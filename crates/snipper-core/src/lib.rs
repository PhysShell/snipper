//! Core domain types for the Snipper expansion engine.
//!
//! This crate contains only value objects and domain logic — no I/O,
//! no LSP types, no Tree-sitter dependencies. It is the portable
//! foundation shared by the LSP adapter and the Reactor mobile editor.

#![forbid(unsafe_code)]

/// A position in a text document expressed as line + UTF-16 character offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Position {
    /// Zero-based line number.
    pub line: u32,
    /// Zero-based UTF-16 character offset.
    pub character: u32,
}

/// A half-open range between two [`Position`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Range {
    /// Start of the range (inclusive).
    pub start: Position,
    /// End of the range (exclusive).
    pub end: Position,
}

/// A single text replacement to apply to a document.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TextEdit {
    /// The range to replace.
    pub range: Range,
    /// Replacement text; may contain LSP snippet tabstops (`$0`, `${1:...}`).
    pub new_text: String,
}
