# S4 — LSP Adapter MVP

| Field | Value |
| --- | --- |
| Status | Done |
| Depends on | [S3](S3-postfix-template-engine.md) |
| ADRs | [ADR-0004](../adr/0004-follow-rust-api-guidelines-on-public-surfaces.md) |

## Goal

Expose the expansion engine over LSP so that any editor supporting the
Language Server Protocol can use postfix completions with prefix matching.

## Inputs → Outputs

**In:** postfix template engine (S3).

**Out:** `snipper-lsp` binary responding to `textDocument/completion` and
`completionItem/resolve`; prefix-matching completion list in a test client.

## Approach

1. Implement `textDocument/completion`: classify cursor, extract partial
   trigger, return templates whose trigger starts with the partial (prefix
   match), sorted by exact-match-first then alphabetical.
2. Implement `completionItem/resolve`: apply `TextEdit` for the chosen
   item.
3. Add INV-5 integration test: `lsp_types` does not appear in the public
   API of `snipper-core` or `snipper-context`.
4. End-to-end smoke test using a minimal LSP client fixture.

## Acceptance criteria

- Typing `users.fo` in a C# document returns a completion list containing
  `fod`, `foreach`, etc.
- Selecting an item applies the correct `TextEdit`.
- INV-5 test passes.
- `cargo test --workspace --all-features` green.

## See also

- [Architecture](../architecture.md)
- [S3 — Postfix template engine](S3-postfix-template-engine.md)
