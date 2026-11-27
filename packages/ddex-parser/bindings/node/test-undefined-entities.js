// A reference to an undefined entity (`&foo;`; DTDs are not read) makes parse throw an
// error that names the entity and gives its line and column, in text and in attribute
// values; the predefined entities and character references still decode. Input: the
// official sample examples/Samples43/1 Audio.xml with `&foo;` injected into the first
// sound recording's title.
// A DOCTYPE is rejected too.
// Run with: npm test (after `npm run build`).

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { DdexParser } = require('./index.js');

const SAMPLE = path.join(__dirname, '..', '..', '..', '..', 'examples', 'Samples43', '1 Audio.xml');
const TITLE = '<TitleText>Yume no Lullaby</TitleText>';
const DISPLAY_TITLE = '<DisplayTitle ApplicableTerritoryCode="Worldwide" IsDefault="true">';

const sample = fs.readFileSync(SAMPLE, 'utf8');
assert.ok(sample.includes(TITLE) && sample.includes(DISPLAY_TITLE));

async function main() {
  const parser = new DdexParser();

  const inText = sample.replace(TITLE, '<TitleText>Yume &foo; Lullaby</TitleText>');
  const textError = "undefined entity '&foo;' at line 73, column 29";
  assert.throws(() => parser.parseSync(inText), (e) => e.message.includes(textError));
  await assert.rejects(parser.parse(inText), (e) => e.message.includes(textError));
  console.log('✓ &foo; in text: the error names it with its line and column');

  const inAttribute = sample.replace(
    DISPLAY_TITLE,
    '<DisplayTitle ApplicableTerritoryCode="World&foo;wide" IsDefault="true">',
  );
  const attributeError = "undefined entity '&foo;' in an attribute value at line 72, column 54";
  assert.throws(() => parser.parseSync(inAttribute), (e) => e.message.includes(attributeError));
  console.log('✓ &foo; in an attribute value: the error names it with its line and column');

  const decoded = 'A &amp; B caf&#233; caf&#xE9;';
  const ok = sample
    .replace(
      '<DisplayTitleText>Yume no Lullaby</DisplayTitleText>',
      `<DisplayTitleText>${decoded}</DisplayTitleText>`,
    )
    .replace(TITLE, `<TitleText>${decoded}</TitleText>`);
  const track = parser
    .parseSync(ok)
    .releases.flatMap((r) => r.tracks)
    .find((t) => t.isrc === 'JPTO09404900');
  assert.equal(track.title, 'A & B café café');
  console.log('✓ &amp;, &#233; and &#xE9; still decode');

  const withDoctype = sample.replace('?>', '?>\n<!DOCTYPE ern:NewReleaseMessage>');
  assert.throws(() => parser.parseSync(withDoctype), /DTD declarations are disabled/);
  await assert.rejects(parser.parse(withDoctype), /DTD declarations are disabled/);
  console.log('✓ a DOCTYPE is rejected');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
