# S0 — Baseline Workspace

| Field | Value |
|---|---|
| Status | TODO (Not started / In progress / Done) |
| Depends on | — |
| ADRs | [ADR-0001](../adr/0001-doc-template.md) |

## Goal

Wire up the Rust workspace so that every quality gate passes on an empty
codebase, giving a known-good baseline for all subsequent work.

## Inputs → Outputs

**In:** empty Git repository.

**Out:** workspace with at least one library crate, all CI jobs green,
`cargo deny` / `cargo fmt` / `cargo clippy` / `cargo test` passing locally.

## Approach

1. Create workspace `Cargo.toml` with `[workspace.lints]` table.
2. Add `my-project-core` stub library crate.
3. Configure `rustfmt.toml`, `clippy.toml`, `deny.toml`, `typos.toml`.
4. Wire up GitHub Actions CI (see `.github/workflows/ci.yml`).
5. Add `fuzz/` crate excluded from the workspace, targeting nightly.
6. Confirm `cargo deny check` passes with chosen licence.

## Acceptance criteria

- `cargo build --workspace` exits 0.
- `cargo fmt --check` exits 0.
- `cargo clippy --workspace -- -D warnings` exits 0.
- `cargo doc --no-deps` exits 0 with no warnings.
- `cargo deny check` exits 0.
- `cargo test --workspace` exits 0.
- All CI jobs in `.github/workflows/ci.yml` pass on the default branch.

## Open questions

- What MSRV will the project target?
- Which licence(s) will be used?

## See also

- [Architecture](../architecture.md)
- [ADR index](../adr/README.md)
