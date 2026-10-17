use crate::{Config, INSTRUCTIONS};
use anyhow::{Result, ensure};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

/// The parsed view of one model response. `output` keeps every item for replay.
pub struct Reply {
    pub output: Vec<Value>,
    pub calls: Vec<ToolCall>,
    pub final_text: Option<String>,
}

#[derive(Deserialize)]
struct Response {
    status: String,
    output: Vec<Value>,
}

/// The output items we read; reasoning and anything else is replayed untouched.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Item {
    FunctionCall(ToolCall),
    Message {
        content: Vec<Content>,
        phase: Option<String>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Content {
    text: Option<String>,
}

pub fn request(config: &Config, input: &[Value]) -> Value {
    json!({
        "model": config.model,
        "instructions": INSTRUCTIONS,
        "input": input,
        "tools": crate::tools::schemas(),
        "store": false,
        "include": ["reasoning.encrypted_content"],
        "max_output_tokens": 131_072
    })
}

pub async fn respond(client: &Client, config: &Config, body: &Value) -> Result<Reply> {
    let response: Response = client
        .post(&config.endpoint)
        .bearer_auth(&config.api_key)
        .json(body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    ensure!(response.status == "completed", "response status is {}", response.status);
    ensure!(!response.output.is_empty(), "response has no output");

    let mut calls = Vec::new();
    let mut final_text = String::new();
    for item in &response.output {
        match Item::deserialize(item)? {
            Item::FunctionCall(call) => calls.push(call),
            Item::Message { content, phase } => {
                if phase.as_deref().is_none_or(|p| p == "final_answer") {
                    final_text.extend(content.into_iter().filter_map(|c| c.text));
                }
            }
            Item::Other => {}
        }
    }

    let final_text = Some(final_text).filter(|text| !text.trim().is_empty());
    Ok(Reply { output: response.output, calls, final_text })
}
