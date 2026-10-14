mod common;
use common::{StubResponses, TempDir, fixture, task_output};
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
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Inspect this directory\nand summarize the result\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        server.requests()[0]["input"][0]["content"],
        "Inspect this directory\nand summarize the result"
    );
    assert_eq!(server.requests().len(), 1);
}
#[test]
fn arguments_are_rejected_before_reading_stdin_or_configuration() {
    let dir = TempDir::new();
    fs::write(dir.path().join(".env"), "bad = \"not closed").unwrap();
    let server = StubResponses::start(vec![]);
    for args in [
        vec!["Inspect README"],
        vec!["-p", "hello"],
        vec!["--print", "hello"],
        vec!["-h"],
        vec!["--help"],
        vec!["--chat"],
        vec!["--workspace", "."],
    ] {
        let mut child = configured(&dir, &server)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // Keep stdin open: invalid arguments must fail without waiting for input.
        let input = child.stdin.take().unwrap();
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
        drop(input);
        let output = child.wait_with_output().unwrap();
        assert!(!timed_out, "arguments waited for stdin: {args:?}");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("v1 takes no arguments"), "{error}");
        assert!(error.contains("README.md"), "{error}");
        assert!(!error.contains("crab>"), "{error}");
        assert!(!error.contains("🦀"), "{error}");
    }
    assert!(server.requests().is_empty());
}

#[test]
fn dotenv_is_loaded_as_data_and_process_environment_wins() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    fs::write(dir.path().join(".env"), format!("export OPENAI_API_KEY='file-key'\nCRABLOOP_MODEL=from-file\nCRABLOOP_API_URL={}\nIGNORED='$(touch unexpected)'\n", server.endpoint())).unwrap();
    let output = task_output(
        command(&dir).env("CRABLOOP_PROVIDER", "openai").env("CRABLOOP_MODEL", "override-model"),
        "hello",
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(server.requests()[0]["model"], "override-model");
    assert_eq!(server.authorizations(), vec!["Bearer file-key"]);
    assert!(!dir.path().join("unexpected").exists());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("file-key"));
}
#[test]
fn existing_deepseek_key_is_enough_for_default_startup() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    fs::write(dir.path().join(".env"), format!("DEEPSEEK_API_KEY=private-deepseek-key\nOPENAI_API_KEY=private-proxy-key\nOPENAI_BASE_URL=https://proxy.invalid/v1\nCRABLOOP_API_URL={}\n", server.endpoint())).unwrap();
    let output = task_output(&mut command(&dir), "hello");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("deepseek-flash"));
    assert!(!stderr.contains("private-"));
    assert_eq!(server.requests()[0]["model"], "deepseek-flash");
    assert_eq!(server.authorizations(), vec!["Bearer private-deepseek-key"]);
}
#[test]
fn explicit_provider_uses_its_own_key_and_default_model() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    fs::write(dir.path().join(".env"), format!("CRABLOOP_PROVIDER=deepseek\nDEEPSEEK_API_KEY=test-key\nOPENAI_API_KEY=wrong-key\nCRABLOOP_API_URL={}\n",server.endpoint())).unwrap();
    let output = task_output(&mut command(&dir), "hello");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(server.requests()[0]["model"], "deepseek-flash");
    assert_eq!(server.authorizations(), vec!["Bearer test-key"]);
}
#[test]
fn dotenv_parse_errors_do_not_echo_credentials() {
    let dir = TempDir::new();
    fs::write(dir.path().join(".env"), "DEEPSEEK_API_KEY=\"secret-not-for-output").unwrap();
    let output = task_output(&mut command(&dir), "hello");
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
fn stdin_runs_once_and_keeps_stdout_plain() {
    let dir = TempDir::new();
    let server = StubResponses::start(vec![fixture("text")]);
    let output = task_output(&mut configured(&dir, &server), "hello");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Done");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("crab>"), "{stderr}");
    assert!(!stderr.contains("🦀"), "{stderr}");
    assert_eq!(server.requests().len(), 1);
}
#[test]
fn empty_task_and_eof_need_no_credentials_or_valid_dotenv() {
    let dir = TempDir::new();
    fs::write(dir.path().join(".env"), "bad = \"not closed").unwrap();
    let server = StubResponses::start(vec![]);
    for task in ["", "  \n\t"] {
        let output = task_output(&mut configured(&dir, &server), task);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty());
        let output = task_output(&mut command(&dir), task);
        assert!(output.status.success());
    }
    assert!(server.requests().is_empty());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn oversized_stdin_is_rejected_before_credentials() {
    let dir = TempDir::new();
    let task = "x".repeat(crabloop::Limits::default().request_bytes + 1);
    let output = task_output(&mut command(&dir), &task);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("task exceeds input budget"));
}

#[test]
fn nonempty_stdin_reports_the_missing_provider_key() {
    let dir = TempDir::new();
    let output = task_output(&mut command(&dir), "Inspect README");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("DEEPSEEK_API_KEY is required"));
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
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"write the file\n").unwrap();
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
fn tool_display_is_readable_bounded_and_preserves_model_feedback() {
    let dir = TempDir::new();
    let contents = format!("alpha\nbeta\n{}", "x".repeat(500));
    fs::write(dir.path().join("notes.md"), &contents).unwrap();
    let server = StubResponses::start(vec![
        serde_json::json!({"status":"completed","output":[
            {"type":"function_call","call_id":"read","name":"read_file","arguments":"{\"path\":\"notes.md\"}"},
            {"type":"function_call","call_id":"missing","name":"read_file","arguments":"{\"path\":\"missing.md\"}"},
            {"type":"function_call","call_id":"bad_command","name":"bash","arguments":serde_json::json!({"command":"printf 'warning\n' >&2; exit 7"}).to_string()}
        ]}),
        fixture("text"),
    ]);
    let output = task_output(&mut configured(&dir, &server), "inspect");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("read_file · notes.md"), "{stderr}");
    assert!(stderr.contains("done"), "{stderr}");
    assert!(stderr.contains("alpha\nbeta"), "{stderr}");
    assert!(stderr.contains("failed"), "{stderr}");
    assert!(stderr.contains("No such file"), "{stderr}");
    assert!(stderr.contains("bash · printf"), "{stderr}");
    assert!(stderr.contains("warning"), "{stderr}");
    assert!(!stderr.contains("tool {"), "{stderr}");
    assert!(!stderr.contains(&"x".repeat(300)), "{stderr}");
    let requests = server.requests();
    let result: serde_json::Value =
        serde_json::from_str(requests[1]["input"][4]["output"].as_str().unwrap()).unwrap();
    assert_eq!(result["content"], contents);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Done");
}
