mod common;
use common::{StubResponses, TempDir, fixture, test_config};
use crabloop::{Config, Stop, run_turn};
use serde_json::{Value, json};
use std::fs;

/// One user task with a fresh history.
async fn run(config: Config) -> anyhow::Result<Stop> {
    let mut input = vec![json!({"role": "user", "content": "Repair the bug"})];
    run_turn(&config, &mut input).await
}

/// A completed response that asks for exactly one tool call.
fn call_reply(id: &str, name: &str, args: Value) -> Value {
    let mut reply = fixture("text");
    reply["output"] = json!([{
        "type": "function_call", "id": format!("fc_{id}"), "call_id": id,
        "name": name, "arguments": args.to_string(), "status": "completed"
    }]);
    reply
}

fn done() -> Stop {
    Stop::Final("Done".into())
}

#[tokio::test]
async fn two_calls_then_final_preserves_ids_and_returns_both_results() {
    let dir = TempDir::new();
    fs::write(dir.path().join("lib.rs"), "A").unwrap();
    fs::write(dir.path().join("tests.rs"), "B").unwrap();
    let server = StubResponses::start(vec![fixture("two_calls"), fixture("text")]);
    let stop = run(test_config(dir.path(), server.endpoint())).await.unwrap();
    assert_eq!(stop, done());
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["tools"].as_array().unwrap().len(), 4);
    let next = requests[1]["input"].as_array().unwrap();
    assert_eq!(next.len(), 5);
    assert_eq!(next[1]["id"], "fc_call_1");
    for (index, id, content) in [(3, "call_1", "A"), (4, "call_2", "B")] {
        assert_eq!(next[index]["call_id"], id);
        let result: Value = serde_json::from_str(next[index]["output"].as_str().unwrap()).unwrap();
        assert_eq!(result["content"], content);
    }
}

#[tokio::test]
async fn tool_error_then_corrected_call_receives_actionable_feedback() {
    let dir = TempDir::new();
    fs::write(dir.path().join("lib.rs"), "OLD").unwrap();
    let server = StubResponses::start(vec![
        call_reply(
            "bad",
            "edit_file",
            json!({"path":"lib.rs","old_text":"missing","new_text":"NEW"}),
        ),
        call_reply("good", "edit_file", json!({"path":"lib.rs","old_text":"OLD","new_text":"NEW"})),
        fixture("text"),
    ]);
    assert_eq!(run(test_config(dir.path(), server.endpoint())).await.unwrap(), done());
    assert_eq!(fs::read_to_string(dir.path().join("lib.rs")).unwrap(), "NEW");
    let sent = server.requests();
    let error: Value =
        serde_json::from_str(sent[1]["input"][2]["output"].as_str().unwrap()).unwrap();
    assert_eq!(error["ok"], false);
    assert!(error["error"].as_str().unwrap().contains("read the file again"));
}

#[tokio::test]
async fn incomplete_never_mutates_the_workspace() {
    let dir = TempDir::new();
    let path = dir.path().join("lib.rs");
    fs::write(&path, "ORIGINAL").unwrap();
    let mut r = call_reply("write", "write_file", json!({"path":"lib.rs","content":"CORRUPTED"}));
    r["status"] = json!("incomplete");
    let server = StubResponses::start(vec![r]);
    assert!(run(test_config(dir.path(), server.endpoint())).await.is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "ORIGINAL");
    assert_eq!(server.requests().len(), 1);
}

#[tokio::test]
async fn commentary_then_final_and_reasoning_are_replayed() {
    let dir = TempDir::new();
    fs::write(dir.path().join("lib.rs"), "code").unwrap();
    let server =
        StubResponses::start(vec![fixture("reasoning"), fixture("commentary"), fixture("text")]);
    let stop = run(test_config(dir.path(), server.endpoint())).await.unwrap();
    assert_eq!(stop, done());
    assert_eq!(server.requests().len(), 3);
    let sent = server.requests();
    assert_eq!(sent[1]["input"][1]["encrypted_content"], "opaque");
    assert!(sent[2]["input"].as_array().unwrap().iter().any(|i| i["phase"] == "commentary"));
}

#[tokio::test]
async fn max_turns_stops_before_an_extra_model_request() {
    let dir = TempDir::new();
    let server =
        StubResponses::start(vec![fixture("commentary"), fixture("commentary"), fixture("text")]);
    let mut config = test_config(dir.path(), server.endpoint());
    config.max_turns = 2;
    assert_eq!(run(config).await.unwrap(), Stop::MaxTurns);
    assert_eq!(server.requests().len(), 2);
}

#[tokio::test]
async fn request_budget_stops_before_network_access() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![]);
    let mut config = test_config(dir.path(), server.endpoint());
    config.request_bytes = 1;
    assert_eq!(run(config).await.unwrap(), Stop::ContextLimit);
    assert_eq!(server.requests().len(), 0);
}

#[tokio::test]
async fn scripted_bugfix_loop_uses_all_tools_and_passes_external_verifier() {
    let dir = TempDir::new();
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/buggy-rust");
    for name in ["lib.rs", "tests.rs"] {
        fs::copy(fixture_dir.join(name), dir.path().join(name)).unwrap();
    }
    let test = "rustc --edition=2024 --test tests.rs -o test-bin && ./test-bin";
    let server = StubResponses::start(vec![
        call_reply("read", "read_file", json!({"path":"lib.rs"})),
        call_reply("before", "bash", json!({"command":test})),
        call_reply(
            "note",
            "write_file",
            json!({"path":"NOTES.md","content":"Repair checked subtraction; test before and after."}),
        ),
        call_reply(
            "edit",
            "edit_file",
            json!({"path":"lib.rs","old_text":"Some(len)","new_text":"len.checked_sub(1)"}),
        ),
        call_reply("after", "bash", json!({"command":test})),
        fixture("text"),
    ]);
    let stop = run(test_config(dir.path(), server.endpoint())).await.unwrap();
    assert_eq!(stop, done());
    assert_eq!(server.requests().len(), 6);
    let requests = server.requests();
    let results: Vec<Value> = requests[5]["input"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["type"] == "function_call_output")
        .map(|i| serde_json::from_str(i["output"].as_str().unwrap()).unwrap())
        .collect();
    assert_eq!(results[1]["ok"], false);
    assert!(results[1]["stdout"].as_str().unwrap().contains("FAILED"));
    assert_eq!(results[4]["ok"], true);
    assert!(results[4]["stdout"].as_str().unwrap().contains("test result: ok"));
    assert_eq!(
        fs::read(dir.path().join("tests.rs")).unwrap(),
        fs::read(fixture_dir.join("tests.rs")).unwrap()
    );
    let verified = std::process::Command::new("bash")
        .arg(format!("{}/scripts/verify-playground.sh", env!("CARGO_MANIFEST_DIR")))
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(verified.status.success(), "{}", String::from_utf8_lossy(&verified.stderr));
}

#[tokio::test]
async fn followup_replays_previous_answer_and_complete_tool_results() {
    let dir = TempDir::new();
    fs::write(dir.path().join("lib.rs"), "A").unwrap();
    fs::write(dir.path().join("tests.rs"), "B").unwrap();
    let server = StubResponses::start(vec![fixture("two_calls"), fixture("text"), fixture("text")]);
    let config = test_config(dir.path(), server.endpoint());
    let mut input = vec![json!({"role":"user","content":"Read the files"})];
    assert_eq!(run_turn(&config, &mut input).await.unwrap(), done());
    assert_eq!(server.requests().len(), 2);
    input.push(json!({"role":"user","content":"Which files did you read?"}));
    let expected = input.clone();
    run_turn(&config, &mut input).await.unwrap();
    assert_eq!(server.requests()[2]["input"], json!(expected));
    assert_eq!(expected[5]["content"][0]["text"], "Done");
    assert_eq!(expected[6]["content"], "Which files did you read?");
    assert_eq!(expected[3]["call_id"], "call_1");
    let result: Value = serde_json::from_str(expected[3]["output"].as_str().unwrap()).unwrap();
    assert_eq!(result["content"], "A");
}

#[tokio::test]
async fn duplicate_call_ids_never_mutate_the_workspace() {
    let dir = TempDir::new();
    let path = dir.path().join("lib.rs");
    fs::write(&path, "ORIGINAL").unwrap();
    let mut response =
        call_reply("shared", "write_file", json!({"path":"lib.rs","content":"FIRST"}));
    let second = call_reply("shared", "write_file", json!({"path":"lib.rs","content":"SECOND"}));
    response["output"].as_array_mut().unwrap().push(second["output"][0].clone());
    let server = StubResponses::start(vec![response]);
    assert!(run(test_config(dir.path(), server.endpoint())).await.is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "ORIGINAL");
    assert_eq!(server.requests().len(), 1);
}
