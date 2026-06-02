# S7 — Multi-Language + Backend Resolution

| Field | Value |
| --- | --- |
| Status | Done |
| Depends on | [S6](S6-surround-selection.md) |
| ADRs | [ADR-0002](../adr/0002-tier1-treesitter-vs-astgrep.md) |

## Goal

Resolve ADR-0002 (Tree-sitter vs ast-grep), introduce the `languages/`
profile system, and add a second supported language.

## Inputs → Outputs

**In:** single-language engine (C# only, S6).

**Out:** `languages/` profile format; second language (Rust or TypeScript);
ADR-0002 closed; optional differential fuzz between backends.

## Approach

1. Close ADR-0002: Tree-sitter is the default CST backend for all
   languages; ast-grep remains optional behind a feature flag or is
   deprecated (decision in the ADR update).
2. Define `languages/<lang>.toml` profile format: grammar crate, feature
   flags, applicable rule packs.
3. Add `TreeSitterBackend::rust()` or `TreeSitterBackend::typescript()`.
4. Port existing postfix + prefix rules to the second language.
5. Add differential fuzz target if two backends are active for a language:
   same input must produce the same `LexicalClass`.

## Acceptance criteria

- `snipper context --language rust` (or `typescript`) classifies correctly.
- `languages/` profiles are loaded at runtime, not hard-coded.
- ADR-0002 status updated to Accepted.
- Differential fuzz (if applicable) passes 60 s clean.

## Open questions

- Second language: Rust (closer to contributors) or TypeScript (larger
  user base)?

## See also

- [Architecture](../architecture.md)
- [ADR-0002](../adr/0002-tier1-treesitter-vs-astgrep.md)
