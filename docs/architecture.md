---
title: Architecture
status: draft
owners: []
updated: 2026-05-30
---

# Architecture

## Context

Snipperper is a portable structural expansion engine. It must work inside an
LSP server, a CLI tool, and eventually the Reactor mobile editor. The engine
is the shared core; adapters are separate crates.

## Goals

- Resolve postfix/prefix/surround/selection triggers to `TextEdit` patches
  using CST context (not heuristic string matching).
- Never expand a trigger inside a string literal, comment, or identifier
  declaration (prime directive).
- Engine crates (`snipper-core`, `snipper-context`) have no dependency on
  LSP types — INV-5.

## Non-goals

- Full language server implementation (LSP protocol handling lives in
  `snipper-lsp`).
- .NET Roslyn integration at MVP (deferred to `sidecar/Snipper.Roslyn/`).
- Binary serialization formats (deferred, see ADR-0003).

## Components

| Crate | Role |
| --- | --- |
| `snipper-core` | Value objects (`TextEdit`, `Range`, `Position`). No I/O. |
| `snipper-context` | CST parser wrapper, cursor classifier, predicate engine. |
| `snipper-lsp` | LSP adapter. Owns `lsp_types` dependency. |
| `snipper-cli` | `snipper` binary; `context` and `expand` subcommands. |

## Data flow

```text
Editor / LSP client
        |
        v
  snipper-lsp  (LSP protocol, lsp_types)
        |
        v
  snipper-context  (CST parse, LexicalClass, PostfixContext)
        |
        v
  snipper-core  (TextEdit, Range, Position)
        |
        v
  Template engine  (expand trigger -> TextEdit[])
```

## Key decisions (ADR index)

- ADR-0001: documentation template and open blockers.
- ADR-0002: Tier-1 backend (Tree-sitter vs ast-grep) — **open**.
- ADR-0003: representation formats and binary format deferral.

## Risks

- **Prime directive regression** — mitigated by INV-1 proptest + golden
  fixtures for string/comment/decl cases.
- **LSP type leakage** — mitigated by INV-5 compilation test.
- **Backend lock-in** — mitigated by differential fuzz test (ADR-0002).

## Definition of Done

- [ ] `just verify` is green (zero markdownlint/fmt/clippy/deny/typos
  warnings).
- [ ] INV-1 through INV-5 property tests are green.
- [ ] Fuzz targets `parse_context`, `receiver_range`, `render_template`
  pass 60 s smoke run.
- [ ] `snipper context --format {tree,sexpr,json}` works and is golden-tested.
- [ ] ADR-0001 blockers are resolved or explicitly deferred.
