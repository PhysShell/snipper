import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

import { registerCommands } from "./commands";
import { resolveServerPath } from "./serverPath";

let client: LanguageClient | undefined;

export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  const serverPath = resolveServerPath(context);

  const serverOptions: ServerOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "csharp" }],
    initializationOptions: buildInitOptions(),
  };

  client = new LanguageClient(
    "snipper",
    "Snipper",
    serverOptions,
    clientOptions,
  );

  await client.start();

  registerCommands(context, client);
}

export async function deactivate(): Promise<void> {
  await client?.stop();
}

/**
 * Build the LSP initializationOptions sent to snipper-lsp on startup.
 *
 * Settings hierarchy (highest priority first):
 *   1. IDE setting  (snipper.roslynPath)
 *   2. initializationOptions  ← what we send here
 *   3. SNIPPER_ROSLYN env var  ← server-side fallback
 */
function buildInitOptions(): Record<string, string> {
  const config = vscode.workspace.getConfiguration("snipper");
  const opts: Record<string, string> = {};

  const roslynPath = config.get<string>("roslynPath", "");
  if (roslynPath) {
    opts["roslynPath"] = roslynPath;
  }

  return opts;
}
