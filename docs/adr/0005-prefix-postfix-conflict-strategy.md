---
title: "ADR-0005: Prefix/postfix conflict-resolution strategy"
status: Accepted
updated: 2026-06-01
---

# ADR-0005: Prefix/postfix conflict-resolution strategy

Date: 2026-06-01
Status: Accepted

## Context

The engine now supports two expansion types:

- **Postfix** — triggered at a `<receiver>.<trigger>` site where the cursor
  is the `name` child of a `member_access_expression` node. Classified as
  `LexicalClass::CodeAfterDot`.
- **Prefix** — triggered at a bare identifier in executable code (not after a
  dot). Classified as `LexicalClass::CodeBareIdentifier`.

A cursor can only occupy one CST node at a time. Because the CST classifier
assigns exactly one `LexicalClass` per position, a cursor that is after a dot
(`CodeAfterDot`) cannot simultaneously be a bare identifier
(`CodeBareIdentifier`), and vice versa. There is therefore no ambiguity at the
cursor level — the two trigger types are structurally disjoint.

The question is: when the same trigger string appears in both rule packs (e.g.
`foreach` exists as both a postfix rule and a prefix rule), which candidates
should the engine return?

## Decision

**The `LexicalClass` at the cursor fully determines the expansion type.**

- `CodeAfterDot` → only postfix rules are matched; prefix rules are ignored.
- `CodeBareIdentifier` → only prefix rules are matched; postfix rules are
  ignored.
- All other classes → no expansion (prime directive or non-expandable site).

Postfix and prefix candidates are never combined in a single completion list.
The rule-kind filter (`RuleKind::Postfix` / `RuleKind::Prefix`) is applied
inside `match_postfix` and `match_prefix` respectively, so even if both packs
are passed to the wrong function the kind guard prevents cross-contamination.

Tie-breaking within a single expansion type is: exact trigger match first,
then alphabetical by trigger. Frequency-based ranking is deferred to S11.

## Consequences

**Good:**
- Zero runtime ambiguity: cursor position unambiguously selects the rule set.
- No "which wins?" edge case to test or document.
- Each rule pack can reuse the same trigger strings for different semantics
  (e.g. `foreach` as postfix loop-body vs. prefix loop-statement) without
  conflict.

**Accepted trade-off:**
- If a future expansion type introduces a genuinely ambiguous cursor site
  (e.g. surround/selection in S6), a new ADR will be required to extend
  the strategy.
- Frequency scoring is deferred; purely alphabetical tie-breaking may not
  match user expectations in all cases — revisit in S11.
