---
title: Architecture
status: draft
owners: []
updated: 2026-05-31
---

# Architecture

## Context

The Snipper engine must work inside an LSP server, a CLI tool, and eventually
the Reactor mobile editor. It is the shared core; all adapters are separate
crates.

## Goals

- Resolve postfix, prefix, surround, selection, and command triggers to
  `TextEdit` patches using CST context (not heuristic string matching).
- Never expand a trigger inside a string literal, comment, or identifier
  declaration (prime directive).
- Engine crates (`snipper-core`, `snipper-context`) have no dependency on
  LSP types — INV-5.
- For C#: enrich context with Roslyn semantic data (receiver type) to enable
  type-aware template filtering (S8).

## Non-goals

- Full language server implementation (LSP protocol handling lives in
  `snipper-lsp`).
- Binary serialization formats (deferred, see ADR-0003).
- Cloud-based telemetry or ranking (stats are local-only, see S11).

## Components

| Crate / Project | Role |
| --- | --- |
| `snipper-core` | Value objects (`TextEdit`, `Range`, `Position`). No I/O. |
| `snipper-context` | CST parser wrapper, cursor classifier, predicate engine. |
| `snipper-lsp` | LSP adapter. Owns `lsp_types` dependency. |
| `snipper-cli` | `snipper` binary; `context` and `expand` subcommands. |
| `sidecar/Snipper.Roslyn` | .NET sidecar; receiver-type info via IPC (S8). |
| `extensions/snipper-vscode` | VS Code extension; thin `vscode-languageclient` wrapper (S12). |
| `extensions/snipper-vs` | Visual Studio VSIX; `ILanguageClient` wrapper (S12). |
| `crates/xtask` | Dev-time code-generation tool; generates extension manifests from TOML (S12). |

## Semantic enrichment strategy

Tree-sitter classifies cursor positions structurally (inside string, comment,
declaration, or after dot). It does not know the _type_ of the receiver.
Reliable type-aware filtering — `fod` only for collections, for example —
requires a language-specific semantic API:

- **C#**: Roslyn sidecar (S8) provides receiver type over a local IPC
  channel. This is the primary path for C# semantic enrichment.
- **Other languages**: language-specific APIs or conservative heuristics,
  resolved per language in S7+.

Tree-sitter remains the CST backbone (fast, cross-language, pure Rust).
Roslyn is a first-class planned component, not a deferred nice-to-have.

## Data flow

```text
Editor / LSP client
        |
        v
  snipper-lsp  (LSP protocol, lsp_types)
        |             |
        v             v
  snipper-context   Snipper.Roslyn sidecar
  (CST parse,       (receiver type — C# only, S8)
   LexicalClass,
   PostfixContext)
        |             |
        v-------------+
  Template engine
  (prefix-match -> type filter -> rank -> TextEdit[])
        |
        v
  snipper-core  (TextEdit, Range, Position)
```

## Key decisions (ADR index)

- ADR-0001: documentation template and open blockers.
- ADR-0002: Tier-1 backend (Tree-sitter vs ast-grep) — **open**.
- ADR-0003: representation formats and binary format deferral.
- ADR-0004: Rust API guidelines on public surfaces.
- ADR-0005: Prefix/postfix conflict-resolution strategy.
- ADR-0006: Surround expansion trigger and prime-directive enforcement.
- ADR-0007: Roslyn sidecar IPC protocol.
- ADR-0008: Editor extension packaging strategy — **proposed**.
- ADR-0009: Generate extension manifests from TOML rules — **proposed**.

## Staged delivery

See [`docs/stages/`](stages/README.md) for the full plan (S0–S12).
Current state: S0–S1, S3–S9 done; S2, S10, S12 not started; S11 deferred.

## Risks

- **Prime directive regression** — mitigated by INV-1 proptest + golden
  fixtures for string/comment/decl cases.
- **LSP type leakage** — mitigated by INV-5 compilation test.
- **Backend lock-in** — mitigated by differential fuzz test (ADR-0002).
- **Roslyn startup latency** — sidecar workspace load can take 2–5 s;
  `classify` must fall back to CST-only during sidecar initialisation.

## Definition of Done

See each stage's acceptance criteria in [`docs/stages/`](stages/).
Global DoD requires all stages S0–S12 green:

- [ ] `just verify` is green (lint, fmt, clippy, deny, typos).
- [ ] INV-1 through INV-5 property tests are green.
- [ ] Fuzz targets `parse_context`, `render_template` pass 60 s smoke run.
- [ ] `snipper context --format {tree,sexpr,json}` is golden-tested.
- [ ] Type-aware filtering via Roslyn sidecar passes S8 acceptance criteria.
- [ ] ADR-0001 blockers are resolved or explicitly deferred.
