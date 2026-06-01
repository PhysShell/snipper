//! Golden-file regression tests for the C# CST classifier.
#![cfg(feature = "lang-csharp")]
use snipper_context::{Backend as _, LexicalClass, TreeSitterBackend};

#[derive(serde::Deserialize)]
struct Fixture {
    source: String,
    offset: usize,
    expected: String,
}

fn expected_class(s: &str) -> LexicalClass {
    match s {
        "CodeAfterDot" => LexicalClass::CodeAfterDot,
        "CodeBareIdentifier" => LexicalClass::CodeBareIdentifier,
        "StringLiteral" => LexicalClass::StringLiteral,
        "Comment" => LexicalClass::Comment,
        "IdentifierDeclaration" => LexicalClass::IdentifierDeclaration,
        "Other" => LexicalClass::Other,
        other => panic!("unknown LexicalClass in fixture: {other}"),
    }
}

fn run_golden(name: &str) {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/snipper-context has parent")
        .parent()
        .expect("crates has parent (workspace root)");
    let path = workspace_root
        .join("tests/golden/csharp")
        .join(format!("{name}.json"));
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"));
    let fixture: Fixture =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("cannot parse {path:?}: {e}"));
    let backend = TreeSitterBackend::csharp();
    let got = backend
        .classify(&fixture.source, fixture.offset)
        .unwrap_or_else(|e| panic!("classify failed for {name}: {e}"));
    assert_eq!(
        got.lexical,
        expected_class(&fixture.expected),
        "golden mismatch for '{name}'"
    );
}

#[test]
fn golden_code_after_dot() {
    run_golden("code_after_dot");
}

#[test]
fn golden_string_literal() {
    run_golden("string_literal");
}

#[test]
fn golden_line_comment() {
    run_golden("line_comment");
}

#[test]
fn golden_block_comment() {
    run_golden("block_comment");
}

#[test]
fn golden_variable_decl() {
    run_golden("variable_decl");
}

#[test]
fn golden_method_decl() {
    run_golden("method_decl");
}

#[test]
fn golden_other() {
    run_golden("other");
}

#[test]
fn golden_bare_identifier() {
    run_golden("bare_identifier");
}

#[test]
fn golden_bare_identifier_has_prefix_context() {
    use snipper_context::PrefixContext;
    let backend = TreeSitterBackend::csharp();
    let source = "ctor";
    let classified = backend.classify(source, 4).expect("classify ok");
    assert_eq!(classified.lexical, LexicalClass::CodeBareIdentifier);
    let _: &PrefixContext = classified
        .prefix
        .as_ref()
        .expect("CodeBareIdentifier must carry PrefixContext");
}

#[test]
fn golden_code_after_dot_has_postfix_context() {
    let backend = TreeSitterBackend::csharp();
    let source = "var y = users.fod;";
    let offset = source.find("fod").unwrap() + "fod".len();
    let classified = backend.classify(source, offset).expect("classify failed");
    assert_eq!(classified.lexical, LexicalClass::CodeAfterDot);
    let postfix = classified
        .postfix
        .expect("code_after_dot must carry PostfixContext");
    assert_eq!(postfix.receiver, "users");
    assert_eq!(postfix.trigger, "fod");
}
