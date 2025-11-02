const fs = require('fs');
const path = require('path');
const onigasm = require('onigasm');
const { Registry, parseRawGrammar } = require('vscode-textmate');

async function loadOnigasmWASM() {
    const wasmPath = require.resolve('onigasm/lib/onigasm.wasm');
    const wasmBin = fs.readFileSync(wasmPath);
    await onigasm.loadWASM(wasmBin.buffer);
}

async function main() {
    const root = path.resolve(__dirname, '..');
    const syntaxesDir = path.resolve(__dirname, '..', 'syntaxes');
    const examplePath = path.resolve(__dirname, '..', 'examples', 'log', 'main.phlow');

    if (!fs.existsSync(examplePath)) {
        console.error('Example file not found:', examplePath);
        process.exit(2);
    }

    // Load wasm
    await loadOnigasmWASM();

    const onigLib = Promise.resolve({
        createOnigScanner: (patterns) => new onigasm.OnigScanner(patterns),
        createOnigString: (str) => new onigasm.OnigString(str)
    });

    const registry = new Registry({
        onigLib,
        loadGrammar: async (scopeName) => {
            const map = {
                'source.phlow': path.join(syntaxesDir, 'phlow.tmLanguage.json'),
                'source.phs': path.join(syntaxesDir, 'phs.tmLanguage.json')
            };
            const file = map[scopeName];
            if (!file || !fs.existsSync(file)) return null;
            const content = fs.readFileSync(file, 'utf8');
            return parseRawGrammar(content, file);
        }
    });

    const grammar = await registry.loadGrammar('source.phlow');
    if (!grammar) {
        console.error('Failed to load grammar source.phlow');
        process.exit(3);
    }

    const text = fs.readFileSync(examplePath, 'utf8');
    let ruleStack = null;
    let containsPhs = false;

    console.log('Tokenizing file:', examplePath);
    console.log('---');

    const lines = text.split(/\r?\n/);
    lines.forEach((line, idx) => {
        const res = grammar.tokenizeLine(line, ruleStack);
        ruleStack = res.ruleStack;
        console.log(`${String(idx + 1).padStart(3)} | ${line}`);
        res.tokens.forEach(t => {
            const tokenText = line.substring(t.startIndex, t.endIndex);
            const scopes = t.scopes.join(' | ');
            // Print the token text followed by its scopes in parentheses
            // Trim only to make output cleaner; preserve empty tokens by showing as quoted string
            const displayText = tokenText.length === 0 ? JSON.stringify(tokenText) : tokenText.trim();
            console.log('    ', displayText, `(${scopes})`);
            if (t.scopes.some(s => s.includes('.phs') || s.includes('phs'))) containsPhs = true;
        });
    });

    console.log('---');
    console.log('Contains tokens with phs scopes?', containsPhs);
    process.exit(0);
}

main().catch(err => {
    console.error('Error:', err);
    process.exit(1);
});
