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
        &make_call(
            "write_1",
            "write_file",
            json!({"path":"hello.rs","content":"你好 🦀"}),
        ),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(written.output["ok"], true);
    assert_eq!(
        fs::read_to_string(dir.path().join("hello.rs")).unwrap(),
        "你好 🦀"
    );
    let read = execute(
        &make_call("read_1", "read_file", json!({"path":"hello.rs"})),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(read.output["content"], "你好 🦀");
    assert_eq!(read.as_input()["type"], "function_call_output");
    assert_eq!(read.as_input()["call_id"], "read_1");
    let decoded: serde_json::Value =
        serde_json::from_str(read.as_input()["output"].as_str().unwrap()).unwrap();
    assert_eq!(decoded["ok"], true);
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
    assert_eq!(result.output["ok"], true);
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
        assert_eq!(r.output["ok"], false);
        assert!(
            r.output["error"].as_str().unwrap().contains(error),
            "{}",
            r.output
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
    }
}
#[tokio::test]
async fn oversized_files_and_edits_are_rejected_without_mutation() {
    let dir = TempDir::new();
    let path = dir.path().join("code.rs");
    fs::write(&path, "12345678").unwrap();
    let limits = Limits {
        file_bytes: 8,
        ..Limits::default()
    };
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
    assert_eq!(r.output["ok"], false);
    assert_eq!(fs::read_to_string(&path).unwrap(), "12345678");
    let r = execute(
        &make_call(
            "write_1",
            "write_file",
            json!({"path":"code.rs","content":"123456789"}),
        ),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(r.output["ok"], false);
    assert_eq!(fs::read_to_string(&path).unwrap(), "12345678");
    fs::write(&path, "123456789").unwrap();
    assert_eq!(
        execute(
            &make_call("r", "read_file", json!({"path":"code.rs"})),
            dir.path(),
            &limits
        )
        .await
        .output["ok"],
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
        assert_eq!(r.output["ok"], false);
        assert_eq!(r.as_input()["call_id"], call.call_id);
        assert!(r.output["error"].as_str().unwrap().len() > 4);
    }
    let mut invalid = make_call("f", "read_file", json!({}));
    invalid.arguments = "{".into();
    assert_eq!(
        execute(&invalid, dir.path(), &Limits::default())
            .await
            .output["ok"],
        false
    );
}
#[tokio::test]
async fn read_truncation_is_utf8_safe_and_non_utf8_is_an_error() {
    let dir = TempDir::new();
    fs::write(dir.path().join("text"), "你好🦀").unwrap();
    let limits = Limits {
        read_output_bytes: 7,
        ..Limits::default()
    };
    let result = execute(
        &make_call("r", "read_file", json!({"path":"text"})),
        dir.path(),
        &limits,
    )
    .await;
    assert_eq!(result.output["content"], "你好");
    assert_eq!(result.output["truncated"], true);
    assert_eq!(clip_utf8("🦀", 3), (String::new(), true));
    fs::write(dir.path().join("binary"), [255u8, 0]).unwrap();
    assert_eq!(
        execute(
            &make_call("r", "read_file", json!({"path":"binary"})),
            dir.path(),
            &limits
        )
        .await
        .output["ok"],
        false
    );
}
#[test]
fn tool_schemas_have_required_string_arguments_and_strict_objects() {
    let defs = schemas();
    assert_eq!(defs.len(), 3);
    let edit = defs.iter().find(|d| d["name"] == "edit_file").unwrap();
    assert_eq!(edit["type"], "function");
    assert_eq!(edit["strict"], true);
    assert_eq!(
        edit["parameters"]["required"],
        json!(["path", "old_text", "new_text"])
    );
    assert_eq!(edit["parameters"]["additionalProperties"], false);
    assert_eq!(edit["parameters"]["properties"]["path"]["type"], "string");
}

#[tokio::test]
async fn oversized_utf8_reports_size_instead_of_encoding() {
    let dir = TempDir::new();
    fs::write(dir.path().join("large"), "你你好").unwrap();
    let limits = Limits {
        file_bytes: 7,
        ..Limits::default()
    };
    let r = execute(
        &make_call("read", "read_file", json!({"path":"large"})),
        dir.path(),
        &limits,
    )
    .await;
    assert!(
        r.output["error"].as_str().unwrap().contains("size"),
        "{}",
        r.output
    );
}
