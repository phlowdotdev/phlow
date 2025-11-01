const fs = require('fs');
const path = require('path');
const onigasm = require('onigasm');
const { OnigScanner, OnigString } = onigasm;
const vscodeTextMate = require('vscode-textmate');

(async () => {
    try {
        const wasmPath = require.resolve('onigasm/lib/onigasm.wasm');
        const wasmBin = fs.readFileSync(wasmPath);
        const wasmArrayBuffer = wasmBin.buffer.slice(wasmBin.byteOffset, wasmBin.byteOffset + wasmBin.byteLength);
        await onigasm.loadWASM(wasmArrayBuffer);
    } catch (e) {
        console.error('Failed to load onigasm wasm.', e);
        process.exit(1);
    }

    const onigLib = Promise.resolve({ createOnigScanner: (patterns) => new OnigScanner(patterns), createOnigString: (s) => new OnigString(s) });
    const registry = new vscodeTextMate.Registry({
        onigLib, loadGrammar: async (scopeName) => {
            const grammarPath = {
                'source.phlow': path.join(__dirname, '..', 'syntaxes', 'phlow.tmLanguage.json'),
                'source.rhai': path.join(__dirname, '..', 'syntaxes', 'rhai.tmLanguage.json')
            }[scopeName];
            if (!grammarPath) return null;
            const content = fs.readFileSync(grammarPath, 'utf8');
            return vscodeTextMate.parseRawGrammar(content, grammarPath);
        }
    });

    const grammar = await registry.loadGrammar('source.phlow');
    if (!grammar) { console.error('Could not load phlow grammar'); process.exit(2); }

    const arg = process.argv[2] || path.join(__dirname, 'fixtures', 'sample.phlow');
    if (!fs.existsSync(arg)) { console.error('File not found:', arg); process.exit(3); }
    const lines = fs.readFileSync(arg, 'utf8').split(/\r?\n/);

    console.log('Tokenizing', arg);
    let ruleStack = vscodeTextMate.INITIAL;
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const toks = grammar.tokenizeLine(line, ruleStack);
        ruleStack = toks.ruleStack;
        // print tokens for lines that contain keywords we care about
        if (/\b(use|case|when|!phs|!import|!include|\{|```|`|\$\{|payload|then|else)\b/.test(line)) {
            console.log(`LINE ${i + 1}: ${line}`);
            for (const tk of toks.tokens) {
                const txt = line.substring(tk.startIndex, tk.endIndex);
                console.log(`  token:'${txt}' -> scopes=[${tk.scopes.join(', ')}]`);
            }
        }
    }
})();
