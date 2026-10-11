use crate::{Limits, model::ToolCall};
use anyhow::{Context, Result, ensure};
use serde_json::{Map, Value, json};
use std::{
    fs,
    io::Read,
    path::{Component, Path},
};

pub struct ToolResult {
    pub call_id: String,
    pub output: Value,
}
impl ToolResult {
    pub fn as_input(&self) -> Value {
        json!({"type":"function_call_output", "call_id":self.call_id, "output":self.output.to_string()})
    }
}
pub fn schemas() -> Vec<Value> {
    [("read_file", "Read a UTF-8 file; output may be truncated.", &["path"][..]),
     ("write_file", "Create or overwrite a UTF-8 file; parent must exist.", &["path", "content"]),
     ("edit_file", "Replace one unique, non-empty exact match.", &["path", "old_text", "new_text"])]
        .into_iter().map(|(name, description, args)| {
            let properties: Map<String, Value> = args.iter().map(|arg| ((*arg).into(), json!({"type":"string"}))).collect();
            json!({"type":"function", "name":name, "description":description, "strict":true,
                "parameters":{"type":"object", "properties":properties, "required":args, "additionalProperties":false}})
        }).collect()
}
pub fn clip_utf8(text: &str, max_bytes: usize) -> (String, bool) {
    let mut end = text.len().min(max_bytes);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].into(), end < text.len())
}
pub async fn execute(call: &ToolCall, workspace: &Path, limits: &Limits) -> ToolResult {
    let output = execute_inner(call, workspace, limits)
        .await
        .unwrap_or_else(|error| json!({"ok":false, "error":format!("{error:#}")}));
    ToolResult {
        call_id: call.call_id.clone(),
        output,
    }
}
async fn execute_inner(call: &ToolCall, workspace: &Path, limits: &Limits) -> Result<Value> {
    let args: Value = serde_json::from_str(&call.arguments)?;
    let arg = |key: &str| {
        args[key]
            .as_str()
            .with_context(|| format!("{key} must be a string"))
    };
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
    let mut text = if call.name == "write_file" {
        arg("content")?.to_owned()
    } else {
        ensure!(
            fs::metadata(&path)?.is_file(),
            "path must be a regular file"
        );
        let mut bytes = Vec::new();
        fs::File::open(&path)?
            .take(limits.file_bytes as u64 + 1)
            .read_to_end(&mut bytes)?;
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
        let start = text
            .find(old)
            .context("old_text did not match; read the file again")?;
        ensure!(
            text.rfind(old) == Some(start),
            "old_text must have a unique match"
        );
        text.replace_range(start..start + old.len(), arg("new_text")?);
    }
    ensure!(
        text.len() <= limits.file_bytes,
        "result exceeds file size limit"
    );
    fs::write(path, text)?;
    Ok(json!({"ok":true}))
}
