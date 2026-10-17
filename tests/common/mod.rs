#![allow(dead_code)]

use crabloop::{Config, model::ToolCall};
use serde_json::Value;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

pub fn test_config(workspace: &Path, endpoint: &str) -> Config {
    Config {
        endpoint: endpoint.into(),
        api_key: "test-key".into(),
        model: "test-model".into(),
        workspace: workspace.to_path_buf(),
        max_turns: 16,
        request_bytes: 4 * 1024 * 1024,
        bash_timeout: Duration::from_secs(30),
    }
}

pub fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/responses/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

pub fn make_call(call_id: &str, name: &str, arguments: Value) -> ToolCall {
    ToolCall { call_id: call_id.into(), name: name.into(), arguments: arguments.to_string() }
}

/// Run the binary with `stdin` as its whole input.
pub fn task_output(command: &mut Command, stdin: &str) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("crabloop-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A one-thread HTTP server that answers each POST with the next prepared response.
pub struct StubResponses {
    endpoint: String,
    requests: Arc<Mutex<Vec<Value>>>,
    authorizations: Arc<Mutex<Vec<String>>>,
    stopped: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl StubResponses {
    pub fn start(replies: Vec<Value>) -> Self {
        Self::with_status(200, replies)
    }
    pub fn with_status(status: u16, replies: Vec<Value>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/v1/responses", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stopped = Arc::new(AtomicBool::new(false));
        let recorded = Arc::clone(&requests);
        let authorizations = Arc::new(Mutex::new(Vec::new()));
        let auth = Arc::clone(&authorizations);
        let stop = Arc::clone(&stopped);
        let worker = thread::spawn(move || {
            let mut replies = replies.into_iter();
            for incoming in listener.incoming() {
                let mut socket = incoming.unwrap();
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut wire = Vec::new();
                let mut byte = [0];
                while !wire.ends_with(b"\r\n\r\n") {
                    if socket.read_exact(&mut byte).is_err() {
                        return;
                    }
                    wire.push(byte[0]);
                }
                let headers = String::from_utf8(wire).unwrap();
                assert!(headers.starts_with("POST /v1/responses HTTP/1.1"));
                auth.lock().unwrap().push(
                    headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("authorization")
                                .then(|| value.trim().to_owned())
                        })
                        .unwrap_or_default(),
                );
                let len = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap())
                    })
                    .unwrap();
                let mut bytes = vec![0; len];
                socket.read_exact(&mut bytes).unwrap();
                recorded.lock().unwrap().push(serde_json::from_slice(&bytes).unwrap());
                let body = replies
                    .next()
                    .unwrap_or(serde_json::json!({"error":"unexpected extra request"}))
                    .to_string();
                let response = format!(
                    "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes());
            }
        });
        Self { endpoint, requests, authorizations, stopped, worker: Some(worker) }
    }
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub fn authorizations(&self) -> Vec<String> {
        self.authorizations.lock().unwrap().clone()
    }
    pub fn requests(&self) -> Vec<Value> {
        self.requests.lock().unwrap().clone()
    }
}
impl Drop for StubResponses {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(
            self.endpoint.trim_start_matches("http://").trim_end_matches("/v1/responses"),
        );
        self.worker.take().unwrap().join().unwrap();
    }
}
