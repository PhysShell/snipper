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

Extensions contain no engine logic. The `snipper-lsp` binary is either bundled in release
builds or discovered via a user-configurable path setting. Path settings are passed to the
server via LSP `initializationOptions`; env vars are the lowest-priority fallback. Platform
differences (`win32` `.exe` suffix, arch-specific subdirectory) are handled in the extension
binary locator, not in the server.

Command identity (trigger, label, command ID) is generated from `snippets/**/*.toml` by the
`xtask generate-extension-manifests` tool (see ADR-0009) so that both extensions stay in
sync with the canonical TOML definitions without hand-maintenance.

Command bodies contain LSP snippet-format tabstops (`${1:...}`, `$0`). Extensions must
receive the body as a `workspace/executeCommand` result string and apply it via the
IDE-native snippet-insertion API (`editor.action.insertSnippet` in VS Code;
`IVsExpansionManager` in Visual Studio). The server must not apply command bodies via
`workspace/applyEdit`.

## Consequences

- Zero-config installation: users install from the marketplace and get completions without
  touching `PATH` or config files.
- Engine and protocol are fully decoupled from editor UI code — a bug fix in `snipper-lsp`
  does not require republishing extensions.
- Each extension must handle binary discovery, platform differences, server restarts on
  crash, and IDE-native snippet insertion.
- Adding a third editor requires only a new thin wrapper with no changes to `snipper-lsp`
  or the engine crates.
- `snipper.*` commands must return the snippet body as a result string; the server is
  authoritative for expansion logic but clients are authoritative for insertion mechanics.
