# ADR 0002: Tier-1 context backend: Tree-sitter vs ast-grep

Date: 2026-05-30
Status: Accepted (updated 2026-06-01)

## Context

The `snipper-context` crate needs a CST parser to classify the cursor
position. Two candidates were under consideration:

- **Tree-sitter** — battle-tested, used by Neovim, Helix, and GitHub
  Copilot; Rust bindings are mature; requires per-language grammar crates
  compiled from C.
- **ast-grep** — higher-level pattern matching over Tree-sitter; simpler
  rule authoring; less direct access to raw CST nodes and byte ranges.

The prime directive requires exact byte ranges for every token. Both
backends expose this via their underlying Tree-sitter layer, but
ast-grep adds an abstraction boundary that may obscure trivia tokens
(whitespace, comments) needed for correct receiver range extraction.

## Decision

**Tree-sitter is the confirmed Tier-1 CST backend for all languages.**

ast-grep is not pursued. Rationale:

1. The `receiver_range` fuzz target demonstrated stable, correct byte-range
   extraction directly from the Tree-sitter CST across thousands of inputs.
2. Direct Tree-sitter access provides the named-field API (`child_by_field_name`)
   needed to identify declaration-name sites precisely — essential for the prime
   directive.
3. The `LanguageRules` abstraction (introduced in S7) makes adding new grammars
   straightforward: supply a `tree-sitter-<lang>` crate, declare node-type
   names in a static `LanguageRules`, and add a factory method.  Two languages
   (C# and TypeScript) are now supported using this approach.
4. The `backend-astgrep` feature flag exists in `Cargo.toml` but is intentionally
   empty and will be removed in a future cleanup unless a compelling use case
   emerges.

## Consequences

- `snipper-context` depends on `tree-sitter` and per-language grammar crates.
  Grammar crates require a C compiler (standard for Rust projects using FFI).
- New languages are added by: (a) adding a `tree-sitter-<lang>` dependency behind
  a `lang-<lang>` feature flag, (b) defining a `static LanguageRules` constant,
  (c) adding a `TreeSitterBackend::<lang>()` factory, and (d) adding rule packs
  under `snippets/<lang>/`.
- Differential fuzz between backends is not required since ast-grep is not pursued.
