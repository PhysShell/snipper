//! CST context classifier for the Snipper expansion engine.
//!
//! Classifies the cursor position within a parsed CST to produce a
//! [`Context`] that drives template selection. Depends on `snipper-core`
//! only — no LSP types appear in the public API (INV-5).

#![forbid(unsafe_code)]

use snippercore::Position;
#[cfg(feature = "backend-treesitter")]
use snippercore::Range;
pub use snippercore::{PostfixContext, PrefixContext};

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

/// Result of classifying the cursor position in source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedContext {
    /// Lexical class of the cursor site.
    pub lexical: LexicalClass,
    /// Postfix context; `Some` only when `lexical == LexicalClass::CodeAfterDot`.
    pub postfix: Option<PostfixContext>,
    /// Prefix context; `Some` only when `lexical == LexicalClass::CodeBareIdentifier`.
    pub prefix: Option<PrefixContext>,
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
    /// `offset` is an LSP insertion-point cursor: it sits after the last
    /// typed character, not on it.
    ///
    /// # Errors
    ///
    /// Returns [`BackendError::ParseFailed`] when the backend cannot
    /// produce a valid CST for `source`.
    fn classify(&self, source: &str, offset: usize) -> Result<ClassifiedContext, BackendError>;
}

/// Tree-sitter-backed CST classifier.
///
/// Construct via a language-specific factory (e.g. [`TreeSitterBackend::csharp`]).
/// Each instance is bound to one language; the `Backend` trait is sealed so only
/// factories inside this crate can create valid instances.
#[cfg(feature = "backend-treesitter")]
pub struct TreeSitterBackend {
    language: tree_sitter::Language,
}

#[cfg(feature = "backend-treesitter")]
impl std::fmt::Debug for TreeSitterBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeSitterBackend").finish_non_exhaustive()
    }
}

#[cfg(feature = "backend-treesitter")]
impl private::Sealed for TreeSitterBackend {}

#[cfg(feature = "backend-treesitter")]
impl Backend for TreeSitterBackend {
    fn classify(&self, source: &str, offset: usize) -> Result<ClassifiedContext, BackendError> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&self.language)
            .map_err(|e| BackendError::ParseFailed {
                reason: e.to_string(),
            })?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| BackendError::ParseFailed {
                reason: "parser timed out or was cancelled".into(),
            })?;
        Ok(classify_at(source, tree.root_node(), offset))
    }
}

#[cfg(feature = "backend-treesitter")]
impl TreeSitterBackend {
    /// Creates a `TreeSitterBackend` for C#.
    #[cfg(feature = "lang-csharp")]
    #[must_use]
    pub fn csharp() -> Self {
        Self {
            language: tree_sitter_c_sharp::LANGUAGE.into(),
        }
    }
}

/// Walk the CST and classify the cursor at `offset` (byte offset).
///
/// Priority order matches the prime directive:
/// 1. String literal           — expansion forbidden
/// 2. Comment                  — expansion forbidden
/// 3. Identifier declaration   — expansion forbidden
/// 4. Code after dot           — postfix trigger site
/// 5. Bare identifier in code  — prefix trigger site
/// 6. Other
#[cfg(feature = "backend-treesitter")]
fn classify_at(source: &str, root: tree_sitter::Node<'_>, offset: usize) -> ClassifiedContext {
    // `offset` is an insertion-point cursor: it sits after the last typed byte.
    // Probe one byte to the left so we land on the token the user just typed.
    let probe = offset.saturating_sub(1);
    let Some(node) = root.descendant_for_byte_range(probe, probe.saturating_add(1)) else {
        return ClassifiedContext {
            lexical: LexicalClass::Other,
            postfix: None,
            prefix: None,
        };
    };

    // Walk ancestors checking for prime-directive contexts first.
    let mut cur = node;
    loop {
        if is_string_node(cur.kind()) {
            return ClassifiedContext {
                lexical: LexicalClass::StringLiteral,
                postfix: None,
                prefix: None,
            };
        }
        if is_comment_node(cur.kind()) {
            return ClassifiedContext {
                lexical: LexicalClass::Comment,
                postfix: None,
                prefix: None,
            };
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }

    if is_declaration_name(node) {
        return ClassifiedContext {
            lexical: LexicalClass::IdentifierDeclaration,
            postfix: None,
            prefix: None,
        };
    }

    if is_postfix_trigger(node) {
        let postfix = extract_postfix_context(source, node);
        return ClassifiedContext {
            lexical: LexicalClass::CodeAfterDot,
            postfix,
            prefix: None,
        };
    }

    // Text-based prefix extraction: if the characters immediately before the
    // cursor form an identifier-like word (letter/underscore start), this is a
    // prefix trigger site.  We do this after all CST prime-directive checks so
    // string/comment/declaration sites are already excluded.
    if let Some(prefix) = extract_prefix_context(source, offset) {
        return ClassifiedContext {
            lexical: LexicalClass::CodeBareIdentifier,
            postfix: None,
            prefix: Some(prefix),
        };
    }

    ClassifiedContext {
        lexical: LexicalClass::Other,
        postfix: None,
        prefix: None,
    }
}

/// Extract [`PostfixContext`] from the `member_access_expression` parent of `name_node`.
#[cfg(feature = "backend-treesitter")]
fn extract_postfix_context(
    source: &str,
    name_node: tree_sitter::Node<'_>,
) -> Option<PostfixContext> {
    let parent = name_node.parent()?; // member_access_expression
    let receiver_node = parent.child_by_field_name("expression")?;
    let receiver = source.get(receiver_node.byte_range())?.to_owned();
    let trigger = source.get(name_node.byte_range())?.to_owned();
    let start = byte_to_position(source, parent.start_byte());
    let end = byte_to_position(source, parent.end_byte());
    Some(PostfixContext {
        receiver,
        trigger,
        range: Range { start, end },
    })
}

/// Extract [`PrefixContext`] from the word immediately before `offset` in `source`.
///
/// Uses source-text scanning (not the CST) to handle error-recovery nodes and
/// keyword tokens that tree-sitter does not classify as `identifier`.
/// Returns `None` when there is no identifier-like word before the cursor.
#[cfg(feature = "backend-treesitter")]
fn extract_prefix_context(source: &str, offset: usize) -> Option<PrefixContext> {
    // Snap to the nearest valid char boundary at or before `offset`.
    // The byte offset from the LSP layer may land inside a multi-byte char.
    let clamped = {
        let c = offset.min(source.len());
        (0..=c)
            .rev()
            .find(|&i| source.is_char_boundary(i))
            .unwrap_or(0)
    };
    let before = &source[..clamped];

    // Scan backwards over identifier chars to find where the word starts.
    let word_start = before
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
        .map_or(0, |(i, c)| i + c.len_utf8());

    let trigger = &source[word_start..clamped];
    if trigger.is_empty() {
        return None;
    }
    // Must start with a letter or underscore (not a digit).
    let first = trigger.chars().next()?;
    if !first.is_alphabetic() && first != '_' {
        return None;
    }
    let start = byte_to_position(source, word_start);
    let end = byte_to_position(source, clamped);
    Some(PrefixContext {
        trigger: trigger.to_owned(),
        range: Range { start, end },
    })
}

/// Convert a byte offset in `source` to an LSP line / UTF-16 character [`Position`].
#[cfg(feature = "backend-treesitter")]
fn byte_to_position(source: &str, byte: usize) -> Position {
    let clamped = byte.min(source.len());
    let before = &source[..clamped];
    let line = u32::try_from(before.bytes().filter(|b| *b == b'\n').count()).unwrap_or(u32::MAX);
    let last_nl = before.rfind('\n').map_or(0, |i| i + 1);
    let character =
        u32::try_from(source[last_nl..clamped].encode_utf16().count()).unwrap_or(u32::MAX);
    Position { line, character }
}

/// C# string-literal node kinds (tree-sitter-c-sharp grammar).
#[cfg(feature = "backend-treesitter")]
fn is_string_node(kind: &str) -> bool {
    matches!(
        kind,
        "string_literal"
            | "verbatim_string_literal"
            | "interpolated_string_expression"
            | "interpolated_verbatim_string_expression"
            | "character_literal"
            | "raw_string_literal"
    )
}

/// C# comment node kinds.
#[cfg(feature = "backend-treesitter")]
fn is_comment_node(kind: &str) -> bool {
    kind == "comment"
}

/// Returns `true` when `node` is the declared name identifier in a
/// declaration context (variable, method, class, parameter, …).
#[cfg(feature = "backend-treesitter")]
fn is_declaration_name(node: tree_sitter::Node<'_>) -> bool {
    if node.kind() != "identifier" {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };

    // type_parameter: the identifier IS the whole node's content (no named field).
    if parent.kind() == "type_parameter" {
        return true;
    }

    // foreach_statement uses the "left" field for the iteration variable.
    if parent.kind() == "foreach_statement" {
        return parent
            .child_by_field_name("left")
            .is_some_and(|n| n.id() == node.id());
    }

    // All other standard declaration kinds use the "name" field.
    if matches!(
        parent.kind(),
        "variable_declarator"
            | "method_declaration"
            | "local_function_statement"
            | "constructor_declaration"
            | "destructor_declaration"
            | "class_declaration"
            | "struct_declaration"
            | "interface_declaration"
            | "record_declaration"
            | "record_struct_declaration"
            | "enum_declaration"
            | "enum_member_declaration"
            | "delegate_declaration"
            | "event_declaration"
            | "property_declaration"
            | "parameter"
            | "catch_declaration"
            | "namespace_declaration"
    ) {
        return parent
            .child_by_field_name("name")
            .is_some_and(|n| n.id() == node.id());
    }

    false
}

/// Returns `true` when `node` is the trigger identifier in a postfix
/// expression (`<receiver>.<trigger>`).
#[cfg(feature = "backend-treesitter")]
fn is_postfix_trigger(node: tree_sitter::Node<'_>) -> bool {
    if node.kind() != "identifier" {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "member_access_expression" {
        return false;
    }
    parent
        .child_by_field_name("name")
        .is_some_and(|n| n.id() == node.id())
}

/// The lexical class of the cursor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum LexicalClass {
    /// Cursor is in executable code, after a dot trigger.
    CodeAfterDot,
    /// Cursor is on a bare identifier in executable code (not after a dot).
    ///
    /// This is the prefix expansion trigger site.
    CodeBareIdentifier,
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
    pub const fn forbids_expansion(&self) -> bool {
        matches!(
            self,
            Self::StringLiteral | Self::Comment | Self::IdentifierDeclaration
        )
    }
}

/// The classified context at a cursor position (for the LSP adapter layer).
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
    /// Prefix context when the cursor is on a bare identifier in code.
    pub prefix: Option<PrefixContext>,
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

    // INV-2: string literals are never eligible for expansion.
    #[cfg(feature = "lang-csharp")]
    proptest::proptest! {
        #[test]
        fn inv2_csharp_string_literal_blocks_expansion(trigger in "[a-z]{2,8}") {
            let backend = TreeSitterBackend::csharp();
            let prefix = r#"var x = ""#;
            let source = format!(r#"{prefix}{trigger}";"#);
            let offset = prefix.len() + trigger.len();
            let classified = backend.classify(&source, offset).unwrap();
            assert!(
                classified.lexical.forbids_expansion(),
                "expected forbidden inside string, got {:?}",
                classified.lexical
            );
        }
    }

    // INV-3: single-line comments are never eligible.
    #[cfg(feature = "lang-csharp")]
    proptest::proptest! {
        #[test]
        fn inv3_csharp_comment_blocks_expansion(trigger in "[a-z]{2,8}") {
            let backend = TreeSitterBackend::csharp();
            let source = format!("// {trigger}");
            let offset = source.find(&trigger).unwrap() + trigger.len();
            let classified = backend.classify(&source, offset).unwrap();
            assert!(
                classified.lexical.forbids_expansion(),
                "expected forbidden inside comment, got {:?}",
                classified.lexical
            );
        }
    }

    #[cfg(feature = "lang-csharp")]
    #[test]
    fn csharp_code_after_dot_is_classified() {
        let backend = TreeSitterBackend::csharp();
        let source = "var y = users.fod;";
        let offset = source.find("fod").unwrap() + "fod".len();
        let classified = backend.classify(source, offset).unwrap();
        assert_eq!(classified.lexical, LexicalClass::CodeAfterDot);
        let postfix = classified
            .postfix
            .expect("CodeAfterDot must have PostfixContext");
        assert_eq!(postfix.receiver, "users");
        assert_eq!(postfix.trigger, "fod");
    }

    #[cfg(feature = "lang-csharp")]
    #[test]
    fn csharp_bare_identifier_is_classified_as_prefix_site() {
        let backend = TreeSitterBackend::csharp();
        // A standalone expression statement: tree-sitter parses `ctor` as an identifier.
        let source = "ctor";
        let offset = source.len();
        let classified = backend.classify(source, offset).unwrap();
        assert_eq!(classified.lexical, LexicalClass::CodeBareIdentifier);
        let prefix = classified
            .prefix
            .expect("CodeBareIdentifier must have PrefixContext");
        assert_eq!(prefix.trigger, "ctor");
    }

    #[cfg(feature = "lang-csharp")]
    #[test]
    fn csharp_variable_declaration_name_is_blocked() {
        let backend = TreeSitterBackend::csharp();
        let source = "int myVar = 0;";
        let offset = source.find("myVar").unwrap() + "myVar".len();
        assert_eq!(
            backend.classify(source, offset).unwrap().lexical,
            LexicalClass::IdentifierDeclaration
        );
    }

    #[cfg(feature = "lang-csharp")]
    #[test]
    fn csharp_method_name_is_blocked() {
        let backend = TreeSitterBackend::csharp();
        let source = "void MyMethod() {}";
        let offset = source.find("MyMethod").unwrap() + "MyMethod".len();
        assert_eq!(
            backend.classify(source, offset).unwrap().lexical,
            LexicalClass::IdentifierDeclaration
        );
    }

    #[cfg(feature = "lang-csharp")]
    #[test]
    fn csharp_block_comment_is_blocked() {
        let backend = TreeSitterBackend::csharp();
        let source = "/* hello fod world */";
        let offset = source.find("fod").unwrap() + "fod".len();
        assert_eq!(
            backend.classify(source, offset).unwrap().lexical,
            LexicalClass::Comment
        );
    }

    #[cfg(feature = "lang-csharp")]
    #[test]
    fn code_bare_identifier_does_not_forbid_expansion() {
        assert!(!LexicalClass::CodeBareIdentifier.forbids_expansion());
    }
}
