use crate::Config;
use anyhow::{Context, Result, ensure};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}
#[derive(Debug)]
pub struct ModelReply {
    pub raw: Value,
    pub output: Vec<Value>,
    pub calls: Vec<ToolCall>,
    pub text: String,
    pub is_final: bool,
    pub usage: Option<Value>,
}
pub fn request(config: &Config, instructions: &str, input: &[Value], tools: &[Value]) -> Value {
    json!({"model": config.model, "instructions": instructions, "input": input,
        "tools": tools, "store": false, "include": ["reasoning.encrypted_content"],
        "max_output_tokens": config.limits.max_output_tokens})
}
pub async fn respond(client: &Client, config: &Config, body: Value) -> Result<ModelReply> {
    let raw: Value = client
        .post(&config.endpoint)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    ensure!(raw["status"] == "completed", "not completed: {}", raw["incomplete_details"]);
    let output = raw["output"].as_array().context("missing response output")?.clone();
    let mut ids = HashSet::new();
    let mut calls = Vec::new();
    let mut text = String::new();
    let mut is_final = false;
    for item in &output {
        if item["type"] == "function_call" {
            let call: ToolCall = serde_json::from_value(item.clone())?;
            ensure!(
                !call.call_id.is_empty() && ids.insert(call.call_id.clone()),
                "invalid call_id"
            );
            calls.push(call);
        }
        if item["type"] == "message" && item["role"] == "assistant" {
            for content in item["content"].as_array().context("missing message content")? {
                if content["type"] == "output_text" {
                    let part = content["text"].as_str().context("invalid output text")?;
                    text.push_str(part);
                    is_final |= !part.trim().is_empty()
                        && (item["phase"].is_null() || item["phase"] == "final_answer");
                }
            }
        }
    }
    ensure!(!calls.is_empty() || !text.trim().is_empty(), "empty response output");
    Ok(ModelReply { output, calls, text, is_final, usage: raw.get("usage").cloned(), raw })
}
