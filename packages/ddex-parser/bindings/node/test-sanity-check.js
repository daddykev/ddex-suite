// sanityCheck on official samples and on bad input.
//
// Official samples: DDEX's ERN 4.2 and 4.3 samples (examples/Samples42, Samples43) and
// YouTube's published ERN 3.8.2 samples (samples/youtube-ern382, fetched with
// scripts/fetch_ern382_samples.sh when missing, as the Rust tests do; DDEX_SKIP_SAMPLES=1
// skips them). The expected version comes from the root element's namespace declaration,
// read here with a regular expression, independently of the parser.
// Run with: npm test (after `npm run build`).

const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const { DdexParser } = require('./index.js');

const REPO = path.join(__dirname, '..', '..', '..', '..');
const NAMESPACE_VERSIONS = {
  'http://ddex.net/xml/ern/382': 'V3_8_2',
  'http://ddex.net/xml/ern/42': 'V4_2',
  'http://ddex.net/xml/ern/43': 'V4_3',
};

const xmlFiles = (dir) =>
  fs.existsSync(dir)
    ? fs.readdirSync(dir).filter((f) => f.endsWith('.xml')).sort().map((f) => path.join(dir, f))
    : [];

function youtubeErn382Samples() {
  const dir = path.join(REPO, 'samples', 'youtube-ern382');
  if (xmlFiles(dir).length === 0) {
    if (process.env.DDEX_SKIP_SAMPLES === '1') return [];
    execFileSync('sh', [path.join(REPO, 'scripts', 'fetch_ern382_samples.sh')], { stdio: 'inherit' });
  }
  return xmlFiles(dir);
}

function expectedVersion(xml) {
  // The first start tag that is not a declaration, comment or processing instruction.
  const root = xml.match(/<(?![?!])([^\s>/]+)([^>]*)>/);
  assert.ok(root, 'root element');
  const prefix = root[1].includes(':') ? root[1].split(':')[0] : null;
  const decl = prefix ? `xmlns:${prefix}` : 'xmlns';
  const ns = root[2].match(new RegExp(`${decl}\\s*=\\s*["']([^"']+)["']`));
  assert.ok(ns, `namespace declaration ${decl}`);
  return NAMESPACE_VERSIONS[ns[1]];
}

async function main() {
  const parser = new DdexParser();
  const ern4 = [
    ...xmlFiles(path.join(REPO, 'examples', 'Samples42')),
    ...xmlFiles(path.join(REPO, 'examples', 'Samples43')),
  ];
  assert.ok(ern4.length > 0, 'official ERN 4.x samples under examples/');
  const samples = [...ern4, ...youtubeErn382Samples()];

  const seen = new Set();
  for (const file of samples) {
    const xml = fs.readFileSync(file, 'utf8');
    const want = expectedVersion(xml);
    seen.add(want);
    const result = await parser.sanityCheck(xml);
    assert.deepEqual(
      { isValid: result.isValid, version: result.version, errors: result.errors, warnings: result.warnings },
      { isValid: true, version: want, errors: [], warnings: [] },
      path.relative(REPO, file),
    );
  }
  if (process.env.DDEX_SKIP_SAMPLES !== '1') {
    assert.deepEqual([...seen].sort(), ['V3_8_2', 'V4_2', 'V4_3']);
  }

  // Malformed: a mismatched end tag. The version is still detected from the root.
  let result = await parser.sanityCheck(
    '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">' +
      '<MessageHeader></Mismatched></ern:NewReleaseMessage>',
  );
  assert.equal(result.isValid, false);
  assert.equal(result.version, 'V4_3');
  assert.equal(result.errors.length, 1);
  assert.match(result.errors[0], /Not well-formed XML/);

  // Malformed: an official sample cut in half.
  const audio = fs.readFileSync(path.join(REPO, 'examples', 'Samples43', '1 Audio.xml'), 'utf8');
  result = await parser.sanityCheck(audio.slice(0, Math.floor(audio.length / 2)));
  assert.equal(result.isValid, false);
  assert.equal(result.version, 'V4_3');
  assert.ok(result.errors.some((e) => e.includes('not closed')), result.errors.join(' | '));

  // Well-formed XML that is not a DDEX message: no version ("Unknown"), invalid.
  result = await parser.sanityCheck('<?xml version="1.0"?><html><body>Hi</body></html>');
  assert.equal(result.isValid, false);
  assert.equal(result.version, 'Unknown');
  assert.equal(result.errors.length, 1);
  assert.match(result.errors[0], /No DDEX ERN namespace found/);

  for (const text of ['', 'not xml at all']) {
    result = await parser.sanityCheck(text);
    assert.equal(result.isValid, false);
    assert.equal(result.version, 'Unknown');
    assert.ok(result.errors.length > 0);
  }

  console.log(`ok - sanityCheck: ${samples.length} official samples valid, bad input reported`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
