// Results the Node builder used to make up are now computed, or null where nothing is
// measured: testRoundTripFidelity (it returned a constant 0.98 "success"), buildWithFidelity's
// statistics and verification, the preset lists, validate, and StreamingDdexBuilder.getXml
// (it returned ""). Round trips use the official sample examples/Samples43/1 Audio.xml.
// Run with: npm test (after `npm run build` here and in ../../../ddex-parser/bindings/node).

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexBuilder, StreamingDdexBuilder } = require('./index.js');

const SAMPLE = path.join(__dirname, '..', '..', '..', '..', 'examples', 'Samples43', '1 Audio.xml');

const header = {
  message_id: 'NODE-HONEST-1',
  message_sender: { party_name: [{ text: 'Label' }], party_id: 'PADPIDA0000000001' },
  message_recipient: { party_name: [{ text: 'DSP' }], party_id: 'PADPIDA0000000002' },
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
    title: [{ text: 'Album' }],
    artist: 'Artist',
    release_type: 'Album',
    tracks: [1, 2].map((n) => ({
      track_id: `T${n}`,
      resource_reference: `A${n}`,
      isrc: `USRT1240000${n}`,
      title: `Track ${n}`,
      duration: 'PT3M42S',
      artist: 'Artist',
      resource_type: 'MusicalWorkSoundRecording',
    })),
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

async function main() {
  const sample = fs.readFileSync(SAMPLE, 'utf8');

  const pass = await new DdexBuilder().testRoundTripFidelity(sample);
  assert.equal(pass.roundTripSuccess, true, pass.issues.join('; '));
  assert.deepEqual(pass.issues, []);
  assert.equal(pass.fidelityScore ?? null, null);
  assert.equal(typeof pass.determinismVerified, 'boolean');
  console.log('✓ testRoundTripFidelity: the official sample round-trips, no score');

  const changed = sample.replace('<ISRC>JPTO09404900</ISRC>', '<ISRC></ISRC>');
  assert.notEqual(changed, sample);
  const fail = await new DdexBuilder().testRoundTripFidelity(changed);
  assert.equal(fail.roundTripSuccess, false);
  assert.match(fail.issues[0], /^round trip failed:/);
  const garbage = await new DdexBuilder().testRoundTripFidelity('<html/>');
  assert.equal(garbage.roundTripSuccess, false);
  console.log('✓ testRoundTripFidelity: a changed field and non-DDEX input fail');

  const built = await new DdexBuilder().buildWithFidelity(request, {
    collectStatistics: true,
    enableVerification: true,
  });
  const xml = built.xml;
  const elements = xml.split('<').length - 1 - (xml.split('</').length - 1) - (xml.split('<?').length - 1);
  assert.equal(built.statistics.elementCount, elements);
  assert.ok(built.statistics.attributeCount > 0 && built.statistics.namespaceCount > 0);
  for (const field of ['memoryUsedBytes', 'extensionCount', 'canonicalizationTimeMs']) {
    assert.equal(built.statistics[field] ?? null, null, field);
  }
  assert.equal(built.verification.roundTripSuccess, true, built.verification.issues.join('; '));
  assert.equal(built.verification.fidelityScore ?? null, null);
  await assert.rejects(
    new DdexBuilder().buildWithFidelity(request, { canonicalization: 'c14n' }),
    /not implemented in 0.5.x/,
  );
  console.log('✓ buildWithFidelity: counted statistics, a real round trip');

  const builder = new DdexBuilder();
  const presets = builder.getAvailablePresets();
  assert.ok(presets.includes('audio_album'), presets.join(', '));
  assert.ok(!presets.some((p) => p.startsWith('spotify') || p.startsWith('apple')), presets.join(', '));
  assert.equal(builder.getPresetInfo('audio_album').name, 'audio_album');
  builder.applyPreset('audio_album');
  assert.throws(() => builder.applyPreset('spotify_album'), /Unknown preset/);
  assert.ok(Array.isArray(builder.getPresetValidationRules('audio_album')));
  console.log("✓ presets are the builder's");

  const empty = new DdexBuilder();
  const validation = await empty.validate();
  assert.equal(validation.isValid, false);
  assert.ok(validation.errors.length > 0);
  assert.equal(empty.getStats().validationErrors, validation.errors.length);
  console.log('✓ validate runs preflight validation');

  const streaming = new StreamingDdexBuilder();
  streaming.startMessage(
    { messageSenderName: 'Label', messageRecipientName: 'DSP', messageId: 'STREAM-1' },
    '3.8.2',
  );
  const ref = streaming.writeResource('A1', 'Track 1', 'Artist', 'USRT12400001', 'PT3M42S', null);
  streaming.finishResourcesStartReleases();
  streaming.writeRelease('R1', 'Album', 'Artist', null, null, null, null, [ref]);
  const stats = streaming.finishMessage();
  assert.ok(stats.bytesWritten > 0);
  const streamed = streaming.getXml();
  assert.ok(streamed.length > 0, 'getXml returns the document');
  assert.ok(streamed.includes('Track 1'));
  console.log('✓ StreamingDdexBuilder.getXml returns the written document');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
