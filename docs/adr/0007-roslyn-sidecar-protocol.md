---
title: "ADR-0007: Roslyn sidecar IPC protocol"
status: Accepted
date: 2026-06-02
deciders: [snipper-team]
---

# ADR-0007 — Roslyn Sidecar IPC Protocol

## Context

Tree-sitter provides structural classification (S1–S7) but cannot resolve
receiver types.  Type-aware filtering — `fod` only for `IEnumerable<T>`,
for example — requires a C#-specific semantic API.  Roslyn is the
authoritative C# compiler API; it runs inside a .NET process.

`snipper-lsp` is a Rust process.  We need a lightweight, reliable bridge
between the Rust host and the .NET Roslyn analysis process.

## Decision

**Transport**: JSON-RPC 2.0 over stdin/stdout, one JSON object per line
(newline-delimited).  No named pipes, no TCP sockets, no shared memory.

**Lifecycle**: one sidecar process per LSP workspace (not per document).
The sidecar is spawned lazily on the first C# `textDocument/didOpen`
notification and kept alive for the session.  `snipper-lsp` detects
broken-pipe errors and falls back to CST-only mode transparently.

**Warm-up**: the sidecar is started in a background task on `didOpen`
so that Roslyn workspace initialisation (typically < 1 s for in-memory
single-file analysis) finishes before the first completion request arrives.

**Request timeout**: 200 ms per query.  On timeout, `snipper-lsp` marks the
sidecar as dead (`None`) and operates in CST-only mode for the remainder of
the session.

**Discovery**: the sidecar executable path is read from the `SNIPPER_ROSLYN`
environment variable.  When the variable is unset or the path does not exist,
the sidecar is silently disabled.

## Protocol

### Method `receiverType`

**Request**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "receiverType",
  "params": {
    "source": "var xs = new List<string>(); xs.fo",
    "offset": 34
  }
}
```

`offset` is a byte offset (0-based) into `source`, pointing inside or
immediately after the trigger text following the dot.

**Response — type resolved**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "types": [
      "System.Collections.Generic.List<string>",
      "System.Collections.Generic.IList<string>",
      "System.Collections.Generic.ICollection<string>",
      "System.Collections.Generic.IEnumerable<string>",
      "System.Collections.IList",
      "System.Collections.ICollection",
      "System.Collections.IEnumerable"
    ]
  }
}
```

`types` is the concrete receiver type followed by all implemented
interfaces and reachable base types, from most- to least-specific.
`System.Object` is omitted.

**Response — type unresolvable**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { "types": [] }
}
```

An empty `types` array means Roslyn could not resolve the type (parse
error, missing references, etc.).  `snipper-lsp` treats this identically
to the sidecar being unavailable — no rule is filtered out.

## Type filtering in `match_postfix`

Rules with `requires = "Enumerable"` in their TOML are filtered as follows:

| Sidecar state | Result |
|---|---|
| Unavailable / timed-out | `receiver_type = None` → rule shown (conservative) |
| Empty types list | `receiver_type = Some([])` → rule shown (conservative) |
| Types list, keyword absent | `receiver_type = Some([...])` → rule **hidden** |
| Types list, keyword present | `receiver_type = Some([...])` → rule shown, ranked first |

`requires` is a comma-separated list; a rule passes if **any** keyword
appears as a substring (case-insensitive) in **any** element of `types`.

## Consequences

- Zero external dependencies for `snipper-lsp` beyond spawning a subprocess.
- Sidecar absence is a no-op; the feature degrades gracefully.
- The .NET sidecar is an isolated crate (`sidecar/Snipper.Roslyn/`) with no
  dependency on `snipper-lsp` internals.
- First-request latency risk is mitigated by eager warm-up in `didOpen`.
- Named-pipe or socket transports can be layered later without changing the
  JSON-RPC message format (only the transport changes).

## Alternatives Considered

| Option | Rejected because |
|---|---|
| Named pipe (Windows `\\.\pipe\…` / POSIX FIFO) | Platform-specific setup; complex security on Windows |
| TCP loopback socket | Requires port management; firewall/proxy issues in some CI environments |
| Shared memory | Requires `unsafe`; violates `#![forbid(unsafe_code)]` in `snipper-lsp` |
| OmniSharp HTTP API | Heavy dependency; licence uncertainty; startup overhead |
