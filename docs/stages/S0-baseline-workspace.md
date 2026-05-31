# S0 — Baseline Workspace

| Field | Value |
| --- | --- |
| Status | Done |
| Depends on | — |
| ADRs | [ADR-0001](../adr/0001-doc-template.md) |

## Goal

Wire up the Rust workspace so that every quality gate passes on an empty
codebase, giving a known-good baseline for all subsequent stages.

## Inputs → Outputs

**In:** empty Git repository.

**Out:** workspace with all crate scaffolds, CI green, all quality tools
configured and passing locally.

## Approach

1. Create workspace `Cargo.toml` with `[workspace.lints]` table.
2. Add crate scaffolds: `snipper-core`, `snipper-context`, `snipper-lsp`,
   `snipper-cli`.
3. Configure `rustfmt.toml`, `clippy.toml`, `deny.toml`, `typos.toml`,
   `rust-toolchain.toml`.
4. Wire up GitHub Actions CI (10 jobs including `public-api`).
5. Add `fuzz/` crate excluded from workspace, targeting nightly.
6. Add ADRs 0001–0004, architecture doc, agent conventions, decisions log.

## Acceptance criteria

- `cargo build --workspace` exits 0.
- `cargo fmt --check` exits 0.
- `cargo clippy --workspace -- -D warnings` exits 0.
- `cargo doc --no-deps` exits 0.
- `cargo deny check` exits 0.
- `cargo test --workspace` exits 0.
- All CI jobs pass.

## See also

- [Architecture](../architecture.md)
- [ADR index](../adr/README.md)
