use crate::{Config, Limits, context::ContextPolicy};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::HashMap,
    io::{self, IsTerminal, Read, Write},
    path::Path,
};

pub const HELP: &str = "usage: crabloop [--workspace DIR] [TASK]\n\n直接启动：在当前目录输入任务\ncrabloop \"检查 README\"\ncrabloop --workspace path/to/project \"修复测试\"\n\n读取当前目录的 .env，进程环境优先。\n推荐配置 DEEPSEEK_API_KEY；也支持 OPENAI_API_KEY / OPENAI_BASE_URL / CRABLOOP_MODEL。\nCRABLOOP_PROVIDER 可显式选择 deepseek 或 openai。";

pub fn load(args: &[String]) -> Result<Option<Config>> {
    let args = args.get(1..).unwrap_or_default();
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        println!("{HELP}");
        return Ok(None);
    }
    let (workspace, task) = match args {
        [] => (".", None),
        [task] if !task.starts_with('-') => (".", Some(task.as_str())),
        [flag, dir] if flag == "--workspace" => (dir.as_str(), None),
        [flag, dir, task] if flag == "--workspace" => (dir.as_str(), Some(task.as_str())),
        _ => bail!(HELP),
    };
    ensure!(task.is_none_or(|t| !t.trim().is_empty()), "任务不能为空\n{HELP}");
    let workspace = std::fs::canonicalize(workspace).context("invalid workspace")?;
    ensure!(workspace.is_dir(), "workspace must be a directory");
    let local: HashMap<String, String> = match dotenvy::from_path_iter(Path::new(".env")) {
        Ok(entries) => entries
            .collect::<std::result::Result<_, _>>()
            .map_err(|_| anyhow::anyhow!("无法解析 .env；请检查 KEY=VALUE 格式"))?,
        Err(dotenvy::Error::Io(e)) if e.kind() == io::ErrorKind::NotFound => HashMap::new(),
        Err(_) => bail!("无法读取当前目录的 .env"),
    };
    let get = |name: &str| {
        std::env::var(name)
            .ok()
            .or_else(|| local.get(name).cloned())
            .filter(|value| !value.trim().is_empty())
    };
    let explicit_generic = get("CRABLOOP_MODEL").is_some()
        || get("CRABLOOP_API_URL").is_some()
        || ["OPENAI_API_KEY", "OPENAI_BASE_URL"]
            .iter()
            .any(|name| std::env::var(name).is_ok_and(|v| !v.trim().is_empty()));
    let provider = get("CRABLOOP_PROVIDER").unwrap_or_else(|| {
        if !explicit_generic && get("DEEPSEEK_API_KEY").is_some() { "deepseek" } else { "openai" }
            .into()
    });
    let (key_name, base, default_model) = match provider.as_str() {
        "deepseek" => ("DEEPSEEK_API_KEY", "https://api.deepseek.com".to_owned(), "deepseek-flash"),
        "openai" => {
            let base = get("OPENAI_BASE_URL").unwrap_or_else(|| "https://api.openai.com/v1".into());
            let deepseek = reqwest::Url::parse(&base)
                .ok()
                .is_some_and(|url| url.host_str() == Some("api.deepseek.com"));
            ("OPENAI_API_KEY", base, if deepseek { "deepseek-flash" } else { "gpt-6-luna" })
        }
        _ => bail!("CRABLOOP_PROVIDER 必须为 deepseek 或 openai"),
    };
    let api_key = get(key_name)
        .with_context(|| format!("缺少 {key_name}；请在当前目录 .env 配置，或导出到终端环境"))?;
    let mut config = Config {
        workspace,
        task: task.unwrap_or_default().into(),
        api_key,
        model: get("CRABLOOP_MODEL").unwrap_or_else(|| default_model.into()),
        endpoint: get("CRABLOOP_API_URL")
            .unwrap_or_else(|| format!("{}/responses", base.trim_end_matches('/'))),
        limits: Limits {
            request_bytes: ContextPolicy::new(
                get("CRABLOOP_CONTEXT_BYTES").unwrap_or_else(|| "49152".into()).parse()?,
            )?
            .hard_bytes,
            ..Limits::default()
        },
    };
    eprintln!("CrabLoop · {provider}/{} · {}", config.model, config.workspace.display());
    if task.is_none() {
        let Some(task) = read_task(config.limits.request_bytes)? else {
            return Ok(None);
        };
        config.task = task;
    }
    Ok(Some(config))
}

fn read_task(limit: usize) -> Result<Option<String>> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        loop {
            eprint!("任务（Ctrl-D 退出）> ");
            io::stderr().flush()?;
            let mut task = String::new();
            if stdin.read_line(&mut task)? == 0 {
                return Ok(None);
            }
            ensure!(task.len() <= limit, "任务超过输入预算");
            if !task.trim().is_empty() {
                return Ok(Some(task.trim().into()));
            }
        }
    }
    let mut bytes = Vec::new();
    stdin.lock().take(limit as u64 + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= limit, "任务超过输入预算");
    let task = String::from_utf8(bytes)?.trim().to_owned();
    Ok((!task.is_empty()).then_some(task))
}
