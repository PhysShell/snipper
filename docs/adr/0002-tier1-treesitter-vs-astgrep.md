# ADR 0002: Tier-1 context backend: Tree-sitter vs ast-grep

Date: 2026-05-30
Status: Proposed

## Context

The `snipper-context` crate needs a CST parser to classify the cursor
position. Two candidates are under consideration:

- **Tree-sitter** — battle-tested, used by Neovim, Helix, and GitHub
  Copilot; Rust bindings are mature; requires per-language grammar crates
  compiled from C.
- **ast-grep** — higher-level pattern matching over Tree-sitter; simpler
  rule authoring; less direct access to raw CST nodes and byte ranges.

The prime directive requires exact byte ranges for every token. Both
backends expose this via their underlying Tree-sitter layer, but
ast-grep adds an abstraction boundary that may obscure trivia tokens
(whitespace, comments) needed for correct receiver range extraction.

The task spec calls for a differential fuzz test: if both backends are
built, `fuzz/fuzz_targets/` must verify that they produce identical
predicate sets and identical receiver ranges on the same input. The
result of that test is the decision mechanism.

## Decision

DECISION NEEDED. Deferred pending differential fuzz test results.
Default scaffold uses Tree-sitter directly (`tree-sitter` crate +
language grammar crates) as the simpler, lower-level option.

## Consequences

- If Tree-sitter is confirmed: `snipper-context` depends on `tree-sitter`
  and per-language grammar crates. Grammar crates require a C compiler.
- If ast-grep is chosen: dependency on `ast-grep-core`; simpler rule
  authoring but less direct range access.
- Either way, the differential fuzz test (`receiver_range` target) must
  pass before this ADR is marked `Accepted`.
