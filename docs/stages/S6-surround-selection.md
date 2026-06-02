# S6 — Surround & Selection Expansions

| Field | Value |
| --- | --- |
| Status | Done |
| Depends on | [S5](S5-prefix-expansions.md) |
| ADRs | [ADR-0006](../adr/0006-surround-prime-directive.md) |

## Goal

Add surround (wrap selected text in a construct) and selection-aware
expansions — the third and fourth expansion types.

## Inputs → Outputs

**In:** prefix + postfix engines (S5).

**Out:** surround rules (`if (`, `try {`, `using (`); LSP
`textDocument/codeAction` integration; selection range in classifier.

## Approach

1. Extend context with selection range awareness.
2. Add `type = "surround"` rules in the template format.
3. Implement surround matcher: selection range → wrapping `TextEdit[]`.
4. Wire `textDocument/codeAction` in `snipper-lsp` for surround
   expansions on selected text.
5. Built-in C# surround rules: `if`, `for`, `foreach`, `try`, `using`,
   `lock`.

## Acceptance criteria

- Selecting `foo.Bar()` and invoking surround shows `if (`, `try {`,
  etc. as code actions.
- Applying a surround action wraps the selection correctly.
- All prior expansion types still work.

## See also

- [Architecture](../architecture.md)
- [S5 — Prefix expansions](S5-prefix-expansions.md)
