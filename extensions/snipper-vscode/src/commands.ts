import * as vscode from "vscode";
import type { LanguageClient } from "vscode-languageclient/node";
import { SNIPPER_COMMANDS } from "./commands.generated";

/**
 * Register all snipper.* VS Code commands.
 *
 * Each command calls workspace/executeCommand on the server, receives the
 * snippet body string, and inserts it via editor.action.insertSnippet so
 * that tabstop placeholders (${1:...}, $0) activate correctly.
 */
export function registerCommands(
  context: vscode.ExtensionContext,
  client: LanguageClient,
): void {
  for (const command of SNIPPER_COMMANDS) {
    context.subscriptions.push(
      vscode.commands.registerCommand(command, () =>
        invokeSnipperCommand(command, client),
      ),
    );
  }
}

async function invokeSnipperCommand(
  command: string,
  client: LanguageClient,
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    return;
  }

  let body: unknown;
  try {
    body = await client.sendRequest("workspace/executeCommand", {
      command,
      arguments: [
        {
          textDocument: { uri: editor.document.uri.toString() },
          position: editor.selection.active,
        },
      ],
    });
  } catch (err) {
    void vscode.window.showErrorMessage(
      `Snipper: command failed — ${String(err)}`,
    );
    return;
  }

  if (typeof body !== "string" || body.length === 0) {
    return;
  }

  await vscode.commands.executeCommand("editor.action.insertSnippet", {
    snippet: body,
  });
}
