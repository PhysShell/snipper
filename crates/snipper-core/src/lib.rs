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
    /// Type hierarchy of the receiver as returned by the Roslyn sidecar (S8).
    ///
    /// Each string is a fully-qualified type name; the list includes the
    /// concrete type followed by all implemented interfaces and base types.
    /// `None` when the sidecar is unavailable or timed out (CST-only mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver_type: Option<Vec<String>>,
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

/// Context for a surround expansion — wrapping the user's selected text.
///
/// Produced by the LSP adapter when the client provides a non-empty selection
/// and invokes `textDocument/codeAction`; consumed by [`match_surround`] to
/// produce [`Candidate`]s.  The `$selection` placeholder in a rule body is
/// replaced with [`selected_text`](SurroundContext::selected_text).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SurroundContext {
    /// The source text that is currently selected.
    pub selected_text: String,
    /// Range of the selection (replaced in full by the expansion).
    pub range: Range,
}

/// Whether a [`Rule`] fires as a postfix (`<receiver>.<trigger>`), a
/// prefix (`<trigger>` without a leading dot), or a surround (wraps selected
/// text with `$selection` substitution).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum RuleKind {
    /// Rule is triggered after a dot: `<receiver>.<trigger>`.
    #[default]
    Postfix,
    /// Rule is triggered by a bare identifier without a preceding dot.
    Prefix,
    /// Rule is triggered when the user has a non-empty selection and invokes
    /// a code action; `$selection` in the body is replaced with the selected text.
    Surround,
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
    /// Comma-separated list of type-name substrings (case-insensitive).
    ///
    /// A postfix rule with `requires` is offered only when **at least one** keyword
    /// appears in at least one string of `PostfixContext::receiver_type`.  When the
    /// field is absent (or when `receiver_type` is `None`), the rule is always offered.
    #[serde(default)]
    pub requires: Option<String>,
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

/// Returns the built-in C# surround rule pack, embedded at compile time
/// from `snippets/csharp/surround.toml`.
///
/// # Panics
///
/// Panics if the embedded TOML is malformed — this indicates a compile-time
/// packaging bug, not a runtime condition.
#[must_use]
pub fn built_in_csharp_surround_rules() -> Vec<Rule> {
    load_rules(include_str!("../../../snippets/csharp/surround.toml"))
}

/// Returns the built-in TypeScript postfix rule pack, embedded at compile time
/// from `snippets/typescript/postfix.toml`.
///
/// # Panics
///
/// Panics if the embedded TOML is malformed — this indicates a compile-time
/// packaging bug, not a runtime condition.
#[must_use]
pub fn built_in_typescript_postfix_rules() -> Vec<Rule> {
    load_rules(include_str!("../../../snippets/typescript/postfix.toml"))
}

/// Returns the built-in TypeScript prefix rule pack, embedded at compile time
/// from `snippets/typescript/prefix.toml`.
///
/// # Panics
///
/// Panics if the embedded TOML is malformed — this indicates a compile-time
/// packaging bug, not a runtime condition.
#[must_use]
pub fn built_in_typescript_prefix_rules() -> Vec<Rule> {
    load_rules(include_str!("../../../snippets/typescript/prefix.toml"))
}

/// Returns the built-in TypeScript surround rule pack, embedded at compile time
/// from `snippets/typescript/surround.toml`.
///
/// # Panics
///
/// Panics if the embedded TOML is malformed — this indicates a compile-time
/// packaging bug, not a runtime condition.
#[must_use]
pub fn built_in_typescript_surround_rules() -> Vec<Rule> {
    load_rules(include_str!("../../../snippets/typescript/surround.toml"))
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
///
/// Rules with a [`Rule::requires`] constraint are filtered out when
/// `PostfixContext::receiver_type` is known and no keyword matches.  When the
/// receiver type is unknown the rule is still offered (conservative mode).
///
/// Ordering tiers (within each tier: exact trigger first, then alphabetical):
/// 1. Type-confirmed rules (`requires` present and matched).
/// 2. Universal rules (no `requires`).
/// 3. Uncertain rules (`requires` present but type unknown).
#[must_use]
pub fn match_postfix(postfix: &PostfixContext, rules: &[Rule]) -> Vec<Candidate> {
    let typed = postfix.trigger.to_ascii_lowercase();
    let type_known = postfix
        .receiver_type
        .as_ref()
        .is_some_and(|t| !t.is_empty());

    let mut tier0: Vec<Candidate> = vec![];
    let mut tier1: Vec<Candidate> = vec![];
    let mut tier2: Vec<Candidate> = vec![];

    for rule in rules.iter().filter(|r| {
        r.kind == RuleKind::Postfix && r.trigger.to_ascii_lowercase().starts_with(typed.as_str())
    }) {
        if !type_filter_passes(rule, postfix.receiver_type.as_ref()) {
            continue;
        }
        let candidate = Candidate {
            trigger: rule.trigger.clone(),
            label: rule.label.clone(),
            edit: TextEdit {
                range: postfix.range,
                new_text: rule.body.replace("$receiver", &postfix.receiver),
            },
        };
        match (&rule.requires, type_known) {
            (Some(_), true) => tier0.push(candidate),
            (None, _) => tier1.push(candidate),
            (Some(_), false) => tier2.push(candidate),
        }
    }

    sort_candidates(&mut tier0, &typed);
    sort_candidates(&mut tier1, &typed);
    sort_candidates(&mut tier2, &typed);
    tier0.extend(tier1);
    tier0.extend(tier2);
    tier0
}

/// Returns `true` when `rule` should be offered given `receiver_type`.
///
/// Conservative: when the sidecar is unavailable (`None`) or returned an empty
/// type list, all rules pass — the type filter is a "deny" mechanism, not an
/// "allow" mechanism.
fn type_filter_passes(rule: &Rule, receiver_type: Option<&Vec<String>>) -> bool {
    match (&rule.requires, receiver_type) {
        (None, _) | (Some(_), None) => true,
        (Some(_), Some(types)) if types.is_empty() => true,
        (Some(req), Some(types)) => {
            let req_lc = req.to_ascii_lowercase();
            req_lc.split(',').map(str::trim).any(|kw| {
                !kw.is_empty() && types.iter().any(|t| t.to_ascii_lowercase().contains(kw))
            })
        }
    }
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

/// Match all surround rules against `ctx`, substituting `$selection` in the body.
///
/// All rules with `kind == RuleKind::Surround` are returned without trigger
/// filtering — the full list is offered to the user as code actions.
#[must_use]
pub fn match_surround(ctx: &SurroundContext, rules: &[Rule]) -> Vec<Candidate> {
    rules
        .iter()
        .filter(|r| r.kind == RuleKind::Surround)
        .map(|r| {
            let new_text = r.body.replace("$selection", &ctx.selected_text);
            Candidate {
                trigger: r.trigger.clone(),
                label: r.label.clone(),
                edit: TextEdit {
                    range: ctx.range,
                    new_text,
                },
            }
        })
        .collect()
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
            receiver_type: None,
        }
    }

    fn postfix_ctx_typed(receiver: &str, trigger: &str, types: Vec<&str>) -> PostfixContext {
        PostfixContext {
            receiver_type: Some(types.into_iter().map(str::to_owned).collect()),
            ..postfix_ctx(receiver, trigger)
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
            requires: None,
        }
    }

    fn prefix_rule(trigger: &str, body: &str) -> Rule {
        Rule {
            kind: RuleKind::Prefix,
            trigger: trigger.to_owned(),
            label: trigger.to_owned(),
            body: body.to_owned(),
            requires: None,
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
    fn type_filter_excludes_requires_rule_for_wrong_type() {
        let rules = vec![
            postfix_rule_with_requires("fod", "$receiver.FirstOrDefault()", "Enumerable"),
            postfix_rule("var", "var ${1:_} = $receiver;"),
        ];
        // string receiver → fod filtered, var shown
        let candidates = match_postfix(
            &postfix_ctx_typed("s", "fo", vec!["string", "System.String"]),
            &rules,
        );
        assert!(!candidates.iter().any(|c| c.trigger == "fod"));
        // var has no requires, so we get it when we type nothing
        let all = match_postfix(&postfix_ctx_typed("s", "", vec!["string"]), &rules);
        assert!(all.iter().any(|c| c.trigger == "var"));
    }

    #[test]
    fn type_filter_includes_requires_rule_for_matching_type() {
        let rules = vec![postfix_rule_with_requires(
            "fod",
            "$receiver.FirstOrDefault()",
            "Enumerable",
        )];
        let types = vec![
            "System.Collections.Generic.List<string>",
            "System.Collections.Generic.IEnumerable<string>",
        ];
        let candidates = match_postfix(&postfix_ctx_typed("xs", "fod", types), &rules);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].trigger, "fod");
    }

    #[test]
    fn type_filter_shows_requires_rule_when_type_unknown() {
        let rules = vec![postfix_rule_with_requires(
            "fod",
            "$receiver.FirstOrDefault()",
            "Enumerable",
        )];
        // No receiver_type → conservative, show fod
        let candidates = match_postfix(&postfix_ctx("xs", "fod"), &rules);
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn type_confirmed_rules_rank_above_universal() {
        let rules = vec![
            postfix_rule_with_requires("fod", "$receiver.FirstOrDefault()", "Enumerable"),
            postfix_rule("var", "var ${1:_} = $receiver;"),
        ];
        let types = vec!["System.Collections.Generic.IEnumerable<int>"];
        let candidates = match_postfix(&postfix_ctx_typed("xs", "", types), &rules);
        // tier0 (type-confirmed fod) before tier1 (universal var)
        assert_eq!(candidates[0].trigger, "fod");
        assert_eq!(candidates[1].trigger, "var");
    }

    fn postfix_rule_with_requires(trigger: &str, body: &str, requires: &str) -> Rule {
        Rule {
            kind: RuleKind::Postfix,
            trigger: trigger.to_owned(),
            label: trigger.to_owned(),
            body: body.to_owned(),
            requires: Some(requires.to_owned()),
        }
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

    fn surround_ctx(selected: &str) -> SurroundContext {
        SurroundContext {
            selected_text: selected.to_owned(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: u32::try_from(selected.len()).unwrap_or(u32::MAX),
                },
            },
        }
    }

    fn surround_rule(trigger: &str, body: &str) -> Rule {
        Rule {
            kind: RuleKind::Surround,
            trigger: trigger.to_owned(),
            label: trigger.to_owned(),
            body: body.to_owned(),
            requires: None,
        }
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn match_surround_ignores_postfix_and_prefix_rules() {
        let rules = vec![
            postfix_rule("fod", "$receiver.FirstOrDefault()"),
            prefix_rule("if", "if (...) { $0 }"),
            surround_rule("if", "if (${1:cond}) {\n    $selection\n}"),
        ];
        let candidates = match_surround(&surround_ctx("x"), &rules);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].trigger, "if");
    }

    #[test]
    fn match_surround_substitutes_selection() {
        let rules = [surround_rule("if", "if (true) {\n    $selection\n}")];
        let candidates = match_surround(&surround_ctx("DoWork();"), &rules);
        assert_eq!(candidates[0].edit.new_text, "if (true) {\n    DoWork();\n}");
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn match_surround_returns_all_rules_regardless_of_selection() {
        let rules = vec![
            surround_rule("if", "if (${1:c}) { $selection }"),
            surround_rule("try", "try { $selection } catch { $0 }"),
        ];
        assert_eq!(match_surround(&surround_ctx("x"), &rules).len(), 2);
        assert_eq!(match_surround(&surround_ctx(""), &rules).len(), 2);
    }

    #[test]
    fn built_in_csharp_surround_rules_loads_without_panic() {
        let rules = built_in_csharp_surround_rules();
        assert!(!rules.is_empty());
        assert!(rules.iter().all(|r| r.kind == RuleKind::Surround));
    }

    #[test]
    fn built_in_typescript_rules_load_without_panic() {
        let post = built_in_typescript_postfix_rules();
        let pre = built_in_typescript_prefix_rules();
        let sur = built_in_typescript_surround_rules();
        assert!(!post.is_empty());
        assert!(!pre.is_empty());
        assert!(!sur.is_empty());
        assert!(post.iter().all(|r| r.kind == RuleKind::Postfix));
        assert!(pre.iter().all(|r| r.kind == RuleKind::Prefix));
        assert!(sur.iter().all(|r| r.kind == RuleKind::Surround));
    }

    proptest::proptest! {
        #[test]
        fn match_surround_never_panics(selected in ".*", body in ".*") {
            let rules = [surround_rule("wrap", &body)];
            let candidates = match_surround(&surround_ctx(&selected), &rules);
            for c in &candidates {
                let _ = c.edit.new_text.len();
            }
        }
    }
}
