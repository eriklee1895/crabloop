mod common;
use common::{StubResponses, fixture};
use crabloop::{Config, model};
use serde_json::json;

#[tokio::test]
async fn sends_stateless_responses_request() {
    let server = StubResponses::start(vec![fixture("text")]);
    let (client, config) = setup(&server);
    let body = model::request(
        &config,
        "Inspect carefully",
        &[json!({"role":"user","content":"hello"})],
        &[],
    );
    let reply = model::respond(&client, &config, body).await.unwrap();
    assert_eq!(reply.text, "Done");
    assert!(reply.is_final);
    let sent = &server.requests()[0];
    assert_eq!(sent["store"], false);
    assert_eq!(sent["model"], "test-model");
    assert_eq!(sent["instructions"], "Inspect carefully");
    assert_eq!(sent["input"][0]["content"], "hello");
    assert_eq!(sent["include"], json!(["reasoning.encrypted_content"]));
    assert!(sent.get("previous_response_id").is_none());
    assert!(sent.get("conversation").is_none());
    assert!(!sent.to_string().contains("test-key"));
}
#[tokio::test]
async fn distinguishes_call_id_from_item_id_and_preserves_reasoning() {
    let server = StubResponses::start(vec![fixture("reasoning"), fixture("two_calls")]);
    let (client, config) = setup(&server);
    let first =
        model::respond(&client, &config, model::request(&config, "", &[], &[])).await.unwrap();
    assert_eq!(first.calls[0].call_id, "call_1");
    assert_eq!(first.output[1]["id"], "fc_call_1");
    assert_eq!(first.output[0]["encrypted_content"], "opaque");
    assert!(!first.is_final);
    let next = model::respond(&client, &config, model::request(&config, "", &first.output, &[]))
        .await
        .unwrap();
    assert_eq!(next.calls.len(), 2);
    assert_eq!(next.calls[1].call_id, "call_2");
    assert_eq!(server.requests()[1]["input"][0]["encrypted_content"], "opaque");
}
#[tokio::test]
async fn commentary_is_preserved_and_not_final() {
    let server = StubResponses::start(vec![fixture("commentary")]);
    let (client, config) = setup(&server);
    let reply =
        model::respond(&client, &config, model::request(&config, "", &[], &[])).await.unwrap();
    assert!(!reply.is_final);
    assert_eq!(reply.output[0]["phase"], "commentary");
}
#[tokio::test]
async fn incomplete_and_missing_or_duplicate_call_ids_are_rejected() {
    let mut duplicate = fixture("two_calls");
    duplicate["output"][1]["call_id"] = json!("call_1");
    for (response, expected) in [
        (fixture("incomplete"), "completed"),
        (fixture("bad_call_id"), "call_id"),
        (duplicate, "call_id"),
    ] {
        let server = StubResponses::start(vec![response]);
        let (client, config) = setup(&server);
        let error = model::respond(&client, &config, model::request(&config, "", &[], &[]))
            .await
            .unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
    }
}
#[tokio::test]
async fn empty_output_is_a_protocol_error_and_http_500_is_not_retried() {
    let mut empty = fixture("text");
    empty["output"] = json!([]);
    for (status, response, expected) in
        [(200, empty, "output"), (500, json!({"error":{"message":"upstream unavailable"}}), "500")]
    {
        let server = StubResponses::with_status(status, vec![response]);
        let (client, config) = setup(&server);
        let error = model::respond(&client, &config, model::request(&config, "", &[], &[]))
            .await
            .unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
        assert_eq!(server.requests().len(), 1);
    }
}
#[tokio::test]
async fn final_text_without_phase_remains_supported() {
    let mut reply = fixture("text");
    reply["output"][0].as_object_mut().unwrap().remove("phase");
    let server = StubResponses::start(vec![reply]);
    let (client, config) = setup(&server);
    assert!(
        model::respond(&client, &config, model::request(&config, "", &[], &[]))
            .await
            .unwrap()
            .is_final
    );
}

#[test]
fn cli_validates_configuration_before_network_access() {
    use std::process::Command;
    let exe = env!("CARGO_BIN_EXE_crabloop");
    let invalid = Command::new(exe).arg("--bad").output().unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("usage"));
    let dir = common::TempDir::new();
    let absent = Command::new(exe)
        .current_dir(dir.path())
        .arg("--workspace")
        .arg(dir.path())
        .arg("hello")
        .env_remove("OPENAI_API_KEY")
        .env("CRABLOOP_MODEL", "test-model")
        .output()
        .unwrap();
    assert!(!absent.status.success());
    assert!(String::from_utf8_lossy(&absent.stderr).contains("OPENAI_API_KEY"));
}
#[test]
fn cli_uses_the_requested_workspace_task_and_model() {
    let dir = common::TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_crabloop"))
        .current_dir(dir.path())
        .arg("--workspace")
        .arg(dir.path())
        .arg("inspect this project")
        .env("OPENAI_API_KEY", "test-key")
        .env("CRABLOOP_MODEL", "test-model")
        .env("CRABLOOP_API_URL", server.endpoint())
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Done"));
    assert_eq!(server.requests()[0]["input"][0]["content"], "inspect this project");
}

#[test]
fn cli_honors_openai_base_url_without_full_endpoint_override() {
    let dir = common::TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_crabloop"))
        .current_dir(dir.path())
        .arg("--workspace")
        .arg(dir.path())
        .arg("base prefix")
        .env("OPENAI_API_KEY", "test-key")
        .env("CRABLOOP_MODEL", "test-model")
        .env_remove("CRABLOOP_API_URL")
        .env("OPENAI_BASE_URL", server.endpoint().trim_end_matches("/responses"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(server.requests()[0]["input"][0]["content"], "base prefix");
}

fn setup(server: &StubResponses) -> (reqwest::Client, Config) {
    (reqwest::Client::new(), common::test_config(std::path::Path::new("."), server.endpoint()))
}
