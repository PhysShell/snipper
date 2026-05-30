# ADR 0004: Follow the Rust API Guidelines on public surfaces

Date: 2026-05-30
Status: Accepted

## Context

Snipperper publishes Rust crates that other code compiles against:
`snipper-core` (value objects, `TextEdit`, `Range`, `Position`) and
`snipper-context` (`Context`, `Backend` trait, `LexicalClass`). These
two crates are the portable foundation shared between the LSP adapter
and the future Reactor mobile editor. Any inconsistency in naming,
trait derivations, error shape, sealed-vs-open traits, or
`#[non_exhaustive]` decisions becomes a downstream-breaking change the
moment we try to fix it.

A second constraint compounds this: `snipper-core` and
`snipper-context` must not expose `lsp_types` in any public signature
(INV-5 from AGENTS.md). The API Guidelines give us the vocabulary to
describe exactly what "public signature" means and why it matters.

The Rust project maintains a checklist for this:
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
(`C-*` recommendations under Naming, Interoperability, Documentation,
Predictability, Flexibility, Type safety, Dependability,
Debugability, Future-proofing, Necessities). Adopting it is cheaper
than inventing a project-local style guide and gives consumers —
including Reactor — the conventions they already expect.

## Decision

### 1. Scope

Every `pub` item in a crate intended for external consumption follows
the Rust API Guidelines. Crates in scope:

- `snipper-core` — always in scope; this is the cross-editor contract.
- `snipper-context` — in scope; the `Backend` trait surface is semver.
- `snipper-lsp` — in scope for its public types.
- `snipper-cli` — binary; naming and doc rules apply, but semver
  guarantees do not (no library surface).

`fuzz/` is **out of scope** (nightly-only harness).

### 2. Non-negotiable C-\* commitments

- **`C-CASE`, `C-CONV`, `C-GETTER`, `C-ITER`, `C-ITER-TY`,
  `C-WORD-ORDER`** — naming. `fn foo()` not `fn get_foo()`;
  iterators return `impl Iterator`; type names follow standard
  Rust conventions.
- **`C-COMMON-TRAITS`** — every public type derives `Debug`, `Clone`,
  `PartialEq`, `Eq` where the inner data permits; `Copy` for
  small value types (`Position`, `Range`, `ByteRange`); `Default`
  where a meaningful zero-value exists.
- **`C-SEND-SYNC`** — public types are `Send + Sync` unless the
  docs explicitly explain why not.
- **`C-GOOD-ERR`** — error types implement `std::error::Error`,
  are `Send + Sync + 'static`, carry source via `thiserror`
  `#[from]` / `#[source]`. No stringly-typed errors on public
  boundaries.
- **`C-DEBUG`, `C-DEBUG-NONEMPTY`** — every public type implements
  `Debug` with non-empty output.
- **`C-NEWTYPE`** — domain identifiers are newtypes where
  type-confusion would cause a bug (e.g. `Line(u32)` vs `Character
  (u32)`).
- **`C-VALIDATE`** — public entry points validate inputs and return
  typed errors. No `unwrap()` or `expect()` on user-controlled or
  parser-derived data in a public library function.
- **`C-NO-PANIC`** — public library functions either do not panic
  or document panic conditions in a `# Panics` doc section.
- **`C-SEALED`** — closed-set traits are sealed. Mandatory sealing
  for the `Backend` trait in `snipper-context`: downstream `impl`
  would bypass the prime-directive invariant check. Sealing is
  required before any release that exposes `Backend` publicly.
- **`C-NON-EXHAUSTIVE`** — applied to enums that may grow without
  a major bump: `LexicalClass`, any future `ExpansionKind`,
  `BackendError`. **Not** applied to enums frozen by design (e.g.
  a fixed set of tabstop variants).
- **`C-DOC`, `C-EXAMPLE`, `C-FAILURE`, `C-LINK`** — every public
  item has a doc comment; every non-trivial public function has
  an `# Examples` section that compiles under `cargo test --doc`;
  `# Errors` and `# Panics` where applicable; intra-doc links to
  related items.

### 3. INV-5 as an API Guideline corollary

The prohibition on `lsp_types` in `snipper-core` and
`snipper-context` public signatures is a Reactor-portability
constraint layered on top of the API Guidelines. It is checked by
a compilation test: adding `lsp_types` to either crate's public
interface must fail `cargo check --package snipper-context` when
`snipper-lsp` is absent from the feature graph.

### 4. Tooling gates (CI)

- `cargo clippy --workspace --all-targets --all-features -D warnings`
- `cargo doc --workspace --all-features --no-deps -D warnings`
  (catches broken intra-doc links and `missing_docs` violations).
- Workspace-level lint attrs in `Cargo.toml`:
  `missing_docs`, `missing_debug_implementations`,
  `missing_copy_implementations`, `unreachable_pub`,
  `single_use_lifetimes`, `unused_qualifications`.
- `cargo deny check` — licenses + supply-chain.
- Pre-v1.0: `cargo public-api --diff-git-checkouts <prev-tag> HEAD`
  runs on every PR; surface diffs require release-note entries.

### 5. Pre-v1.0 audit

Before the first crates.io release, a line-by-line walk of the API
Guidelines checklist against `snipper-core` and `snipper-context`.
Findings file issues; release does not happen until every finding is
resolved or explicitly waived in `docs/decisions.log.md` with a
rationale.

## Consequences

- Public surfaces are predictable for Reactor and any future consumer.
  Reviewers can cite C-* rule codes rather than argue style from first
  principles.
- `missing_docs` + `cargo doc -D warnings` forces every public item to
  carry a doc comment from day one. Non-trivial upfront cost; pays off
  when Reactor team reads crate docs cold.
- Sealing `Backend` closes the door on third-party backend impls.
  The trade-off is intentional: the prime-directive invariant is more
  valuable than pluggability.
- `#[non_exhaustive]` on `LexicalClass` costs downstream a wildcard
  match arm. The cost is the honest price of growing the classifier
  without a major version bump.
- `cargo public-api` baseline becomes part of release ritual.
  Accidental SemVer breaks become a CI failure instead of a user-filed
  bug months later.

## See also

- External: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
  (checklist at `/checklist.html`); `rust-lang/api-guidelines` on GitHub.
- External tools: `cargo-public-api`, `cargo-semver-checks`,
  `cargo-deny`.
- ADR-0001 (doc template), ADR-0002 (Backend trait — the public
  surface most affected by C-SEALED), ADR-0003 (representation
  formats — public output format is part of the semver contract).
- [AGENTS.md](../../AGENTS.md) INV-5 (Reactor API boundary).
