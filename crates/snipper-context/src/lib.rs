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

// ---------------------------------------------------------------------------
// Language rules — static configuration for each supported grammar
// ---------------------------------------------------------------------------

/// A single rule for recognising a declaration-name site.
///
/// When a CST node matches `node_kinds` and its parent matches `parent_kind`,
/// the node is a declaration name. If `field_name` is `Some`, the node must
/// additionally be the child at that named field; `None` means any child
/// (used for grammars where the name is the parent node itself, e.g. C#
/// `type_parameter`).
#[cfg(feature = "backend-treesitter")]
struct DeclNameRule {
    parent_kind: &'static str,
    field_name: Option<&'static str>,
    node_kinds: &'static [&'static str],
}

/// Language-specific CST node type configuration.
///
/// A `&'static LanguageRules` is stored in each [`TreeSitterBackend`] instance
/// and threaded through the classifier so all prime-directive checks and
/// postfix-trigger detection are language-agnostic.
#[cfg(feature = "backend-treesitter")]
struct LanguageRules {
    /// Node kinds that represent string literals (prime directive).
    string_node_kinds: &'static [&'static str],
    /// Node kinds that represent comments (prime directive).
    comment_node_kinds: &'static [&'static str],
    /// Kind of the member-access expression node.
    postfix_member_kind: &'static str,
    /// Named field within the member-access node for the trigger identifier.
    postfix_trigger_field: &'static str,
    /// Node kinds acceptable as the trigger identifier.
    postfix_trigger_kinds: &'static [&'static str],
    /// Named field within the member-access node for the receiver expression.
    postfix_receiver_field: &'static str,
    /// Rules for recognising declaration-name sites.
    decl_name_rules: &'static [DeclNameRule],
}

// C# language rules (tree-sitter-c-sharp grammar)
#[cfg(feature = "lang-csharp")]
static CSHARP_RULES: LanguageRules = LanguageRules {
    string_node_kinds: &[
        "string_literal",
        "verbatim_string_literal",
        "interpolated_string_expression",
        "interpolated_verbatim_string_expression",
        "character_literal",
        "raw_string_literal",
    ],
    comment_node_kinds: &["comment"],
    postfix_member_kind: "member_access_expression",
    postfix_trigger_field: "name",
    postfix_trigger_kinds: &["identifier"],
    postfix_receiver_field: "expression",
    decl_name_rules: &[
        // type_parameter: the identifier IS the type param — no field check.
        DeclNameRule {
            parent_kind: "type_parameter",
            field_name: None,
            node_kinds: &["identifier"],
        },
        // foreach_statement uses "left" for the loop variable.
        DeclNameRule {
            parent_kind: "foreach_statement",
            field_name: Some("left"),
            node_kinds: &["identifier"],
        },
        // All other standard declaration kinds use the "name" field.
        DeclNameRule {
            parent_kind: "variable_declarator",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "method_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "local_function_statement",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "constructor_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "destructor_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "class_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "struct_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "interface_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "record_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "record_struct_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "enum_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "enum_member_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "delegate_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "event_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "property_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "parameter",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "catch_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "namespace_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
    ],
};

// TypeScript language rules (tree-sitter-typescript grammar)
#[cfg(feature = "lang-typescript")]
static TYPESCRIPT_RULES: LanguageRules = LanguageRules {
    string_node_kinds: &["string", "template_string"],
    comment_node_kinds: &["comment"],
    postfix_member_kind: "member_expression",
    postfix_trigger_field: "property",
    postfix_trigger_kinds: &["property_identifier"],
    postfix_receiver_field: "object",
    decl_name_rules: &[
        DeclNameRule {
            parent_kind: "function_declaration",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "variable_declarator",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "class_declaration",
            field_name: Some("name"),
            node_kinds: &["type_identifier"],
        },
        DeclNameRule {
            parent_kind: "method_definition",
            field_name: Some("name"),
            node_kinds: &["property_identifier"],
        },
        DeclNameRule {
            parent_kind: "required_parameter",
            field_name: Some("pattern"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "optional_parameter",
            field_name: Some("pattern"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "interface_declaration",
            field_name: Some("name"),
            node_kinds: &["type_identifier"],
        },
        DeclNameRule {
            parent_kind: "type_alias_declaration",
            field_name: Some("name"),
            node_kinds: &["type_identifier"],
        },
        DeclNameRule {
            parent_kind: "function_signature",
            field_name: Some("name"),
            node_kinds: &["identifier"],
        },
        DeclNameRule {
            parent_kind: "abstract_method_signature",
            field_name: Some("name"),
            node_kinds: &["property_identifier"],
        },
    ],
};

// ---------------------------------------------------------------------------
// Tree-sitter backend
// ---------------------------------------------------------------------------

/// Tree-sitter-backed CST classifier.
///
/// Construct via a language-specific factory (e.g. [`TreeSitterBackend::csharp`]).
/// Each instance is bound to one language; the `Backend` trait is sealed so only
/// factories inside this crate can create valid instances.
#[cfg(feature = "backend-treesitter")]
pub struct TreeSitterBackend {
    language: tree_sitter::Language,
    rules: &'static LanguageRules,
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
        Ok(classify_at(source, tree.root_node(), offset, self.rules))
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
            rules: &CSHARP_RULES,
        }
    }

    /// Creates a `TreeSitterBackend` for TypeScript.
    #[cfg(feature = "lang-typescript")]
    #[must_use]
    pub fn typescript() -> Self {
        Self {
            language: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            rules: &TYPESCRIPT_RULES,
        }
    }
}

// ---------------------------------------------------------------------------
// Core classifier
// ---------------------------------------------------------------------------

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
fn classify_at(
    source: &str,
    root: tree_sitter::Node<'_>,
    offset: usize,
    rules: &LanguageRules,
) -> ClassifiedContext {
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
        if rules.string_node_kinds.contains(&cur.kind()) {
            return ClassifiedContext {
                lexical: LexicalClass::StringLiteral,
                postfix: None,
                prefix: None,
            };
        }
        if rules.comment_node_kinds.contains(&cur.kind()) {
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

    if is_declaration_name(node, rules) {
        return ClassifiedContext {
            lexical: LexicalClass::IdentifierDeclaration,
            postfix: None,
            prefix: None,
        };
    }

    if is_postfix_trigger(node, rules) {
        let postfix = extract_postfix_context(source, node, rules);
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

/// Extract [`PostfixContext`] from the member-access parent of `name_node`.
#[cfg(feature = "backend-treesitter")]
fn extract_postfix_context(
    source: &str,
    name_node: tree_sitter::Node<'_>,
    rules: &LanguageRules,
) -> Option<PostfixContext> {
    let parent = name_node.parent()?;
    let receiver_node = parent.child_by_field_name(rules.postfix_receiver_field)?;
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

/// Returns `true` when `node` is the declared name identifier in a declaration
/// context according to the language-specific `rules`.
#[cfg(feature = "backend-treesitter")]
fn is_declaration_name(node: tree_sitter::Node<'_>, rules: &LanguageRules) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    for rule in rules.decl_name_rules {
        if parent.kind() != rule.parent_kind {
            continue;
        }
        if !rule.node_kinds.contains(&node.kind()) {
            continue;
        }
        match rule.field_name {
            // No field check — parent + node kind match is sufficient.
            None => return true,
            Some(field) => {
                if parent
                    .child_by_field_name(field)
                    .is_some_and(|n| n.id() == node.id())
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns `true` when `node` is the trigger identifier in a postfix
/// expression (`<receiver>.<trigger>`) according to the language-specific `rules`.
#[cfg(feature = "backend-treesitter")]
fn is_postfix_trigger(node: tree_sitter::Node<'_>, rules: &LanguageRules) -> bool {
    if !rules.postfix_trigger_kinds.contains(&node.kind()) {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != rules.postfix_member_kind {
        return false;
    }
    parent
        .child_by_field_name(rules.postfix_trigger_field)
        .is_some_and(|n| n.id() == node.id())
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

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

    // TypeScript classifier tests
    #[cfg(feature = "lang-typescript")]
    #[test]
    fn typescript_code_after_dot_is_classified() {
        let backend = TreeSitterBackend::typescript();
        let source = "users.map";
        let offset = source.find("map").unwrap() + "map".len();
        let classified = backend.classify(source, offset).unwrap();
        assert_eq!(classified.lexical, LexicalClass::CodeAfterDot);
        let postfix = classified
            .postfix
            .expect("CodeAfterDot must have PostfixContext");
        assert_eq!(postfix.receiver, "users");
        assert_eq!(postfix.trigger, "map");
    }

    #[cfg(feature = "lang-typescript")]
    #[test]
    fn typescript_string_literal_blocks_expansion() {
        let backend = TreeSitterBackend::typescript();
        let source = r#"const s = "hello";"#;
        let offset = source.find("hello").unwrap() + "hello".len();
        assert!(backend
            .classify(source, offset)
            .unwrap()
            .lexical
            .forbids_expansion());
    }

    #[cfg(feature = "lang-typescript")]
    #[test]
    fn typescript_comment_blocks_expansion() {
        let backend = TreeSitterBackend::typescript();
        let source = "// hello world";
        let offset = source.len();
        assert!(backend
            .classify(source, offset)
            .unwrap()
            .lexical
            .forbids_expansion());
    }

    #[cfg(feature = "lang-typescript")]
    #[test]
    fn typescript_function_name_is_blocked() {
        let backend = TreeSitterBackend::typescript();
        let source = "function myFunc() {}";
        let offset = source.find("myFunc").unwrap() + "myFunc".len();
        assert_eq!(
            backend.classify(source, offset).unwrap().lexical,
            LexicalClass::IdentifierDeclaration
        );
    }

    #[cfg(feature = "lang-typescript")]
    #[test]
    fn typescript_bare_identifier_is_prefix_site() {
        let backend = TreeSitterBackend::typescript();
        let source = "ctor";
        let offset = source.len();
        let classified = backend.classify(source, offset).unwrap();
        assert_eq!(classified.lexical, LexicalClass::CodeBareIdentifier);
        assert_eq!(
            classified.prefix.expect("must have PrefixContext").trigger,
            "ctor"
        );
    }
}
