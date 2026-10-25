mod common;
use common::{StubResponses, fixture};
use crabloop::{Config, model};
use serde_json::{Value, json};

fn setup(server: &StubResponses) -> (reqwest::Client, Config) {
    (reqwest::Client::new(), common::test_config(std::path::Path::new("."), server.endpoint()))
}

async fn respond(server: &StubResponses, input: &[Value]) -> anyhow::Result<model::Reply> {
    let (client, config) = setup(server);
    model::respond(&client, &config, &model::request(&config, input)).await
}

#[tokio::test]
async fn sends_stateless_responses_request() {
    let server = StubResponses::start(vec![fixture("text")]);
    let reply = respond(&server, &[json!({"role":"user","content":"hello"})]).await.unwrap();
    assert_eq!(reply.final_text.as_deref(), Some("Done"));
    let sent = &server.requests()[0];
    assert_eq!(sent["store"], false);
    assert_eq!(sent["max_output_tokens"], 131_072);
    assert_eq!(sent["model"], "test-model");
    assert_eq!(sent["input"][0]["content"], "hello");
    assert_eq!(sent["include"], json!(["reasoning.encrypted_content"]));
    assert!(sent.get("previous_response_id").is_none());
    assert_eq!(server.authorizations()[0], "Bearer test-key");
}

#[tokio::test]
async fn distinguishes_call_id_from_item_id_and_preserves_reasoning() {
    let server = StubResponses::start(vec![fixture("reasoning"), fixture("two_calls")]);
    let first = respond(&server, &[]).await.unwrap();
    assert_eq!(first.calls[0].call_id, "call_1");
    assert_eq!(first.output[1]["id"], "fc_call_1");
    assert_eq!(first.output[0]["encrypted_content"], "opaque");
    assert!(first.final_text.is_none());
    let next = respond(&server, &first.output).await.unwrap();
    assert_eq!(next.calls[1].call_id, "call_2");
    assert_eq!(server.requests()[1]["input"][0]["encrypted_content"], "opaque");
}

#[tokio::test]
async fn commentary_is_preserved_and_not_final_but_phaseless_text_is() {
    let server = StubResponses::start(vec![fixture("commentary")]);
    let reply = respond(&server, &[]).await.unwrap();
    assert!(reply.final_text.is_none());
    assert_eq!(reply.output[0]["phase"], "commentary");
    let mut text = fixture("text");
    text["output"][0].as_object_mut().unwrap().remove("phase");
    let server = StubResponses::start(vec![text]);
    assert!(respond(&server, &[]).await.unwrap().final_text.is_some());
}

#[tokio::test]
async fn incomplete_missing_call_id_empty_output_and_http_500_are_errors() {
    let mut empty = fixture("text");
    empty["output"] = json!([]);
    for (status, response, expected) in [
        (200, fixture("incomplete"), "status is incomplete"),
        (200, fixture("bad_call_id"), "call_id"),
        (200, empty, "no output"),
        (500, json!({"error":{"message":"down"}}), "500"),
    ] {
        let server = StubResponses::with_status(status, vec![response]);
        let error = respond(&server, &[]).await.err().unwrap();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
        assert_eq!(server.requests().len(), 1);
    }
}

#[tokio::test]
async fn mixed_phases_keep_only_final_answer_text_and_every_item() {
    let mut reply = fixture("commentary");
    let answer = fixture("text")["output"][0].clone();
    reply["output"].as_array_mut().unwrap().push(answer);
    let server = StubResponses::start(vec![reply]);
    let reply = respond(&server, &[]).await.unwrap();
    assert_eq!(reply.final_text.as_deref(), Some("Done"));
    assert_eq!(reply.output.len(), 2);
}

#[tokio::test]
async fn empty_and_duplicate_call_ids_are_rejected() {
    let mut empty = fixture("two_calls");
    empty["output"][0]["call_id"] = json!("");
    let mut duplicate = fixture("two_calls");
    duplicate["output"][1]["call_id"] = json!("call_1");
    let mut accepted = Vec::new();
    for (name, response) in [("empty", empty), ("duplicate", duplicate)] {
        let server = StubResponses::start(vec![response]);
        if respond(&server, &[]).await.is_ok() {
            accepted.push(name);
        }
    }
    assert!(accepted.is_empty(), "accepted invalid call IDs: {accepted:?}");
}

#[tokio::test]
async fn only_assistant_output_text_can_be_a_final_answer() {
    for (role, kind) in [("user", "output_text"), ("assistant", "summary_text")] {
        let mut response = fixture("text");
        response["output"][0]["role"] = json!(role);
        response["output"][0]["content"][0]["type"] = json!(kind);
        let original = response["output"].as_array().unwrap().clone();
        let server = StubResponses::start(vec![response]);
        let reply = respond(&server, &[]).await.unwrap();
        assert!(reply.final_text.is_none(), "accepted {role}/{kind} as final text");
        assert_eq!(reply.output, original);
    }
}

#[tokio::test]
async fn output_text_requires_its_text_field() {
    let mut response = fixture("text");
    response["output"][0]["content"][0].as_object_mut().unwrap().remove("text");
    let server = StubResponses::start(vec![response]);
    assert!(respond(&server, &[]).await.is_err());
}
