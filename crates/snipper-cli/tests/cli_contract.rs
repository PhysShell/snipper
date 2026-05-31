//! Contract tests for the `snipper` command-line interface.

use std::process::{Command, Stdio};

use serde_json::Value;

fn run_snipper(args: &[&str], stdin: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_snipper"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("snipper binary starts");

    {
        use std::io::Write as _;
        let mut child_stdin = child.stdin.take().expect("stdin pipe is available");
        child_stdin
            .write_all(stdin.as_bytes())
            .expect("source text is written to stdin");
    }

    let output = child.wait_with_output().expect("snipper process exits");
    assert!(
        output.status.success(),
        "snipper failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("snipper output is UTF-8")
}

#[test]
fn expand_emits_text_edit_array_contract() {
    let stdout = run_snipper(
        &["expand", "--language", "csharp", "--offset", "17"],
        "var y = users.fod;",
    );
    let json: Value = serde_json::from_str(&stdout).expect("expand output is JSON");
    let edits = json.as_array().expect("expand output is a JSON array");

    assert_eq!(edits.len(), 1);
    assert!(edits[0].get("range").is_some());
    assert_eq!(edits[0]["new_text"], "users.FirstOrDefault()");
    assert!(
        edits[0].get("trigger").is_none(),
        "expand must emit TextEdit[] rather than Candidate[]"
    );
    assert!(
        edits[0].get("label").is_none(),
        "expand must emit TextEdit[] rather than Candidate[]"
    );
    assert!(
        edits[0].get("edit").is_none(),
        "expand must emit TextEdit[] rather than Candidate[]"
    );
}

#[test]
fn context_json_includes_postfix_replacement_range() {
    let stdout = run_snipper(
        &[
            "context",
            "--language",
            "csharp",
            "--offset",
            "17",
            "--format",
            "json",
        ],
        "var y = users.fod;",
    );
    let json: Value = serde_json::from_str(&stdout).expect("context output is JSON");

    assert_eq!(json["postfix"]["receiver"], "users");
    assert_eq!(json["postfix"]["trigger"], "fod");
    assert_eq!(json["postfix"]["range"]["start"]["line"], 0);
    assert_eq!(json["postfix"]["range"]["start"]["character"], 8);
    assert_eq!(json["postfix"]["range"]["end"]["line"], 0);
    assert_eq!(json["postfix"]["range"]["end"]["character"], 17);
}
