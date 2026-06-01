---
title: "ADR-0006: Surround expansion trigger and prime-directive enforcement"
status: Accepted
updated: 2026-06-01
---

# ADR-0006: Surround expansion trigger and prime-directive enforcement

Date: 2026-06-01
Status: Accepted

## Context

S6 introduces a third expansion type — **surround** — that wraps selected text
with a code construct (`if`, `try`, `using`, …).  Unlike postfix and prefix
expansions, surround is not triggered by a cursor position but by a non-empty
text selection combined with a `textDocument/codeAction` request.

Two questions arise:

1. **When is surround offered?**  The trigger is structural: selection is
   non-empty.  There is no typed trigger string to match — all available surround
   rules are offered as code actions and the user picks one.

2. **How is the prime directive enforced?**  For postfix/prefix, the CST
   classifier returns a `LexicalClass` that enforces the prime directive before
   any candidates are produced.  Surround bypasses the classifier's trigger
   logic.  We still must not expand inside strings, comments, or declaration
   sites.

## Decision

**Trigger:** A non-empty selection (`range.start != range.end`) inside a C#
document triggers the surround path.  All `RuleKind::Surround` rules in the
active pack are offered without trigger filtering.

**Prime-directive check:** Before producing candidates, the LSP adapter
classifies the source text at the byte corresponding to `selection.start + 1`
(one byte into the selection).  If the returned `LexicalClass` satisfies
`forbids_expansion()` — i.e. the selection start is inside a string literal,
comment, or identifier declaration — the adapter returns an empty action list.

Rationale: the same CST classifier used for postfix/prefix already encodes the
prime-directive rules.  Reusing it for the selection-start probe avoids
duplicating that logic.  Probing one byte *into* the selection (rather than
the boundary itself) ensures the check targets the selection content, not a
character that precedes it.

**No new LexicalClass variant:** Surround is structurally orthogonal to
postfix/prefix.  It uses a different LSP RPC (`codeAction` vs `completion`)
and a different trigger mechanism (selection vs cursor).  Adding a
`LexicalClass::CodeSelection` would conflate two independent axes and is
therefore not done.

## Consequences

**Good:**
- Prime directive is enforced consistently: the same forbidden-zone rules apply
  to all three expansion types.
- No new CST classification variant is required; surround integrates cleanly
  alongside existing expansions.
- Postfix, prefix, and surround are never offered in the same UI widget
  (completion vs code-action), eliminating visual conflicts.

**Accepted trade-off:**
- Probing `selection.start + 1` may misclassify an empty or single-byte
  selection that starts exactly at a character boundary.  This edge case is
  acceptable: the result is conservatively no expansion (not a wrong expansion).
- Template rendering replaces `$selection` verbatim; if the selected text
  contains `$` characters (e.g. shell variables), the editor may interpret them
  as additional tabstops.  Escaping is deferred to a future stage.
