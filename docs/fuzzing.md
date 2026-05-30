---
title: Fuzzing Policy
status: stable
owners: []
updated: 2026-05-30
---

# Fuzzing Policy

> Authoritative policy for the fuzz layer; see also ADR-0003.

## What fuzzing is and is not in Snipper

Fuzzing is a **robustness layer**, not a correctness proof. It guarantees that
the codepaths touching externally produced bytes — context parsing, template
rendering — never panic, hang, or allocate without bound on arbitrary input.
Functional correctness lives in unit, property, and golden tests.

## Hard rules

1. The targets in the catalogue below are **mandatory**. A PR landing a new
   wire-touching component without a fuzz target does not merge.
2. The `fuzz/` crate is **isolated** from the main workspace: its own
   `rust-toolchain.toml` pins nightly.
3. The bounded smoke run is a **blocking** CI gate; deep fuzzing runs
   scheduled, non-blocking.
4. Every closed crash ships a minimised input committed under
   `fuzz/corpus/<target>/` and stays there forever.
5. Every oracle asserts at minimum: no panic, no hang (`-timeout=10`),
   no unbounded allocation (`-rss_limit_mb=2048`).

## Target catalogue

| Target | Priority | Input | Oracle additions |
| --- | --- | --- | --- |
| `parse_context` | P0 | arbitrary UTF-8 + cursor position | no panic; terminates |
| `receiver_range` | P0 | `<expr>.<trigger>` with cursor | range in bounds; no receiver duplication |
| `render_template` | P0 | template body + captures | no panic; tabstops valid; `{{expr}}` substituted once |

## Property invariants (proptest, `crates/*/tests/`)

- **INV-1** (prime directive): trigger inside string literal or comment →
  expansion candidates are empty.
- **INV-2** (round-trip): postfix edit applied + re-parse → no duplicate
  receiver node.
- **INV-3** (bounds): all `TextEdit.range` within document bounds; no
  overlap with unrelated text.
- **INV-4** (determinism): same input → same context and render output
  (no hash-map order effects).
- **INV-5** (Reactor API boundary): compilation test confirms `snipper-core`
  and `snipper-context` public APIs contain no `lsp_types` types.

## Corpus layout

```text
fuzz/
├── Cargo.toml
├── rust-toolchain.toml
├── fuzz_targets/
│   ├── parse_context.rs
│   ├── receiver_range.rs
│   └── render_template.rs
└── corpus/          <- git-ignored; seed files added with git add -f
    ├── parse_context/
    ├── receiver_range/
    └── render_template/
```

Seed corpora are hand-picked minimal valid inputs. Regression corpora are
minimised crash/hang inputs from closed findings.

## Oracle template (raw bytes)

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // guard: only process valid UTF-8
    let Ok(_s) = std::str::from_utf8(data) else { return; };
    // call engine function here; must not panic
});
```

## CI policy

- **Blocking per PR**: 60 s smoke run per P0 target.
- **Non-blocking nightly**: 15 min per target; artifacts uploaded to GitHub
  Actions (see `.github/workflows/fuzz-nightly.yml`).

## Known findings

When the first finding lands, append:

> **F-001 — `<target>` — `<short description>`**
> Seed: `fuzz/corpus/<target>/<file>`.
> Found: `<date>`. Fixed: `<commit>`.
