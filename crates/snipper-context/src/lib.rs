//! CST context classifier for the Snipper expansion engine.
//!
//! Classifies the cursor position within a parsed CST to produce a
//! [`Context`] that drives template selection. Depends on `snipper-core`
//! only — no LSP types appear in the public API (INV-5).

#![forbid(unsafe_code)]

use snippercore::Position;

/// Sealing token — prevents external crates from implementing [`Backend`].
mod private {
    pub trait Sealed {}
}

/// Error produced by a [`Backend`] during CST classification.
///
/// `#[non_exhaustive]` — new variants may be added without a major bump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BackendError {
    /// The source text could not be parsed by the backend.
    #[error("parse failed: {reason}")]
    ParseFailed {
        /// Human-readable reason for the failure.
        reason: String,
    },
}

/// CST classification backend (sealed — see ADR-0004).
///
/// Determines the [`LexicalClass`] at a given byte offset in source text.
/// The trait is sealed so downstream crates cannot add implementations;
/// this preserves the prime-directive invariant enforced inside this crate.
///
/// # Thread safety
///
/// Implementations must be `Send + Sync` so a single backend instance
/// can be shared across LSP handler threads without wrapping.
pub trait Backend: private::Sealed + Send + Sync {
    /// Classify the cursor position in `source` at byte `offset`.
    ///
    /// # Errors
    ///
    /// Returns [`BackendError::ParseFailed`] when the backend cannot
    /// produce a valid CST for `source`.
    fn classify(&self, source: &str, offset: usize) -> Result<LexicalClass, BackendError>;
}

/// Tree-sitter-backed CST classifier.
///
/// This is the default backend (enabled by the `backend-treesitter` feature).
/// The real grammar walkers are added per-language; this scaffold always
/// returns [`LexicalClass::Other`].
#[cfg(feature = "backend-treesitter")]
#[derive(Debug, Clone, Copy, Default)]
pub struct TreeSitterBackend {
    _priv: (),
}

#[cfg(feature = "backend-treesitter")]
impl private::Sealed for TreeSitterBackend {}

#[cfg(feature = "backend-treesitter")]
impl Backend for TreeSitterBackend {
    fn classify(&self, _source: &str, _offset: usize) -> Result<LexicalClass, BackendError> {
        // TODO(tree-sitter): walk the CST at offset and return the real class
        Ok(LexicalClass::Other)
    }
}

#[cfg(feature = "backend-treesitter")]
impl TreeSitterBackend {
    /// Creates a new `TreeSitterBackend`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// The lexical class of the cursor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    pub range: snippercore::Range,
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest::proptest! {
        #[test]
        fn inv1_forbidden_contexts_always_block(
            _s in proptest::string::string_regex(".{0,64}").unwrap()
        ) {
            assert!(LexicalClass::StringLiteral.forbids_expansion());
            assert!(LexicalClass::Comment.forbids_expansion());
            assert!(LexicalClass::IdentifierDeclaration.forbids_expansion());
        }
    }
}
