mod common;
use common::{TempDir, make_call};
use crabloop::{
    Limits,
    tools::{clip_utf8, execute, schemas},
};
use serde_json::json;
use std::fs;

#[tokio::test]
async fn read_write_roundtrip_and_output_correlation() {
    let dir = TempDir::new();
    let limits = Limits::default();
    let written = execute(
        &make_call("write_1", "write_file", json!({"path":"hello.rs","content":"Hello 🦀"})),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(written["ok"], true);
    assert_eq!(fs::read_to_string(dir.path().join("hello.rs")).unwrap(), "Hello 🦀");
    let read =
        execute(&make_call("read_1", "read_file", json!({"path":"hello.rs"})), dir.path(), &limits)
            .await;
    assert_eq!(read["content"], "Hello 🦀");
    assert_eq!(read["ok"], true);
}
#[tokio::test]
async fn edit_unique_match_changes_only_the_requested_text() {
    let dir = TempDir::new();
    let path = dir.path().join("code.rs");
    fs::write(&path, "prefix OLD suffix").unwrap();
    let result = execute(
        &make_call(
            "edit_1",
            "edit_file",
            json!({"path":"code.rs","old_text":"OLD","new_text":"NEW"}),
        ),
        dir.path(),
        &Limits::default(),
    )
    .await;
    assert_eq!(result["ok"], true);
    assert_eq!(fs::read_to_string(path).unwrap(), "prefix NEW suffix");
}
#[tokio::test]
async fn failed_edits_leave_the_original_file_unchanged() {
    let dir = TempDir::new();
    let path = dir.path().join("code.rs");
    for (before, old, new, error) in [
        ("same same", "same", "new", "unique"),
        ("aaaa", "aaa", "x", "unique"),
        ("before", "missing", "new", "match"),
        ("before", "", "new", "empty"),
    ] {
        fs::write(&path, before).unwrap();
        let r = execute(
            &make_call(
                "edit_1",
                "edit_file",
                json!({"path":"code.rs","old_text":old,"new_text":new}),
            ),
            dir.path(),
            &Limits::default(),
        )
        .await;
        assert_eq!(r["ok"], false);
        assert!(r["error"].as_str().unwrap().contains(error), "{}", r);
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
    }
}
#[tokio::test]
async fn oversized_files_and_edits_are_rejected_without_mutation() {
    let dir = TempDir::new();
    let path = dir.path().join("code.rs");
    fs::write(&path, "12345678").unwrap();
    let limits = Limits { file_bytes: 8, ..Limits::default() };
    let r = execute(
        &make_call(
            "edit_1",
            "edit_file",
            json!({"path":"code.rs","old_text":"1","new_text":"TOO-LONG"}),
        ),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(r["ok"], false);
    assert_eq!(fs::read_to_string(&path).unwrap(), "12345678");
    let r = execute(
        &make_call("write_1", "write_file", json!({"path":"code.rs","content":"123456789"})),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(r["ok"], false);
    assert_eq!(fs::read_to_string(&path).unwrap(), "12345678");
    fs::write(&path, "123456789").unwrap();
    assert_eq!(
        execute(&make_call("r", "read_file", json!({"path":"code.rs"})), dir.path(), &limits).await
            ["ok"],
        false
    );
}
#[tokio::test]
async fn invalid_arguments_paths_and_unknown_tools_return_correlated_errors() {
    let dir = TempDir::new();
    let cases = [
        make_call("a", "read_file", json!({"path":42})),
        make_call("b", "read_file", json!({"path":"../outside"})),
        make_call("c", "write_file", json!({"path":"/absolute","content":"x"})),
        make_call("d", "unknown", json!({})),
        make_call("e", "read_file", json!({"path":"missing.rs"})),
    ];
    for call in cases {
        let r = execute(&call, dir.path(), &Limits::default()).await;
        assert_eq!(r["ok"], false);
        assert!(r["error"].as_str().unwrap().len() > 4);
    }
    let mut invalid = make_call("f", "read_file", json!({}));
    invalid.arguments = "{".into();
    assert_eq!(execute(&invalid, dir.path(), &Limits::default()).await["ok"], false);
}
#[tokio::test]
async fn read_truncation_is_utf8_safe_and_non_utf8_is_an_error() {
    let dir = TempDir::new();
    fs::write(dir.path().join("text"), "café🦀").unwrap();
    let limits = Limits { read_output_bytes: 5, ..Limits::default() };
    let result =
        execute(&make_call("r", "read_file", json!({"path":"text"})), dir.path(), &limits).await;
    assert_eq!(result["content"], "café");
    assert_eq!(result["truncated"], true);
    assert_eq!(clip_utf8("🦀", 3), (String::new(), true));
    fs::write(dir.path().join("binary"), [255u8, 0]).unwrap();
    assert_eq!(
        execute(&make_call("r", "read_file", json!({"path":"binary"})), dir.path(), &limits).await
            ["ok"],
        false
    );
}
#[test]
fn tool_schemas_have_required_string_arguments_and_strict_objects() {
    let defs = schemas();
    assert_eq!(defs.len(), 4);
    let edit = defs.iter().find(|d| d["name"] == "edit_file").unwrap();
    assert_eq!(edit["type"], "function");
    assert_eq!(edit["strict"], true);
    assert_eq!(edit["parameters"]["required"], json!(["path", "old_text", "new_text"]));
    assert_eq!(edit["parameters"]["additionalProperties"], false);
    assert_eq!(edit["parameters"]["properties"]["path"]["type"], "string");
}

#[tokio::test]
async fn oversized_utf8_reports_size_instead_of_encoding() {
    let dir = TempDir::new();
    fs::write(dir.path().join("large"), "éééé").unwrap();
    let limits = Limits { file_bytes: 7, ..Limits::default() };
    let r = execute(&make_call("read", "read_file", json!({"path":"large"})), dir.path(), &limits)
        .await;
    assert!(r["error"].as_str().unwrap().contains("size"), "{}", r);
}

#[tokio::test]
async fn bash_reports_exit_and_stderr() {
    let dir = TempDir::new();
    let r = execute(
        &make_call("b", "bash", json!({"command":"printf ok; printf problem >&2; exit 7"})),
        dir.path(),
        &Limits::default(),
    )
    .await;
    assert_eq!(r["ok"], false);
    assert_eq!(r["exit_code"], 7);
    assert_eq!(r["stdout"], "ok");
    assert_eq!(r["stderr"], "problem");
    assert_eq!(r["timed_out"], false);
}
#[tokio::test]
async fn bash_respects_workspace_and_closes_stdin() {
    let dir = TempDir::new();
    let r = execute(
        &make_call("b", "bash", json!({"command":"cat; pwd"})),
        dir.path(),
        &Limits::default(),
    )
    .await;
    assert_eq!(r["ok"], true);
    assert_eq!(
        fs::canonicalize(r["stdout"].as_str().unwrap().trim()).unwrap(),
        fs::canonicalize(dir.path()).unwrap()
    );
}
#[tokio::test]
async fn bash_bounds_both_pipes_without_blocking_the_process() {
    let dir = TempDir::new();
    let r = execute(
        &make_call("b", "bash", json!({"command":"printf '%100000s' x; printf '%100000s' y >&2"})),
        dir.path(),
        &Limits::default(),
    )
    .await;
    assert_eq!(r["ok"], true);
    assert_eq!(r["truncated"], true);
    assert_eq!(r["stdout"].as_str().unwrap().len(), 4096);
    assert_eq!(r["stderr"].as_str().unwrap().len(), 4096);
}
#[tokio::test]
async fn bash_timeout_reaps_direct_child() {
    let dir = TempDir::new();
    let limits =
        Limits { bash_timeout: std::time::Duration::from_millis(100), ..Limits::default() };
    let r = execute(
        &make_call(
            "b",
            "bash",
            json!({"command":"echo $$ > timeout.pid; printf started; exec sleep 10"}),
        ),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(r["ok"], false);
    assert_eq!(r["timed_out"], true);
    assert_eq!(r["stdout"], "started");
    let pid = fs::read_to_string(dir.path().join("timeout.pid")).unwrap();
    assert!(
        !std::process::Command::new("kill")
            .args(["-0", pid.trim()])
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success()
    );
}

#[tokio::test]
async fn bash_bounds_pipe_wait_after_parent_exit() {
    let dir = TempDir::new();
    let limits =
        Limits { bash_timeout: std::time::Duration::from_millis(100), ..Limits::default() };
    let start = std::time::Instant::now();
    let result = execute(
        &make_call("b", "bash", json!({"command":"sleep 0.5 & exit 0"})),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(result["timed_out"], true);
    assert_eq!(result["exit_code"], 0);
    assert!(start.elapsed() < std::time::Duration::from_secs(2));
}
