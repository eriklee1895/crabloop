mod common;
use common::{TempDir, make_call, test_config};
use crabloop::{Config, tools};
use serde_json::{Value, json};
use std::fs;
use std::time::Duration;

fn config(dir: &TempDir) -> Config {
    test_config(dir.path(), "http://127.0.0.1:9/v1/responses")
}

async fn run(dir: &TempDir, name: &str, args: Value) -> Value {
    tools::execute(&make_call("call_1", name, args), &config(dir)).await
}

#[test]
fn tool_schemas_are_strict_string_objects() {
    let schemas = tools::schemas();
    assert!(schemas.iter().all(|s| s["strict"] == true));
    let edit = schemas.iter().find(|s| s["name"] == "edit_file").unwrap();
    assert_eq!(edit["type"], "function");
    assert_eq!(edit["strict"], true);
    assert_eq!(edit["parameters"]["required"], json!(["path", "old_text", "new_text"]));
    assert_eq!(edit["parameters"]["additionalProperties"], false);
    assert_eq!(edit["parameters"]["properties"]["path"]["type"], "string");
}

#[tokio::test]
async fn write_then_read_roundtrip() {
    let dir = TempDir::new();
    let written = run(&dir, "write_file", json!({"path": "hello.rs", "content": "你好 🦀"})).await;
    assert_eq!(written["ok"], true);
    assert_eq!(fs::read_to_string(dir.path().join("hello.rs")).unwrap(), "你好 🦀");
    let read = run(&dir, "read_file", json!({"path": "hello.rs"})).await;
    assert_eq!(read["ok"], true);
    assert_eq!(read["content"], "你好 🦀");
}

#[tokio::test]
async fn edit_unique_match_changes_only_the_requested_text() {
    let dir = TempDir::new();
    let path = dir.path().join("code.rs");
    fs::write(&path, "prefix OLD suffix").unwrap();
    let args = json!({"path": "code.rs", "old_text": "OLD", "new_text": "NEW"});
    assert_eq!(run(&dir, "edit_file", args).await["ok"], true);
    assert_eq!(fs::read_to_string(path).unwrap(), "prefix NEW suffix");
}

#[tokio::test]
async fn failed_edits_leave_the_original_file_unchanged() {
    let dir = TempDir::new();
    let path = dir.path().join("code.rs");
    for (before, old, error) in [
        ("same same", "same", "unique"),
        ("aaaa", "aaa", "unique"),
        ("before", "missing", "match"),
        ("before", "", "empty"),
    ] {
        fs::write(&path, before).unwrap();
        let args = json!({"path": "code.rs", "old_text": old, "new_text": "new"});
        let result = run(&dir, "edit_file", args).await;
        assert_eq!(result["ok"], false);
        assert!(result["error"].as_str().unwrap().contains(error), "{result}");
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
    }
}

#[tokio::test]
async fn size_limit_is_checked_before_decoding_and_before_writing() {
    let dir = TempDir::new();
    let large = "你".repeat(30_000);
    fs::write(dir.path().join("large.txt"), &large).unwrap();
    let read = run(&dir, "read_file", json!({"path": "large.txt"})).await;
    assert!(read["error"].as_str().unwrap().contains("size"), "{read}");
    fs::write(dir.path().join("small.txt"), "small").unwrap();
    let write = run(&dir, "write_file", json!({"path": "small.txt", "content": large})).await;
    assert_eq!(write["ok"], false);
    assert_eq!(fs::read_to_string(dir.path().join("small.txt")).unwrap(), "small");
}

#[tokio::test]
async fn read_truncation_is_utf8_safe_and_non_utf8_is_an_error() {
    let dir = TempDir::new();
    fs::write(dir.path().join("text"), format!("{}🦀", "a".repeat(8 * 1024 - 1))).unwrap();
    let read = run(&dir, "read_file", json!({"path": "text"})).await;
    assert_eq!(read["content"].as_str().unwrap().len(), 8 * 1024 - 1);
    assert_eq!(read["truncated"], true);
    assert_eq!(tools::clip("你好🦀", 7), ("你好", true));
    fs::write(dir.path().join("binary"), [255u8, 0]).unwrap();
    assert_eq!(run(&dir, "read_file", json!({"path": "binary"})).await["ok"], false);
}

#[tokio::test]
async fn bad_arguments_paths_and_unknown_tools_become_tool_errors() {
    let dir = TempDir::new();
    for (name, args) in [
        ("read_file", json!({"path": 42})),
        ("read_file", json!({"path": "../outside"})),
        ("write_file", json!({"path": "/absolute", "content": "x"})),
        ("read_file", json!({"path": "missing.rs"})),
        ("unknown", json!({})),
    ] {
        let result = run(&dir, name, args).await;
        assert_eq!(result["ok"], false);
        assert!(!result["error"].as_str().unwrap().is_empty());
    }
    let mut invalid = make_call("call_1", "read_file", json!({}));
    invalid.arguments = "{".into();
    assert_eq!(tools::execute(&invalid, &config(&dir)).await["ok"], false);
}

#[tokio::test]
async fn write_file_rejects_a_fifo_without_blocking() {
    let dir = TempDir::new();
    let fifo = dir.path().join("pipe");
    assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
    let result = run(&dir, "write_file", json!({"path": "pipe", "content": "x"})).await;
    assert!(result["error"].as_str().unwrap().contains("regular file"), "{result}");
}

#[tokio::test]
async fn bash_reports_exit_code_and_both_streams() {
    let dir = TempDir::new();
    let result =
        run(&dir, "bash", json!({"command": "printf ok; printf problem >&2; exit 7"})).await;
    assert_eq!(result["ok"], false);
    assert_eq!(result["exit_code"], 7);
    assert_eq!(result["stdout"], "ok");
    assert_eq!(result["stderr"], "problem");
    assert_eq!(result["timed_out"], false);
}

#[tokio::test]
async fn bash_runs_in_the_workspace_with_closed_stdin_and_no_api_key() {
    let dir = TempDir::new();
    let command = "cat; pwd; printenv OPENAI_API_KEY || echo no-key";
    let result = run(&dir, "bash", json!({"command": command})).await;
    let stdout = result["stdout"].as_str().unwrap();
    let (cwd, key) = stdout.trim().split_once('\n').unwrap();
    assert_eq!(fs::canonicalize(cwd).unwrap(), fs::canonicalize(dir.path()).unwrap());
    assert_eq!(key, "no-key");
}

#[tokio::test]
async fn bash_bounds_both_pipes_without_blocking_the_process() {
    let dir = TempDir::new();
    let command = "printf '%100000s' x; printf '%100000s' y >&2";
    let result = run(&dir, "bash", json!({"command": command})).await;
    assert_eq!(result["ok"], true);
    assert_eq!(result["truncated"], true);
    assert_eq!(result["stdout"].as_str().unwrap().len(), 4096);
    assert_eq!(result["stderr"].as_str().unwrap().len(), 4096);
}

#[tokio::test]
async fn bash_timeout_reaps_direct_child_and_keeps_partial_output() {
    let dir = TempDir::new();
    let config = Config { bash_timeout: Duration::from_millis(100), ..config(&dir) };
    let command = "echo $$ > timeout.pid; printf started; exec sleep 10";
    let result =
        tools::execute(&make_call("call_1", "bash", json!({"command": command})), &config).await;
    assert_eq!(result["ok"], false);
    assert_eq!(result["timed_out"], true);
    assert_eq!(result["stdout"], "started");
    let pid = fs::read_to_string(dir.path().join("timeout.pid")).unwrap();
    let alive = std::process::Command::new("kill")
        .args(["-0", pid.trim()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(!alive.success());
}

#[tokio::test]
async fn bash_timeout_also_bounds_pipes_held_after_parent_exit() {
    let dir = TempDir::new();
    let config = Config { bash_timeout: Duration::from_millis(100), ..config(&dir) };
    let started = std::time::Instant::now();
    let call = make_call("call_1", "bash", json!({"command": "sleep 0.5 & exit 0"}));
    let result = tools::execute(&call, &config).await;
    assert_eq!(result["timed_out"], true);
    assert_eq!(result["exit_code"], 0);
    assert!(started.elapsed() < Duration::from_secs(2));
}
