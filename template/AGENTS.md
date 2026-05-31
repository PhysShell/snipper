# AGENTS.md

**my-project** — [TODO: one sentence describing what this project does].

> **Customize:** Replace every `my-project` with your project name and fill in
> every `[TODO: ...]` section below.

## Prime directive

[TODO: State the invariant that must never be violated. This is the single rule
that overrides all other considerations. Example: "The engine must never mutate
source text outside the exact byte range requested by a TextEdit."]

## Project map

| Crate             | Role                     | Key public types |
|-------------------|--------------------------|------------------|
| `my-project-core` | Foundational value types | `Example`        |

[TODO: Add a row per crate; remove the placeholder `Example` type.]

## Constitution

1. Red CI is never acceptable on `main`.
2. A failing fuzz corpus seed is a P0 bug.
3. Every public item has a doc comment; `cargo doc -D warnings` is a required
   CI gate.
4. No `unwrap()` or `expect()` on user-controlled or parser-derived data in
   public library functions.
5. All public types implement `Debug`.
6. Error types implement `std::error::Error + Send + Sync + 'static`.
7. `unsafe` is forbidden (`#![forbid(unsafe_code)]`) in all workspace crates.

## Commands

```sh
# Full gate: lint + test + fuzz-smoke
just verify

# Individual steps
just lint        # markdownlint + typos + rustfmt + clippy + cargo-deny
just test        # cargo test --workspace --all-features
just fuzz-smoke  # 60-second bounded smoke run of all fuzz targets
```

## TDD workflow

1. Write a failing test that names the invariant (e.g. `test_inv1_...`).
2. Run `cargo test --workspace` — confirm it fails.
3. Implement the minimum code to make the test pass.
4. Run `just verify` — confirm the full gate is green.
5. Commit with a message that references the invariant or issue.

## Invariants

[TODO: Define INV-1, INV-2, etc. Each invariant should be: named, stated as a
boolean assertion, verified by at least one test, and referenced in fuzz
oracles where applicable.]

**INV-1** — [TODO: state your first invariant].

## Hard constraints

### Rust API Guidelines (C-\*)

Every `pub` item in a crate intended for external consumption follows the
[Rust API Guidelines](<https://rust-lang.github.io/api-guidelines/>).

- **`C-CASE`, `C-CONV`, `C-GETTER`, `C-ITER`** — naming: `fn foo()` not
  `fn get_foo()`; iterators return `impl Iterator`.
- **`C-COMMON-TRAITS`** — every public type derives `Debug`, `Clone`,
  `PartialEq`, `Eq`; `Copy` for small value types; `Default` where a
  meaningful zero-value exists.
- **`C-SEND-SYNC`** — public types are `Send + Sync` unless the docs
  explicitly explain why not.
- **`C-GOOD-ERR`** — error types implement `std::error::Error`, are
  `Send + Sync + 'static`, use `thiserror`. No stringly-typed errors on
  public boundaries.
- **`C-DEBUG`, `C-DEBUG-NONEMPTY`** — every public type implements `Debug`
  with non-empty output.
- **`C-VALIDATE`** — validate inputs at public entry points; return typed
  errors. No `unwrap()` on user-controlled data in a public library function.
- **`C-NO-PANIC`** — document panic conditions in `# Panics` doc sections.
- **`C-SEALED`** — closed-set traits are sealed with a private `Sealed`
  supertrait.
- **`C-NON-EXHAUSTIVE`** — apply `#[non_exhaustive]` to enums that may grow
  without a major version bump.
- **`C-DOC`, `C-EXAMPLE`, `C-FAILURE`, `C-LINK`** — every public item has
  a doc comment; non-trivial functions have an `# Examples` section that
  compiles under `cargo test --doc`; `# Errors` and `# Panics` where
  applicable; intra-doc links to related items.

## Anti-patterns

1. `unwrap()` or `expect()` in library code on user-controlled data.
2. Re-exporting upstream types into your public API when they are not part of
   your stability contract.
3. `allow(clippy::...)` without a comment explaining the exception.
4. Committing generated corpus files (`fuzz/corpus/`).
5. Merging a PR with a failing CI job by bypassing branch protection.
6. A public type without `Debug`.
7. An error type that is not `Send + Sync + 'static`.
8. Intra-doc link that does not resolve — caught by `cargo doc -D warnings`.
9. Wildcard dependency version (`*`) — caught by `cargo deny`.
10. A `pub` item missing a doc comment — caught by the `missing_docs` lint.

## Open blockers

[TODO: List open decisions that block implementation. Remove when none remain.]
