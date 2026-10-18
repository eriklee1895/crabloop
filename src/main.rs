use crabloop::{Config, Stop, run_turn};
use std::io::BufRead;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
    let config = Config::from_env()?;
    let mut input = Vec::new();

    eprint!("crab> ");
    for line in std::io::stdin().lock().lines() {
        let task = line?;
        if !task.trim().is_empty() {
            input.push(serde_json::json!({"role": "user", "content": task}));
            match run_turn(&config, &mut input).await? {
                Stop::Final(text) => println!("{text}"),
                stop => {
                    eprintln!("stopped: {stop:?}");
                    return Ok(ExitCode::from(2));
                }
            }
        }
        eprint!("crab> ");
    }

    Ok(ExitCode::SUCCESS)
}
