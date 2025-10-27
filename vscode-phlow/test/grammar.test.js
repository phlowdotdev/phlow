// Simple grammar test using vscode-textmate + onigasm
// Install deps first: npm install vscode-textmate onigasm

const fs = require('fs');
const path = require('path');
const onigasm = require('onigasm');
const { OnigScanner, OnigString } = onigasm;
const vscodeTextMate = require('vscode-textmate');

(async () => {
  try {
    // Load the onigasm wasm binary correctly in Node
    const wasmPath = require.resolve('onigasm/lib/onigasm.wasm');
    const wasmBin = fs.readFileSync(wasmPath);
    // Convert Node Buffer to ArrayBuffer slice covering the data
    const wasmArrayBuffer = wasmBin.buffer.slice(wasmBin.byteOffset, wasmBin.byteOffset + wasmBin.byteLength);
    await onigasm.loadWASM(wasmArrayBuffer);
  } catch (e) {
    console.error('Failed to load onigasm wasm. Ensure onigasm is installed.');
    console.error(e);
    process.exit(1);
  }

  // Provide onigasm onigLib implementation required by vscode-textmate
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

  const grammar = await registry.loadGrammar('source.phlow');
  if (!grammar) {
    console.error('Could not load phlow grammar');
    process.exit(2);
  }

  const sample = fs.readFileSync(path.join(__dirname, 'fixtures', 'sample.phlow'), 'utf8');
  const lines = sample.split(/\r?\n/);

  console.log('Tokenizing sample.phlow...');
  let ruleStack = vscodeTextMate.INITIAL;
  let foundPhs = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const lineTokens = grammar.tokenizeLine(line, ruleStack);
    ruleStack = lineTokens.ruleStack;

    for (const t of lineTokens.tokens) {
      const tokenText = line.substring(t.startIndex, t.endIndex);
      if (t.scopes.some(s => s.includes('meta.embedded.block.phs') || s.includes('source.rhai'))) {
        foundPhs = true;
      }
    }
  }

  if (!foundPhs) {
    console.log('\n--- Diagnostics: showing tokens for lines that look relevant ---');
    let ruleStack2 = vscodeTextMate.INITIAL;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.includes('payload') || line.includes('!phs') || line.includes('{') || line.includes('when') || line.includes('?')) {
        const toks = grammar.tokenizeLine(line, ruleStack2);
        ruleStack2 = toks.ruleStack;
        console.log(`LINE ${i + 1}: ${line}`);
        for (const tk of toks.tokens) {
          const txt = line.substring(tk.startIndex, tk.endIndex);
          console.log(`  token:'${txt}' -> scopes=[${tk.scopes.join(', ')}]`);
        }
      }
    }
    console.log('--- end diagnostics ---\n');
  }

  if (foundPhs) {
    console.log('PASS: Embedded !phs blocks/tokenization detected (rhai scopes present)');
    process.exit(0);
  } else {
    console.error('FAIL: No embedded !phs/rhai tokens detected');
    process.exit(3);
  }
})();
