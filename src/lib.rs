pub mod model;
pub mod tools;
use std::{path::PathBuf, time::Duration};
#[derive(Clone)]
pub struct Limits {
    pub max_turns: usize,
    pub http_timeout: Duration,
    pub bash_timeout: Duration,
    pub request_bytes: usize,
    pub file_bytes: usize,
    pub read_output_bytes: usize,
    pub stream_bytes: usize,
    pub max_output_tokens: u32,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_turns: 16,
            http_timeout: Duration::from_secs(60),
            bash_timeout: Duration::from_secs(30),
            request_bytes: 131072,
            file_bytes: 65536,
            read_output_bytes: 8192,
            stream_bytes: 4096,
            max_output_tokens: 2048,
        }
    }
}
pub struct Config {
    pub workspace: PathBuf,
    pub task: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: String,
    pub limits: Limits,
}

impl Config {
    pub fn from_args_env(args: &[String]) -> anyhow::Result<Self> {
        use anyhow::{Context, ensure};
        let [_, flag, workspace, task] = args else {
            anyhow::bail!("usage: crabloop --workspace DIR TASK");
        };
        ensure!(
            flag == "--workspace" && !task.trim().is_empty(),
            "usage: crabloop --workspace DIR TASK"
        );
        let workspace = std::fs::canonicalize(workspace).context("invalid workspace")?;
        ensure!(workspace.is_dir(), "workspace must be a directory");
        let required = |name| -> anyhow::Result<String> {
            let value = std::env::var(name).with_context(|| format!("{name} is required"))?;
            ensure!(!value.trim().is_empty(), "{name} must not be empty");
            Ok(value)
        };
        Ok(Self {
            workspace,
            task: task.clone(),
            api_key: required("OPENAI_API_KEY")?,
            model: required("CRABLOOP_MODEL")?,
            endpoint: std::env::var("CRABLOOP_API_URL").unwrap_or_else(|_| {
                format!(
                    "{}/responses",
                    std::env::var("OPENAI_BASE_URL")
                        .unwrap_or_else(|_| "https://api.openai.com/v1".into())
                        .trim_end_matches('/')
                )
            }),
            limits: Limits::default(),
        })
    }
}
