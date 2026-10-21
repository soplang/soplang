"use strict";

const assert = require("node:assert/strict");
const vscode = require("vscode");

async function eventually(operation, message) {
    const expires = Date.now() + 15000;
    let value;
    while (Date.now() < expires) {
        value = await operation();
        if (value) return value;
        await new Promise((resolve) => setTimeout(resolve, 100));
    }
    assert.fail(`${message}: ${JSON.stringify(value)}`);
}

async function run() {
    const extension = vscode.extensions.getExtension("soplang.soplang");
    assert.ok(extension, "Soplang extension must be installed in the extension host");
    await extension.activate();
    const document = await vscode.workspace.openTextDocument({ language: "soplang", content: 'hawl salaan(){celi 42}\nqor(salaan())\n' });
    await vscode.window.showTextDocument(document);
    const position = new vscode.Position(1, 6);
    const definitions = await eventually(async () => {
        const rows = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", document.uri, position);
        return rows?.length ? rows : undefined;
    }, "definition provider did not become ready");
    assert.equal((definitions[0].uri || definitions[0].targetUri).toString(), document.uri.toString());
    const hover = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, position);
    assert.ok(hover.length > 0);
    const renamed = await vscode.commands.executeCommand("vscode.executeDocumentRenameProvider", document.uri, position, "salaan_cusub");
    assert.ok(renamed.get(document.uri).length >= 2);
    const formatted = await vscode.commands.executeCommand("vscode.executeFormatDocumentProvider", document.uri, { tabSize: 4, insertSpaces: true });
    assert.ok(formatted.length > 0);

    const invalid = await vscode.workspace.openTextDocument({ language: "soplang", content: "door x = @\n" });
    await vscode.window.showTextDocument(invalid);
    await eventually(() => vscode.languages.getDiagnostics(invalid.uri).some((row) => row.severity === vscode.DiagnosticSeverity.Error), "parser diagnostic was not published");
    await vscode.commands.executeCommand("soplang.restartServer");
    await eventually(async () => {
        const rows = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", document.uri, position);
        return rows?.length > 0;
    }, "server restart did not restore navigation");
    console.log("SOPLANG_VSCODE_INTEGRATION_OK");
}

module.exports = { run };
