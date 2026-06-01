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

/// Context for a prefix trigger `<trigger>` (bare identifier, not after a dot).
///
/// Produced by the CST classifier when a `CodeBareIdentifier` site is found;
/// consumed by [`match_prefix`] to produce [`Candidate`]s.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrefixContext {
    /// The identifier text typed by the user (e.g. `"ctor"`, `"if"`).
    pub trigger: String,
    /// Range covering the trigger in the document.
    pub range: Range,
}

/// Whether a [`Rule`] fires as a postfix (`<receiver>.<trigger>`) or a
/// prefix (`<trigger>` without a leading dot) expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleKind {
    /// Rule is triggered after a dot: `<receiver>.<trigger>`.
    #[default]
    Postfix,
    /// Rule is triggered by a bare identifier without a preceding dot.
    Prefix,
}

/// A single template rule loaded from a rule pack.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Rule {
    /// Whether this rule fires as postfix or prefix expansion.
    #[serde(rename = "type", default)]
    pub kind: RuleKind,
    /// Trigger prefix; matched case-insensitively.
    pub trigger: String,
    /// Short label shown in the completion menu.
    pub label: String,
    /// LSP snippet body.
    ///
    /// For postfix rules, `$receiver` is substituted with the receiver expression
    /// text. For prefix rules the body is used verbatim (no `$receiver`).
    pub body: String,
}

/// A single expansion candidate produced by [`match_postfix`] or [`match_prefix`].
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
///
/// # Panics
///
/// Panics if the embedded TOML is malformed — this indicates a compile-time
/// packaging bug, not a runtime condition.
#[must_use]
pub fn built_in_csharp_postfix_rules() -> Vec<Rule> {
    load_rules(include_str!("../../../snippets/csharp/postfix.toml"))
}

/// Returns the built-in C# prefix rule pack, embedded at compile time
/// from `snippets/csharp/prefix.toml`.
///
/// # Panics
///
/// Panics if the embedded TOML is malformed — this indicates a compile-time
/// packaging bug, not a runtime condition.
#[must_use]
pub fn built_in_csharp_prefix_rules() -> Vec<Rule> {
    load_rules(include_str!("../../../snippets/csharp/prefix.toml"))
}

fn load_rules(raw: &str) -> Vec<Rule> {
    #[derive(serde::Deserialize)]
    struct Pack {
        rules: Vec<Rule>,
    }
    toml::from_str::<Pack>(raw)
        .expect("built-in rule pack is valid TOML")
        .rules
}

/// Match `postfix.trigger` (case-insensitive prefix) against `rules`.
///
/// Only rules with `kind == RuleKind::Postfix` are considered.
/// Returns candidates ordered with exact matches first, then alphabetically
/// by trigger.
#[must_use]
pub fn match_postfix(postfix: &PostfixContext, rules: &[Rule]) -> Vec<Candidate> {
    let typed = postfix.trigger.to_ascii_lowercase();
    let mut candidates: Vec<Candidate> = rules
        .iter()
        .filter(|r| {
            r.kind == RuleKind::Postfix
                && r.trigger.to_ascii_lowercase().starts_with(typed.as_str())
        })
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
    sort_candidates(&mut candidates, &typed);
    candidates
}

/// Match `prefix.trigger` (case-insensitive prefix) against `rules`.
///
/// Only rules with `kind == RuleKind::Prefix` are considered.
/// Returns candidates ordered with exact matches first, then alphabetically
/// by trigger.
#[must_use]
pub fn match_prefix(prefix: &PrefixContext, rules: &[Rule]) -> Vec<Candidate> {
    let typed = prefix.trigger.to_ascii_lowercase();
    let mut candidates: Vec<Candidate> = rules
        .iter()
        .filter(|r| {
            r.kind == RuleKind::Prefix && r.trigger.to_ascii_lowercase().starts_with(typed.as_str())
        })
        .map(|r| Candidate {
            trigger: r.trigger.clone(),
            label: r.label.clone(),
            edit: TextEdit {
                range: prefix.range,
                new_text: r.body.clone(),
            },
        })
        .collect();
    sort_candidates(&mut candidates, &typed);
    candidates
}

fn sort_candidates(candidates: &mut [Candidate], typed: &str) {
    candidates.sort_by(|a, b| {
        let a_exact = a.trigger.to_ascii_lowercase() == typed;
        let b_exact = b.trigger.to_ascii_lowercase() == typed;
        b_exact
            .cmp(&a_exact)
            .then_with(|| a.trigger.cmp(&b.trigger))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn postfix_ctx(receiver: &str, trigger: &str) -> PostfixContext {
        PostfixContext {
            receiver: receiver.to_owned(),
            trigger: trigger.to_owned(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
            },
        }
    }

    fn prefix_ctx(trigger: &str) -> PrefixContext {
        PrefixContext {
            trigger: trigger.to_owned(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: u32::try_from(trigger.len()).unwrap_or(u32::MAX),
                },
            },
        }
    }

    fn postfix_rule(trigger: &str, body: &str) -> Rule {
        Rule {
            kind: RuleKind::Postfix,
            trigger: trigger.to_owned(),
            label: trigger.to_owned(),
            body: body.to_owned(),
        }
    }

    fn prefix_rule(trigger: &str, body: &str) -> Rule {
        Rule {
            kind: RuleKind::Prefix,
            trigger: trigger.to_owned(),
            label: trigger.to_owned(),
            body: body.to_owned(),
        }
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn match_prefix_returns_prefix_rules_only() {
        let rules = vec![
            postfix_rule("fod", "$receiver.FirstOrDefault()"),
            prefix_rule("if", "if (${1:cond}) {\n    $0\n}"),
            prefix_rule("ctor", "public $1() {\n    $0\n}"),
        ];
        let candidates = match_prefix(&prefix_ctx("i"), &rules);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].trigger, "if");
    }

    #[test]
    fn match_prefix_exact_match_first() {
        let rules = vec![
            prefix_rule("for", "for (…)"),
            prefix_rule("foreach", "foreach (…)"),
        ];
        let candidates = match_prefix(&prefix_ctx("for"), &rules);
        assert_eq!(candidates[0].trigger, "for", "exact match must be first");
    }

    #[test]
    fn match_postfix_ignores_prefix_rules() {
        let rules = vec![
            postfix_rule("fod", "$receiver.FirstOrDefault()"),
            prefix_rule("for", "for (…)"),
        ];
        let candidates = match_postfix(&postfix_ctx("xs", "fo"), &rules);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].trigger, "fod");
    }

    #[test]
    fn built_in_csharp_prefix_rules_loads_without_panic() {
        let rules = built_in_csharp_prefix_rules();
        assert!(!rules.is_empty());
        assert!(rules.iter().all(|r| r.kind == RuleKind::Prefix));
    }

    proptest::proptest! {
        #[test]
        fn match_prefix_never_panics(trigger in ".*", rule_trigger in ".*", body in ".*") {
            let rules = [prefix_rule(&rule_trigger, &body)];
            let _ = match_prefix(&prefix_ctx(&trigger), &rules);
        }
    }
}
