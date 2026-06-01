//! End-to-end LSP smoke test using a minimal client fixture.
//!
//! Starts the `snipper-lsp` binary over stdio, drives the standard
//! initialize → didOpen → completion → shutdown sequence, and verifies
//! that typing `users.fod` returns a `FirstOrDefault()` completion item.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

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

#[test]
fn lsp_completion_smoke() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_snipper-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("snipper-lsp binary starts");

    let mut stdin = child.stdin.take().unwrap();
    let stdout_raw = child.stdout.take().unwrap();
    let mut stdout = BufReader::new(stdout_raw);

    // 1. initialize
    let init_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": null,
            "capabilities": {}
        }
    })
    .to_string();
    write_msg(&mut stdin, &init_body);
    let init_resp = read_response(&mut stdout, 1);
    assert!(
        init_resp["result"]["capabilities"].is_object(),
        "initialize returns capabilities"
    );

    // 2. initialized notification (no response)
    let initialized_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    })
    .to_string();
    write_msg(&mut stdin, &initialized_body);

    // 3. textDocument/didOpen — cursor at end of "users.fod" (character 17)
    let source = "var y = users.fod;";
    let did_open_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///test.cs",
                "languageId": "csharp",
                "version": 1,
                "text": source
            }
        }
    })
    .to_string();
    write_msg(&mut stdin, &did_open_body);

    // 4. textDocument/completion — position after "fod" (line 0, char 17)
    let completion_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/completion",
        "params": {
            "textDocument": { "uri": "file:///test.cs" },
            "position": { "line": 0, "character": 17 }
        }
    })
    .to_string();
    write_msg(&mut stdin, &completion_body);
    let completion_resp = read_response(&mut stdout, 2);

    let items = completion_resp["result"]
        .as_array()
        .expect("completion result is a JSON array");
    assert!(!items.is_empty(), "at least one completion item returned");

    let labels: Vec<&str> = items.iter().filter_map(|i| i["label"].as_str()).collect();
    assert!(
        labels.iter().any(|l| l.contains("FirstOrDefault")),
        "fod → FirstOrDefault must appear in completions; got: {labels:?}"
    );

    // 5. shutdown
    let shutdown_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "shutdown",
        "params": null
    })
    .to_string();
    write_msg(&mut stdin, &shutdown_body);
    let _ = read_response(&mut stdout, 3);

    // 6. exit notification
    let exit_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "exit",
        "params": null
    })
    .to_string();
    write_msg(&mut stdin, &exit_body);
    drop(stdin);

    child.wait().expect("snipper-lsp exits cleanly");
}
