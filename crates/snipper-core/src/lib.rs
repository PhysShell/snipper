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

/// Context for a postfix trigger `<expr>.<trigger>`.
///
/// Produced by the CST classifier when a `CodeAfterDot` site is found;
/// consumed by [`match_postfix`] to produce [`Candidate`]s.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PostfixContext {
    /// The receiver expression text (e.g. `"users"`).
    pub receiver: String,
    /// The trigger word typed after the dot (e.g. `"fod"`).
    pub trigger: String,
    /// Range covering `<receiver>.<trigger>` in the document.
    pub range: Range,
}

/// A single postfix template rule loaded from a rule pack.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Rule {
    /// Trigger prefix; matched case-insensitively against [`PostfixContext::trigger`].
    pub trigger: String,
    /// Short label shown in the completion menu.
    pub label: String,
    /// LSP snippet body; `$receiver` is substituted with the receiver text.
    pub body: String,
}

/// A single expansion candidate produced by [`match_postfix`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Candidate {
    /// Full trigger that matched.
    pub trigger: String,
    /// Label for the completion menu.
    pub label: String,
    /// Text edit that applies the expansion.
    pub edit: TextEdit,
}

/// Returns the built-in C# postfix rule pack, embedded at compile time
/// from `snippets/csharp/postfix.toml`.
#[must_use]
pub fn built_in_csharp_postfix_rules() -> Vec<Rule> {
    #[derive(serde::Deserialize)]
    struct Pack {
        rules: Vec<Rule>,
    }
    let raw = include_str!("../../../snippets/csharp/postfix.toml");
    let pack: Pack = toml::from_str(raw).expect("built-in snippets/csharp/postfix.toml is valid TOML");
    pack.rules
}

/// Match `postfix.trigger` (case-insensitive prefix) against `rules`.
///
/// Returns candidates ordered with exact matches first, then alphabetically
/// by trigger.
#[must_use]
pub fn match_postfix(postfix: &PostfixContext, rules: &[Rule]) -> Vec<Candidate> {
    let typed = postfix.trigger.to_ascii_lowercase();
    let mut candidates: Vec<Candidate> = rules
        .iter()
        .filter(|r| r.trigger.to_ascii_lowercase().starts_with(&*typed))
        .map(|r| {
            let new_text = r.body.replace("$receiver", &postfix.receiver);
            Candidate {
                trigger: r.trigger.clone(),
                label: r.label.clone(),
                edit: TextEdit {
                    range: postfix.range,
                    new_text,
                },
            }
        })
        .collect();
    candidates.sort_by(|a, b| {
        let a_exact = a.trigger.to_ascii_lowercase() == typed;
        let b_exact = b.trigger.to_ascii_lowercase() == typed;
        b_exact
            .cmp(&a_exact)
            .then_with(|| a.trigger.cmp(&b.trigger))
    });
    candidates
}
