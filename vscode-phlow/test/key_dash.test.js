const fs = require('fs');
const path = require('path');

// Simple test to ensure keys that appear after a list marker '- key:' are detected
// by the same detection logic (we just reproduce the key-detection regex here).

const filePath = path.resolve(__dirname, '..', '..', 'examples', 'openapi', 'main.phlow');
const txt = fs.readFileSync(filePath, 'utf8');
const lines = txt.split(/\r?\n/);

let foundDashId = false;
let foundDashPayload = false;
let foundAction = false;

for (const line of lines) {
  const dashKey = line.match(/^(\s*)-\s+([A-Za-z0-9_\-\.]+)\s*:/);
  if (dashKey) {
    const name = dashKey[2];
    if (name === 'id') foundDashId = true;
    if (name === 'payload') foundDashPayload = true;
  }
  const propKey = line.match(/^\s*([A-Za-z0-9_\-\.]+)\s*:/);
  if (propKey && propKey[1] === 'action') foundAction = true;
}

if (!foundDashId) {
  console.error('FAIL: did not find dash key `id` in examples/openapi/main.phlow');
  process.exit(1);
}
if (!foundDashPayload) {
  console.error('FAIL: did not find dash key `payload` in examples/openapi/main.phlow');
  process.exit(2);
}
if (!foundAction) {
  console.error('FAIL: did not find property key `action` (nested) in examples/openapi/main.phlow');
  process.exit(3);
}

console.log('PASS: dash keys (e.g. - id:, - payload:) and nested keys (action) detected');
process.exit(0);
