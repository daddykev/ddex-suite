// The WASM builder is the core builder: it builds ERN 3.8.2 / 4.2 / 4.3 messages that are valid
// against the official DDEX XSDs, rejects incomplete requests with the core's errors, and
// writes the same bytes for the same request.
//
// Run with: npm test (builds pkg-node with `wasm-pack build --target nodejs` first).
//
// XSD validation needs `xmllint` (libxml2) and the schemas fetched by
// scripts/fetch_ddex_schemas.sh (run here when they are missing). If either is
// unavailable the test FAILS; set DDEX_SKIP_XSD=1 to skip XSD validation explicitly.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');

const { WasmDdexBuilder, version } = require('./pkg-node/ddex_builder_wasm.js');

const repoRoot = path.resolve(__dirname, '..', '..', '..', '..');

const header = {
  message_id: 'WASM-1',
  message_sender: { party_name: [{ text: 'Example Label' }], party_id: 'PADPIDA0000000001' },
  message_recipient: { party_name: [{ text: 'Example DSP' }], party_id: 'PADPIDA0000000002' },
  message_control_type: 'LiveMessage',
  message_created_date_time: '2024-09-13T12:00:00Z',
};

function track(n, extra = {}) {
  return {
    track_id: `T${n}`,
    resource_reference: `A${n}`,
    isrc: `USRT1240000${n}`,
    title: `Track ${n}`,
    duration: 'PT3M42S',
    artist: 'Example Artist',
    resource_type: 'MusicalWorkSoundRecording',
    parental_warning_type: 'NotExplicit',
    ...extra,
  };
}

function request(versionId, tracks) {
  return {
    header,
    version: versionId,
    language_and_script_code: 'en',
    avs_version_id: versionId === '4.3' ? '12' : undefined,
    releases: [{
      release_id: 'A1-ABCDE-0000000001-M',
      release_reference: 'R0',
      title: [{ text: 'Example & "Album" <Live>' }],
      artist: 'Example Artist',
      label: 'Example Records',
      upc: '123456789012',
      release_type: tracks.length > 1 ? 'Album' : 'Single',
      genre: 'Pop',
      parental_warning_type: 'NotExplicit',
      tracks,
    }],
    deals: [{
      deal_terms: {
        commercial_model_type: 'SubscriptionModel',
        territory_code: ['Worldwide'],
        start_date: '2024-01-01',
        use_types: ['OnDemandStream'],
      },
      release_references: ['R0'],
    }],
  };
}

const cases = [];
for (const v of ['4.2', '4.3']) {
  cases.push([`${v} single`, request(v, [track(1)])]);
  cases.push([`${v} album`, request(v, [1, 2, 3].map((n) => track(n, { sequence_number: n })))]);
  cases.push([`${v} multi-disc album`, request(v, [
    track(1, { disc_number: 1, sequence_number: 1 }),
    track(2, { disc_number: 1, sequence_number: 2 }),
    track(3, { disc_number: 2, sequence_number: 1 }),
  ])]);
  cases.push([`${v} video`, request(v, [track(1, {
    resource_kind: 'Video', resource_type: 'ShortFormMusicalWorkVideo', duration: 'PT4M01S',
  })])]);
}

// ERN 3.8.2: no ReleaseType/Genre/ParentalWarningType requirements; deals inside the release.
{
  const r = request('3.8.2', [1, 2].map((n) => track(n, { sequence_number: n })));
  delete r.language_and_script_code;
  cases.push(['3.8.2 album', r]);
}

function xsdPath(versionId) {
  const dir = versionId.replace(/\./g, '');
  return path.join(repoRoot, 'schemas', 'ern', dir, 'release-notification.xsd');
}

function xsdAvailable() {
  if (process.env.DDEX_SKIP_XSD === '1') return false;
  try {
    execFileSync('xmllint', ['--version'], { stdio: 'ignore' });
  } catch {
    throw new Error('xmllint (libxml2) is required for the XSD check; install it or set DDEX_SKIP_XSD=1');
  }
  if (!['3.8.2', '4.2', '4.3'].every((v) => fs.existsSync(xsdPath(v)))) {
    execFileSync('sh', [path.join(repoRoot, 'scripts', 'fetch_ddex_schemas.sh')], { stdio: 'inherit' });
  }
  return true;
}

function assertSchemaValid(name, versionId, xml) {
  const file = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'ddex-wasm-')), 'out.xml');
  fs.writeFileSync(file, xml);
  try {
    execFileSync('xmllint', ['--noout', '--schema', xsdPath(versionId), file], { stdio: 'pipe' });
  } catch (err) {
    throw new Error(`${name}: not valid against the ERN ${versionId} XSD:\n${err.stderr}`);
  }
}

const checkXsd = xsdAvailable();
const builder = new WasmDdexBuilder();

assert.match(version(), /^\d+\.\d+\.\d+/);

for (const [name, req] of cases) {
  const xml = builder.build(req);
  const ns = req.version.replace(/\./g, '');
  assert.match(xml, new RegExp(`xmlns:ern="http://ddex.net/xml/ern/${ns}"`), name);
  assert.match(xml, /Example &amp; "Album" &lt;Live&gt;|Example &amp; &quot;Album&quot; &lt;Live&gt;/, name);
  // Deterministic: the same request gives the same bytes.
  assert.equal(new WasmDdexBuilder().build(req), xml, `${name}: output is not deterministic`);
  if (checkXsd) assertSchemaValid(name, req.version, xml);
}

// The core's validation and errors.
assert.throws(() => builder.build({ header }), /missing field `version`/);
assert.throws(() => builder.build({ ...request('4.3', [track(1)]), version: '9.9' }), /9\.9/);
const incomplete = request('4.3', [track(1)]);
delete incomplete.releases[0].genre;
assert.throws(
  () => builder.build(incomplete),
  /ERN 4\.3 requires Release\/Genre\/GenreText \(releases\[0\]\.genre\)/,
);
assert.throws(() => builder.build('not a request'), /must be an object|Invalid build request/);

console.log(
  `ok - WASM builder: ${cases.length} ERN 3.8.2/4.x builds through the core builder` +
    (checkXsd ? ', XSD-valid' : ' (XSD check skipped: DDEX_SKIP_XSD=1)'),
);
