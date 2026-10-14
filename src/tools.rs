use crate::{Limits, model::ToolCall};
use anyhow::{Context, Result, ensure};
use serde_json::{Map, Value, json};
use std::{
    fs,
    io::Read,
    path::{Component, Path},
    process::Stdio,
};

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
            args.iter().map(|arg| ((*arg).into(), json!({"type": "string"}))).collect();

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

pub fn clip_utf8(text: &str, max_bytes: usize) -> (String, bool) {
    let mut end = text.len().min(max_bytes);
    while !text.is_char_boundary(end) {
        end -= 1;
    }

    (text[..end].into(), end < text.len())
}

pub async fn execute(call: &ToolCall, workspace: &Path, limits: &Limits) -> Value {
    execute_inner(call, workspace, limits)
        .await
        .unwrap_or_else(|error| json!({"ok":false, "error":format!("{error:#}")}))
}

async fn execute_inner(call: &ToolCall, workspace: &Path, limits: &Limits) -> Result<Value> {
    let args: Value = serde_json::from_str(&call.arguments)?;
    let arg = |key: &str| args[key].as_str().with_context(|| format!("{key} must be a string"));

    if call.name == "bash" {
        return bash(arg("command")?, workspace, limits).await;
    }

    ensure!(
        ["read_file", "write_file", "edit_file"].contains(&call.name.as_str()),
        "unknown tool: {}",
        call.name
    );

    let relative = Path::new(arg("path")?);
    ensure!(
        !relative.is_absolute() && !relative.components().any(|c| c == Component::ParentDir),
        "path must be relative without parent traversal"
    );
    let path = workspace.join(relative);
    ensure!(!path.exists() || path.is_file(), "path must be a regular file");

    let mut text = if call.name == "write_file" {
        arg("content")?.to_owned()
    } else {
        let mut bytes = Vec::new();
        fs::File::open(&path)?.take(limits.file_bytes as u64 + 1).read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= limits.file_bytes, "file exceeds size limit");
        String::from_utf8(bytes)?
    };

    if call.name == "read_file" {
        let (content, truncated) = clip_utf8(&text, limits.read_output_bytes);
        return Ok(json!({"ok":true,"content":content,"truncated":truncated}));
    }

    if call.name == "edit_file" {
        let old = arg("old_text")?;
        ensure!(!old.is_empty(), "old_text must not be empty");
        let start = text.find(old).context("old_text did not match; read the file again")?;
        ensure!(text.rfind(old) == Some(start), "old_text must have a unique match");
        text.replace_range(start..start + old.len(), arg("new_text")?);
    }

    ensure!(text.len() <= limits.file_bytes, "result exceeds file size limit");
    fs::write(path, text)?;
    Ok(json!({"ok":true}))
}

async fn drain(
    mut reader: impl tokio::io::AsyncRead + Unpin,
    kept: &mut Vec<u8>,
    limit: usize,
) -> std::io::Result<bool> {
    use tokio::io::AsyncReadExt;

    let mut chunk = [0; 4096];
    let mut truncated = false;

    loop {
        let n = reader.read(&mut chunk).await?;
        if n == 0 {
            return Ok(truncated);
        }

        let take = n.min(limit.saturating_sub(kept.len()));
        kept.extend_from_slice(&chunk[..take]);
        truncated |= take < n;
    }
}

async fn bash(command: &str, workspace: &Path, limits: &Limits) -> Result<Value> {
    let mut child = tokio::process::Command::new("bash")
        .args(["-c", command])
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let out = child.stdout.take().context("stdout pipe missing")?;
    let err = child.stderr.take().context("stderr pipe missing")?;
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

    let completion = tokio::time::timeout(limits.bash_timeout, async {
        tokio::try_join!(
            child.wait(),
            drain(out, &mut stdout, limits.stream_bytes),
            drain(err, &mut stderr, limits.stream_bytes)
        )
    })
    .await;

    let (status, timed_out, truncated) = match completion {
        Ok(Ok((status, out_cut, err_cut))) => (Some(status), false, out_cut || err_cut),
        other => {
            let observed_status = child.try_wait()?;
            if observed_status.is_none() {
                child.kill().await?;
            }

            if let Ok(Err(error)) = other {
                return Err(error.into());
            }
            (observed_status, true, true)
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
