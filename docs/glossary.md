---
title: Glossary
---

# Glossary

Domain terms used across the Snipper codebase, documentation, and ADRs.
Sorted alphabetically.

---

## A

**ADR** (Architecture Decision Record) — A lightweight document in `docs/adr/` that
records a significant design decision: its context, the decision made, and its
consequences. The format follows the Nygard template
(Status / Context / Decision / Consequences). Append-only; superseded ADRs change
status to _Superseded_ and link to the replacement.

**ast-grep** — A structural code-search and rewrite tool built on tree-sitter grammars.
Candidate for the optional `backend-astgrep` feature in `snipper-context`; see ADR-0002.

## B

**Backend** — The sealed trait in `snipper-context` that abstracts the CST parsing
implementation. Downstream crates cannot implement it (C-SEALED); only the crates
shipped with Snipper may provide implementations. See ADR-0002 and ADR-0004.

**BackendError** — The `#[non_exhaustive]` error enum returned by `Backend::classify`.
Implements `std::error::Error + Send + Sync + 'static` (C-GOOD-ERR).

**byte offset** — An index into a source text measured in bytes (`usize`). Passed to
`Backend::classify` to identify the cursor position before conversion to `Position`.

## C

**classify** — The single method of the `Backend` trait:
`fn classify(&self, source: &str, offset: usize) -> Result<LexicalClass, BackendError>`.

**CodeAfterDot** — A `LexicalClass` variant indicating the cursor follows a dot-trigger
pattern and is inside executable code. Expansion is permitted here.

**corpus** — The directory of seed inputs fed to a fuzz target by the fuzzer engine.
Stored under `fuzz/corpus/<target-name>/` and excluded from git via `.gitignore`.

**CST** (Concrete Syntax Tree) — A parse tree that preserves every source token,
including whitespace and comments. Snipper uses CSTs rather than ASTs so that
`TextEdit` patches reference exact byte ranges without re-serialising the tree.

## E

**expansion** — The operation that replaces a postfix trigger pattern
(`<expr>.<trigger>`) with template-generated code. Must never fire inside a string
literal, a comment, or at an identifier declaration site (prime directive).

## I

**INV** (Invariant) — A numbered property that must hold at all times, defined in
`AGENTS.md` (INV-1 through INV-5). Invariants are verified by property tests, fuzz
oracles, and compilation checks.

## L

**LexicalClass** — The `#[non_exhaustive]` enum in `snipper-context` that classifies
the cursor's syntactic position: `CodeAfterDot`, `StringLiteral`, `Comment`,
`IdentifierDeclaration`, or `Other`.

**LSP** (Language Server Protocol) — A JSON-RPC protocol for communication between an
editor client and a language analysis server. `snipper-lsp` implements the LSP adapter.
`lsp_types` must not appear in `snipper-core` or `snipper-context` public APIs (INV-5).

## P

**Position** — The `snipper-core` value type `{ line: u32, character: u32 }` (0-indexed,
UTF-16 code units as per LSP). Derives `Copy`.

**postfix trigger** — A trigger pattern of the form `<expr>.<word>`, e.g. `xs.fod`.
The engine replaces the entire `<expr>.<word>` span with expanded code.

**prime directive** — The non-negotiable rule: the engine must _never_ expand a trigger
inside a string literal, a comment, or at an identifier declaration position. Encoded
as `LexicalClass::forbids_expansion()` and verified by INV-1.

**proptest** — The property-based testing library used to verify the INV-\* invariants.
Generates random inputs and checks that structural properties hold for all of them.

## R

**Range** — The `snipper-core` value type `{ start: Position, end: Position }` covering
a span in a document. Used in `TextEdit` and `PostfixContext`. Derives `Copy`.

**Reactor** — A planned mobile code editor that will consume `snipper-core` and
`snipper-context` directly. The portability constraint (INV-5, no `lsp_types` in the
public API) exists to keep these crates usable outside an LSP host.

**receiver** — The expression to the left of the dot in a postfix trigger. For `xs.fod`,
`xs` is the receiver. Stored as a `String` in `PostfixContext`.

## S

**S-expression (sexpr)** — A text representation of a CST node tree used as the
canonical format for golden test snapshots. Stable, human-readable, and diffable with
standard tools. See `docs/representation-formats.md` and ADR-0003.

## T

**TextEdit** — The `snipper-core` value type `{ range: Range, new_text: String }` that
describes a single atomic edit to apply to a document. The primary output of the
expansion engine.

**tree-sitter** — An incremental, error-recovering parsing library. The default CST
backend (`backend-treesitter` feature, enabled by default). See ADR-0002.

**trigger** — The word suffix that activates a postfix expansion, e.g. `fod` or
`foreach`. Stored in `PostfixContext.trigger`.

**typos** — A fast spell-checking CLI tool used in the lint gate. Domain-specific words
that are intentionally non-standard are listed in `typos.toml` under
`[default.extend-words]`.

## W

**workspace** — The Cargo workspace root at the repository root. Contains
`snipper-core`, `snipper-context`, `snipper-lsp`, and `snipper-cli` as members;
`fuzz/` is excluded and uses its own `Cargo.toml` with a nightly toolchain.
