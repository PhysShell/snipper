# S3 — Postfix Template Engine

| Field | Value |
|---|---|
| Status | Done |
| Depends on | [S2](S2-test-harness.md) |
| ADRs | [ADR-0003](../adr/0003-representation-formats.md) |

## Goal

Implement the first expansion type end-to-end: postfix triggers
(`<receiver>.fod`) resolve to `TextEdit[]` patches.

## Inputs → Outputs

**In:** classified `PostfixContext` from S1/S2.

**Out:** `snipper expand` CLI subcommand emitting `TextEdit[]`; built-in
C# postfix rule pack in `snippets/csharp/postfix.toml`.

## Approach

1. Define the TOML template format: trigger, expansion body, optional
   `requires` metadata (used for type-aware filtering in S8).
2. Implement the matcher: `PostfixContext` + rule pack → ranked `TextEdit[]`
   filtered by prefix match on trigger.
3. Built-in C# postfix rules: `fod`, `foreach`, `if`, `null`, `not`,
   `var`, `tolist`, `tod`.
4. Wire `snipper expand --offset --language` CLI subcommand.
5. INV-4 (determinism) property tests pass via S2 harness.
6. Fuzz target `render_template`: arbitrary `PostfixContext` + rule;
   assert no panic.

## Acceptance criteria

- `echo 'var y = users.fod;' | snipper expand --language csharp --offset 17`
  emits valid `TextEdit[]` JSON with `FirstOrDefault()` expansion.
- INV-4 property tests pass.
- Fuzz target `render_template` builds clean.
- Built-in rule pack passes golden snapshot tests.

## Open questions

- Add `requires` field to `Rule` for type-aware filtering (S8).
- Ranking tie-break within matched candidates: exact match first ✓, then
  alphabetical ✓ — implemented.

## See also

- [Architecture](../architecture.md)
- [Fuzzing](../fuzzing.md)
- [Representation formats](../representation-formats.md)
