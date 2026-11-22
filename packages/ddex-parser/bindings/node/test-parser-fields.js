// Smoke test for the 0.4.6 parser fields through the Node binding: MessageControlType,
// release identifiers (ISRC, ProprietaryId), dates (default + territorial lists, partial
// dates as written), P/C lines, genres, parental warnings, labels, 3.8.x track artists and
// layout, TrackRelease genre, resource details and deal EndDate.
// Run with: npm test (after `npm run build`).

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexParser } = require('./index.js');

const fixture = (name) =>
  fs.readFileSync(path.join(__dirname, '..', '..', 'tests', 'fixtures', name), 'utf8');

function ern43() {
  const parsed = new DdexParser().parseSync(fixture('ern43_details.xml'));
  assert.equal(parsed.messageControlType, 'TestMessage');
  const r = parsed.releases[0];
  assert.equal(r.icpn, '123456789012');
  assert.equal(r.labelName, 'Details Records');
  assert.equal(r.genre, 'Pop');
  assert.equal(r.subGenre, 'Synth-pop');
  assert.deepEqual(
    r.genres.map((g) => [g.genre, g.territoryCodes, g.isDefault ?? null]),
    [['J-Pop', ['JP'], null], ['Pop', [], true]],
  );
  assert.equal(r.parentalWarning, 'Explicit');
  assert.deepEqual(r.parentalWarnings, ['Explicit']);
  assert.equal(r.pLine.text, '(P) 2024 Details Records');
  assert.equal(r.pLine.year, 2024);
  assert.equal(r.cLine.text, '(C) 2024 Details Records', 'worldwide C line is the default');
  assert.equal(r.releaseDate, '2024-09-20T00:00:00+00:00');
  assert.deepEqual(r.releaseDates.map((d) => [d.date, d.territoryCodes]), [
    ['2024-09-18', ['JP']],
    ['2024-09-20', []],
  ]);
  assert.equal(r.originalReleaseDate ?? null, null, 'partial date: no full date value');
  assert.deepEqual(r.originalReleaseDates.map((d) => d.date), ['2016']);
  const t = r.tracks[0];
  assert.equal(t.parentalWarning, 'ExplicitContentEdited', 'IsDefault warning');
  assert.equal(t.pLine.company, 'Details Records');
  assert.equal(t.trackReleaseGenre, 'Pop');
  assert.equal(t.trackReleaseSubGenre, 'Electropop');
  assert.equal(t.genre ?? null, null, '4.x resources carry no genre');
  // resources: camelCase keys, like the rest of the API (JsResource)
  assert.equal(parsed.resources.A2.cLine.text, '(C) 2024 Details Records');
  assert.equal(parsed.resources.A2.parentalWarning, 'NotExplicit');
  assert.equal(parsed.deals[0].endDate, '2025-09-19T00:00:00+00:00');
}

function ern382() {
  const parsed = new DdexParser().parseSync(fixture('ern382_details.xml'));
  assert.equal(parsed.messageControlType, 'TestMessage');
  const r = parsed.releases[0];
  assert.equal(r.icpn, '123456789012');
  assert.deepEqual(r.proprietaryIds, [{ namespace: 'DPID:PADPIDA0000000001', value: 'REL-0001' }]);
  assert.equal(r.labelName, 'Details Records', 'Worldwide block is the default');
  assert.equal(r.genre, 'Pop');
  assert.equal(r.parentalWarning, 'Explicit');
  assert.equal(r.pLine.year, 2024);
  assert.deepEqual(r.pLine.territoryCodes, ['Worldwide']);
  assert.equal(r.releaseDate, '2024-09-20T00:00:00+00:00');
  assert.deepEqual(r.releaseDates.map((d) => [d.date, d.territoryCodes]), [
    ['2024-09-18', ['JP']],
    ['2024-09-20', ['Worldwide']],
  ]);
  assert.deepEqual(r.originalReleaseDates.map((d) => d.date), ['2016']);
  const [t1, t2] = r.tracks;
  assert.equal(t1.artist, 'Details Artist feat. Guest Artist');
  assert.deepEqual(t1.artists.map((a) => [a.name, a.role]), [
    ['Details Artist', 'MainArtist'],
    ['Guest Artist', 'FeaturedArtist'],
  ]);
  assert.deepEqual([t1.trackNumber, t2.trackNumber], [1, 2]);
  assert.deepEqual(t1.groupPath, [{ sequenceNumber: 1 }]);
  assert.equal(t1.genre, 'Pop');
  assert.equal(t1.labelName, 'Details Records');
  assert.equal(t2.parentalWarning, 'NotExplicit');
  assert.equal(parsed.deals[0].endDate, '2025-09-19T00:00:00+00:00');
}

ern43();
ern382();
console.log('ok - Node binding exposes the 0.4.6 parser fields (ERN 4.3 and 3.8.2)');
