#!/usr/bin/env node
// Install test for the release packages: require ddex-parser and ddex-builder (installed
// from the packed tarballs), parse an official sample, and build a small ERN 4.3 message
// that the parser reads back.
//
// Usage (from the directory where the packages are installed):
//   node scripts/release_smoke.js "examples/Samples43/1 Audio.xml"

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const resolveFrom = (name) => require.resolve(name, { paths: [process.cwd()] });
const { DdexParser } = require(resolveFrom('ddex-parser'));
const { DdexBuilder } = require(resolveFrom('ddex-builder'));
const versionOf = (name) =>
  JSON.parse(fs.readFileSync(path.join(path.dirname(resolveFrom(name)), 'package.json'), 'utf8')).version;

async function main(samplePath) {
  const parser = new DdexParser();
  const parsed = parser.parseSync(fs.readFileSync(samplePath, 'utf8'));
  assert.ok(parsed.releases.length >= 1, 'no release parsed');
  console.log(`parsed ${samplePath}: ${parsed.releases.length} release(s), ${parsed.dealCount} deal(s)`);

  const party = (name, id) => ({ party_name: [{ text: name }], party_id: id });
  const xml = await new DdexBuilder().build({
    header: {
      message_id: 'RELEASE-SMOKE',
      message_sender: party('Smoke Label', 'PADPIDA0000000001'),
      message_recipient: party('Smoke DSP', 'PADPIDA0000000002'),
      message_created_date_time: '2024-09-13T12:00:00Z',
    },
    version: '4.3',
    avs_version_id: '12',
    releases: [{
      release_id: 'A1-ABCDE-0000000001-M',
      title: [{ text: 'Smoke Single' }],
      artist: 'Smoke Artist',
      release_type: 'Single',
      genre: 'Pop',
      parental_warning_type: 'NotExplicit',
      tracks: [{
        track_id: 'T1',
        isrc: 'USRT12400001',
        title: 'Smoke Track',
        duration: 'PT3M42S',
        artist: 'Smoke Artist',
        resource_type: 'MusicalWorkSoundRecording',
        parental_warning_type: 'NotExplicit',
      }],
    }],
    deals: [],
  });
  const rebuilt = parser.parseSync(xml);
  assert.equal(rebuilt.messageId, 'RELEASE-SMOKE');
  assert.equal(rebuilt.releases.length, 1);
  console.log(`built ${xml.length} bytes of ERN 4.3 and parsed it back`);
  console.log(`ok - ddex-parser ${versionOf('ddex-parser')}, ddex-builder ${versionOf('ddex-builder')}`);
}

main(process.argv[2]).catch((err) => {
  console.error(err);
  process.exit(1);
});
