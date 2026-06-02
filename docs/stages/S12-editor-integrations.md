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

## Snippet tabstop insertion

The S9 `execute_command` handler inserts command bodies via
`workspace/applyEdit` (`InsertTextFormat::PlainText`). LSP `workspace/applyEdit`
has no snippet-format field, so tabstop markers (`${1:ClassName}`, `$0`) are
inserted as literal text rather than activating.

S12 must fix this at two levels:

1. **Server change** — `execute_command` must return the body as a
   `serde_json::Value::String` result instead of applying the edit itself.
   The server becomes a pure data provider; clients decide how to insert.
2. **Extension insertion** — the VS Code extension must call
   `workspace/executeCommand`, receive the body string, then apply it with
   `vscode.commands.executeCommand('editor.action.insertSnippet', { snippet: body })`.
   The Visual Studio extension must use its equivalent snippet-insertion API.

Command bodies in `snippets/csharp/commands.toml` must not be changed;
the tabstop syntax is correct and will activate once extensions handle
insertion properly.

## Inputs → Outputs

**In:** `snipper-lsp` binary (S4); `workspace/executeCommand` handler (S9).

**Out:** two editor extensions (`extensions/snipper-vscode/`,
`extensions/snipper-vs/`); marketplace-ready `.vsix` artefacts;
updated `execute_command` handler that returns body as result value.

## Acceptance criteria

### S12.1 VS Code

- Installing the VSIX and opening a `.cs` file triggers Snipper completions
  (postfix, prefix, surround) without any manual configuration.
- "Snipper: Scaffold constructor" appears in the command palette, inserts the
  constructor stub at the cursor, and activates tabstop navigation
  (`${1:ClassName}` is selected; `$0` is the final cursor position).
- Tabstop markers are never inserted as literal text.
- Extension activates only on C# documents (no activation on `.rs`, `.py`,
  etc.).
- `snipper.serverPath` setting overrides the bundled binary path.
- Extension handles `snipper-lsp` crash by restarting the server.

### S12.2 Visual Studio

- Installing the VSIX and opening a `.cs` file triggers Snipper completions.
- "Snipper: Scaffold constructor" appears in the Visual Studio command
  palette, inserts the stub, and activates tabstop navigation.
- Tabstop markers are never inserted as literal text.
- Extension activates only on C# documents.
- Options page exposes the binary path override.

## See also

- [Architecture](../architecture.md)
- [ADR-0008 — Editor extension packaging](../adr/0008-editor-extension-packaging.md)
- [S9 — Command expansions](S9-command-expansions.md)
- [S4 — LSP adapter MVP](S4-lsp-adapter-mvp.md)
