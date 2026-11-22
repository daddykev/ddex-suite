// Every key of the parse output matches the generated TypeScript types (index.d.ts):
// walks the whole ParsedMessage of the official ERN 4.3 (and 4.2) samples recursively,
// including resources, tracks and deals, and checks that each key is declared on its
// interface with a value of the declared type, and that every required key is present.
// Also checks that the streaming options removed in 0.5.0 are rejected.
//
// Run with: npm test (after `npm run build`).

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexParser } = require('./index.js');

const examples = path.join(__dirname, '..', '..', '..', '..', 'examples');

// --- index.d.ts: interface name -> { field -> { type, optional } }
function parseInterfaces(dts) {
  const interfaces = {};
  const re = /export interface (\w+) \{([\s\S]*?)\n\}/g;
  let m;
  while ((m = re.exec(dts))) {
    const body = m[2].replace(/\/\*[\s\S]*?\*\//g, '');
    const fields = {};
    for (const line of body.split('\n')) {
      const f = line.trim().match(/^(\w+)(\?)?: (.+)$/);
      if (f) fields[f[1]] = { type: f[3].trim(), optional: Boolean(f[2]) };
    }
    interfaces[m[1]] = fields;
  }
  return interfaces;
}

const interfaces = parseInterfaces(fs.readFileSync(path.join(__dirname, 'index.d.ts'), 'utf8'));
const visited = {};

function check(value, type, where) {
  let inner;
  if ((inner = type.match(/^Array<(.+)>$/))) {
    assert.ok(Array.isArray(value), `${where}: expected an array (${type}), got ${typeof value}`);
    value.forEach((v, i) => check(v, inner[1], `${where}[${i}]`));
  } else if ((inner = type.match(/^Record<string, (.+)>$/))) {
    assert.ok(value && typeof value === 'object' && !Array.isArray(value), `${where}: expected an object (${type})`);
    for (const [k, v] of Object.entries(value)) check(v, inner[1], `${where}[${JSON.stringify(k)}]`);
  } else if (type === 'string' || type === 'number' || type === 'boolean') {
    assert.equal(typeof value, type, `${where}: expected ${type}, got ${JSON.stringify(value)}`);
  } else if (interfaces[type]) {
    checkInterface(value, type, where);
  } else {
    assert.fail(`${where}: the test does not know the type ${type}`);
  }
}

function checkInterface(value, name, where) {
  const fields = interfaces[name];
  assert.ok(value && typeof value === 'object', `${where}: expected ${name}, got ${JSON.stringify(value)}`);
  visited[name] = (visited[name] || 0) + 1;
  for (const [key, v] of Object.entries(value)) {
    const field = fields[key];
    assert.ok(field, `${where}.${key}: key not declared on ${name} in index.d.ts`);
    if (v === undefined || v === null) {
      assert.ok(field.optional, `${where}.${key}: required ${name}.${key} is ${v}`);
      continue;
    }
    check(v, field.type, `${where}.${key}`);
  }
  for (const [key, field] of Object.entries(fields)) {
    if (!field.optional) assert.ok(key in value, `${where}: required key ${name}.${key} is missing`);
  }
}

// --- The samples
const files = ['Samples43', 'Samples42'].flatMap((dir) =>
  fs.readdirSync(path.join(examples, dir))
    .filter((f) => f.endsWith('.xml'))
    .map((f) => path.join(examples, dir, f)),
);
assert.ok(files.length >= 10, 'expected the official 4.2 / 4.3 samples in examples/');

const parser = new DdexParser();
for (const file of files) {
  const xml = fs.readFileSync(file, 'utf8');
  const where = path.relative(examples, file);
  checkInterface(parser.parseSync(xml), 'ParsedMessage', where);
  // statistics / fidelityInfo are only present with options.
  checkInterface(parser.parseSync(xml, { collectStatistics: true }), 'ParsedMessage', where);
}

for (const name of ['ParsedMessage', 'JsRelease', 'JsTrack', 'JsResource', 'JsDeal', 'JsCopyright',
  'JsArtist', 'JsTrackGroup', 'JsGenre', 'ParseStatistics', 'FidelityInfo']) {
  assert.ok(visited[name] > 0, `no ${name} in the samples' output: the walk did not cover it`);
}

// --- Removed streaming options are rejected, not ignored
const sample = fs.readFileSync(path.join(examples, 'Samples43', '1 Audio.xml'), 'utf8');
for (const option of [{ mode: 'stream' }, { autoThreshold: 1 }, { enableStreaming: true }, { streamingThreshold: 1 }]) {
  const name = Object.keys(option)[0];
  assert.throws(() => parser.parseSync(sample, option), new RegExp(`${name}.*removed in 0\\.5\\.0`));
}
assert.equal(typeof parser.stream, 'undefined', 'stream() was removed in 0.5.0');

console.log(`ok - parse output keys match index.d.ts (${files.length} samples; ` +
  `${visited.JsResource} resources, ${visited.JsTrack} tracks, ${visited.JsDeal} deals)`);
