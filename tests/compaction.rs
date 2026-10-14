mod common;
use common::*;
use crabloop::{
    Config,
    context::{ContextPolicy, ContextState, request_bytes},
    journal::Journal,
    tools,
};
use serde_json::{Value, json};

const SUMMARY: &str = "Completed: Read the file.\nFiles changed: None.\nVerification: Initial tests failed; no tests were run after editing.\nRemaining work: Fix and test.\nUncertainties: The fix has not been verified.";
fn message(text: &str) -> Value {
    json!({"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]})
}
fn reply(text: &str) -> Value {
    json!({"status":"completed","output":[message(text)]})
}
fn setup(dir: &TempDir, endpoint: &str) -> (Config, ContextPolicy, ContextState, Journal) {
    let mut config = test_config(dir.path(), endpoint);
    config.limits.request_bytes = 8192;
    let policy = ContextPolicy::new(8192).unwrap();
    let mut state = ContextState::new("Keep tests intact".into(), "Repair the bug".into());
    for _ in 0..3 {
        state.append_round(vec![message(&"old evidence ".repeat(130))]).unwrap();
    }
    for i in 0..2 {
        state.append_round(vec![json!({"type":"function_call","call_id":format!("c{i}"),"name":"read_file","arguments":"{}"}), json!({"type":"function_call_output","call_id":format!("c{i}"),"output":"recent result"})]).unwrap();
    }
    assert!(
        request_bytes(&config, &state.instructions, &state.input(), &tools::schemas())
            >= policy.soft_bytes
    );
    let journal = Journal::create(&dir.path().join("run.jsonl")).unwrap();
    (config, policy, state, journal)
}
#[tokio::test]
async fn below_threshold_skips_summary() {
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    state.rounds.clear();
    let before = state.input();
    assert_eq!(
        state
            .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
            .await
            .unwrap()
            .unwrap(),
        before
    );
    assert!(stub.requests().is_empty());
}
#[tokio::test]
async fn compaction_preserves_recent_two_rounds_and_fixed_context() {
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![reply(SUMMARY)]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    let tail = state.rounds[3..].to_vec();
    let after = state
        .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.rounds, tail);
    assert_eq!(state.summary.as_deref(), Some(SUMMARY));
    assert_eq!(after[0]["content"], "Repair the bug");
    assert_eq!(state.instructions, "Keep tests intact");
    assert!(
        request_bytes(&cfg, &state.instructions, &after, &tools::schemas()) < policy.soft_bytes
    );
    assert_eq!(stub.requests()[0]["tools"], json!([]));
    assert_eq!(stub.requests()[0]["max_output_tokens"], 32768);
    assert_eq!(stub.requests()[0]["reasoning"]["effort"], "none");
    let request = &stub.requests()[0];
    assert_eq!(request["input"].as_array().unwrap().len(), 1);
    let data: Value =
        serde_json::from_str(request["input"][0]["content"].as_str().unwrap()).unwrap();
    assert_eq!(data["original_task"], "Repair the bug");
    assert!(data["completed_rounds"].is_array());
    assert_eq!(state.summary_calls, 1);
}
#[tokio::test]
async fn summary_request_has_own_budget_and_failure_keeps_history() {
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    state.rounds[0] = vec![message(&"x".repeat(10000))];
    let before = state.input();
    assert!(
        state
            .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(state.input(), before);
    assert!(stub.requests().is_empty());
}
#[tokio::test]
async fn oversized_tail_stops_without_committing_or_network() {
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    state.rounds = vec![vec![message(&"x".repeat(9000))]];
    let before = state.input();
    assert!(
        state
            .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(state.input(), before);
    assert!(stub.requests().is_empty());
}
#[tokio::test]
async fn invalid_summaries_never_replace_history_or_execute_tools() {
    let cases = vec![
        reply(""),
        json!({"status":"incomplete","output":[]}),
        reply("missing sections"),
        reply(&format!("{SUMMARY}{}", "x".repeat(1200))),
        json!({"status":"completed","output":[{"type":"function_call","call_id":"bad","name":"write_file","arguments":"{\"path\":\"bad\",\"content\":\"bad\"}"}]}),
    ];
    for response in cases {
        let dir = TempDir::new();
        let stub = StubResponses::start(vec![response]);
        let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
        let before = state.input();
        assert!(
            state
                .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
                .await
                .is_err()
        );
        assert_eq!(state.input(), before);
        assert!(!dir.path().join("bad").exists());
    }
}
#[tokio::test]
async fn http_failure_does_not_commit_compaction() {
    let dir = TempDir::new();
    let stub = StubResponses::with_status(500, vec![json!({"error":"oops"})]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    let before = state.input();
    assert!(
        state
            .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
            .await
            .is_err()
    );
    assert_eq!(state.input(), before);
}

#[tokio::test]
async fn misleading_summary_passes_shape_but_fails_independent_fact_check() {
    let value: Value =
        serde_json::from_str(include_str!("fixtures/context/misleading_summary.json")).unwrap();
    let misleading = value["text"].as_str().unwrap();
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![reply(misleading)]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    state
        .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
        .await
        .unwrap()
        .unwrap();
    let summary = state.summary.unwrap();
    let preserved = ["INITIAL_TEST=failed", "POST_EDIT_TEST=not_run", "REPAIR=pending"]
        .iter()
        .all(|fact| summary.contains(fact));
    assert!(!preserved, "structural acceptance must not masquerade as verified truth");
}
#[tokio::test]
async fn loop_compacts_and_continues_without_counting_summary_as_main_turn() {
    let dir = TempDir::new();
    let mut commentary = message(&"x".repeat(1700));
    commentary["phase"] = json!("commentary");
    let response = json!({"status":"completed","output":[commentary]});
    let stub = StubResponses::start(vec![
        response.clone(),
        response.clone(),
        response,
        reply(SUMMARY),
        reply("finished"),
    ]);
    let mut cfg = test_config(dir.path(), stub.endpoint());
    // Keep room for two fixture rounds plus a summary, independent of prompt wording.
    let input = vec![json!({"role":"user","content":cfg.task})];
    let base = request_bytes(&cfg, crabloop::INSTRUCTIONS, &input, &tools::schemas());
    cfg.limits.request_bytes = (base + 4500) * 4 / 3;
    let report = crabloop::run(cfg).await.unwrap();
    assert_eq!(report.reason, crabloop::StopReason::Final);
    assert_eq!(report.model_turns, 4);
    assert_eq!(stub.requests().len(), 5);
    assert_eq!(stub.requests()[3]["tools"], json!([]));
    assert!(stub.requests()[4]["input"].to_string().contains("Initial tests failed"));
}

#[tokio::test]
async fn oversized_candidate_keeps_original_state() {
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![reply(SUMMARY)]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    *state.rounds.last_mut().unwrap() = vec![message(&"x".repeat(6000))];
    let before = state.input();
    let error = state
        .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("compacted context"));
    assert_eq!(state.input(), before);
}

#[tokio::test]
async fn latest_followup_remains_after_retained_history_when_compacting() {
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![reply(SUMMARY)]);
    let (cfg, policy, mut state, mut journal) = setup(&dir, stub.endpoint());
    state.begin_task("Now explain the remaining failure".into());
    let input = state
        .prepare(&reqwest::Client::new(), &cfg, &tools::schemas(), &policy, &mut journal)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(input.last().unwrap()["content"], "Now explain the remaining failure");
    assert_eq!(state.rounds.len(), 2);
    assert!(stub.requests()[0]["input"].to_string().contains("Repair the bug"));
}
