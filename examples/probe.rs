//! Send one request and print the parsed reply. Nothing is executed.
//!
//! cargo run --example probe -- "只回复 CRABLOOP_PROTOCOL_OK"

use crabloop::{Config, model};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let task = std::env::args().nth(1).unwrap_or_else(|| "只回复 CRABLOOP_PROTOCOL_OK".into());

    let request = model::request(&config, &[json!({"role": "user", "content": task})]);
    let reply = model::respond(&reqwest::Client::new(), &config, &request).await?;

    println!("{}", serde_json::to_string_pretty(&reply.output)?);
    eprintln!("tool calls: {}; final text: {:?}", reply.calls.len(), reply.final_text);
    Ok(())
}
