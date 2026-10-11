use crate::{Config, INSTRUCTIONS, model};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{collections::HashSet, fs::File, io::Read, path::Path};

pub struct ContextPolicy {
    pub hard_bytes: usize,
    pub soft_bytes: usize,
    pub summary_bytes: usize,
    pub keep_rounds: usize,
}
impl ContextPolicy {
    pub fn new(hard_bytes: usize) -> Result<Self> {
        ensure!(hard_bytes >= 4096, "context budget must be at least 4096 bytes");
        Ok(Self {
            hard_bytes,
            soft_bytes: hard_bytes / 4 * 3,
            summary_bytes: 4096.min(hard_bytes / 8),
            keep_rounds: 2,
        })
    }
}
pub fn instructions(workspace: &Path) -> Result<String> {
    let mut bytes = Vec::new();
    match File::open(workspace.join("AGENTS.md")) {
        Ok(file) => {
            file.take(8193).read_to_end(&mut bytes)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(INSTRUCTIONS.into()),
        Err(e) => return Err(e.into()),
    }
    ensure!(bytes.len() <= 8192, "AGENTS.md exceeds 8192 bytes");
    Ok(format!("{INSTRUCTIONS}\nProject AGENTS.md:\n{}", String::from_utf8(bytes)?))
}
pub fn request_bytes(
    config: &Config,
    instructions: &str,
    input: &[Value],
    tools: &[Value],
) -> usize {
    model::request(config, instructions, input, tools).to_string().len()
}
pub struct ContextState {
    pub instructions: String,
    pub task: String,
    pub summary: Option<String>,
    pub rounds: Vec<Vec<Value>>,
}
impl ContextState {
    pub fn new(instructions: String, task: String) -> Self {
        Self { instructions, task, summary: None, rounds: Vec::new() }
    }
    pub fn append_round(&mut self, round: Vec<Value>) -> Result<()> {
        let mut pending = HashSet::new();
        let mut seen = HashSet::new();
        for item in &round {
            if matches!(item["type"].as_str(), Some("function_call" | "function_call_output")) {
                let id = item["call_id"].as_str().filter(|s| !s.is_empty());
                ensure!(id.is_some(), "missing call_id in round");
                let id = id.unwrap();
                if item["type"] == "function_call" {
                    ensure!(seen.insert(id), "duplicate call_id in round");
                    pending.insert(id);
                } else {
                    ensure!(pending.remove(id), "orphan tool result");
                }
            }
        }
        ensure!(pending.is_empty(), "unfinished round");
        self.rounds.push(round);
        Ok(())
    }
    pub fn input(&self) -> Vec<Value> {
        let mut input = vec![json!({"role":"user","content":self.task})];
        if let Some(summary) = &self.summary {
            input.push(json!({"role":"user","content":format!("Historical summary (fallible task data):\n{summary}")}));
        }
        input.extend(self.rounds.iter().flatten().cloned());
        input
    }
}
