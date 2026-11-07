mod common;
use common::{StubResponses, TempDir, fixture, task_output};
use serde_json::json;
use std::process::Command;

fn crabloop(dir: &TempDir, server: &StubResponses) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_crabloop"));
    command
        .current_dir(dir.path())
        .env("OPENAI_API_KEY", "test-key")
        .env("OPENAI_BASE_URL", server.endpoint().trim_end_matches("/responses"))
        .env("CRABLOOP_MODEL", "test-model");
    command
}

#[test]
fn missing_key_is_reported_before_any_request() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![]);
    let output = task_output(crabloop(&dir, &server).env_remove("OPENAI_API_KEY"), "hello\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("OPENAI_API_KEY"));
    assert!(server.requests().is_empty());
}

#[test]
fn a_stdin_line_runs_tools_in_the_startup_directory() {
    let dir = TempDir::new();
    std::fs::write(dir.path().join("note.txt"), "from cwd").unwrap();
    let mut call = fixture("text");
    call["output"] = json!([{
        "type": "function_call", "call_id": "call_1", "name": "read_file",
        "arguments": "{\"path\":\"note.txt\"}"
    }]);
    let server = StubResponses::start(vec![call, fixture("text")]);
    let output = task_output(&mut crabloop(&dir, &server), "read note.txt\n");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Done\n");
    let result = &server.requests()[1]["input"][2];
    assert_eq!(result["call_id"], "call_1");
    assert!(result["output"].as_str().unwrap().contains("from cwd"));
}

#[test]
fn each_stdin_line_is_a_task_and_history_is_kept() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text"), fixture("text")]);
    let output = task_output(&mut crabloop(&dir, &server), "first\n\nsecond\n");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Done\nDone\n");
    let second = &server.requests()[1]["input"];
    assert_eq!(second[0]["content"], "first");
    assert_eq!(second[1]["content"][0]["text"], "Done");
    assert_eq!(second[2]["content"], "second");
}
