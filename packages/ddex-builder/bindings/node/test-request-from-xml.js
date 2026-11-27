// requestFromXml: parse -> modify -> build, on the official samples
// (examples/Samples43/1 Audio.xml and 8 DjMix.xml).
// Run with: npm test (after `npm run build` here and in ../../../ddex-parser/bindings/node).

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexBuilder, requestFromXml } = require('./index.js');
const { DdexParser } = require(path.join(__dirname, '..', '..', '..', 'ddex-parser', 'bindings', 'node', 'index.js'));

const samples = path.join(__dirname, '..', '..', '..', '..', 'examples', 'Samples43');
const audio = fs.readFileSync(path.join(samples, '1 Audio.xml'), 'utf8');
const djMix = fs.readFileSync(path.join(samples, '8 DjMix.xml'), 'utf8');

// The fields the tests compare: release titles, track ISRCs, deal territories.
function summary(xml) {
  const message = new DdexParser().parseSync(xml);
  return {
    titles: message.releases.map((r) => r.title ?? null),
    isrcs: message.releases.flatMap((r) => r.tracks.map((t) => t.isrc ?? null)),
    territories: message.deals.map((d) => d.territories),
  };
}

// Every leaf that differs, as `path: a -> b`.
function diff(where, a, b, out) {
  if (a && b && typeof a === 'object' && typeof b === 'object' && Array.isArray(a) === Array.isArray(b)
      && (!Array.isArray(a) || a.length === b.length)) {
    for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) {
      diff(Array.isArray(a) ? `${where}[${key}]` : `${where}.${key}`, a[key] ?? null, b[key] ?? null, out);
    }
  } else if (JSON.stringify(a) !== JSON.stringify(b)) {
    out.push(`${where}: ${JSON.stringify(a)} -> ${JSON.stringify(b)}`);
  }
  return out;
}

// The request of `xml`, with the values the builder fills in (deal references, a default
// resource group) cleared where `original` left them out.
function mapped(xml, original) {
  const { request } = requestFromXml(xml);
  if (original) {
    request.deals.forEach((d, i) => { if (original.deals[i]?.deal_reference == null) d.deal_reference = null; });
    request.releases.forEach((r, i) => { if (original.releases[i]?.resource_group == null) r.resource_group = null; });
  }
  return request;
}

async function main() {
  // Build the mapped request: the compared fields equal the original's.
  const { request, unmapped } = requestFromXml(audio);
  const rebuilt = await new DdexBuilder().build(request);
  assert.deepEqual(summary(rebuilt), summary(audio));
  console.log('✓ requestFromXml -> build: titles, ISRCs and deal territories round-trip');

  // What the request cannot hold
  for (const p of ['header.message_thread_id', 'attributes.ReleaseProfileVersionId', 'resources[0].p_line']) {
    assert.ok(unmapped.includes(p), `${p} not in ${JSON.stringify(unmapped)}`);
  }
  console.log('✓ unmapped lists what the request cannot hold');

  // Modify the title: only that field differs
  const modified = requestFromXml(audio).request;
  modified.releases[0].title[0].text = 'Changed Title';
  const changed = await new DdexBuilder().build(modified);
  const diffs = diff('request', mapped(audio), mapped(changed, modified), []);
  assert.equal(diffs.length, 1, diffs.join('; '));
  assert.match(diffs[0], /^request\.releases\[0\]\.title\[0\]\.text: .* -> "Changed Title"$/);
  console.log('✓ modifying the title changes only the title');

  // DjMix: an empty <MessageId/> stays empty, so the build fails naming it
  const dj = requestFromXml(djMix);
  assert.equal(dj.request.header.message_id, '');
  await assert.rejects(new DdexBuilder().build(dj.request), /requires MessageHeader\/MessageId \(header\.message_id\)/);
  console.log('✓ DjMix: the empty MessageId fails the build, named');

  assert.throws(() => requestFromXml('<not ddex/>'), /Parse error/);
  console.log('✓ a parse error throws');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
