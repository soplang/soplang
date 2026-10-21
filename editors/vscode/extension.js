"use strict";

const path = require("node:path");
const vscode = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;
let output;
let watcher;
let disposed = false;
let pending = Promise.resolve();

async function stopClient() {
    const previous = client;
    client = undefined;
    if (previous) {
        try {
            await previous.stop();
        } finally {
            await previous.dispose();
        }
    }
}

async function restart() {
    await stopClient();
    if (disposed || !vscode.workspace.isTrusted) return;
    const folder = vscode.workspace.workspaceFolders?.[0];
    const config = vscode.workspace.getConfiguration("soplang");
    if (!config.get("server.enabled", true)) return;
    let command = config.get("server.path", "soplang").trim();
    if (!command) throw new Error("soplang.server.path ma madhnaan karo.");
    if (!path.isAbsolute(command) && /[/\\]/.test(command)) {
        command = path.resolve(folder?.uri.fsPath || process.cwd(), command);
    }
    client = new LanguageClient("soplang", "Soplang", {
        command,
        args: ["lsp"],
        transport: TransportKind.stdio,
        options: folder ? { cwd: folder.uri.fsPath } : {},
    }, {
        documentSelector: [
            { language: "soplang", scheme: "file" },
            { language: "soplang", scheme: "untitled" },
            { language: "soplang", scheme: "soplang-stdlib" },
        ],
        outputChannel: output,
        traceOutputChannel: output,
        synchronize: { fileEvents: watcher },
        markdown: { isTrusted: false },
    });
    await client.start();
    if (disposed) await stopClient();
}

function requestRestart() {
    pending = pending.then(restart).catch(async (error) => {
        await stopClient().catch(() => {});
        if (disposed) return;
        output.appendLine(String(error?.stack || error));
        const action = await vscode.window.showErrorMessage(
            "Adeegga luuqadda Soplang lama bilaabi karin. Hubi rakibidda ama sax soplang.server.path.",
            "Fur dejimaha",
        );
        if (action === "Fur dejimaha") {
            await vscode.commands.executeCommand("workbench.action.openSettings", "soplang.server.path");
        }
    });
    return pending;
}

function activate(context) {
    disposed = false;
    output = vscode.window.createOutputChannel("Soplang");
    context.subscriptions.push(output);
    watcher = vscode.workspace.createFileSystemWatcher("**/*.{sop,toml,lock}");
    context.subscriptions.push(watcher);
    context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider("soplang-stdlib", {
        provideTextDocumentContent: async (uri) => {
            if (!client) throw new Error("Adeegga luuqadda Soplang ma furna.");
            return (await client.sendRequest("soplang/source", { uri: uri.toString() })) || "";
        },
    }));
    context.subscriptions.push(vscode.commands.registerCommand("soplang.restartServer", requestRestart));
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("soplang.server")) void requestRestart();
    }));
    context.subscriptions.push(vscode.workspace.onDidGrantWorkspaceTrust(requestRestart));
    return requestRestart();
}

async function deactivate() {
    disposed = true;
    await pending;
    await stopClient();
}

module.exports = { activate, deactivate };
