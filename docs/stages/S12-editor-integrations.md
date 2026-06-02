# S12 — Editor Integrations

| Field | Value |
| --- | --- |
| Status | Not started |
| Depends on | [S9](S9-command-expansions.md) |
| ADRs | [ADR-0008](../adr/0008-editor-extension-packaging.md) |

## Goal

Package `snipper-lsp` as a first-class editor extension for VS Code and
Visual Studio so that all five expansion types are available in real editors
without any manual server configuration.

## Subtasks

### S12.1 — VS Code extension

Build a TypeScript VS Code extension that:

1. Bundles or locates the `snipper-lsp` binary (bundled for release builds;
   overridable via `snipper.serverPath` setting).
2. Starts `snipper-lsp` as a language server via `vscode-languageclient`.
3. Registers `snipper.*` commands in the VS Code command palette; each
   command invokes `workspace/executeCommand` with a cursor argument.
4. Contributes a minimal settings point: `snipper.serverPath` (binary
   override) and `snipper.roslynPath` (Roslyn sidecar path).
5. Activates only on C# documents (`.cs`) to avoid unnecessary startup cost.

**Inputs:** `snipper-lsp` binary (S4), command registry (S9).

**Outputs:** `extensions/snipper-vscode/` TypeScript package; `snipper-*.vsix` artefact.

### S12.2 — Visual Studio extension

Build a C# VSIX extension that:

1. Implements `ILanguageClient` (`Microsoft.VisualStudio.LanguageServer.Client 17.x`)
   wrapping `snipper-lsp`.
2. Activates on `ContentType("CSharp")` documents.
3. Starts `snipper-lsp` as a child process; handles restarts on crash.
4. Exposes `snipper.*` commands through Visual Studio command routing
   (Tools menu or command palette).
5. Provides a Visual Studio options page for `snipper-lsp` binary path and
   `SNIPPER_ROSLYN` env-var override.

**Inputs:** `snipper-lsp` binary (S4), command registry (S9).

**Outputs:** `extensions/snipper-vs/` C# project; `Snipper.VS.vsix` artefact.

### Deferred: other editors

Neovim, Emacs, JetBrains Rider, and other LSP-capable editors can connect
to `snipper-lsp` directly with user-side configuration. Dedicated wrappers
are deferred until community demand is established.

## Inputs → Outputs

**In:** `snipper-lsp` binary (S4); `workspace/executeCommand` handler (S9).

**Out:** two editor extensions (`extensions/snipper-vscode/`,
`extensions/snipper-vs/`); marketplace-ready `.vsix` artefacts.

## Acceptance criteria

### S12.1 VS Code

- Installing the VSIX and opening a `.cs` file triggers Snipper completions
  (postfix, prefix, surround) without any manual configuration.
- "Snipper: Scaffold constructor" appears in the command palette and inserts
  the constructor stub at the cursor position.
- Extension activates only on C# documents (no activation on `.rs`, `.py`,
  etc.).
- `snipper.serverPath` setting overrides the bundled binary path.
- Extension handles `snipper-lsp` crash by restarting the server.

### S12.2 Visual Studio

- Installing the VSIX and opening a `.cs` file triggers Snipper completions.
- "Snipper: Scaffold constructor" appears in the Visual Studio command
  palette and inserts the stub.
- Extension activates only on C# documents.
- Options page exposes the binary path override.

## See also

- [Architecture](../architecture.md)
- [ADR-0008 — Editor extension packaging](../adr/0008-editor-extension-packaging.md)
- [S9 — Command expansions](S9-command-expansions.md)
- [S4 — LSP adapter MVP](S4-lsp-adapter-mvp.md)
