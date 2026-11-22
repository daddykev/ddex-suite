// Smoke test: the Node binding builds an ERN 3.8.2 message (with deals) with the core
// builder, and ddex-parser (its Node binding) reads it back.
// Run with: npm run test:smoke (after `npm run build` here and in ../../../ddex-parser/bindings/node)

const assert = require('node:assert/strict');
const path = require('node:path');
const { DdexBuilder } = require('./index.js');
const { DdexParser } = require(path.join(__dirname, '..', '..', '..', 'ddex-parser', 'bindings', 'node', 'index.js'));

const request = {
  header: {
    message_id: 'NODE-SMOKE-382',
    message_sender: { party_name: [{ text: 'Smoke Label' }], party_id: 'PADPIDA0000000001' },
    message_recipient: { party_name: [{ text: 'Smoke DSP' }], party_id: 'PADPIDA0000000002' },
    message_created_date_time: '2024-09-13T12:00:00Z',
  },
  version: '3.8.2',
  releases: [{
    release_id: 'A1-ABCDE-0000000001-M',
    release_reference: 'R0',
    title: [{ text: 'Smoke & "Mirrors" <Live>' }],
    artist: 'Smoke Artist',
    label: 'Smoke Records',
    upc: '123456789012',
    release_type: 'Album',
    tracks: [1, 2].map((n) => ({
      track_id: `T${n}`,
      resource_reference: `A${n}`,
      isrc: `USRT1240000${n}`,
      title: `Track ${n}`,
      duration: 'PT3M42S',
      artist: 'Smoke Artist',
    })),
  }],
  deals: [{
    deal_terms: { commercial_model_type: 'SubscriptionModel', territory_code: ['Worldwide'], start_date: '2024-01-01', use_types: ['OnDemandStream'] },
    release_references: ['R0'],
  }],
};

async function main() {
  const xml = await new DdexBuilder().build(request);
  assert.match(xml, /xmlns:ern="http:\/\/ddex\.net\/xml\/ern\/382"/);
  assert.match(xml, /<SoundRecordingId>/);
  assert.match(xml, /<ReleaseDeal>/);
  const parsed = new DdexParser().parseSync(xml);
  assert.equal(parsed.messageId, 'NODE-SMOKE-382');
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

  // Values the 3.8.2 schema requires are the core builder's errors, naming the field
  const incomplete = structuredClone(request);
  delete incomplete.deals[0].deal_terms.use_types;
  await assert.rejects(
    new DdexBuilder().build(incomplete),
    /ERN 3\.8\.2 requires DealTerms\/Usage\/UseType \(deals\[0\]\.deal_terms\.use_types\)/,
  );

  console.log('ok - Node binding builds ERN 3.8.2 that ddex-parser reads back');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
