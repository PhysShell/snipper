# S5 — Prefix Expansions + Conflict Strategy

| Field | Value |
| --- | --- |
| Status | Not started |
| Depends on | [S4](S4-lsp-adapter-mvp.md) |
| ADRs | — (new ADR required for conflict strategy) |

## Goal

Add the second expansion type — prefix snippets triggered at the start
of an expression without a dot — and define how the engine resolves
ambiguity when multiple rules match the same partial trigger.

## Inputs → Outputs

**In:** postfix engine + LSP adapter (S3/S4).

**Out:** prefix expansion rules for C# (`ctor`, `prop`, `if`, `for`,
`switch`, `try`); a documented conflict-resolution strategy; ADR.

## Approach

1. Extend the TOML template format with `type = "prefix"` rules.
2. Add `snippets/csharp/prefix.toml` with built-in prefix rules.
3. Implement prefix trigger matching: bare identifier (not after dot)
   triggers prefix candidates.
4. Define conflict strategy: postfix wins when cursor follows a dot;
   prefix wins otherwise. Tie-break alphabetical (frequency scoring
   deferred to S11).
5. Write an ADR for the conflict-resolution decision.

## Acceptance criteria

- Typing `if` at expression start yields prefix expansion candidates.
- Postfix and prefix candidates do not conflict at the same cursor.
- Conflict strategy ADR accepted.
- All existing golden tests still pass.

## Open questions

- Should prefix and postfix candidates ever appear together in one list?

## See also

- [Architecture](../architecture.md)
- [S3 — Postfix template engine](S3-postfix-template-engine.md)
