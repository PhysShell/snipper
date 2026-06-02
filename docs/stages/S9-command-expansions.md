# S9 — Command Expansions

| Field | Value |
| --- | --- |
| Status | Done |
| Depends on | [S4](S4-lsp-adapter-mvp.md) |
| ADRs | — |

## Goal

Add the fifth expansion type: command expansions triggered by name
regardless of cursor position, surfaced via `workspace/executeCommand`.

## Inputs → Outputs

**In:** LSP adapter (S4).

**Out:** command registry; `snippets/csharp/commands.toml`; commands
visible in the editor command palette.

## Approach

1. Extend the template format with `type = "command"`.
2. Implement a command registry in `snipper-lsp`: commands registered on
   server init, invoked via `workspace/executeCommand`.
3. Built-in C# commands: scaffold constructor, scaffold property,
   implement interface stub.
4. Wire into the LSP `initialize` response with the supported commands
   list.

## Acceptance criteria

- Running "Snipper: Scaffold constructor" inserts a constructor stub at
  the cursor.
- Command names are deterministic across restarts.

## Known limitations

`execute_command` applies the body via `workspace/applyEdit`
(`InsertTextFormat::PlainText`). LSP `workspace/applyEdit` carries no snippet
format, so tabstop markers (`${1:ClassName}`, `$0`) are inserted as literal
text when invoked from a generic LSP client. S12 resolves this: the handler
will return the body as a result value and extensions will apply it via their
native snippet-insertion API (e.g. `editor.action.insertSnippet` in VS Code).

## See also

- [Architecture](../architecture.md)
- [S4 — LSP adapter MVP](S4-lsp-adapter-mvp.md)
- [S12 — Editor integrations](S12-editor-integrations.md)
