//! Golden-file regression tests for the TypeScript CST classifier.
#![cfg(feature = "lang-typescript")]
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
        .join("tests/golden/typescript")
        .join(format!("{name}.json"));
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"));
    let fixture: Fixture =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("cannot parse {path:?}: {e}"));
    let backend = TreeSitterBackend::typescript();
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
fn golden_function_decl() {
    run_golden("function_decl");
}

#[test]
fn golden_variable_decl() {
    run_golden("variable_decl");
}

#[test]
fn golden_bare_identifier() {
    run_golden("bare_identifier");
}

#[test]
fn golden_code_after_dot_has_postfix_context() {
    let backend = TreeSitterBackend::typescript();
    let source = "users.map";
    let offset = source.find("map").unwrap() + "map".len();
    let classified = backend.classify(source, offset).expect("classify failed");
    assert_eq!(classified.lexical, LexicalClass::CodeAfterDot);
    let postfix = classified
        .postfix
        .expect("code_after_dot must carry PostfixContext");
    assert_eq!(postfix.receiver, "users");
    assert_eq!(postfix.trigger, "map");
}

#[test]
fn golden_bare_identifier_has_prefix_context() {
    use snipper_context::PrefixContext;
    let backend = TreeSitterBackend::typescript();
    let source = "ctor";
    let classified = backend.classify(source, 4).expect("classify ok");
    assert_eq!(classified.lexical, LexicalClass::CodeBareIdentifier);
    let _: &PrefixContext = classified
        .prefix
        .as_ref()
        .expect("CodeBareIdentifier must carry PrefixContext");
}
