# ADR 0003: Normative representation formats and binary format deferral

Date: 2026-05-30
Status: Accepted

## Context

Snipperper produces and consumes structured data in several contexts: internal
CST manipulation, user-defined rules, CLI output for humans, CLI output for
tooling, test snapshots, and eventually a sidecar JSON-RPC contract. Using
one format for everything creates mismatches (JSON is poor for diffable
snapshots; S-expressions are poor for tooling consumption). Using arbitrary
formats per subsystem creates drift and makes the codebase harder to reason
about.

The task spec (§9) provides a normative table of format choices. A binary
format (protobuf, bincode) has been requested for a future cache/IPC
scenario but is not needed at MVP.

## Decision

We adopt the normative table in `docs/representation-formats.md` verbatim.
Key choices:

- Internal tree manipulation uses **CST with ranges** (Tree-sitter parse
  tree), not a pure AST.
- User rules use **TOML**.
- Sidecar JSON-RPC uses **JSON**.
- `snipper context --format tree` for humans, `--format sexpr` for golden
  snapshots, `--format json` for tooling.
- **S-expression is the canonical golden-snapshot format** (stable under
  cosmetic changes, easy to diff).
- Binary formats are **deferred** until the cache/IPC milestone.
- Zipper navigation model and edge/adjacency list are **deferred** for
  Reactor.

This ADR supersedes any informal format choices made during scaffolding.

## Consequences

- `snipper context` must implement all three output formats; they are
  covered by golden tests.
- Future contributors must update this ADR (or supersede it) before
  introducing a new format.
- Binary format deferral means the MVP has no persistent cache; acceptable
  at this stage.
- The `fuzz/` crate is the only place where the workspace-level
  `unsafe_code = "forbid"` does not apply (libFuzzer harness).
