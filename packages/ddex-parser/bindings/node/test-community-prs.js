// Smoke test for issue #1 and community PRs #2-#4 through the Node.js binding.
// Absent optional values must come through as null/undefined, never as invented values.
// Run with: node test-community-prs.js (after `npm run build`)

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexParser } = require('./index.js');

const fixtures = path.join(__dirname, '..', '..', 'tests', 'fixtures');
const read = (name) => fs.readFileSync(path.join(fixtures, name), 'utf8');
const parser = new DdexParser();
const isAbsent = (value) => value === null || value === undefined;

const checks = {
  'deal without DealReference has no dealId'() {
    const result = parser.parseSync(read('deal_without_reference.xml'));
    assert.equal(result.deals.length, 1);
    assert.ok(isAbsent(result.deals[0].dealId), `dealId should be absent, got ${result.deals[0].dealId}`);
    assert.deepEqual(result.deals[0].releases, ['R1']);
    assert.deepEqual(result.deals[0].territories, ['Worldwide']);
    assert.ok(isAbsent(result.deals[0].commercialModel), 'commercialModel must not be hard-coded');
  },

  'deals are linked to their releases'() {
    const result = parser.parseSync(read('deals_linked_to_releases.xml'));
    assert.deepEqual(
      result.deals.map((d) => [d.dealId ?? null, d.releases]),
      [['D1', ['R1', 'R2']], [null, ['R1', 'R2']], ['D3', ['R2']]],
    );
  },

  'image without ReferenceTitle has no title and is not a track'() {
    const result = parser.parseSync(read('image_without_reference_title.xml'));
    const image = result.resources.A2;
    assert.ok(image, 'image resource A2 should be present');
    assert.ok(isAbsent(image.title), `image title should be absent, got ${image.title}`);
    const tracks = result.releases[0].tracks;
    assert.equal(tracks.length, 1);
    assert.equal(tracks[0].trackId, 'A1');
    assert.equal(tracks[0].position, 1);
    assert.equal(tracks[0].title, 'Example Track');
    assert.equal(tracks[0].duration, '3:20');
  },

  'sound recording without Duration has no duration'() {
    const result = parser.parseSync(read('sound_recording_without_duration.xml'));
    const track = result.releases[0].tracks[0];
    assert.equal(track.title, 'Track Without Duration');
    assert.ok(isAbsent(track.duration), `duration should be absent, got ${track.duration}`);
  },

  'ERN 3.8.1 is reported as V3_8_1'() {
    const xml = read('ern_381.xml');
    assert.equal(parser.detectVersion(xml), 'V3_8_1');
    assert.equal(parser.parseSync(xml).version, 'V3_8_1');
  },
};

let failed = 0;
for (const [name, check] of Object.entries(checks)) {
  try {
    check();
    console.log(`✓ ${name}`);
  } catch (err) {
    failed += 1;
    console.error(`✗ ${name}\n  ${err.message}`);
  }
}
if (failed > 0) {
  console.error(`${failed} check(s) failed`);
  process.exit(1);
}
console.log('All community PR smoke checks passed');
