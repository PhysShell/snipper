# Snipper — agent guide

## What

Snipperper is not the project name. The project is called **Snipper** — a
portable structural expansion engine for code editors. It resolves a trigger
(`fod`, `whr`, `foreach`) at the cursor into expansion candidates by parsing
the CST context, matching postfix/prefix/surround templates, and emitting
`TextEdit` patches. The engine is editor-agnostic: LSP and the Roslyn sidecar
are adapters, not the core.

## Prime directive (non-negotiable)

The engine **must never** expand a trigger that appears inside a string
literal, a comment, or an identifier declaration position. A false positive
expansion is worse than ten missed expansions. Every fuzz target, property
test, and golden fixture in this repository exists primarily to guard this
invariant.

A green CI with a broken prime-directive invariant is a regression under a
green badge — not a passing build.

## Project map

- `crates/snipper-core/` — domain types (`TextEdit`, `Range`, `Position`).
  No I/O, no LSP. `#![forbid(unsafe_code)]`.
- `crates/snipper-context/` — CST context classifier (Tree-sitter parse
  tree, lexical and semantic predicates). `#![forbid(unsafe_code)]`.
- `crates/snipper-lsp/` — LSP adapter (`textDocument/completion`,
  `completionItem/resolve`).
- `crates/snipper-cli/` — `snipper` binary (`snipper context`,
  `snipper expand`).
- `fuzz/` — isolated nightly cargo-fuzz crate (ADR-0003).
- `docs/` — knowledge base; start at
  [`docs/architecture.md`](docs/architecture.md).
- `languages/` — language profiles (grammar bindings and context rules).
- `snippets/` — built-in default rule packs (TOML).
- `sidecar/Snipper.Roslyn/` — .NET semantic sidecar (deferred).
- `tests/golden/` — golden fixtures `(file, cursor, expected sexpr snapshot)`.

## Constitution

All architectural decisions live in [`docs/adr/`](docs/adr/). On any design
conflict, the most recent `Accepted` ADR is authoritative. Extend ADRs
rather than inventing policy in code.

## Commands

```text
just verify     # lint + test + fuzz-smoke
just lint       # markdownlint + fmt + clippy + deny + typos
just test       # cargo test --workspace --all-features
just fuzz-smoke # 60 s/target smoke run (P0 targets)
```

## Routing

- Architectural decision → new ADR in `docs/adr/`
  (template: `docs/adr/0000-template.md`).
- Format question →
  [`docs/representation-formats.md`](docs/representation-formats.md).
- Fuzz policy → [`docs/fuzzing.md`](docs/fuzzing.md).
- Convention question →
  [`docs/agent-conventions.md`](docs/agent-conventions.md).

## TDD and property-test workflow

1. **`snipper-core`, `snipper-context`** — strict red → green → refactor
   with `proptest` invariants. Write failing tests first; never commit a
   new public function in the same commit as its tests.
2. **`snipper-lsp`, `snipper-cli`** — integration-test driven; TDD not
   required.

Property invariants that must always hold (full list in `docs/fuzzing.md`):

- `INV-1` (prime directive): trigger inside a string literal, comment, or
  identifier declaration → expansion candidates are empty.
- `INV-2` (round-trip): applying a postfix edit and re-parsing does not
  duplicate the receiver node.
- `INV-3` (bounds): all `TextEdit.range` values are within document bounds
  and do not overlap unrelated text.
- `INV-4` (determinism): context classification and template rendering are
  deterministic (no hash-map order dependence).
- `INV-5` (Reactor API boundary): public APIs of `snipper-core` and
  `snipper-context` contain no `lsp_types` types.

## Hard constraints

- `unsafe_code = "forbid"` workspace-wide. Allowed only in Tree-sitter FFI
  grammar bindings, with a `// SAFETY:` comment per call site.
- `snipper-core` and `snipper-context` must not expose `lsp_types` in any
  public signature (INV-5). These crates are the portable core shared with
  the Reactor mobile editor; LSP is an adapter.
- All repository text is English. Code, comments, commit messages, docs,
  and fixtures — English only.
- Fuzzing is mandatory for context parsing and template rendering (ADR-0003).
  A PR landing a new wire-touching component without a fuzz target does not
  merge.
- `snipper context` CLI subcommand must support
  `--format {tree,sexpr,json}`. `sexpr` is the canonical golden-snapshot
  format: stable and diffable.
- MSRV is `1.85` (matches `rust-toolchain.toml` and `clippy.toml`).

## Anti-patterns (block in review)

1. Expanding a trigger inside a string literal, comment, or identifier
   declaration. This is the prime directive. No exceptions.
2. Using `unwrap()` or `expect()` on user-controlled or parser-derived
   data in a public library function.
3. Depending on `lsp_types` in `snipper-core` or `snipper-context`.
4. Adding a public item without a doc comment in a published crate
   (`missing_docs` lint fires).
5. Introducing a binary serialization format before the cache/IPC milestone
   (see ADR-0003).
6. Treating fuzz targets as optional. They are first-class citizens.
7. Replacing golden-snapshot diffing with ad-hoc string comparisons.

## Open blockers (§11 of task spec)

These require human input; working on defaults until resolved, with status
recorded in ADR-0001:

1. **MSRV** — set to `1.85` following SemantixTrace; override if needed.
2. **License** — set to `MIT` following SemantixTrace; override if needed.
3. **Tier-1 backend** (Tree-sitter vs ast-grep) — open; see ADR-0002.
4. **Documentation language** — set to English following SemantixTrace.
