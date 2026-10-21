"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const test = require("node:test");

function extension({ trusted = true, executable = "soplang", start } = {}) {
    const instances = [], errors = [], commands = new Map(), events = {}, logs = [];
    const config = { "server.path": executable, "server.enabled": true };
    const workspace = {
        isTrusted: trusted,
        createFileSystemWatcher: () => ({ dispose() {} }),
        registerTextDocumentContentProvider: () => ({ dispose() {} }),
        workspaceFolders: [{ uri: { fsPath: path.resolve("workspace with spaces") } }],
        getConfiguration: () => ({ get: (key, fallback) => config[key] ?? fallback }),
        onDidChangeConfiguration: (handler) => { events.changed = handler; return { dispose() {} }; },
        onDidGrantWorkspaceTrust: (handler) => { events.trust = handler; return { dispose() {} }; },
    };
    class LanguageClient {
        constructor(id, name, server, options) {
            Object.assign(this, { id, name, server, options, stopped: 0, disposed: 0 });
            instances.push(this);
        }
        async start() { if (start) await start(); }
        async stop() { this.stopped++; }
        async dispose() { this.disposed++; }
    }
    const api = {
        workspace,
        window: {
            createOutputChannel: () => ({ appendLine: (text) => logs.push(text), dispose() {} }),
            showErrorMessage: async (message) => { errors.push(message); },
        },
        commands: {
            registerCommand: (name, handler) => { commands.set(name, handler); return { dispose() {} }; },
            executeCommand: async () => {},
        },
    };
    const module = { exports: {} };
    vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../extension.js"), "utf8"), {
        module, process,
        require: (name) => name === "vscode" ? api : name === "vscode-languageclient/node"
            ? { LanguageClient, TransportKind: { stdio: 0 } } : require(name),
    }, { filename: "extension.js" });
    return { ...module.exports, workspace, config, instances, errors, commands, events, logs };
}

test("starts the configured executable over stdio and disposes it on restart", async () => {
    const ext = extension({ executable: "./tools/soplang" });
    await ext.activate({ subscriptions: [] });
    assert.equal(ext.instances.length, 1);
    const first = ext.instances[0];
    assert.equal(first.server.command, path.resolve("workspace with spaces/tools/soplang"));
    assert.equal(JSON.stringify(first.server.args), '["lsp"]');
    assert.equal(first.server.transport, 0);
    assert.equal(first.server.options.shell, undefined);
    assert.equal(first.options.markdown.isTrusted, false);
    assert.equal(JSON.stringify(first.options.documentSelector), '[{"language":"soplang","scheme":"file"},{"language":"soplang","scheme":"untitled"},{"language":"soplang","scheme":"soplang-stdlib"}]');
    ext.config["server.path"] = "another-soplang";
    await ext.commands.get("soplang.restartServer")();
    assert.equal(first.stopped, 1);
    assert.equal(first.disposed, 1);
    assert.equal(ext.instances[1].server.command, "another-soplang");
    await ext.deactivate();
    assert.equal(ext.instances[1].stopped, 1);
    assert.equal(ext.instances[1].disposed, 1);
});

test("untrusted and disabled workspaces do not execute the language server", async () => {
    const ext = extension({ trusted: false });
    await ext.activate({ subscriptions: [] });
    assert.equal(ext.instances.length, 0);
    ext.workspace.isTrusted = true;
    await ext.events.trust();
    assert.equal(ext.instances.length, 1);
    ext.config["server.enabled"] = false;
    ext.events.changed({ affectsConfiguration: (name) => name === "soplang.server" });
    await ext.commands.get("soplang.restartServer")();
    assert.equal(ext.instances.length, 1);
    assert.equal(ext.instances[0].stopped, 1);
    await ext.deactivate();
});

test("failed startup reports a useful message and cleans up the client", async () => {
    const ext = extension({ start: async () => { throw new Error("ENOENT original detail"); } });
    await ext.activate({ subscriptions: [] });
    assert.equal(ext.errors.length, 1);
    assert.match(ext.errors[0], /soplang.server.path/);
    assert.ok(ext.logs.some((line) => line.includes("ENOENT original detail")));
    assert.equal(ext.instances[0].disposed, 1);
    await ext.deactivate();
});

test("deactivation during startup waits and stops the child exactly once", async () => {
    let finish;
    const gate = new Promise((resolve) => { finish = resolve; });
    const ext = extension({ start: () => gate });
    const active = ext.activate({ subscriptions: [] });
    while (ext.instances.length === 0) await Promise.resolve();
    const stopped = ext.deactivate();
    finish();
    await Promise.all([active, stopped]);
    assert.equal(ext.instances[0].stopped, 1);
    assert.equal(ext.instances[0].disposed, 1);
});
