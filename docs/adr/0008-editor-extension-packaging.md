# ADR-0008: Package editor extensions as thin LSP-client wrappers

Date: 2026-06-02
Status: Proposed

## Context

`snipper-lsp` is a standards-compliant Language Server Protocol server.
Any LSP-capable editor can connect to it once the binary is on `PATH`, but
users should not be required to locate the binary or start the server
manually. Each first-class editor therefore needs a thin wrapper that
discovers the binary, starts the process, and wires up the LSP connection.

Two editors have been chosen as initial targets for S12:

- **VS Code** — large C# developer market share; straightforward to
  iterate; TypeScript extension model and `vscode-languageclient` make LSP
  integration low-friction.
- **Visual Studio** — the primary C# IDE for Windows; required to make
  the Roslyn sidecar (S8) accessible to its natural audience.

Other LSP-capable editors (Neovim, Emacs, JetBrains Rider) can connect
to `snipper-lsp` directly with user-side configuration; dedicated wrappers
are deferred until community demand is established.

## Decision

We ship one extension per editor, each acting as a minimal LSP-client
wrapper around `snipper-lsp`:

- **VS Code** (`extensions/snipper-vscode/`) — TypeScript extension using
  `vscode-languageclient`. Activates on C# documents (`.cs`); starts
  `snipper-lsp` as a child process; registers `snipper.*` VS Code commands
  that delegate to `workspace/executeCommand`.
- **Visual Studio** (`extensions/snipper-vs/`) — C# VSIX implementing
  `ILanguageClient` (`Microsoft.VisualStudio.LanguageServer.Client 17.x`).
  Activates on `ContentType("CSharp")`; starts `snipper-lsp` as a child
  process; exposes `snipper.*` commands through VS command routing.

Extensions contain no engine logic. The `snipper-lsp` binary is either
bundled in release builds or discovered via a user-configurable path
setting (`snipper.roslynPath` for VS Code; equivalent for Visual Studio).
Platform differences (`win32` `.exe` suffix, etc.) are handled in the
extension, not in the server.

## Consequences

- Zero-config installation: users install from the marketplace and get
  completions without touching `PATH` or config files.
- Engine and protocol are fully decoupled from editor UI code — a bug fix
  in `snipper-lsp` does not require republishing extensions.
- Each extension must handle binary discovery, platform differences, and
  server restarts on crash.
- Adding a third editor requires only a new thin wrapper with no changes
  to `snipper-lsp` or the engine crates.
- `snipper.*` commands must be forwarded via `workspace/executeCommand`;
  the server remains authoritative for all expansion logic (consistent with
  S9 design).
