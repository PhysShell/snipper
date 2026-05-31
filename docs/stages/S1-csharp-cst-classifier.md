# S1 — C# CST Classifier

| Field | Value |
| --- | --- |
| Status | Done |
| Depends on | [S0](S0-baseline-workspace.md) |
| ADRs | [ADR-0002](../adr/0002-tier1-treesitter-vs-astgrep.md) |

## Goal

Implement the real C# CST classifier so that `classify_at` returns the
correct `LexicalClass` for all cursor positions, enforcing the prime
directive.

## Inputs → Outputs

**In:** S0 workspace with stub `TreeSitterBackend` returning `Other`.

**Out:** working CST classifier for C# with property tests and a
functioning `snipper context` CLI subcommand.

## Approach

1. Add `tree-sitter` + `tree-sitter-c-sharp` as optional deps behind
   `backend-treesitter` / `lang-csharp` feature flags.
2. Implement `classify_at`: probe `offset.saturating_sub(1)` (LSP
   insertion-point cursor), walk ancestor chain for string/comment nodes,
   then check declaration and postfix predicates.
3. Implement `is_string_node`, `is_comment_node`, `is_declaration_name`,
   `is_postfix_trigger` for the C# grammar.
4. Add INV-2 and INV-3 property tests with cursor-at-end offsets.
5. Wire `TreeSitterBackend::csharp()` into `snipper context` CLI with
   `--offset`, `--language`, `--format {tree,sexpr,json}`.

## Acceptance criteria

- `classify` returns `StringLiteral`/`Comment` inside those node kinds.
- `classify` returns `IdentifierDeclaration` for variable, method, class,
  parameter, and type-parameter name positions.
- `classify` returns `CodeAfterDot` for `<receiver>.<trigger>` positions.
- INV-2 and INV-3 property tests pass with cursor-at-end offsets.
- `snipper context --language csharp --offset N --format sexpr` works.
- All CI jobs pass.

## See also

- [Architecture](../architecture.md)
- [Fuzzing](../fuzzing.md)
