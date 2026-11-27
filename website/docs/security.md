---
sidebar_position: 9
title: Security model
description: The XML threats the parser handles, how text passes through the builder, what is not validated at runtime, and the open security issues.
---

# Security model

DDEX messages come from partners, so the parser treats its input as untrusted. The libraries
defend against **XML-level threats**; they do not judge the metadata itself.

## What the parser rejects

The default configuration (`SecurityConfig::strict()` in Rust; the Node.js and Python
bindings always use it) applies these rules before any value is read:

| Threat | Handling |
|---|---|
| DTDs (`<!DOCTYPE ...>`) | Rejected by every reader (`parse` and the Rust streaming and selective readers). This also stops internal entity declarations |
| External entities (XXE) | Rejected; nothing is read from files or URLs |
| Entity expansion ("billion laughs") | No entity is ever expanded (a reference to anything but the five predefined entities and character references is an error), so there is no reference limit |
| Deep nesting | Limited to 100 levels of elements |
| Oversized input | The Node.js binding takes up to 100 MB of XML; Rust and Python set no size limit |
| Network access | None: no schema, DTD or entity is ever fetched |

A rejected message is an ordinary parse error that says which rule it broke:

```js
const assert = require('node:assert/strict');
const { DdexParser } = require('ddex-parser');

const xxe = `<?xml version="1.0"?>
<!DOCTYPE ern:NewReleaseMessage [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
  <MessageHeader><MessageId>&xxe;</MessageId></MessageHeader>
</ern:NewReleaseMessage>`;

assert.throws(() => new DdexParser().parseSync(xxe), /DTD declarations are disabled/);
```

In Rust, `DDEXParser::with_config` takes a different `SecurityConfig` (the reference and
depth limits). A DOCTYPE and undefined entities are rejected whatever the configuration.

## Text passes through, escaped

Metadata that looks like SQL, HTML or script (`'; DROP TABLE`, `<script>`) is legitimate
text as far as DDEX is concerned. The parser returns it unchanged, and the builder writes it
XML-escaped, so it reads back exactly as given. Escaping it for your own database, HTML page
or shell is up to you.

The builder rejects only characters that XML 1.0 does not allow at all (C0 control
characters other than tab, line feed and carriage return, U+FFFE, U+FFFF), naming the
field. See [escaping and illegal characters](./build-request.mdx#escaping-and-illegal-characters).

## What is not validated at runtime

- **No XSD validation.** Neither package validates a message against the DDEX XSD at
  runtime. The parser reads what it understands and ignores the rest; the builder checks the
  fields the target version requires, and its output is validated against the official XSDs
  in CI ([ERN version support](./ern-versions.md)).
- **No AVS / business-rule validation.** Allowed-value sets, identifier checksums and
  partner rules are not enforced. Use [DDEX Workbench](https://ddex-workbench.org) for that.
- **Well-formedness gaps.** A `]]>` sequence in character data is accepted (it is not
  well-formed XML) and the surrounding text is mangled; mismatched tags are reported as a
  generic XML error rather than a specific one.

## Known security issues

These are in helper APIs of `ddex-builder` that `build` does not depend on, and are pinned by
ignored tests until they are fixed:

- `PathValidator` accepts some unsafe paths and rejects some valid ones.
- `InputValidator` is regex-based: it rejects legitimate metadata text (a title like
  "Delete Me" trips its keyword blocklist) and misses some malformed input.
- `OutputSanitizer::sanitize_xml_output` rewrites the documents it accepts instead of
  validating them, and accepts truncated XML.
- Secure log messages can still contain sensitive paths.

The full list, with the tests that pin each one, is in
[KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md).

## Supply chain and reporting

- `Cargo.lock` is committed. CI runs `cargo-deny` (advisories, banned crates, licenses,
  sources) with the policy in [`deny.toml`](https://github.com/daddykev/ddex-suite/blob/main/deny.toml).
- Every wheel and npm package is installed and smoke-tested before a release is published;
  publishing is a manual step.
- From 0.5.1, release artifacts are built in GitHub Actions with signed build-provenance
  attestations and SBOMs, verifiable with `gh attestation verify <file> --repo daddykev/ddex-suite`,
  provided the published files are the ones release-build produced. Publishing is manual,
  so there is no npm `--provenance` or PyPI trusted-publishing attestation.

Report security problems privately by email to [security@ddex-suite.org](mailto:security@ddex-suite.org), not in a public issue; see
[SECURITY.md](https://github.com/daddykev/ddex-suite/blob/main/SECURITY.md).
