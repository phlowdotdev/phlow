// Test that core phlow names are tokenized as constant.language.phlow when used as values
// Run: node test/tokenize_constants.js

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

  const grammar = await registry.loadGrammar('source.phlow');
  if (!grammar) {
    console.error('Could not load phlow grammar');
    process.exit(2);
  }

  const file = path.join(__dirname, '..', '..', 'examples', 'log', 'main.phlow');
  if (!fs.existsSync(file)) {
    console.error('File not found:', file);
    process.exit(3);
  }
  const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/);

  let ruleStack = vscodeTextMate.INITIAL;
  let failures = [];

  const checks = [
    { name: 'payload: main', re: /payload:\s*main\b/ },
    { name: 'payload: envs', re: /payload:\s*envs\b/ },
    { name: 'spread in array ...main', re: /\.\.\.main\b/ },
    { name: 'spread object ...main[0]', re: /\.\.\.main\[0\]/ },
  ];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const toks = grammar.tokenizeLine(line, ruleStack);
    ruleStack = toks.ruleStack;

    for (const check of checks) {
      if (check.re.test(line)) {
        // find token representing 'main' or 'envs' in this line
        const found = toks.tokens.some(t => {
          const txt = line.substring(t.startIndex, t.endIndex);
          return (txt === 'main' || txt === 'envs' || txt === '...main' || txt === '...main[0]' || txt.includes('...main'))
            && t.scopes.some(s => s.includes('constant.language.phlow'));
        });
        if (!found) {
          failures.push({ line: i + 1, text: line.trim(), check: check.name });
        }
      }
    }
  }

  if (failures.length > 0) {
    console.error('FAIL: Some constant value tokenization checks failed');
    for (const f of failures) {
      console.error(`  Line ${f.line}: (${f.check}) -> ${f.text}`);
    }
    process.exit(4);
  } else {
    console.log('PASS: All constant value checks passed (main, envs, spreads recognized as constant.language.phlow)');
    process.exit(0);
  }
})();
