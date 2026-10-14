use crabloop::{StopReason, cli, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Some(config) = cli::load(&std::env::args().collect::<Vec<_>>())? else {
        return Ok(());
    };

    let report = run(config).await?;

    println!("{}", report.final_text);
    eprintln!("stop: {:?}; model turns: {}", report.reason, report.model_turns);

    if report.reason != StopReason::Final {
        std::process::exit(2);
    }

    Ok(())
}
