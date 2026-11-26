// validateStructure reads the whole document: a reference to an undefined entity (`&foo;`;
// DTDs are not read) makes it invalid, with the entity's name, line and column, and so does
// a syntax error after the first element (before 0.5.1 only the first event was read).
// Input: the official sample examples/Samples43/1 Audio.xml.
// Run with: npm test (after `npm run build`).

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { validateStructure } = require('./index.js');

const SAMPLE = path.join(__dirname, '..', '..', '..', '..', 'examples', 'Samples43', '1 Audio.xml');
const TITLE = '<TitleText>Yume no Lullaby</TitleText>';
const DISPLAY_TITLE = '<DisplayTitle ApplicableTerritoryCode="Worldwide" IsDefault="true">';

async function main() {
  const sample = fs.readFileSync(SAMPLE, 'utf8');
  assert.ok(sample.includes(TITLE) && sample.includes(DISPLAY_TITLE));

  assert.deepEqual(await validateStructure(sample), { isValid: true, errors: [], warnings: [] });
  console.log('✓ The official sample is valid');

  const inText = await validateStructure(
    sample.replace(TITLE, '<TitleText>Yume &foo; Lullaby</TitleText>'),
  );
  assert.equal(inText.isValid, false);
  assert.deepEqual(inText.errors, [
    "XML parsing error: undefined entity '&foo;' at line 73, column 29",
  ]);
  console.log('✓ &foo; in text is reported with its line and column');

  const inAttribute = await validateStructure(
    sample.replace(
      DISPLAY_TITLE,
      '<DisplayTitle ApplicableTerritoryCode="World&foo;wide" IsDefault="true">',
    ),
  );
  assert.equal(inAttribute.isValid, false);
  assert.deepEqual(inAttribute.errors, [
    "XML parsing error: undefined entity '&foo;' in an attribute value at line 72, column 54",
  ]);
  console.log('✓ &foo; in an attribute value is reported with its line and column');

  const mismatched = await validateStructure(sample.replace('</TitleText>', '</Title>'));
  assert.equal(mismatched.isValid, false);
  assert.match(mismatched.errors[0], /^XML parsing error:/);
  console.log('✓ A mismatched end tag after the first element is reported');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
