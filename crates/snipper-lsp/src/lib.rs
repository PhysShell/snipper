//! LSP adapter for the Snipper expansion engine.
//!
//! This crate bridges `snipper-context` and `snipper-core` to the Language
//! Server Protocol. LSP-specific types live **here only** — they must not
//! appear in `snipper-core` or `snipper-context` public APIs (INV-5).
