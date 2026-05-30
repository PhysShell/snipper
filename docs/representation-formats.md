---
title: Representation Formats
status: stable
owners: []
updated: 2026-05-30
---

# Representation Formats

> Normative. Mirrors task spec §9. Rationale: ADR-0003.

## Principle

There is no single ideal format for all tasks. Snipper deliberately uses
different representations for different purposes. Trying to unify everything
into one format is the mistake that breaks on the first real user.

## Normative selection table

| Task | Format | Status |
| --- | --- | --- |
| Internal tree for edits | **CST with ranges** (Tree-sitter parse tree) | required |
| User-defined rules | **TOML** | required |
| Sidecar / JSON-RPC contract | **JSON** | required |
| CLI debug output for humans | **indented ASCII tree** | required |
| Context snapshots in tests | **S-expression** | required |
| Machine CLI output for tooling | **JSON** | required |
| Diagrams in docs | **Mermaid** (DOT for complex graphs) | as needed |
| Internal test ADT dumps | constructor notation (ML/Rust style) | as needed |
| Node paths in golden tests | path notation (`/unit/class[0]/...`) | as needed |
| Binary format (protobuf, etc.) | — | **deferred** (cache/IPC, not MVP) |
| Zipper navigation model | — | **deferred** (for Reactor) |
| Edge/adjacency list | — | **deferred** (Reactor indexes) |

## Why CST, not AST

Snipperper makes **text edits** and needs exact `range` for every token:

```text
users.fod
^^^^^ receiver
     ^ dot
      ^^^ trigger
```

A pure AST loses the dot, parentheses, and trivia. An edit built on it
duplicates the receiver. The internal engine always works on a CST/parse
tree with ranges.

## CLI: three required output formats

The `snipper context` command must support `--format {tree,sexpr,json}`.

### `--format tree` (human)

```text
context
├── language: csharp
├── cursor: 12:20
├── lexical: code, after_dot
├── postfix
│   ├── receiver: users.Where(x => x.Active)
│   ├── trigger: fod
│   └── range: 12:8..12:35
└── semantic
    ├── type: IEnumerable<User>
    └── predicates: enumerable
```

### `--format sexpr` (snapshot tests, stable, diffable)

```text
(context
  (language "csharp")
  (cursor 12 20)
  (lexical code after_dot)
  (postfix
    (receiver "users.Where(x => x.Active)")
    (trigger "fod")
    (range "12:8" "12:35"))
  (semantic
    (type "IEnumerable<User>")
    (predicate enumerable)))
```

`sexpr` is the canonical format for golden snapshots: stable under cosmetic
changes, easy to diff.

### `--format json` (tooling)

```json
{
  "language": "csharp",
  "cursor": { "line": 12, "character": 20 },
  "lexical": ["code", "after_dot"],
  "postfix": {
    "receiver": "users.Where(x => x.Active)",
    "trigger": "fod",
    "range": {
      "start": { "line": 12, "character": 8 },
      "end":   { "line": 12, "character": 35 }
    }
  },
  "semantic": {
    "type": "IEnumerable<User>",
    "predicates": ["enumerable"]
  }
}
