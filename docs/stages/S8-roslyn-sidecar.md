# S8 — Roslyn Sidecar + Semantic Enrichment

| Field | Value |
| --- | --- |
| Status | Not started |
| Depends on | [S7](S7-multi-language.md) |
| ADRs | — (new ADR required for sidecar protocol) |

## Goal

Provide reliable C# receiver-type information via a Roslyn sidecar.
Tree-sitter gives structure; Roslyn gives semantics. Type-aware filtering
ensures `fod` (FirstOrDefault) appears only when the receiver is a
collection type.

## Inputs → Outputs

**In:** CST-only classifier (S1–S7).

**Out:** `sidecar/Snipper.Roslyn/` .NET project; sidecar IPC protocol;
`PostfixContext` enriched with receiver type; template `requires` metadata
honoured at filter time.

## Approach

1. Write an ADR for the sidecar protocol (JSON-RPC over named pipe or
   stdin/stdout) and lifecycle (per-workspace vs per-document).
2. Scaffold `sidecar/Snipper.Roslyn/` as a minimal .NET console app that
   accepts a document + offset and returns the receiver type symbol.
3. Extend `PostfixContext` with `receiver_type: Option<String>`.
4. In `snipper-lsp`, spawn the sidecar on first C# document open; query
   it on each completion request.
5. Extend the template TOML: `requires = "IEnumerable"` filters `fod`
   unless the receiver type matches.
6. Ranking: type-matched templates rank above non-matched ones.

## Acceptance criteria

- `fod` does not appear when receiver is a plain `object` or `string`.
- `fod`, `foreach`, `tolist` appear when receiver is `List<T>` or
  `IEnumerable<T>`.
- Sidecar crash falls back to CST-only classification without crashing
  `snipper-lsp`.
- ADR for sidecar protocol accepted.

## Open questions

- Sidecar lifecycle: one process per workspace or per document?
- First-request latency: Roslyn workspace load is 2–5 s; mitigate how?

## See also

- [Architecture](../architecture.md)
- [S3 — Postfix template engine](S3-postfix-template-engine.md)
- [S11 — Smart ranking](S11-smart-ranking.md)
