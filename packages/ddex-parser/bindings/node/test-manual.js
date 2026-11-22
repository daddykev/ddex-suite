// Smoke test: the native binding loads, parses a valid message and rejects invalid input.
// Run with: node test-manual.js (after `npm run build` or `npm run build:debug`)

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexParser } = require('./index');

const parser = new DdexParser({ debug: false });
console.log('✓ Parser loaded');

const valid = fs.readFileSync(
  path.join(__dirname, '..', '..', 'tests', 'fixtures', 'ern43_basic.xml'),
  'utf8',
);
const result = parser.parseSync(valid);
assert.equal(result.messageId, 'MSG_TV7HAUF8KQ');
console.log('✓ Parses a valid ERN 4.3 message');

assert.throws(
  () => parser.parseSync('<NewReleaseMessage></NewReleaseMessage>'),
  /namespace|missing|required/i,
  'a message without the ERN namespace must be rejected',
);
console.log('✓ Rejects incomplete XML');

console.log('All Node.js binding smoke checks passed');
