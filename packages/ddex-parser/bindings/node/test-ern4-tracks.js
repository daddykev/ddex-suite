// Smoke test for ERN 4.x track extraction through the Node.js binding.
// Run with: node test-ern4-tracks.js (after `npm run build`)

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexParser } = require('./index.js');

const examples = path.join(__dirname, '..', '..', '..', '..', 'examples');
const read = (dir, name) => fs.readFileSync(path.join(examples, dir, name), 'utf8');
const parser = new DdexParser();
const isAbsent = (value) => value === null || value === undefined;

const checks = {
  'official 4.3 Audio sample returns its 21 tracks'() {
    const result = parser.parseSync(read('Samples43', '1 Audio.xml'));
    assert.equal(result.releases.length, 1);
    const release = result.releases[0];
    assert.equal(release.displayArtist, 'Saeko Shu');
    assert.equal(release.tracks.length, 21);
    assert.equal(result.trackCount, 21);
    const first = release.tracks[0];
    assert.equal(first.trackId, 'A1');
    assert.equal(first.resourceReference, 'A1');
    assert.equal(first.isrc, 'JPTO09404900');
    assert.equal(first.title, 'Yume no Lullaby');
    assert.equal(first.artist, 'Saeko Shu');
    assert.equal(first.duration, '2:28');
    assert.equal(first.position, 1);
    assert.equal(first.trackNumber, 1);
    assert.equal(first.trackReleaseReference, 'R1');
    assert.equal(first.trackReleaseId, '00094631432057_JPTO09404900_R1');
    assert.deepEqual(first.groupPath, []);
  },

  'deals on TrackReleases resolve to tracks'() {
    const result = parser.parseSync(read('Samples43', '1 Audio.xml'));
    const tracks = result.releases[0].tracks;
    const trackDeal = result.deals.find((d) => d.releases.includes('R1'));
    assert.ok(trackDeal, 'a deal references TrackRelease R1');
    const resolved = trackDeal.releases.map(
      (ref) => tracks.find((t) => t.trackReleaseReference === ref)?.trackId,
    );
    assert.equal(resolved.length, 21);
    assert.ok(resolved.every((id) => typeof id === 'string'));
  },

  'nested Component groups give disc numbers and group paths'() {
    const tracks = parser.parseSync(read('Samples43', '3 MixedMedia.xml')).releases[0].tracks;
    assert.equal(tracks.length, 17);
    const video = tracks[14];
    assert.equal(video.trackId, 'A15');
    assert.equal(video.discNumber, 2);
    assert.equal(video.position, 1);
    assert.deepEqual(video.groupPath, [{ groupType: 'Component', sequenceNumber: 2 }]);
  },

  'DjMix without sender PartyName parses with no senderName'() {
    const result = parser.parseSync(read('Samples43', '8 DjMix.xml'));
    assert.ok(isAbsent(result.senderName), `senderName should be absent, got ${result.senderName}`);
    assert.equal(result.releases[0].tracks.length, 1);
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
console.log('All ERN 4.x track smoke checks passed');
