use crate::{Config, Limits};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::HashMap,
    io::{self, BufRead, IsTerminal, Read},
};

pub fn load(args: &[String]) -> Result<Option<Config>> {
    ensure!(
        args.len() <= 1,
        "v1 takes no arguments; enter a task on stdin. See README.md for setup."
    );
    let limits = Limits::default();
    let stdin = io::stdin();
    let mut input = stdin.lock().take(limits.request_bytes as u64 + 1);
    let mut task = String::new();
    if stdin.is_terminal() {
        eprint!("Task (Ctrl-D to exit)> ");
        input.read_line(&mut task)?;
    } else {
        input.read_to_string(&mut task)?;
    }
    ensure!(task.len() <= limits.request_bytes, "task exceeds input budget");
    if task.trim().is_empty() {
        return Ok(None);
    }
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
    let api_key = get(key).with_context(|| format!("{key} is required in .env or environment"))?;
    let model = get("CRABLOOP_MODEL").unwrap_or_else(|| model.into());
    eprintln!("CrabLoop · {model} · {}", workspace.display());
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
