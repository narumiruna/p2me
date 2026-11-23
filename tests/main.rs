#[path = "../src/cli/mod.rs"]
mod cli;

mod server;

use std::{
    env, fs,
    io::ErrorKind,
    path::PathBuf,
    process::{Command, ExitCode, Output},
};

use serde_json::{Value, json};
use server::{Reply, Request, invoke};
use tempfile::TempDir;

const TEST_PORT: &str = "P2ME_TEST_PORT";
const KEY: &str = "PROVE2ME_API_KEY";
const ENV_KEY: &str = "p2m_dummy_env_key";
const FILE_KEY: &str = "p2m_dummy_file_key";
const TOKEN: &str = "dummy_access_token";

fn main() -> ExitCode {
    // This executable can run the CLI against a fixture. The shipped binary cannot.
    if let Ok(port) = env::var(TEST_PORT) {
        let port: u16 = port.parse().expect("test server port");
        return cli::run(&format!("http://127.0.0.1:{port}/api/v1"));
    }
    for (name, test) in [
        (
            "production arguments and help",
            production_arguments as fn(),
        ),
        ("credential sources", credentials),
        (
            "authentication and redirect failures",
            authentication_failures,
        ),
        ("theorem search and detail", theorems),
        ("proof and disproof submission", submissions),
        ("one-shot verification verdicts", verdicts),
    ] {
        test();
        println!("ok - {name}");
    }
    ExitCode::SUCCESS
}

struct Fixture {
    temp: TempDir,
}

impl Fixture {
    fn new() -> Self {
        Self {
            temp: tempfile::tempdir().unwrap(),
        }
    }

    fn production(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_p2me"));
        command.env_remove(KEY).current_dir(self.temp.path());
        command
    }

    fn run(&self, args: &[&str], key: Option<&str>, replies: Vec<Reply>) -> (Output, Vec<Request>) {
        let mut command = Command::new(env::current_exe().unwrap());
        command
            .env_remove(KEY)
            .current_dir(self.temp.path())
            .args(args);
        if let Some(key) = key {
            command.env(KEY, key);
        }
        invoke(command, replies)
    }

    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.temp.path().join(name);
        fs::write(&path, text).unwrap();
        path
    }
}

fn ok(value: Value) -> Reply {
    Reply::Json(200, value)
}

fn token() -> Reply {
    ok(json!({ "access_token": TOKEN, "version": "fixture" }))
}

fn check_output(output: &Output, expected: Value) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        expected
    );
    let mut pretty = serde_json::to_string_pretty(&expected).unwrap();
    pretty.push('\n');
    assert_eq!(output.stdout, pretty.as_bytes());
}

fn check_failure(output: &Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(diagnostic), "{stderr:?}");
    for secret in [ENV_KEY, FILE_KEY, TOKEN] {
        assert!(!stderr.contains(secret), "a credential reached stderr");
    }
}

fn check_auth(request: &Request, key: &str) {
    assert_eq!(request.line, "POST /api/v1/agent/refresh HTTP/1.1");
    assert!(!request.headers.contains_key("authorization"));
    assert_eq!(
        serde_json::from_slice::<Value>(&request.body).unwrap(),
        json!({ "api_key": key })
    );
}

fn check_protected(request: &Request, line: &str) {
    assert_eq!(request.line, line);
    assert_eq!(request.headers["authorization"], format!("Bearer {TOKEN}"));
}

fn production_arguments() {
    let fixture = Fixture::new();
    for (args, expected) in [
        (&["--help"][..], "p2me theorems search poincare"),
        (&["theorems", "--help"][..], "search"),
        (&["verify", "--help"][..], "status"),
        (&["verify", "submit", "--help"][..], "--proof-type"),
    ] {
        let output = fixture.production().args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let output = fixture
        .production()
        .args(["verify", "status"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SUBMISSION_ID"));
}

fn credentials() {
    let fixture = Fixture::new();
    let args = ["theorems", "search", "calculus"];
    let result = json!({ "theorems": [], "total": 0 });
    let (output, requests) = fixture.run(&args, Some(ENV_KEY), vec![token(), ok(result.clone())]);
    check_output(&output, result.clone());
    check_auth(&requests[0], ENV_KEY);
    assert_eq!(fs::read_dir(fixture.temp.path()).unwrap().count(), 0);
    let contents = format!("# Local secrets\nOTHER=ignored\n{KEY} = {FILE_KEY}\n");
    let dotenv = fixture.write(".env", &contents);
    let (output, requests) = fixture.run(&args, None, vec![token(), ok(result.clone())]);
    check_output(&output, result.clone());
    check_auth(&requests[0], FILE_KEY);
    let (output, requests) = fixture.run(&args, Some(ENV_KEY), vec![token(), ok(result.clone())]);
    check_output(&output, result);
    check_auth(&requests[0], ENV_KEY);
    assert_eq!(fs::read_to_string(&dotenv).unwrap(), contents);
    assert_eq!(fs::read_dir(fixture.temp.path()).unwrap().count(), 1);
    for key in [Some(""), Some("  ")] {
        let (output, requests) = fixture.run(&args, key, vec![]);
        check_failure(&output, "set PROVE2ME_API_KEY");
        assert!(requests.is_empty());
    }
    fixture.write(".env", &format!("{KEY}=\"{FILE_KEY}\"\n"));
    let (output, requests) = fixture.run(&args, None, vec![]);
    check_failure(&output, "unquoted PROVE2ME_API_KEY");
    assert!(requests.is_empty());
    let subdir = fixture.temp.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(dotenv, contents).unwrap();
    let mut child = Command::new(env::current_exe().unwrap());
    child.env_remove(KEY).current_dir(subdir).args(args);
    let (output, requests) = invoke(child, vec![]);
    check_failure(&output, "set PROVE2ME_API_KEY");
    assert!(requests.is_empty());
    assert!(
        matches!(fs::metadata(fixture.temp.path().join("credentials.json")), Err(error) if error.kind() == ErrorKind::NotFound)
    );
}

fn authentication_failures() {
    let fixture = Fixture::new();
    let args = ["verify", "status", "sub-123"];
    for (replies, diagnostic, count) in [
        (
            vec![Reply::Json(401, json!({ "detail": ENV_KEY }))],
            "HTTP 401",
            1,
        ),
        (
            vec![ok(json!({ "detail": ENV_KEY }))],
            "did not return an access token",
            1,
        ),
        (vec![Reply::Redirect], "authentication failed (HTTP 307)", 1),
        (
            vec![token(), Reply::Redirect],
            "request failed (HTTP 307)",
            2,
        ),
        (
            vec![token(), Reply::Json(500, json!({ "detail": TOKEN }))],
            "HTTP 500",
            2,
        ),
    ] {
        let (output, requests) = fixture.run(&args, Some(ENV_KEY), replies);
        check_failure(&output, diagnostic);
        assert_eq!(
            requests.len(),
            count,
            "no redirected request should reach the target"
        );
        check_auth(&requests[0], ENV_KEY);
    }
}

fn theorems() {
    let fixture = Fixture::new();
    let search = json!({ "theorems": [{ "theorem_id": "thm-123", "theorem_name": "poincare" }], "total": 1 });
    let (output, requests) = fixture.run(
        &["theorems", "search", "poincaré & x+y"],
        Some(ENV_KEY),
        vec![token(), ok(search.clone())],
    );
    check_output(&output, search.clone());
    check_auth(&requests[0], ENV_KEY);
    assert_eq!(requests.len(), 2);
    check_protected(
        &requests[1],
        "GET /api/v1/theorems?q=poincar%C3%A9+%26+x%2By HTTP/1.1",
    );

    let id = search["theorems"][0]["theorem_id"].as_str().unwrap();
    let detail = json!({ "theorem_id": id, "formal_statement": "theorem example : True := by sorry", "preamble": "import Mathlib", "upstream_number": 123456789012345678901234567890_u128 });
    let (output, requests) = fixture.run(
        &["theorems", "get", id],
        Some(ENV_KEY),
        vec![token(), ok(detail.clone())],
    );
    check_output(&output, detail);
    assert_eq!(requests.len(), 2);
    check_protected(&requests[1], "GET /api/v1/theorems/thm-123 HTTP/1.1");

    let (output, requests) = fixture.run(&["theorems", "get", "../verify"], Some(ENV_KEY), vec![]);
    check_failure(&output, "invalid theorem ID");
    assert!(requests.is_empty());
}

fn submissions() {
    const LEAN: &str = "theorem solution : True := by trivial\n";
    let fixture = Fixture::new();
    fixture.write("solution.lean", LEAN);
    let receipt = json!({ "submission_id": "sub-123", "status": "PENDING", "message": "queued" });
    let submit = ["verify", "submit", "thm-123", "solution.lean"];
    for kind in ["prove", "disprove"] {
        let mut args = submit.to_vec();
        if kind == "disprove" {
            args.extend([
                "--proof-type",
                "disprove",
                "--explanation",
                "A counterexample.",
            ]);
        }
        let (output, requests) =
            fixture.run(&args, Some(ENV_KEY), vec![token(), ok(receipt.clone())]);
        check_output(&output, receipt.clone());
        assert_eq!(requests.len(), 2, "upload should not call status");
        let upload = &requests[1];
        check_protected(upload, "POST /api/v1/verify HTTP/1.1");
        assert!(upload.headers["content-type"].starts_with("multipart/form-data; boundary="));
        let body = String::from_utf8_lossy(&upload.body);
        assert!(body.contains("name=\"theorem_id\"\r\n\r\nthm-123\r\n"));
        assert!(body.contains(&format!("name=\"proof_type\"\r\n\r\n{kind}\r\n")));
        assert!(body.contains("name=\"file\"; filename=\"solution.lean\""));
        assert!(body.contains(LEAN));
        assert_eq!(
            body.contains("name=\"explanation\"\r\n\r\nA counterexample.\r\n"),
            kind == "disprove"
        );

        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        let id = response["submission_id"].as_str().unwrap();
        let result = json!({ "id": id, "theorem_id": "thm-123", "status": "ACCEPTED" });
        let (output, requests) = fixture.run(
            &["verify", "status", id],
            Some(ENV_KEY),
            vec![token(), ok(result.clone())],
        );
        check_output(&output, result);
        assert_eq!(requests.len(), 2);
        check_protected(
            &requests[1],
            "GET /api/v1/verify?submission_id=sub-123 HTTP/1.1",
        );
    }

    let (output, requests) = fixture.run(&submit, Some(ENV_KEY), vec![token(), Reply::Disconnect]);
    check_failure(&output, "request failed");
    assert_eq!(requests.len(), 2, "lost response must not retry the upload");
    check_protected(&requests[1], "POST /api/v1/verify HTTP/1.1");

    let (output, requests) = fixture.run(
        &["verify", "submit", "thm-123", "missing.lean"],
        Some(ENV_KEY),
        vec![],
    );
    check_failure(&output, "could not open the Lean file");
    assert!(requests.is_empty());
}

fn verdicts() {
    let fixture = Fixture::new();
    let args = ["verify", "status", "sub-123"];
    for status in ["PENDING", "ACCEPTED", "SKETCH_ACCEPTED", "WA", "ERROR"] {
        let result = json!({ "id": "sub-123", "status": status, "error_message": "fixture" });
        let (output, requests) =
            fixture.run(&args, Some(ENV_KEY), vec![token(), ok(result.clone())]);
        check_output(&output, result);
        assert_eq!(
            requests.len(),
            2,
            "one token exchange and one status lookup"
        );
        check_protected(
            &requests[1],
            "GET /api/v1/verify?submission_id=sub-123 HTTP/1.1",
        );
    }
    for (reply, diagnostic) in [
        (Reply::Disconnect, "request failed"),
        (
            Reply::Json(404, json!({ "detail": "unknown submission" })),
            "HTTP 404",
        ),
        (Reply::InvalidJson, "response as JSON"),
    ] {
        let (output, requests) = fixture.run(&args, Some(ENV_KEY), vec![token(), reply]);
        check_failure(&output, diagnostic);
        assert_eq!(requests.len(), 2);
    }
}
