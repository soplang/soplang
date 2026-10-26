"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { runTests } = require("@vscode/test-electron");

async function main() {
    const extensionDevelopmentPath = path.resolve(__dirname, "..");
    const repository = path.resolve(extensionDevelopmentPath, "../..");
    const binary = process.env.SOPLANG_TEST_BINARY || path.join(repository, "target", "debug", process.platform === "win32" ? "soplang.exe" : "soplang");
    if (!fs.existsSync(binary)) throw new Error(`Build Soplang first: ${binary}`);
    const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "soplang-vscode-"));
    const workspace = path.join(temporary, "workspace");
    fs.mkdirSync(path.join(workspace, ".vscode"), { recursive: true });
    fs.writeFileSync(path.join(workspace, ".vscode", "settings.json"), JSON.stringify({ "soplang.server.path": binary }));
    fs.writeFileSync(path.join(workspace, "main.sop"), "door n = 1\nqor(n)\n");
    try {
        const snippets = JSON.parse(fs.readFileSync(path.join(extensionDevelopmentPath, "snippets/soplang.json")));
        fs.writeFileSync(path.join(workspace, "module.sop"), "door value = 1\n");
        for (const [name, snippet] of Object.entries(snippets)) {
            const defaults = new Map();
            const text = snippet.body.join("\n").replace(/\$\{(\d+)(?::([^}]*))?\}/g, (_, id, value) => {
                if (value !== undefined) defaults.set(id, value);
                return defaults.get(id) || "";
            }).replace(/\$(\d+)/g, (_, id) => defaults.get(id) || "");
            const file = path.join(workspace, "snippet.sop");
            fs.writeFileSync(file, text);
            const checked = spawnSync(binary, ["check", file], { encoding: "utf8" });
            if (checked.status !== 0) throw new Error(`Invalid ${name} snippet: ${checked.stderr || checked.error}`);
        }
        const schemaSnippets = JSON.parse(fs.readFileSync(path.join(extensionDevelopmentPath, "snippets/schema.json")));
        for (const [name, snippet] of Object.entries(schemaSnippets)) {
            const source = snippet.body.join("\n").replace(/\$\{\d+:([^}]*)\}/g, "$1");
            const file = path.join(workspace, "schema.sop");
            fs.writeFileSync(file, source);
            const checked = spawnSync(binary, ["check", file], { encoding: "utf8" });
            if (checked.status !== 0) throw new Error(`Invalid schema ${name} snippet: ${checked.stderr || checked.error}`);
            fs.rmSync(file);
        }
        fs.writeFileSync(path.join(workspace, "schema.sop"), "jadwal qof {\n id abn furaha\n}\n");
        await runTests({
            extensionDevelopmentPath,
            extensionTestsPath: path.join(__dirname, "integration"),
            vscodeExecutablePath: process.env.SOPLANG_VSCODE_EXECUTABLE,
            version: process.env.SOPLANG_VSCODE_VERSION || "stable",
            launchArgs: [
                workspace,
                "--disable-workspace-trust", "--skip-welcome", "--skip-release-notes",
                "--disable-telemetry", "--disable-gpu", "--no-sandbox",
                `--user-data-dir=${path.join(temporary, "user")}`,
                `--extensions-dir=${path.join(temporary, "extensions")}`,
                ...(process.platform === "linux" ? ["--ozone-platform=headless"] : []),
            ],
        });
    } finally {
        fs.rmSync(temporary, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
    }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
