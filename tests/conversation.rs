mod common;
use common::*;
use crabloop::{Session, StopReason};
use serde_json::{Value, json};

fn reply(text: &str) -> Value {
    json!({"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}]})
}
#[tokio::test]
async fn successive_user_turns_keep_prior_question_and_final_answer_in_order() {
    let dir = TempDir::new();
    let stub = StubResponses::start(vec![reply("The code is COBALT."), reply("COBALT")]);
    let mut session = Session::new(test_config(dir.path(), stub.endpoint())).unwrap();
    session.turn("Remember code COBALT", |_| {}).await.unwrap();
    let result = session.turn("What was the code?", |_| {}).await.unwrap();
    assert_eq!(result.reason, StopReason::Final);
    let sent = stub.requests();
    let input = sent[1]["input"].as_array().unwrap();
    assert_eq!(input.len(), 3);
    assert_eq!(input[0]["content"], "Remember code COBALT");
    assert_eq!(input[1]["content"][0]["text"], "The code is COBALT.");
    assert_eq!(input[2]["content"], "What was the code?");
}
#[tokio::test]
async fn completed_tool_groups_and_latest_followup_survive_new_turn() {
    let dir = TempDir::new();
    let call = json!({"status":"completed","output":[{"type":"function_call","call_id":"write","name":"write_file","arguments":"{\"path\":\"note.txt\",\"content\":\"COBALT\"}"}]});
    let stub = StubResponses::start(vec![call, reply("Saved note.txt"), reply("COBALT")]);
    let mut session = Session::new(test_config(dir.path(), stub.endpoint())).unwrap();
    session.turn("Save COBALT", |_| {}).await.unwrap();
    session.turn("What did you save?", |_| {}).await.unwrap();
    let sent = stub.requests();
    let input = sent[2]["input"].as_array().unwrap();
    assert_eq!(input[1]["type"], "function_call");
    assert_eq!(input[2]["type"], "function_call_output");
    assert_eq!(input[1]["call_id"], input[2]["call_id"]);
    assert_eq!(input.last().unwrap()["content"], "What did you save?");
}
#[tokio::test]
async fn failed_or_interrupted_session_requires_explicit_new_session() {
    let dir = TempDir::new();
    let stub =
        StubResponses::start(vec![json!({"status":"incomplete","output":[]}), reply("fresh")]);
    let mut session = Session::new(test_config(dir.path(), stub.endpoint())).unwrap();
    assert!(session.turn("first", |_| {}).await.is_err());
    assert!(session.turn("again", |_| {}).await.unwrap_err().to_string().contains("/new"));
    assert_eq!(stub.requests().len(), 1);
    let mut fresh = Session::new(test_config(dir.path(), stub.endpoint())).unwrap();
    fresh.turn("new task", |_| {}).await.unwrap();
    assert_eq!(stub.requests()[1]["input"].as_array().unwrap().len(), 1);
    fresh.interrupt().unwrap();
    assert!(fresh.turn("do not retry", |_| {}).await.is_err());
    assert_eq!(stub.requests().len(), 2);
}
