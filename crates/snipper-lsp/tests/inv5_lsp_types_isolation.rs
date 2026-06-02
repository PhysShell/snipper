//! INV-5: `lsp_types` must not appear in the public API of `snipper-core` or
//! `snipper-context`. These crates are the portable core shared with the Reactor
//! mobile editor; LSP is an adapter concern confined to `snipper-lsp`.

const CORE_CARGO_TOML: &str = include_str!("../../snipper-core/Cargo.toml");
const CONTEXT_CARGO_TOML: &str = include_str!("../../snipper-context/Cargo.toml");

#[test]
fn snipper_core_does_not_depend_on_lsp_types() {
    assert!(
        !CORE_CARGO_TOML.contains("lsp-types"),
        "snipper-core Cargo.toml must not reference lsp-types (INV-5)"
    );
}

#[test]
fn snipper_context_does_not_depend_on_lsp_types() {
    assert!(
        !CONTEXT_CARGO_TOML.contains("lsp-types"),
        "snipper-context Cargo.toml must not reference lsp-types (INV-5)"
    );
}
