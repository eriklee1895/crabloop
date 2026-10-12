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
        .env("CRABLOOP_PROVIDER", "openai")
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
    assert_eq!(server.requests().len(), 1);
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
    assert!(String::from_utf8_lossy(&help.stdout).contains("-p"));
    fs::remove_file(dir.path().join(".env")).unwrap();
    let server = StubResponses::start(vec![]);
    let empty = configured(&dir, &server).stdin(Stdio::null()).output().unwrap();
    assert!(empty.status.success());
    assert!(server.requests().is_empty());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
#[test]
fn dotenv_is_loaded_as_data_and_process_environment_wins() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    fs::write(dir.path().join(".env"), format!("export OPENAI_API_KEY='file-key'\nCRABLOOP_MODEL=from-file\nCRABLOOP_API_URL={}\nIGNORED='$(touch unexpected)'\n", server.endpoint())).unwrap();
    let output = command(&dir)
        .env("CRABLOOP_PROVIDER", "openai")
        .env("CRABLOOP_MODEL", "override-model")
        .arg("hello")
        .output()
        .unwrap();
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
fn stdin_task_runs_tools_in_the_startup_directory() {
    let dir = TempDir::new();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();
    let server = StubResponses::start(vec![
        serde_json::json!({"status":"completed","output":[{
            "type":"function_call","call_id":"write","name":"write_file",
            "arguments":"{\"path\":\"note.txt\",\"content\":\"inside work\"}"
        }]}),
        fixture("text"),
    ]);
    let mut child = configured(&dir, &server)
        .current_dir(&work)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"hello\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read_to_string(work.join("note.txt")).unwrap(), "inside work");
    assert!(!dir.path().join("note.txt").exists());
    assert_eq!(server.requests()[0]["input"][0]["content"], "hello");
}

#[test]
fn print_mode_runs_once_and_keeps_stdout_plain() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    let output = configured(&dir, &server).args(["-p", "hello"]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Done");
    assert_eq!(server.requests().len(), 1);
}
#[test]
fn chat_is_reserved_for_later_versions() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![]);
    let output = configured(&dir, &server).arg("--chat").stdin(Stdio::null()).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage"));
    assert!(server.requests().is_empty());
}
#[test]
fn empty_task_exits_without_calling_the_model() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![]);
    let output = configured(&dir, &server).args(["-p", "  "]).output().unwrap();
    assert!(output.status.success());
    assert!(server.requests().is_empty());
}
