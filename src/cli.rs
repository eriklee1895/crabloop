use crate::{Config, Limits};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::HashMap,
    io::{self, IsTerminal, Read},
    path::Path,
};

pub const HELP: &str = "usage: crabloop [--workspace DIR] [-p TASK | --chat]\n\ncrabloop                              多轮对话（默认当前目录）\ncrabloop -p \"检查 README\"              单次任务，回答后退出\ncrabloop --workspace path/to/project   在指定目录对话\ncrabloop --chat                        强制聊天（包括管道输入）\n\n-p / --print TASK；也兼容旧的 TASK 位置参数。无参数管道输入作为单次任务。\n聊天：/new 清空会话，/help 帮助，/exit 或 Ctrl-D 退出。\n读取当前目录的 .env，进程环境优先。\n推荐 DEEPSEEK_API_KEY；也支持 OPENAI_API_KEY / OPENAI_BASE_URL / CRABLOOP_MODEL。\nCRABLOOP_PROVIDER 可显式选择 deepseek 或 openai。";

pub struct Launch {
    pub config: Config,
    pub chat: bool,
}
pub fn load(args: &[String]) -> Result<Option<Launch>> {
    let args = args.get(1..).unwrap_or_default();
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        println!("{HELP}");
        return Ok(None);
    }
    let (mut workspace, mut task, mut chat) = (".", None, false);
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = args.next().context("--workspace 需要目录")?,
            "--chat" => chat = true,
            "-p" | "--print" if task.is_none() => {
                task = Some(args.next().context("-p 需要任务")?.as_str())
            }
            value if !value.starts_with('-') && task.is_none() => task = Some(value),
            _ => bail!(HELP),
        }
    }
    ensure!(!chat || task.is_none(), "--chat 不能与单次任务同时使用");
    ensure!(task.is_none_or(|t| !t.trim().is_empty()), "任务不能为空");
    chat |= task.is_none() && io::stdin().is_terminal();
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
    let request_bytes = get("CRABLOOP_CONTEXT_BYTES")
        .unwrap_or_else(|| Limits::default().request_bytes.to_string())
        .parse()?;
    ensure!(request_bytes >= 4096, "context budget must be at least 4096 bytes");
    let mut config = Config {
        workspace,
        task: task.unwrap_or_default().into(),
        api_key,
        model: get("CRABLOOP_MODEL").unwrap_or_else(|| default_model.into()),
        endpoint: get("CRABLOOP_API_URL")
            .unwrap_or_else(|| format!("{}/responses", base.trim_end_matches('/'))),
        limits: Limits { request_bytes, ..Limits::default() },
    };
    eprintln!("CrabLoop · {provider}/{} · {}", config.model, config.workspace.display());
    if task.is_none() && !chat {
        let Some(task) = read_task(config.limits.request_bytes)? else {
            return Ok(None);
        };
        config.task = task;
    }
    Ok(Some(Launch { config, chat }))
}

fn read_task(limit: usize) -> Result<Option<String>> {
    let stdin = io::stdin();
    let mut bytes = Vec::new();
    stdin.lock().take(limit as u64 + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= limit, "任务超过输入预算");
    let task = String::from_utf8(bytes)?.trim().to_owned();
    Ok((!task.is_empty()).then_some(task))
}
