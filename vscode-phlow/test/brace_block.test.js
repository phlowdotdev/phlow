const fs = require('fs');
const path = require('path');

// This test reproduces the scanner logic used in extension.js to detect
// brace-delimited blocks that start inside a list item (e.g. `- payload: { ... }`)
// and ensures braces inside strings are ignored.

function findBraceBlockRange(lines, startLineIndex, itemIndent, firstBraceIdxInLine) {
  let braceCount = 0;
  let closed = false;
  let endLine = startLineIndex;
  let endCol = null;
  let inString = null;
  let escaped = false;

  for (let k = startLineIndex; k < lines.length; k++) {
    const text = lines[k];
    let startIdx = 0;
    if (k === startLineIndex) startIdx = firstBraceIdxInLine;
    for (let p = startIdx; p < text.length; p++) {
      const ch = text[p];
      if (escaped) { escaped = false; continue; }
      if (ch === '\\') { escaped = true; continue; }
      if (inString) {
        if (ch === inString) inString = null;
        continue;
      }
      if (ch === '"' || ch === '\'' || ch === '`') { inString = ch; continue; }
      if (ch === '{') braceCount++; else if (ch === '}') braceCount--;
      if (braceCount === 0) {
        endLine = k; endCol = p + 1; closed = true; break;
      }
    }
    if (closed) break;
  }
  return { closed, endLine, endCol };
}

function findStepsBlockRanges(text) {
  const lines = text.split(/\r?\n/);
  const ranges = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^\s*steps\s*:/.test(line)) {
      // scan following lines for items
      for (let j = i + 1; j < lines.length; j++) {
        const l = lines[j];
        if (/^\s*-\s+/.test(l)) {
          const m = l.match(/^(\s*)-\s+(.*)$/);
          const indent = m[1].length;
          const after = l.slice(indent + 2);
          const braceIdx = l.indexOf('{', indent + 2);
          if (braceIdx !== -1) {
            const result = findBraceBlockRange(lines, j, indent, braceIdx);
            ranges.push({ startLine: j, endLine: result.endLine, closed: result.closed });
            // advance j past block
            if (result.closed) j = result.endLine;
          }
        } else if (l.trim() === '') {
          continue;
        } else if (!/^\s+/.test(l)) {
          break; // end of steps block
        }
      }
    }
  }
  return ranges;
}

(function main(){
  const fixturePath = path.join(__dirname, 'fixtures', 'brace_test.phlow');
  const txt = fs.readFileSync(fixturePath, 'utf8');
  const ranges = findStepsBlockRanges(txt);
  if (!ranges.length) {
    console.error('No brace block ranges detected'); process.exit(1);
  }
  // Expect the first range to be closed and to span several lines (> 4)
  const r = ranges[0];
  if (!r.closed) {
    console.error('Brace block was not closed by scanner'); process.exit(2);
  }
  if (r.endLine - r.startLine < 4) {
    console.error('Brace block was too short, expected multi-line block'); process.exit(3);
  }
  console.log('PASS: brace block scanner correctly detected closing brace and ignored braces in strings');
  process.exit(0);
})();
