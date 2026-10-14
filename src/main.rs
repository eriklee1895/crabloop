use crabloop::{StopReason, cli, run_turn};
use std::io::IsTerminal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let chat = std::io::stdin().is_terminal();
    let Some(config) = cli::load(&std::env::args().collect::<Vec<_>>())? else {
        return Ok(());
    };
    let mut input = vec![serde_json::json!({"role":"user","content":config.task})];

    loop {
        let report = run_turn(&config, &mut input).await?;
        println!("{}", report.final_text);
        eprintln!("stop: {:?}; model turns: {}", report.reason, report.model_turns);
        if report.reason != StopReason::Final {
            std::process::exit(2);
        }
        if !chat {
            break;
        }
        let Some(task) = cli::read_task(config.limits.request_bytes)? else {
            break;
        };
        input.push(serde_json::json!({"role":"user","content":task}));
    }
    Ok(())
}
