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
    const libraryUse = await vscode.workspace.openTextDocument({ language: "soplang", content: 'keen "web" sida w\ndoor app = cusub w.Web()\n' });
    await vscode.window.showTextDocument(libraryUse);
    const libraryDefinitions = await eventually(async () => {
        const rows = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", libraryUse.uri, libraryUse.positionAt(libraryUse.getText().indexOf("Web") + 1));
        return rows?.length ? rows : undefined;
    }, "bundled class definition was not provided");
    const libraryUri = libraryDefinitions[0].uri || libraryDefinitions[0].targetUri;
    assert.equal(libraryUri.scheme, "soplang-stdlib");
    const library = await vscode.workspace.openTextDocument(libraryUri);
    assert.ok(library.getText().includes("qaab Web"));
    const schema = await vscode.workspace.openTextDocument({ language: "soplang-schema", content: "# People\njadwal qof {\n id abn furaha\n magac integer\n}\n" });
    await vscode.window.showTextDocument(schema);
    const schemaError = await eventually(() => vscode.languages.getDiagnostics(schema.uri).find((row) => row.code === "E059"), "schema diagnostic was not published");
    assert.equal(schemaError.range.start.line, 3);
    assert.equal(schemaError.range.start.character, 7);
    const schemaCompletion = await vscode.commands.executeCommand("vscode.executeCompletionItemProvider", schema.uri, new vscode.Position(3, 7));
    assert.ok(schemaCompletion.items.some((item) => item.label === "qoraal"));
    const schemaFile = await vscode.workspace.openTextDocument(vscode.Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "schema.sop"));
    assert.equal(schemaFile.languageId, "soplang-schema");
    console.log("SOPLANG_VSCODE_INTEGRATION_OK");
}

module.exports = { run };
