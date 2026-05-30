//! CST context classifier for the Snipper expansion engine.
//!
//! Classifies the cursor position within a parsed CST to produce a
//! [`Context`] that drives template selection. Depends on `snipper-core`
//! only — no LSP types appear in the public API (INV-5).

#![forbid(unsafe_code)]

use snipper_core::Position;

/// The lexical class of the cursor position.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum LexicalClass {
    /// Cursor is in executable code, after a dot trigger.
    CodeAfterDot,
    /// Cursor is inside a string literal — expansion is forbidden (prime directive).
    StringLiteral,
    /// Cursor is inside a comment — expansion is forbidden (prime directive).
    Comment,
    /// Cursor is at an identifier declaration site — expansion is forbidden.
    IdentifierDeclaration,
    /// Any other lexical context; not eligible for expansion.
    Other,
}

impl LexicalClass {
    /// Returns `true` when expansion candidates must be empty (prime directive).
    #[must_use]
    pub fn forbids_expansion(&self) -> bool {
        matches!(
            self,
            Self::StringLiteral | Self::Comment | Self::IdentifierDeclaration
        )
    }
}

/// The classified context at a cursor position.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Context {
    /// Source language identifier (e.g. `"rust"`, `"csharp"`).
    pub language: String,
    /// Cursor position in the document.
    pub cursor: Position,
    /// Lexical classification of the cursor site.
    pub lexical: LexicalClass,
    /// Postfix context when the cursor follows a dot-trigger pattern.
    pub postfix: Option<PostfixContext>,
}

/// Context for a postfix trigger `<expr>.<trigger>`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostfixContext {
    /// The receiver expression text.
    pub receiver: String,
    /// The trigger word (e.g. `"fod"`, `"foreach"`).
    pub trigger: String,
    /// Range covering `<receiver>.<trigger>` in the document.
    pub range: snipper_core::Range,
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest::proptest! {
        #[test]
        fn inv1_string_literal_forbids_expansion(_s in proptest::string::string_regex(".{0,64}").unwrap()) {
            assert!(LexicalClass::StringLiteral.forbids_expansion());
            assert!(LexicalClass::Comment.forbids_expansion());
            assert!(LexicalClass::IdentifierDeclaration.forbids_expansion());
        }
    }
}
