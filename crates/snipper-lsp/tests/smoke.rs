//! End-to-end LSP smoke tests using a minimal client fixture.
//!
//! Each test starts the `snipper-lsp` binary over stdio and drives a
//! request sequence against it.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

fn write_msg(w: &mut impl Write, body: &str) {
    write!(w, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
    w.flush().unwrap();
}

fn read_msg(r: &mut impl BufRead) -> serde_json::Value {
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(v) = trimmed.strip_prefix("Content-Length: ") {
            content_length = v.parse().unwrap();
        }
    }
    let mut body = vec![0u8; content_length];
    r.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// Read messages until we find the response whose `id` matches `req_id`.
/// Notifications (no `id` field) are silently skipped.
fn read_response(r: &mut impl BufRead, req_id: u64) -> serde_json::Value {
    loop {
        let msg = read_msg(r);
        if msg.get("id").and_then(serde_json::Value::as_u64) == Some(req_id) {
            return msg;
        }
    }
}

/// Spawn snipper-lsp, send initialize + initialized + didOpen for a C# fixture.
///
/// Returns `(stdin, stdout_reader, child)`. The caller must send shutdown + exit
/// and wait on `child` after the test body.
fn start_server(
    source: &str,
    uri: &str,
) -> (ChildStdin, BufReader<std::process::ChildStdout>, Child) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_snipper-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("snipper-lsp binary starts");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    write_msg(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "processId": null, "rootUri": null, "capabilities": {} }
        })
        .to_string(),
    );
    let resp = read_response(&mut stdout, 1);
    assert!(resp["result"]["capabilities"].is_object(), "initialize ok");

    write_msg(
        &mut stdin,
        &serde_json::json!({"jsonrpc":"2.0","method":"initialized","params":{}}).to_string(),
    );

    write_msg(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "csharp",
                    "version": 1,
                    "text": source
                }
            }
        })
        .to_string(),
    );

    (stdin, stdout, child)
}

fn shutdown_server(
    mut stdin: ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
    id: u64,
    child: &mut Child,
) {
    write_msg(
        &mut stdin,
        &serde_json::json!({"jsonrpc":"2.0","id":id,"method":"shutdown","params":null})
            .to_string(),
    );
    let _ = read_response(stdout, id);
    write_msg(
        &mut stdin,
        &serde_json::json!({"jsonrpc":"2.0","method":"exit","params":null}).to_string(),
    );
    drop(stdin);
    child.wait().expect("snipper-lsp exits cleanly");
}

/// Typing `users.fod` must return a `FirstOrDefault()` completion item.
#[test]
fn lsp_completion_smoke() {
    let (mut stdin, mut stdout, mut child) =
        start_server("var y = users.fod;", "file:///test.cs");

    write_msg(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": "file:///test.cs" },
                "position": { "line": 0, "character": 17 }
            }
        })
        .to_string(),
    );
    let resp = read_response(&mut stdout, 2);

    let items = resp["result"].as_array().expect("completion returns array");
    assert!(!items.is_empty(), "at least one completion item");
    let labels: Vec<&str> = items.iter().filter_map(|i| i["label"].as_str()).collect();
    assert!(
        labels.iter().any(|l| l.contains("FirstOrDefault")),
        "fod → FirstOrDefault must appear; got: {labels:?}"
    );

    shutdown_server(stdin, &mut stdout, 3, &mut child);
}

/// `workspace/executeCommand` for a built-in command must return a snippet body
/// string containing tabstop placeholders — never `null`.
#[test]
fn lsp_execute_command_returns_body() {
    let (mut stdin, mut stdout, mut child) = start_server("", "file:///scaffold.cs");

    write_msg(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "workspace/executeCommand",
            "params": {
                "command": "snipper.scaffoldConstructor",
                "arguments": [{
                    "textDocument": { "uri": "file:///scaffold.cs" },
                    "position": { "line": 0, "character": 0 }
                }]
            }
        })
        .to_string(),
    );
    let resp = read_response(&mut stdout, 2);

    let body = resp["result"]
        .as_str()
        .expect("executeCommand result must be a string, not null");
    assert!(
        body.contains("${1:"),
        "body must contain tabstop placeholders; got: {body:?}"
    );
    assert!(
        body.contains("$0"),
        "body must contain final cursor tabstop; got: {body:?}"
    );

    shutdown_server(stdin, &mut stdout, 3, &mut child);
}
