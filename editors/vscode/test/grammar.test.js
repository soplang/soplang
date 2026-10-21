"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const tm = require("vscode-textmate");
const onig = require("vscode-oniguruma");

const ready = (async () => {
    const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
    await onig.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
    const registry = new tm.Registry({
        onigLib: Promise.resolve({ createOnigScanner: (patterns) => new onig.OnigScanner(patterns), createOnigString: (text) => new onig.OnigString(text) }),
        loadGrammar: async () => tm.parseRawGrammar(fs.readFileSync(path.join(__dirname, "../syntaxes/soplang.tmLanguage.json"), "utf8"), "soplang.json"),
    });
    return registry.loadGrammar("source.soplang");
})();

async function tokens(source) {
    const grammar = await ready;
    let state = tm.INITIAL;
    const output = [];
    for (const line of source.split("\n")) {
        const result = grammar.tokenizeLine(line, state);
        state = result.ruleStack;
        output.push(result.tokens.map((token) => ({ text: line.slice(token.startIndex, token.endIndex), scopes: token.scopes })));
    }
    return output.flat();
}

test("comments, keywords and numbers compile as real TextMate rules", async () => {
    const result = await tokens("door n = 42 // comment\n/* block\n comment */\nqor(n)\n# not a Soplang comment");
    assert.ok(result.some((token) => token.text === "door" && token.scopes.includes("keyword.control.soplang")));
    assert.ok(result.some((token) => token.text === "42" && token.scopes.includes("constant.numeric.soplang")));
    assert.ok(result.some((token) => token.text.includes("comment") && token.scopes.includes("comment.line.double-slash.soplang")));
    assert.ok(result.some((token) => token.text.includes("comment") && token.scopes.includes("comment.block.soplang")));
    assert.ok(result.filter((token) => token.text.includes("#")).every((token) => !token.scopes.some((scope) => scope.startsWith("comment"))));
});

test("raw and ordinary backslashes differ from escaped strings", async () => {
    const result = await tokens(String.raw`door a = "a\nb"` + "\n" + String.raw`door b = r'a\nb'` + "\n" + String.raw`door c = e"a\nb"`);
    const escapes = result.filter((token) => token.scopes.includes("constant.character.escape.soplang"));
    assert.equal(escapes.length, 1);
    assert.equal(escapes[0].text, String.raw`\n`);
    assert.ok(result.some((token) => token.scopes.includes("string.quoted.single.r.soplang")));
});

test("interpolation includes nested expressions and strings", async () => {
    const result = await tokens('door s = f"hello {{ { { n: 2 }.n } }} {qor(\'inside\')}"');
    assert.ok(result.some((token) => token.text === "2" && token.scopes.includes("meta.interpolation.soplang") && token.scopes.includes("constant.numeric.soplang")));
    assert.ok(result.some((token) => token.text === "inside" && token.scopes.includes("string.quoted.single.r.soplang")));
    assert.ok(result.some((token) => token.text === "{{" && token.scopes.includes("constant.character.escape.soplang")));
});

test("triple strings and block comments end before following source", async () => {
    const result = await tokens('door text = e"""first\nsecond\\n\n"""\ndoor n = 3');
    const declarations = result.filter((token) => token.text === "door");
    assert.equal(declarations.length, 2);
    assert.ok(declarations[1].scopes.every((scope) => !scope.startsWith("string")));
    assert.ok(result.some((token) => token.text.includes("second") && token.scopes.includes("string.quoted.double.e.multiline.soplang")));
});

test("language configuration recognizes Unicode words and correct comment delimiters", () => {
    const config = JSON.parse(fs.readFileSync(path.join(__dirname, "../language-configuration.json")));
    assert.equal(config.comments.lineComment, "//");
    assert.deepEqual(config.comments.blockComment, ["/*", "*/"]);
    const words = new RegExp(config.wordPattern, "g");
    assert.deepEqual("magac.waalid qof_2 magacaé".match(words), ["magac", "waalid", "qof_2", "magacaé"]);
});

test("all quote/prefix combinations release their string scopes at the delimiter", async () => {
    for (const prefix of ["", "r", "e", "f"]) {
        for (const quote of ['"', "'"]) {
            for (const width of [1, 3]) {
                const delimiter = quote.repeat(width);
                const contents = prefix === "f" ? "text {1 + 2}" : "text";
                const result = await tokens(`door s = ${prefix}${delimiter}${contents}${delimiter}; door next = 9`);
                const declarations = result.filter((token) => token.text === "door");
                assert.equal(declarations.length, 2, `${prefix}${delimiter}`);
                assert.ok(declarations[1].scopes.every((scope) => !scope.startsWith("string")), `${prefix}${delimiter}`);
            }
        }
    }
});
