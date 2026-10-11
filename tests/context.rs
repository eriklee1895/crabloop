mod common;
use common::*;
use crabloop::{
    context::{ContextPolicy, ContextState, instructions, request_bytes},
    journal::Journal,
    tools,
};
use serde_json::{Value, json};
use std::fs;

fn round() -> Vec<Value> {
    vec![
        json!({"type":"reasoning","encrypted_content":"opaque"}),
        json!({"type":"function_call","call_id":"a","name":"read_file","arguments":"{}"}),
        json!({"type":"function_call_output","call_id":"a","output":"result"}),
    ]
}
#[test]
fn complete_round_preserves_all_items_and_fixed_context() {
    let mut state = ContextState::new("project rules".into(), "original task".into());
    let items = round();
    state.append_round(items.clone()).unwrap();
    state.append_round(items.clone()).unwrap();
    state.rounds.remove(0);
    let input = state.input();
    assert_eq!(input[0]["content"], "original task");
    assert_eq!(&input[1..], &items);
    assert_eq!(state.instructions, "project rules");
}
#[test]
fn rejects_orphan_missing_duplicate_or_out_of_order_results() {
    let items = round();
    for bad in [
        vec![items[2].clone()],
        items[..2].to_vec(),
        vec![items[1].clone(), items[2].clone(), items[2].clone()],
        vec![items[2].clone(), items[1].clone()],
    ] {
        let mut state = ContextState::new("rules".into(), "task".into());
        assert!(state.append_round(bad).is_err());
        assert!(state.rounds.is_empty());
    }
}
#[test]
fn request_bytes_include_tools_instructions_and_ciphertext() {
    let dir = TempDir::new();
    let cfg = test_config(dir.path(), "unused");
    let input = round();
    let short = request_bytes(&cfg, "", &[], &[]);
    assert!(request_bytes(&cfg, "rules", &input, &tools::schemas()) > short + 1000);
    assert_eq!(
        request_bytes(&cfg, "rules", &input, &tools::schemas()),
        serde_json::to_vec(&crabloop::model::request(&cfg, "rules", &input, &tools::schemas()))
            .unwrap()
            .len()
    );
}
#[test]
fn journal_is_append_only_and_refuses_overwrite() {
    let dir = TempDir::new();
    let path = dir.path().join("run.jsonl");
    let mut journal = Journal::create(&path).unwrap();
    journal.record("request", &json!({"input":"task"})).unwrap();
    let prefix = fs::read(&path).unwrap();
    journal.record("result", &json!({"ok":true})).unwrap();
    let all = fs::read_to_string(&path).unwrap();
    assert!(all.as_bytes().starts_with(&prefix));
    assert_eq!(all.lines().count(), 2);
    assert_eq!(
        serde_json::from_str::<Value>(all.lines().nth(1).unwrap()).unwrap()["kind"],
        "result"
    );
    assert!(Journal::create(&path).is_err());
}
#[test]
fn invalid_budget_is_rejected_and_defaults_are_derived() {
    assert!(ContextPolicy::new(4095).is_err());
    let p = ContextPolicy::new(49152).unwrap();
    assert_eq!(
        (p.hard_bytes, p.soft_bytes, p.summary_bytes, p.keep_rounds),
        (49152, 36864, 4096, 2)
    );
}
#[test]
fn root_agents_is_optional_but_invalid_content_fails() {
    let dir = TempDir::new();
    assert_eq!(instructions(dir.path()).unwrap(), crabloop::INSTRUCTIONS);
    let path = dir.path().join("AGENTS.md");
    fs::write(&path, "Keep tests intact").unwrap();
    assert!(instructions(dir.path()).unwrap().contains("Keep tests intact"));
    fs::write(&path, [255]).unwrap();
    assert!(instructions(dir.path()).is_err());
    fs::write(&path, "x".repeat(8193)).unwrap();
    assert!(instructions(dir.path()).is_err());
}
#[tokio::test]
async fn run_journals_protocol_tools_and_stop_without_credentials() {
    let dir = TempDir::new();
    fs::write(dir.path().join("AGENTS.md"), "Keep tests intact").unwrap();
    let stub = StubResponses::start(vec![
        json!({"status":"completed","output":[{"type":"function_call","call_id":"a","name":"write_file","arguments":"{\"path\":\"note\",\"content\":\"hello\"}"}]}),
        fixture("text"),
    ]);
    crabloop::run(test_config(dir.path(), stub.endpoint())).await.unwrap();
    let path = fs::read_dir(dir.path().join(".crabloop")).unwrap().next().unwrap().unwrap().path();
    let log = fs::read_to_string(path).unwrap();
    assert!(!log.contains("test-key"));
    for kind in ["request", "response", "tool_started", "tool_finished", "stop"] {
        assert!(log.contains(kind));
    }
    assert!(
        stub.requests()
            .iter()
            .all(|r| r["instructions"].as_str().unwrap().contains("Keep tests intact"))
    );
}
