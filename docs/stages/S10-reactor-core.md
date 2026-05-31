# S10 — Reactor Portable Core Hardening

| Field | Value |
| --- | --- |
| Status | Not started |
| Depends on | [S8](S8-roslyn-sidecar.md) |
| ADRs | — (new ADR required for mobile API contract) |

## Goal

Verify that `snipper-core` and `snipper-context` compile and run on
non-desktop targets so they can be embedded in the Reactor mobile editor.

## Inputs → Outputs

**In:** full engine (S8).

**Out:** `wasm32-unknown-unknown` CI passing; no OS-specific or std-I/O
deps in core crates; documented embedding API; mobile ADR.

## Approach

1. Add `wasm32-unknown-unknown` to CI matrix for `snipper-core` and
   `snipper-context`.
2. Audit all public types for platform-specific bounds; replace with
   portable alternatives where possible.
3. Ensure `snipper-core` and `snipper-context` have no `std::io` or
   `std::fs` in their public surface.
4. Document the embedding API: how Reactor calls `classify` and `expand`
   without the LSP transport layer.
5. Write ADR for the Reactor API contract.

## Acceptance criteria

- `cargo build --target wasm32-unknown-unknown` exits 0 for `snipper-core`
  and `snipper-context`.
- No `std::io`, `std::fs`, or platform-specific types in public API.
- Embedding guide committed to `docs/`.

## See also

- [Architecture](../architecture.md)
- [S8 — Roslyn sidecar](S8-roslyn-sidecar.md)
