#![cfg(all(feature = "test-endpoint", debug_assertions))]

mod server;

use std::{
    collections::BTreeMap,
    fs::{self, File},
    process::{Command, Output, Stdio},
};

use reqwest::Url;
use serde_json::{Value, json};
use server::{Reply, Request, invoke};
use tempfile::TempDir;

const TEST_PORT: &str = "P2ME_TEST_PORT";
const KEY: &str = "PROVE2ME_API_KEY";
const API_KEY: &str = "fixture-key";
const AGENT: &str = "fixture-agent-token";
const SESSION: &str = "fixture-session-token";
const LEAN: &[u8] = b"theorem solution : True := by trivial\n";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfixture image";

struct Fixture(TempDir);

impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        fixture.write(
            "session.json",
            json!({ "access_token": SESSION }).to_string(),
        );
        fixture.write("solution.lean", LEAN);
        fixture.write("avatar.png", PNG);
        fixture
    }

    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
        fs::write(self.0.path().join(name), bytes).unwrap();
    }

    fn command(&self, args: &[&str], has_key: bool) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_p2me"));
        command
            .env_remove(KEY)
            .current_dir(self.0.path())
            .args(args);
        if has_key {
            command.env(KEY, API_KEY);
        }
        command
    }
}

fn text(value: &Value) -> &str {
    value.as_str().unwrap()
}

fn json_reply(value: Value) -> Reply {
    Reply::Json(200, value)
}

fn auth_reply() -> Reply {
    json_reply(json!({"access_token": AGENT}))
}

fn assert_json_output(output: &Output, expected: &Value) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        output.stdout,
        format!("{}\n", serde_json::to_string_pretty(expected).unwrap()).as_bytes()
    );
}

fn assert_exchange(request: &Request) {
    assert_eq!(request.line, "POST /api/v1/agent/refresh HTTP/1.1");
    assert!(!request.headers.contains_key("authorization"));
    assert_json_body(request, &json!({"api_key": API_KEY}));
}

fn assert_json_body(request: &Request, body: &Value) {
    assert_eq!(request.headers["content-type"], "application/json");
    assert_eq!(
        &serde_json::from_slice::<Value>(&request.body).unwrap(),
        body
    );
}

fn assert_multipart(request: &Request, kind: &str) {
    assert!(request.headers["content-type"].starts_with("multipart/form-data; boundary="));
    let body = String::from_utf8_lossy(&request.body);
    match kind {
        "lean" => {
            assert!(body.contains("name=\"file\"; filename=\"solution.lean\""));
            assert!(body.contains("name=\"theorem_id\"\r\n\r\ntheorem-id\r\n"));
            assert!(body.contains("name=\"proof_type\"\r\n\r\ndisprove\r\n"));
            assert!(body.contains("name=\"explanation\"\r\n\r\nA counterexample.\r\n"));
            assert!(request.body.windows(LEAN.len()).any(|part| part == LEAN));
        }
        "avatar" => {
            assert!(body.contains("name=\"file\"; filename=\"avatar.png\""));
            assert!(
                body.lines()
                    .any(|line| line.eq_ignore_ascii_case("content-type: image/png"))
            );
            assert!(request.body.windows(PNG.len()).any(|part| part == PNG));
        }
        _ => panic!("unknown multipart fixture: {kind}"),
    }
}

#[test]
fn every_documented_endpoint_from_the_built_cli() {
    let mut documented = BTreeMap::new();
    for row in include_str!("../docs/prove2me-api-coverage.md").lines() {
        let cells: Vec<_> = row.split(" | ").collect();
        let Some(endpoint) = cells.get(1).and_then(|cell| cell.strip_prefix('`')) else {
            continue;
        };
        let (method, path) = endpoint.trim_end_matches('`').split_once(' ').unwrap();
        if !path.starts_with('/') {
            continue;
        }
        let command = cells[3]
            .strip_prefix("`p2me ")
            .unwrap()
            .split_once('`')
            .unwrap()
            .0;
        let mut words = command.split_whitespace();
        let group = words.next().unwrap();
        let operation = words.next().unwrap();
        assert!(
            documented
                .insert(format!("{group} {operation}"), (method, path))
                .is_none()
        );
    }
    assert_eq!(documented.len(), 96);

    let fixture = Fixture::new();
    for line in include_str!("fixtures/api-coverage.jsonl").lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let args: Vec<_> = case["args"].as_array().unwrap().iter().map(text).collect();
        let method = text(&case["method"]);
        let path = text(&case["path"]);
        let (doc_method, doc_path) = documented.remove(&args[..2].join(" ")).unwrap_or_else(|| {
            panic!("extra or duplicate command in test cases: {args:?}");
        });
        assert_eq!(method, doc_method, "{args:?}");
        let path_segments: Vec<_> = path.split('/').collect();
        let doc_segments: Vec<_> = doc_path.split('/').collect();
        assert_eq!(path_segments.len(), doc_segments.len(), "{args:?}");
        for (segment, pattern) in path_segments.iter().zip(doc_segments) {
            if pattern.starts_with('{') {
                assert!(!segment.is_empty(), "{args:?}");
            } else {
                assert_eq!(*segment, pattern, "{args:?}");
            }
        }

        let auth = case["auth"].as_str().unwrap_or("bearer");
        let mut replies = Vec::new();
        if auth == "bearer" {
            replies.push(auth_reply());
        }
        if args.contains(&"--json-file") {
            fixture.write("payload.json", case["json"].to_string());
        }
        let expected = json!({"result": args[..2]});
        let no_content = case["empty"].as_bool().unwrap_or(false);
        replies.push(if no_content {
            Reply::Json(204, Value::Null)
        } else {
            json_reply(expected.clone())
        });
        let (output, requests) = invoke(
            fixture.command(&args, matches!(auth, "bearer" | "api-key")),
            replies,
        );
        if no_content {
            assert_eq!(output.status.code(), Some(0), "{args:?}: {output:?}");
            assert!(
                output.stdout.is_empty() && output.stderr.is_empty(),
                "{args:?}: {output:?}"
            );
        } else {
            assert_json_output(&output, &expected);
        }
        let request = if auth == "bearer" {
            assert_eq!(requests.len(), 2, "{args:?}");
            assert_exchange(&requests[0]);
            &requests[1]
        } else {
            assert_eq!(requests.len(), 1, "{args:?}");
            &requests[0]
        };
        let authorization = request.headers.get("authorization").map(String::as_str);
        match auth {
            "bearer" => assert_eq!(
                authorization,
                Some("Bearer fixture-agent-token"),
                "{args:?}"
            ),
            "session" => assert_eq!(
                authorization,
                Some("Bearer fixture-session-token"),
                "{args:?}"
            ),
            "api-key" | "public" => assert_eq!(authorization, None, "{args:?}"),
            _ => panic!("unknown auth fixture: {auth}"),
        }
        let parts: Vec<_> = request.line.split_whitespace().collect();
        assert_eq!(parts[0], method, "{args:?}");
        let url = Url::parse(&format!("http://example.invalid{}", parts[1])).unwrap();
        assert_eq!(url.path(), format!("/api/v1{path}"), "{args:?}");
        let mut query = BTreeMap::<_, Vec<_>>::new();
        for (key, value) in url.query_pairs() {
            query.entry(key).or_default().push(value);
        }
        let empty = json!({});
        assert_eq!(
            json!(query),
            *case.get("query").unwrap_or(&empty),
            "{args:?}"
        );
        if let Some(target) = case.get("json") {
            assert_json_body(request, target);
        } else if let Some(kind) = case["multipart"].as_str() {
            assert_multipart(request, kind);
        } else {
            assert!(request.body.is_empty(), "{args:?}");
        }
    }
    assert!(
        documented.is_empty(),
        "missing fixture cases: {documented:?}"
    );
}

#[test]
fn batch_queries_and_browse_modes() {
    let fixture = Fixture::new();
    for (args, expected) in [
        (vec!["theorems", "search"], "/theorems"),
        (
            vec!["definitions", "search", "--tags", "a,b"],
            "/definitions?tags=a%2Cb",
        ),
        (
            vec![
                "theorems", "search", "first é", "--q", "next & +", "--q", "third",
            ],
            "/theorems?q=first+%C3%A9&q=next+%26+%2B&q=third",
        ),
        (
            vec!["theorems", "submissions", "thm", "--first", "true"],
            "/theorems/thm/submissions?first=true",
        ),
        (
            vec![
                "theorems",
                "graph",
                "thm",
                "--expand-node",
                "node",
                "--sketch-offset",
                "2",
            ],
            "/theorems/thm/graph?expand_node=node&sketch_offset=2",
        ),
    ] {
        let (output, requests) = invoke(
            fixture.command(&args, true),
            vec![auth_reply(), json_reply(json!({}))],
        );
        assert_json_output(&output, &json!({}));
        assert_eq!(requests[1].line, format!("GET /api/v1{expected} HTTP/1.1"));
    }
}

#[test]
fn reads_json_from_stdin_and_passes_alternative_bodies() {
    let fixture = Fixture::new();
    for (args, payload, expected) in [
        (
            vec!["mission-proposals", "create"],
            json!({"campaign_id":"campaign", "visibility":"private"}),
            "POST /api/v1/mission-proposals HTTP/1.1",
        ),
        (
            vec!["mission-proposals", "add-item", "p"],
            json!({"kind":"reference", "theorem_id":"thm"}),
            "POST /api/v1/mission-proposals/p/items HTTP/1.1",
        ),
        (
            vec!["mission-proposals", "add-item", "p"],
            json!({"kind":"definition", "definition_name":"d", "definition_title":"D", "definition":"def d := 0"}),
            "POST /api/v1/mission-proposals/p/items HTTP/1.1",
        ),
        (
            vec!["theorems", "submit"],
            json!({"theorem_name":"trivial", "theorem_title":"Trivial", "formal_statement":"theorem trivial : True := by sorry", "natural_language_statement":"Truth."}),
            "POST /api/v1/submit-problem HTTP/1.1",
        ),
        (
            vec!["submissions", "update", "s"],
            json!({"deprecated":true}),
            "PATCH /api/v1/submissions/s HTTP/1.1",
        ),
        (
            vec!["submissions", "update", "s"],
            json!({"explanation":null}),
            "PATCH /api/v1/submissions/s HTTP/1.1",
        ),
    ] {
        fixture.write("stdin.json", payload.to_string());
        let mut command = fixture.command(&args, true);
        command.args(["--json-file", "-"]).stdin(Stdio::from(
            File::open(fixture.0.path().join("stdin.json")).unwrap(),
        ));
        let (output, requests) = invoke(
            command,
            vec![auth_reply(), Reply::Json(202, json!({"ok":true}))],
        );
        assert_json_output(&output, &json!({"ok":true}));
        assert_eq!(requests[1].line, expected);
        assert_json_body(&requests[1], &payload);
    }
}

#[test]
fn rejects_invalid_inputs_without_sending_a_request() {
    let fixture = Fixture::new();
    fixture.write("invalid.json", "fixture-private-content not JSON");
    fixture.write("array.json", "[]");
    fixture.write(
        "not-session.json",
        "{\"api_key\":\"fixture-private-content\"}",
    );
    fixture.write("not-image.png", "not image bytes");
    for (args, expected) in [
        (
            vec!["auth", "login", "--json-file", "invalid.json"],
            "could not read a JSON object",
        ),
        (
            vec!["auth", "login", "--json-file", "array.json"],
            "JSON input must be an object",
        ),
        (
            vec!["auth", "api-key", "--session-file", "not-session.json"],
            "session JSON does not contain an access token",
        ),
        (
            vec!["resources", "get", "m", "../verify"],
            "invalid path parameter",
        ),
        (
            vec!["me", "upload-avatar", "not-image.png"],
            "the avatar must be a PNG, JPEG, or WebP image",
        ),
    ] {
        let (output, requests) = invoke(fixture.command(&args, true), vec![]);
        assert!(requests.is_empty(), "{args:?}");
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(expected), "{stderr:?}");
        assert!(!stderr.contains("fixture-private-content"));
    }
}

#[test]
fn public_login_failures_are_not_retried_or_echoed() {
    let fixture = Fixture::new();
    fixture.write(
        "login.json",
        r#"{"email":"fake@example.invalid","password":"fixture-private-content"}"#,
    );
    for (reply, expected) in [
        (Reply::Disconnect, "request failed"),
        (Reply::InvalidJson, "response as JSON"),
        (
            Reply::Json(401, json!({"detail":"fixture-private-content"})),
            "HTTP 401",
        ),
    ] {
        let command = fixture.command(&["auth", "login", "--json-file", "login.json"], false);
        let (output, requests) = invoke(command, vec![reply]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(requests.len(), 1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(expected), "{stderr:?}");
        assert!(!stderr.contains("fixture-private-content"));
    }
}

#[test]
fn avatar_supports_jpeg_and_webp() {
    let fixture = Fixture::new();
    for (bytes, mime) in [
        (b"\xff\xd8\xfffixture".as_slice(), "image/jpeg"),
        (b"RIFF\x04\0\0\0WEBP".as_slice(), "image/webp"),
    ] {
        fixture.write("input", bytes);
        let (output, requests) = invoke(
            fixture.command(&["me", "upload-avatar", "input"], true),
            vec![
                auth_reply(),
                json_reply(json!({"avatar_url":"https://example.invalid/avatar"})),
            ],
        );
        assert_json_output(
            &output,
            &json!({"avatar_url":"https://example.invalid/avatar"}),
        );
        let request = &requests[1];
        assert!(
            String::from_utf8_lossy(&request.body)
                .lines()
                .any(|line| line.eq_ignore_ascii_case(&format!("content-type: {mime}")))
        );
        assert!(request.body.windows(bytes.len()).any(|part| part == bytes));
    }
}

#[test]
fn login_json_can_feed_key_minting() {
    let fixture = Fixture::new();
    fixture.write(
        "login.json",
        json!({"email":"fake@example.invalid","password":"fixture-password"}).to_string(),
    );
    let session =
        json!({"access_token": SESSION, "refresh_token":"fixture-refresh", "version":"fixture"});
    let (login, requests) = invoke(
        fixture.command(&["auth", "login", "--json-file", "login.json"], false),
        vec![json_reply(session.clone())],
    );
    assert_json_output(&login, &session);
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].headers.contains_key("authorization"));

    fixture.write("pipe.json", login.stdout);
    let mut mint = fixture.command(&["auth", "api-key", "--session-file", "-"], false);
    mint.stdin(Stdio::from(
        File::open(fixture.0.path().join("pipe.json")).unwrap(),
    ));
    let key = json!({"api_key": API_KEY});
    let (output, requests) = invoke(mint, vec![json_reply(key.clone())]);
    assert_json_output(&output, &key);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].line, "POST /api/v1/agent/api-key HTTP/1.1");
    assert_eq!(
        requests[0].headers["authorization"],
        format!("Bearer {SESSION}")
    );
    assert!(requests[0].body.is_empty());
}

#[test]
fn explicit_session_is_not_forwarded_to_redirects() {
    let fixture = Fixture::new();
    let (output, requests) = invoke(
        fixture.command(
            &["auth", "api-key", "--session-file", "session.json"],
            false,
        ),
        vec![Reply::Redirect],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("HTTP 307"));
    assert!(!stderr.contains(SESSION));
    assert_eq!(requests.len(), 1);
}
