use crabloop::{Config, StopReason, run};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let report = run(Config::from_args_env(&std::env::args().collect::<Vec<_>>())?).await?;
    println!("{}", report.final_text);
    eprintln!("stop: {:?}; model turns: {}", report.reason, report.model_turns);
    if report.reason != StopReason::Final {
        std::process::exit(2);
    }
    Ok(())
}
