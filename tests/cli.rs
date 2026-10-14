mod common;
use common::{StubResponses, TempDir, fixture};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

fn command(dir: &TempDir) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_crabloop"));
    command.current_dir(dir.path()).env_clear().env("PATH", std::env::var_os("PATH").unwrap());
    command
}
fn configured(dir: &TempDir, server: &StubResponses) -> Command {
    let mut command = command(dir);
    command
        .env("OPENAI_API_KEY", "test-key")
        .env("CRABLOOP_MODEL", "test-model")
        .env("CRABLOOP_API_URL", server.endpoint());
    command
}
#[test]
fn no_arguments_reads_a_task_from_stdin_in_current_directory() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    let mut child = configured(&dir, &server)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all("请检查这个目录\n并说明结果\n".as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(server.requests()[0]["input"][0]["content"], "请检查这个目录\n并说明结果");
    assert!(dir.path().join(".crabloop").is_dir());
}
#[test]
fn task_argument_defaults_to_current_workspace() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    let output = configured(&dir, &server).arg("检查 README").output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(server.requests()[0]["input"][0]["content"], "检查 README");
}
#[test]
fn help_needs_no_credentials_and_eof_does_not_call_model() {
    let dir = TempDir::new();
    fs::write(dir.path().join(".env"), "bad = \"not closed").unwrap();
    let help = command(&dir).arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--workspace"));
    fs::remove_file(dir.path().join(".env")).unwrap();
    let server = StubResponses::start(vec![]);
    let empty = configured(&dir, &server).stdin(Stdio::null()).output().unwrap();
    assert!(empty.status.success());
    assert!(server.requests().is_empty());
    assert!(!dir.path().join(".crabloop").exists());
}
#[test]
fn dotenv_is_loaded_as_data_and_process_environment_wins() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    fs::write(dir.path().join(".env"), format!("export OPENAI_API_KEY='file-key'\nCRABLOOP_MODEL=from-file\nCRABLOOP_API_URL={}\nIGNORED='$(touch unexpected)'\n", server.endpoint())).unwrap();
    let output =
        command(&dir).env("CRABLOOP_MODEL", "override-model").arg("hello").output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(server.requests()[0]["model"], "override-model");
    assert_eq!(server.authorizations(), vec!["Bearer file-key"]);
    assert!(!dir.path().join("unexpected").exists());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("file-key"));
}
#[test]
fn existing_deepseek_key_is_enough_for_default_startup() {
    let dir = TempDir::new();
    fs::write(dir.path().join(".env"), "DEEPSEEK_API_KEY=private-deepseek-key\nOPENAI_API_KEY=private-proxy-key\nOPENAI_BASE_URL=https://proxy.invalid/v1\n").unwrap();
    let output = command(&dir).stdin(Stdio::null()).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("deepseek-flash"));
    assert!(!stderr.contains("private-"));
}
#[test]
fn explicit_provider_uses_its_own_key_and_default_model() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    fs::write(dir.path().join(".env"), format!("CRABLOOP_PROVIDER=deepseek\nDEEPSEEK_API_KEY=test-key\nOPENAI_API_KEY=wrong-key\nCRABLOOP_API_URL={}\n",server.endpoint())).unwrap();
    let output = command(&dir).arg("hello").output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(server.requests()[0]["model"], "deepseek-flash");
    assert_eq!(server.authorizations(), vec!["Bearer test-key"]);
}
#[test]
fn dotenv_parse_errors_do_not_echo_credentials() {
    let dir = TempDir::new();
    fs::write(dir.path().join(".env"), "DEEPSEEK_API_KEY=\"secret-not-for-output").unwrap();
    let output = command(&dir).arg("hello").output().unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains(".env"));
    assert!(!error.contains("secret-not-for-output"));
}
#[test]
fn explicit_workspace_without_task_reads_stdin() {
    let dir = TempDir::new();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();
    let server = StubResponses::start(vec![fixture("text")]);
    let mut child = configured(&dir, &server)
        .args(["--workspace", "work"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"hello\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(work.join(".crabloop").is_dir());
}

#[test]
fn print_mode_runs_once_and_keeps_stdout_plain() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    let output =
        configured(&dir, &server).args(["-p", "hello", "--workspace", "."]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Done");
    assert_eq!(server.requests().len(), 1);
}
#[test]
fn chat_keeps_history_and_new_resets_it() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text"), fixture("text"), fixture("text")]);
    let mut child = configured(&dir, &server)
        .arg("--chat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"first\nsecond\n/new\nfresh\n/exit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let requests = server.requests();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[1]["input"].as_array().unwrap().len(), 3);
    assert_eq!(requests[2]["input"].as_array().unwrap().len(), 1);
    assert_eq!(requests[2]["input"][0]["content"], "fresh");
}

#[test]
fn empty_chat_and_local_commands_need_no_model_or_journal() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![]);
    let mut child = configured(&dir, &server)
        .arg("--chat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"\n/help\n/new\n/exit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(server.requests().is_empty());
    assert!(!dir.path().join(".crabloop").exists());
}

#[cfg(unix)]
#[test]
fn write_file_rejects_fifo_without_blocking_the_run() {
    let dir = TempDir::new();
    assert!(Command::new("mkfifo").arg(dir.path().join("pipe")).status().unwrap().success());
    let server = StubResponses::start(vec![
        serde_json::json!({"status":"completed","output":[{
            "type":"function_call","call_id":"fifo","name":"write_file",
            "arguments":"{\"path\":\"pipe\",\"content\":\"x\"}"
        }]}),
        fixture("text"),
    ]);
    let mut child = configured(&dir, &server)
        .arg("write the file")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut timed_out = false;
    while child.try_wait().unwrap().is_none() {
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            timed_out = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(!timed_out, "write_file blocked on an existing FIFO");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let requests = server.requests();
    let result = &requests[1]["input"][2];
    assert_eq!(result["call_id"], "fifo");
    let body: serde_json::Value = serde_json::from_str(result["output"].as_str().unwrap()).unwrap();
    assert_eq!(body["ok"], false);
    assert!(body["error"].as_str().unwrap().contains("regular file"));
}

#[test]
fn commentary_is_printed_once_per_response_and_final_stdout_stays_clean() {
    let dir = TempDir::new();
    let mut first = fixture("commentary");
    first["output"][0]["content"] = serde_json::json!([
        {"type":"output_text","text":"准备检查。\n"},
        {"type":"output_text","text":"读取项目。"}
    ]);
    let mut action = fixture("commentary");
    action["output"][0]["content"][0]["text"] = "现在写入记录。".into();
    action["output"].as_array_mut().unwrap().push(serde_json::json!({
        "type":"function_call","call_id":"note","name":"write_file",
        "arguments":"{\"path\":\"note.txt\",\"content\":\"saved\"}"
    }));
    let mut final_reply = fixture("commentary");
    final_reply["output"][0]["content"][0]["text"] = "记录已写入。".into();
    final_reply["output"].as_array_mut().unwrap().push(fixture("text")["output"][0].clone());
    let server = StubResponses::start(vec![first.clone(), action, final_reply]);
    let output = configured(&dir, &server).args(["-p", "hello"]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Done\n");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for text in ["准备检查。\n读取项目。", "现在写入记录。", "记录已写入。"]
    {
        assert_eq!(stderr.matches(text).count(), 1, "{stderr}");
    }
    assert!(stderr.find("现在写入记录。").unwrap() < stderr.find("tool ").unwrap());
    assert!(stderr.find("tool ").unwrap() < stderr.find("记录已写入。").unwrap());
    assert_eq!(fs::read_to_string(dir.path().join("note.txt")).unwrap(), "saved");
    let sent = server.requests();
    assert_eq!(sent.len(), 3);
    assert_eq!(sent[1]["input"][1], first["output"][0]);
}

#[test]
fn chat_renders_commentary_separately_from_the_final_answer() {
    let dir = TempDir::new();
    let mut reply = fixture("commentary");
    reply["output"][0]["content"][0]["text"] = "检查完成。".into();
    reply["output"].as_array_mut().unwrap().push(fixture("text")["output"][0].clone());
    let server = StubResponses::start(vec![reply]);
    let mut child = configured(&dir, &server)
        .arg("--chat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"hello\n/exit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.matches("检查完成。").count(), 1, "{stderr}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Done"));
    assert!(!stdout.contains("检查完成。"));
}
