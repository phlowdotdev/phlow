const fs = require('fs');
const path = require('path');
const onigasm = require('onigasm');
const { OnigScanner, OnigString } = onigasm;
const vscodeTextMate = require('vscode-textmate');

(async () => {
    const wasmPath = require.resolve('onigasm/lib/onigasm.wasm');
    const wasmBin = fs.readFileSync(wasmPath);
    const wasmArrayBuffer = wasmBin.buffer.slice(wasmBin.byteOffset, wasmBin.byteLength + wasmBin.byteOffset);
    await onigasm.loadWASM(wasmArrayBuffer);

    const onigLib = Promise.resolve({ createOnigScanner: (p) => new OnigScanner(p), createOnigString: (s) => new OnigString(s) });
    const registry = new vscodeTextMate.Registry({
        onigLib, loadGrammar: async (s) => {
            const grammarPath = { 'source.phlow': path.join(__dirname, '..', 'syntaxes', 'phlow.tmLanguage.json'), 'source.rhai': path.join(__dirname, '..', 'syntaxes', 'rhai.tmLanguage.json') }[s];
            return vscodeTextMate.parseRawGrammar(fs.readFileSync(grammarPath, 'utf8'), grammarPath);
        }
    });

    const grammar = await registry.loadGrammar('source.phlow');
    const testLines = [
        '      message: true',
        '      message: "Warning! Something might be wrong."',
        '      message: [1, "true", false]'
    ];

    for (const line of testLines) {
        const toks = grammar.tokenizeLine(line, vscodeTextMate.INITIAL);
        console.log('LINE:', line);
        for (const tk of toks.tokens) {
            const txt = line.substring(tk.startIndex, tk.endIndex);
            console.log(`  token:'${txt}' -> scopes=[${tk.scopes.join(', ')}]`);
        }
    }
})();
