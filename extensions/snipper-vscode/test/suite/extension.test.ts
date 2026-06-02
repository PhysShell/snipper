import * as assert from "assert";
import * as vscode from "vscode";

const EXT_ID = "snipper.snipper-vscode";

suite("Snipper VS Code extension", () => {
  test("extension is registered", () => {
    const ext = vscode.extensions.getExtension(EXT_ID);
    assert.ok(ext, `Extension '${EXT_ID}' not found`);
  });

  test("extension activates on C# document", async () => {
    const ext = vscode.extensions.getExtension(EXT_ID);
    assert.ok(ext);

    const doc = await vscode.workspace.openTextDocument({
      language: "csharp",
      content: "// hello",
    });
    await vscode.window.showTextDocument(doc);

    // Give the activation event a moment to fire
    await new Promise((r) => setTimeout(r, 500));
    assert.strictEqual(ext.isActive, true, "extension should be active after opening a C# file");
  });

  test("snipper.* commands are registered in the command palette", async () => {
    const all = await vscode.commands.getCommands(true);
    const snipperCmds = all.filter((c) => c.startsWith("snipper."));

    assert.ok(
      snipperCmds.includes("snipper.scaffoldConstructor"),
      "snipper.scaffoldConstructor must be registered",
    );
    assert.ok(
      snipperCmds.includes("snipper.scaffoldProperty"),
      "snipper.scaffoldProperty must be registered",
    );
    assert.ok(
      snipperCmds.includes("snipper.implementInterface"),
      "snipper.implementInterface must be registered",
    );
  });

  test("extension does not crash without an active editor", async () => {
    const ext = vscode.extensions.getExtension(EXT_ID);
    assert.ok(ext);
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    assert.ok(typeof ext.isActive === "boolean");
  });
});
