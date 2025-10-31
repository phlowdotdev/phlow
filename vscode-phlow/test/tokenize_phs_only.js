const fs = require('fs');
const path = require('path');
const onigasm = require('onigasm');
const { OnigScanner, OnigString } = onigasm;
const vscodeTextMate = require('vscode-textmate');

(async () => {
    try {
        const wasmPath = require.resolve('onigasm/lib/onigasm.wasm');
        const wasmBin = fs.readFileSync(wasmPath);
        const wasmArrayBuffer = wasmBin.buffer.slice(wasmBin.byteOffset, wasmBin.byteLength + wasmBin.byteOffset);
        await onigasm.loadWASM(wasmArrayBuffer);
    } catch (e) {
        console.error('Failed to load onigasm wasm.');
        console.error(e);
        process.exit(1);
    }

    const onigLib = Promise.resolve({
        createOnigScanner: (patterns) => new OnigScanner(patterns),
        createOnigString: (s) => new OnigString(s),
    });

    const registry = new vscodeTextMate.Registry({
        onigLib,
        loadGrammar: async (scopeName) => {
            const grammarPath = {
                'source.phlow': path.join(__dirname, '..', 'syntaxes', 'phlow.tmLanguage.json'),
                'source.rhai': path.join(__dirname, '..', 'syntaxes', 'rhai.tmLanguage.json')
            }[scopeName];
            if (!grammarPath) return null;
            const content = fs.readFileSync(grammarPath, 'utf8');
            return vscodeTextMate.parseRawGrammar(content, grammarPath);
        }
    });

    // Load grammar
    const grammar = await registry.loadGrammar('source.phlow');

    // Test specific case that might cause ! and phs to split
    const testLines = [
        'key: !phs value',
        'another: !phs test',
        '!phs standalone'
    ];

    console.log('Testing !phs directive tokenization specifically...');

    let ruleStack = vscodeTextMate.INITIAL;
    for (let i = 0; i < testLines.length; i++) {
        const line = testLines[i];
        console.log(`\nLINE ${i + 1}: ${line}`);

        const result = grammar.tokenizeLine(line, ruleStack);
        ruleStack = result.ruleStack;

        for (const token of result.tokens) {
            const tokenText = line.substring(token.startIndex, token.endIndex);
            console.log(`  token:'${tokenText}' -> scopes=[${token.scopes.join(', ')}]`);
        }
    }
})().catch(console.error);