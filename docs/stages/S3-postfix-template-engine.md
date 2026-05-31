# S3 — Postfix Template Engine

| Field | Value |
|---|---|
| Status | Not started |
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
   `var`, `tolist`, `count`.
4. Wire `snipper expand --offset --language` CLI subcommand.
5. Add INV-2 (round-trip: apply edit and re-parse; receiver not duplicated)
   and INV-3 (bounds: all `TextEdit.range` within document) property tests.
6. Add fuzz target `render_template`: arbitrary `PostfixContext` + rule;
   assert no panic and bounds invariant holds.

## Acceptance criteria

- `echo 'users.fod' | snipper expand --language csharp --offset 9`
  emits valid `TextEdit[]` JSON.
- INV-2 and INV-3 property tests pass.
- Fuzz target `render_template` runs 60 s clean.
- Built-in rule pack passes golden snapshot tests.

## Open questions

- Final TOML schema for template rules — needs a decisions log entry.
- Ranking tie-break within matched candidates: exact match first, then
  alphabetical?

## See also

- [Architecture](../architecture.md)
- [Fuzzing](../fuzzing.md)
- [Representation formats](../representation-formats.md)
