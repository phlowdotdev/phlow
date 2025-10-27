const fs = require('fs');
const path = require('path');
const onigasm = require('onigasm');
const { OnigScanner, OnigString } = require('onigasm');
const vscodeTextMate = require('vscode-textmate');

(async () => {
  try {
    const wasmPath = require.resolve('onigasm/lib/onigasm.wasm');
    const wasmBin = fs.readFileSync(wasmPath);
    const wasmArrayBuffer = wasmBin.buffer.slice(wasmBin.byteOffset, wasmBin.byteOffset + wasmBin.byteLength);
    await onigasm.loadWASM(wasmArrayBuffer);
  } catch (e) {
    console.error('Failed to load onigasm wasm. Ensure onigasm is installed.');
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

  const fixturePath = path.resolve(__dirname, 'fixtures', 'sample.phlow');
  const raw = fs.readFileSync(fixturePath, 'utf8');
  const lines = raw.split(/\r?\n/);

  console.log('Tokenizing sample.phlow for key/value checks...');
  let ruleStack = vscodeTextMate.INITIAL;
  let foundAny = false;
  const failures = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const r = grammar.tokenizeLine(line, ruleStack);
    ruleStack = r.ruleStack;
    const tokens = r.tokens;

    if (line.includes('payload:')) {
      foundAny = true;
      let matched = false;
      for (let t = 0; t < tokens.length; t++) {
        const token = tokens[t];
        const start = token.startIndex;
        const end = (t + 1 < tokens.length) ? tokens[t + 1].startIndex : line.length;
        const text = line.slice(start, end);
        if (text.includes('payload')) {
          const scopes = token.scopes || [];
          const hasKeyScope = scopes.some(s => s.includes('entity.name.tag') || s.includes('punctuation.separator.key-value') || s.includes('entity.name'));
          const hasRhaiScope = scopes.some(s => s.includes('source.rhai'));
          if (!hasKeyScope) {
            failures.push({line: i + 1, reason: 'payload key not scoped as entity.name.tag', scopes, text});
          }
          if (hasRhaiScope) {
            failures.push({line: i + 1, reason: 'payload key incorrectly scoped as source.rhai', scopes, text});
          }
          matched = true;
          break;
        }
      }
      if (!matched) {
        failures.push({line: i + 1, reason: 'payload text not found within any token on the line', text: line});
      }
    }
  }

  if (!foundAny) {
    console.error('No lines with payload: found in fixture.');
    process.exit(2);
  }

  if (failures.length) {
    console.error('KEY-VALUE TEST FAILURES:');
    failures.forEach(f => {
      console.error(`Line ${f.line}: ${f.reason}`);
      if (f.scopes) console.error('  scopes=', JSON.stringify(f.scopes));
      if (f.text) console.error('  text=', JSON.stringify(f.text));
    });
    process.exit(1);
  }

  console.log('PASS: payload key tokenization looks correct (key scoped, not rhai)');
  process.exit(0);

})().catch(err => {
  console.error(err);
  process.exit(99);
});
