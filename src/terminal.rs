use crate::{Config, Progress, Session, StopReason};
use anyhow::Result;
use rustyline::{DefaultEditor, error::ReadlineError};
use std::io::IsTerminal;

const HELP: &str = "Type a task to begin. Up/Down: input history.\n/new: fresh conversation · /exit or Ctrl-D: quit · Ctrl-C: cancel\n";

pub async fn chat(config: Config) -> Result<()> {
    let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let (accent, reset) = if color { ("\x1b[36;1m", "\x1b[0m") } else { ("", "") };
    println!("\n{accent}🦀 CrabLoop{reset} · a small Rust agent\n{HELP}");
    let mut editor = DefaultEditor::new()?;
    let mut session: Option<Session> = None;
    loop {
        let line = match editor.readline(&format!("{accent}crab>{reset} ")) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(error) => return Err(error.into()),
        };
        let task = line.trim();
        match task {
            "" => continue,
            "/exit" | "/quit" => break,
            "/help" => {
                println!("{HELP}");
                continue;
            }
            "/new" => {
                session = None;
                println!("New conversation. Files are kept.\n");
                continue;
            }
            _ => {}
        }
        if task.len() > config.limits.request_bytes {
            eprintln!("Task exceeds input budget. Shorten it and try again.");
            continue;
        }
        editor.add_history_entry(task)?;
        if session.is_none() {
            session = Some(Session::new(config.clone())?);
        }
        let session = session.as_mut().unwrap();
        let started = std::time::Instant::now();
        let outcome = tokio::select! {
            result = session.turn(task, progress) => Some(result),
            signal = tokio::signal::ctrl_c() => { signal?; None }
        };
        match outcome {
            Some(Ok(report)) => {
                if !report.final_text.is_empty() {
                    println!("\n{accent}CrabLoop{reset}\n{}", report.final_text);
                }
                eprintln!(
                    "  {:?} · {} steps · {:.1}s\n",
                    report.reason,
                    report.model_turns,
                    started.elapsed().as_secs_f64()
                );
                if report.reason != StopReason::Final {
                    eprintln!("Task stopped before completion. /new clears the context.");
                }
            }
            Some(Err(error)) => {
                eprintln!("Error: {error:#}\nInspect workspace changes; use /new to continue.\n")
            }
            None => {
                session.interrupt()?;
                eprintln!("Cancelled. Effects may be incomplete; inspect files, then /new.\n");
            }
        }
    }
    println!("Bye.");
    Ok(())
}

fn progress(event: Progress<'_>) {
    match event {
        Progress::Model(step) => eprintln!("  · step {step} · 处理中…"),
        Progress::Commentary(text) => eprintln!("\n过程\n{text}\n"),
        Progress::ToolStart(call) => {
            let args: serde_json::Value = serde_json::from_str(&call.arguments).unwrap_or_default();
            let detail = args["path"].as_str().or_else(|| args["command"].as_str()).unwrap_or("");
            let detail: String = detail.chars().filter(|c| !c.is_control()).take(100).collect();
            eprintln!("  → {}  {detail}", call.name);
        }
        Progress::ToolResult(_, result) => {
            eprintln!(
                "    {}",
                if result["ok"] == true { "✓ done" } else { "! tool reported an error" }
            );
        }
    }
}
