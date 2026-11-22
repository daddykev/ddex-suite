// Smoke test: the Node binding builds an ERN 4.3 message with the core builder, and
// ddex-parser (its Node binding) reads it back.
// Run with: npm test (after `npm run build` here and in ../../../ddex-parser/bindings/node)

const assert = require('node:assert/strict');
const path = require('node:path');
const { DdexBuilder } = require('./index.js');
const { DdexParser } = require(path.join(__dirname, '..', '..', '..', 'ddex-parser', 'bindings', 'node', 'index.js'));

const header = {
  message_id: 'NODE-SMOKE-1',
  message_sender: { party_name: [{ text: 'Smoke Label' }], party_id: 'PADPIDA0000000001' },
  message_recipient: { party_name: [{ text: 'Smoke DSP' }], party_id: 'PADPIDA0000000002' },
  message_created_date_time: '2024-09-13T12:00:00Z',
};

const request = {
  header,
  version: '4.3',
  language_and_script_code: 'en',
  avs_version_id: '12',
  releases: [{
    release_id: 'A1-ABCDE-0000000001-M',
    release_reference: 'R0',
    title: [{ text: 'Smoke & "Mirrors" <Live>' }],
    artist: 'Smoke Artist',
    label: 'Smoke Records',
    upc: '123456789012',
    release_type: 'Album',
    genre: 'Pop',
    parental_warning_type: 'NotExplicit',
    tracks: [1, 2].map((n) => ({
      track_id: `T${n}`,
      resource_reference: `A${n}`,
      isrc: `USRT1240000${n}`,
      title: `Track ${n}`,
      duration: 'PT3M42S',
      artist: 'Smoke Artist',
      resource_type: 'MusicalWorkSoundRecording',
      parental_warning_type: 'NotExplicit',
    })),
  }],
  deals: [{
    deal_terms: { commercial_model_type: 'SubscriptionModel', territory_code: ['Worldwide'], start_date: '2024-01-01', use_types: ['OnDemandStream'] },
    release_references: ['R0'],
  }],
};

async function main() {
  const parser = new DdexParser();

  // 1. A full build request
  const xml = await new DdexBuilder().build(request);
  assert.match(xml, /xmlns:ern="http:\/\/ddex\.net\/xml\/ern\/43"/);
  assert.match(xml, /<SoundRecordingEdition>/);
  const parsed = parser.parseSync(xml);
  assert.equal(parsed.messageId, 'NODE-SMOKE-1');
  assert.equal(parsed.releases.length, 1);
  const release = parsed.releases[0];
  assert.equal(release.defaultTitle, 'Smoke & "Mirrors" <Live>');
  assert.equal(release.displayArtist, 'Smoke Artist');
  assert.equal(release.releaseType, 'Album');
  assert.deepEqual(release.tracks.map((t) => [t.isrc, t.title]), [
    ['USRT12400001', 'Track 1'],
    ['USRT12400002', 'Track 2'],
  ]);
  assert.equal(parsed.dealCount, 1);

  // 2. Stored releases/resources plus the request's header and version
  const builder = new DdexBuilder();
  builder.addRelease({
    releaseId: 'A1-ABCDE-0000000002-M', releaseType: 'Single', title: 'Stored Single',
    artist: 'Smoke Artist', label: 'Smoke Records', genre: 'Pop',
    parentalWarningType: 'NotExplicit', trackIds: ['A9'],
  });
  builder.addResource({
    resourceId: 'A9', resourceType: 'SoundRecording', ddexType: 'MusicalWorkSoundRecording',
    parentalWarningType: 'NotExplicit', title: 'Stored Track', artist: 'Smoke Artist',
    isrc: 'USRT12400009', duration: 'PT2M30S',
  });
  const storedXml = await builder.build({ header, version: '4.3', language_and_script_code: 'en', avs_version_id: '12' });
  const stored = parser.parseSync(storedXml);
  assert.equal(stored.releases[0].tracks[0].title, 'Stored Track');

  // 3. Nothing is filled in: missing data is the core builder's error
  await assert.rejects(new DdexBuilder().build({ header }), /missing field `version`/);
  const incomplete = structuredClone(request);
  delete incomplete.releases[0].genre;
  await assert.rejects(
    new DdexBuilder().build(incomplete),
    /ERN 4\.3 requires Release\/Genre\/GenreText \(releases\[0\]\.genre\)/,
  );

  console.log('ok - Node binding builds ERN 4.3 that ddex-parser reads back');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
