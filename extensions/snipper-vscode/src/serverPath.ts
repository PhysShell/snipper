import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

/**
 * Resolve the path to the snipper-lsp binary.
 *
 * Resolution order:
 *   1. `snipper.serverPath` user setting (explicit override)
 *   2. Bundled binary at `bin/<platform>/snipper-lsp[.exe]`
 *   3. `snipper-lsp` on PATH (dev / non-packaged installs)
 */
export function resolveServerPath(context: vscode.ExtensionContext): string {
  const configured = vscode.workspace
    .getConfiguration("snipper")
    .get<string>("serverPath", "");
  if (configured) {
    return configured;
  }

  const platformDir = getPlatformDir();
  const exe = process.platform === "win32" ? "snipper-lsp.exe" : "snipper-lsp";
  const bundled = path.join(context.extensionPath, "bin", platformDir, exe);
  if (fs.existsSync(bundled)) {
    return bundled;
  }

  return exe; // fall through to PATH
}

function getPlatformDir(): string {
  const { platform, arch } = process;
  if (platform === "win32") {
    return "win32-x64";
  }
  if (platform === "darwin") {
    return arch === "arm64" ? "darwin-arm64" : "darwin-x64";
  }
  return "linux-x64";
}
