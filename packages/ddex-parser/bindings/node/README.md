# ddex-parser (Node.js)

DDEX ERN XML parser for Node.js, a native addon over the Rust
[`ddex-parser`](https://crates.io/crates/ddex-parser) crate. Part of
[DDEX Suite](https://github.com/daddykev/ddex-suite). It reads ERN 3.8.1, 3.8.2, 4.2 and
4.3 messages; TypeScript types are included (`index.d.ts`).

> **0.6.0 has breaking changes.** `ParseOptions` and the second argument of `parse` /
> `parseSync` were removed, `sanityCheck` returns `version: null` for an unrecognised
> version and has no `warnings`, and Node.js 22 or newer is required. See the [0.6.0
> migration
> guide](https://github.com/daddykev/ddex-suite/blob/main/website/docs/migration-0.6.md).

## Installation

```bash
npm install ddex-parser
```

Prebuilt binaries are installed for macOS (arm64, x64), Linux x64 (glibc) and Windows
x64, through the platform packages `ddex-parser-<platform>`. Node.js 22 or newer.

## Example

```js
const fs = require('node:fs');
const { DdexParser } = require('ddex-parser');

const parser = new DdexParser();
const message = parser.parseSync(fs.readFileSync('release.xml', 'utf8'));

console.log(message.messageId, message.version, message.messageControlType);
for (const release of message.releases) {
  console.log(release.releaseId, release.title, 'by', release.displayArtist);
  console.log('  release dates (as written):', release.releaseDates.map((d) => d.date));
  console.log('  genres:', release.genres.map((g) => g.genre), 'label:', release.labelName);
  console.log('  (P)', release.pLine?.year, release.pLine?.text, 'warning:', release.parentalWarning);
  for (const track of release.tracks) {
    const groups = track.groupPath.map((g) => `${g.groupType} ${g.sequenceNumber}`).join(' / ');
    console.log(`  ${track.position}. ${track.title} [${track.isrc}] disc ${track.discNumber} (${groups})`);
  }
}
for (const [reference, resource] of Object.entries(message.resources)) {
  console.log(reference, resource.resourceType, resource.title, resource.durationSeconds);
}
for (const deal of message.deals) {
  console.log('deal', deal.releases, deal.startDate, deal.endDate, deal.territories);
}
```

`parse(xml)` is the same, returning a `Promise<ParsedMessage>`:

```js
const fs = require('node:fs');
const { DdexParser } = require('ddex-parser');

const parser = new DdexParser();
const xml = fs.readFileSync('release.xml', 'utf8');

parser.parse(xml).then((message) => {
  console.log(`${message.releaseCount} release(s), ${message.dealCount} deal(s)`);
});
```

## API

- `new DdexParser()`
- `parseSync(xml: string): ParsedMessage` and `parse(xml: string): Promise<ParsedMessage>`
  take only the XML (`ParseOptions`, whose fields all threw, was removed in 0.6.0). They run
  the complete parser, whatever the input size (up to 100 MB of XML; there is no
  streaming in Node). Absent values are omitted, never invented, and text is kept as
  written.
- `detectVersion(xml)`, `sanityCheck(xml)` (ERN version detection and an XML
  well-formedness check, not XSD validation; `version` is `null` when no ERN version is
  recognised), `getDetailedError(xml)`.

`ParsedMessage` has the message header (`messageId`, `messageType`, `messageDate`,
`senderName`, `senderId`, `recipientName`, `recipientId`, `version`, `profile`,
`messageControlType`), counts, `releases` (each with `tracks`, `releaseDates`, `genres`,
`pLine` / `cLine`, `parentalWarning`, `labelName`, identifiers), `resources` keyed by
resource reference, and `deals`. See `index.d.ts` for every field.

## Security

The parser rejects DTDs and external entities (XXE) and references to undefined entities
(so no entity is expanded), limits element depth and input size, and never fetches from
the network. It does not validate against
the DDEX XSD or the AVS business rules.

## Links

- Documentation: [ddex-suite.org](https://ddex-suite.org/docs/parser)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.6.0 migration guide](https://github.com/daddykev/ddex-suite/blob/main/website/docs/migration-0.6.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
