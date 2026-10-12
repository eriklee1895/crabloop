use crate::{Config, Limits};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::HashMap,
    io::{self, BufRead, IsTerminal, Read},
};

const HELP: &str = "usage: crabloop [-p TASK]\n无参数：在当前目录输入一个任务；-p/--print：单次任务。\n读取当前目录 .env；DEEPSEEK_API_KEY 默认使用 deepseek-flash。\nCRABLOOP_PROVIDER=openai 使用 OPENAI_API_KEY / OPENAI_BASE_URL / CRABLOOP_MODEL。";
pub fn load(args: &[String]) -> Result<Option<Config>> {
    if args.len() == 2 && matches!(args[1].as_str(), "-h" | "--help") {
        println!("{HELP}");
        return Ok(None);
    }
    let task = match args.get(1..).unwrap_or_default() {
        [] => None,
        [task] if !task.starts_with('-') => Some(task.clone()),
        [flag, task] if matches!(flag.as_str(), "-p" | "--print") => Some(task.clone()),
        _ => bail!(HELP),
    };
    let workspace = std::env::current_dir()?;
    let local: HashMap<String, String> = match dotenvy::from_path_iter(".env") {
        Ok(entries) => entries
            .collect::<std::result::Result<_, _>>()
            .map_err(|_| anyhow::anyhow!("Invalid .env; check KEY=VALUE syntax"))?,
        Err(dotenvy::Error::Io(e)) if e.kind() == io::ErrorKind::NotFound => HashMap::new(),
        Err(_) => bail!("Cannot read .env"),
    };
    let get = |name: &str| {
        std::env::var(name)
            .ok()
            .or_else(|| local.get(name).cloned())
            .filter(|v| !v.trim().is_empty())
    };
    let provider = get("CRABLOOP_PROVIDER").unwrap_or_else(|| "deepseek".into());
    let (key, base, model) = match provider.as_str() {
        "deepseek" => ("DEEPSEEK_API_KEY", "https://api.deepseek.com".into(), "deepseek-flash"),
        "openai" => (
            "OPENAI_API_KEY",
            get("OPENAI_BASE_URL").unwrap_or_else(|| "https://api.openai.com/v1".into()),
            "gpt-6-luna",
        ),
        _ => bail!("CRABLOOP_PROVIDER must be deepseek or openai"),
    };
    let limits = Limits::default();
    let api_key = get(key).with_context(|| format!("{key} is required in .env or environment"))?;
    let model = get("CRABLOOP_MODEL").unwrap_or_else(|| model.into());
    eprintln!("CrabLoop · {model} · {}", workspace.display());
    let task = match task {
        Some(task) => task,
        None => {
            let stdin = io::stdin();
            let mut input = stdin.lock().take(limits.request_bytes as u64 + 1);
            let mut text = String::new();
            if stdin.is_terminal() {
                eprint!("任务（Ctrl-D 退出）> ");
                input.read_line(&mut text)?;
            } else {
                input.read_to_string(&mut text)?;
            }
            text
        }
    };
    ensure!(task.len() <= limits.request_bytes, "task exceeds input budget");
    if task.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(Config {
        workspace,
        task: task.trim().into(),
        api_key,
        model,
        limits,
        endpoint: get("CRABLOOP_API_URL")
            .unwrap_or_else(|| format!("{}/responses", base.trim_end_matches('/'))),
    }))
}
