use crate::{Config, model::ToolCall};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Stdio,
};
use tokio::io::{AsyncRead, AsyncReadExt};

const FILE_BYTES: usize = 64 * 1024;
const READ_BYTES: usize = 8 * 1024;
const PIPE_BYTES: u64 = 4 * 1024;

pub fn schemas() -> Vec<Value> {
    [
        ("read_file", "Read a UTF-8 file; output may be truncated.", &["path"][..]),
        (
            "write_file",
            "Create or overwrite a UTF-8 file; parent must exist.",
            &["path", "content"],
        ),
        (
            "edit_file",
            "Replace one unique, non-empty exact match.",
            &["path", "old_text", "new_text"],
        ),
        ("bash", "Run Bash in the workspace with bounded output and a timeout.", &["command"]),
    ]
    .into_iter()
    .map(|(name, description, args)| {
        let properties: Map<String, Value> =
            args.iter().map(|arg| (arg.to_string(), json!({"type": "string"}))).collect();

        json!({
            "type": "function",
            "name": name,
            "description": description,
            "strict": true,
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": args,
                "additionalProperties": false
            }
        })
    })
    .collect()
}

pub fn clip(text: &str, max_bytes: usize) -> (&str, bool) {
    let end = text.floor_char_boundary(max_bytes);
    (&text[..end], end < text.len())
}

/// Run one tool call. Failures become `{"ok": false, ...}` results for the model.
pub async fn execute(call: &ToolCall, config: &Config) -> Value {
    dispatch(call, config)
        .await
        .unwrap_or_else(|error| json!({"ok": false, "error": format!("{error:#}")}))
}

async fn dispatch(call: &ToolCall, config: &Config) -> Result<Value> {
    let args: Value = serde_json::from_str(&call.arguments)?;
    let arg = |key: &str| args[key].as_str().with_context(|| format!("{key} must be a string"));
    let path = || resolve(&config.workspace, arg("path")?);

    match call.name.as_str() {
        "bash" => bash(arg("command")?, config).await,
        "read_file" => {
            let text = read(&path()?)?;
            let (content, truncated) = clip(&text, READ_BYTES);
            Ok(json!({"ok": true, "content": content, "truncated": truncated}))
        }
        "write_file" => write(&path()?, arg("content")?),
        "edit_file" => {
            let (path, old) = (path()?, arg("old_text")?);
            ensure!(!old.is_empty(), "old_text must not be empty");
            let mut text = read(&path)?;
            let start = text.find(old).context("old_text did not match; read the file again")?;
            ensure!(text.rfind(old) == Some(start), "old_text must have a unique match");
            text.replace_range(start..start + old.len(), arg("new_text")?);
            write(&path, &text)
        }
        name => bail!("unknown tool: {name}"),
    }
}

/// Paths are relative to the workspace; this is a path contract, not a sandbox.
fn resolve(workspace: &Path, relative: &str) -> Result<PathBuf> {
    let relative = Path::new(relative);
    ensure!(
        !relative.is_absolute() && !relative.components().any(|c| c == Component::ParentDir),
        "path must be relative without parent traversal"
    );
    let path = workspace.join(relative);
    ensure!(!path.exists() || path.is_file(), "path must be a regular file");
    Ok(path)
}

fn read(path: &Path) -> Result<String> {
    ensure!(fs::metadata(path)?.len() <= FILE_BYTES as u64, "file exceeds size limit");
    Ok(fs::read_to_string(path)?)
}

fn write(path: &Path, text: &str) -> Result<Value> {
    ensure!(text.len() <= FILE_BYTES, "result exceeds file size limit");
    fs::write(path, text)?;
    Ok(json!({"ok": true}))
}

async fn bash(command: &str, config: &Config) -> Result<Value> {
    let mut child = tokio::process::Command::new("bash")
        .args(["-c", command])
        .current_dir(&config.workspace)
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let out = child.stdout.take().context("stdout pipe missing")?;
    let err = child.stderr.take().context("stderr pipe missing")?;
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let done = tokio::time::timeout(config.bash_timeout, async {
        tokio::try_join!(child.wait(), drain(out, &mut stdout), drain(err, &mut stderr))
    })
    .await;

    let (status, timed_out, truncated) = match done {
        Ok(result) => {
            let (status, out_cut, err_cut) = result?;
            (Some(status), false, out_cut || err_cut)
        }
        Err(_) => {
            if child.try_wait()?.is_none() {
                child.kill().await?;
            }
            (child.try_wait()?, true, true)
        }
    };

    Ok(json!({
        "ok": !timed_out && status.is_some_and(|s| s.success()),
        "exit_code": status.and_then(|s| s.code()),
        "stdout": String::from_utf8_lossy(&stdout),
        "stderr": String::from_utf8_lossy(&stderr),
        "timed_out": timed_out,
        "truncated": truncated
    }))
}

/// Keep the first bytes of a pipe, but keep reading so the child never blocks on a full pipe.
async fn drain(mut pipe: impl AsyncRead + Unpin, kept: &mut Vec<u8>) -> std::io::Result<bool> {
    (&mut pipe).take(PIPE_BYTES).read_to_end(kept).await?;
    Ok(tokio::io::copy(&mut pipe, &mut tokio::io::sink()).await? > 0)
}
