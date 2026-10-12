use crabloop::{StopReason, cli, run, terminal};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Some(launch) = cli::load(&std::env::args().collect::<Vec<_>>())? else {
        return Ok(());
    };
    if launch.chat {
        return terminal::chat(launch.config).await;
    }
    let report = run(launch.config).await?;
    println!("{}", report.final_text);
    eprintln!("stop: {:?}; model turns: {}", report.reason, report.model_turns);
    if report.reason != StopReason::Final {
        std::process::exit(2);
    }
    Ok(())
}
