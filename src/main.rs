use crabloop::{Config, model::ModelClient};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_args_env(&std::env::args().collect::<Vec<_>>())?;
    let client = ModelClient::new(
        &config.endpoint,
        &config.model,
        &config.api_key,
        &config.limits,
    )?;
    let reply = client
        .respond(client.request(
            "Inspect the requested coding task.",
            &[serde_json::json!({"role":"user","content":config.task})],
            &[],
        ))
        .await?;
    println!("{}", reply.text);
    for call in reply.calls {
        eprintln!("tool request: {} ({})", call.name, call.call_id);
    }
    Ok(())
}
