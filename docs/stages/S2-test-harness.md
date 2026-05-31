# S2 — Test Harness: Golden Fixtures + Fuzz

| Field | Value |
| --- | --- |
| Status | Not started |
| Depends on | [S1](S1-csharp-cst-classifier.md) |
| ADRs | [ADR-0003](../adr/0003-representation-formats.md) |

## Goal

Lock in the classifier's behaviour as ground truth before the template
engine lands. A broken prime directive under a green badge is a regression.

## Inputs → Outputs

**In:** working C# CST classifier (S1).

**Out:** `tests/golden/` directory with sexpr snapshots for all five
`LexicalClass`es; fuzz target `parse_context`; INV-4 and INV-5 green.

## Approach

1. Create `tests/golden/csharp/` with `.cs` source fixtures and `.sexpr`
   snapshot files for each `LexicalClass`.
2. Add a test harness that runs `snipper context --format sexpr` on each
   fixture and diffs against the snapshot (`UPDATE_GOLDEN=1` to refresh).
3. Add fuzz target `parse_context`: feed arbitrary bytes as source and a
   random `usize` as offset; assert no panic and a valid `LexicalClass`.
4. Add INV-4 property test: same `(source, offset)` always returns the
   same `LexicalClass`.
5. Add INV-5 compilation test: `snipper-core` and `snipper-context`
   dependency trees contain no `lsp_types` crate.
6. Record `cargo public-api` baseline in CI.

## Acceptance criteria

- Golden diff tests pass with committed snapshots.
- Fuzz target `parse_context` runs 60 s without panic or OOM.
- INV-4 and INV-5 property tests pass.
- `cargo public-api` baseline recorded in CI.

## See also

- [Fuzzing](../fuzzing.md)
- [Architecture](../architecture.md)
