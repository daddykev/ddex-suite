# DB-C14N/1.0

**DDEX Builder Canonical XML 1.0**: the canonical form `ddex-builder` writes. This page
describes what the implementation (`ddex_builder::canonical::DB_C14N`,
`packages/ddex-builder/src/canonical/`) does. It is the project's own form, not a W3C
canonicalization (it is not C14N or C14N 1.1).

## Where it is used

- `DDEXBuilder::build` canonicalizes its output by default (`BuildOptions::output.canon_mode`
  is `CanonMode::DbC14n`). With `CanonMode::Pretty` the generator's output is returned as
  written.
- `DDEXBuilder::canonicalize(xml, &options)` canonicalizes any XML when
  `options.canonicalization` is `CanonicalizationAlgorithm::DbC14N` (the default), and
  returns the input unchanged when it is `None`. There are no other algorithms and no
  options: the C14N, C14N 1.1 and custom algorithms and the preservation options of
  earlier versions were not implemented and were removed in 0.6.0.
- `DB_C14N::new().canonicalize(xml)` is the canonicalizer itself.
  `DB_C14N::canonical_hash(xml)` is the SHA-256 (lowercase hex) of the string it is given;
  it does not canonicalize first.

## Input

The input must be a well-formed XML document with one root element. A reference to an
entity other than the five predefined ones (and character references) is an error. The ERN
version is taken from the first ERN namespace URI found in the text
(`http://ddex.net/xml/ern/381`, `382`, `42` or `43`); without one, the canonicalizer's
version is used (4.3 for `DB_C14N::new()`, or the one given to `DB_C14N::with_version`).

## Rules

1. **XML declaration.** The output starts with `<?xml version="1.0" encoding="UTF-8"?>`
   and a line feed, whatever the input declared.
2. **What is kept.** Elements, attributes, text and comments inside the root element.
   Processing instructions, a DOCTYPE, and comments outside the root element are dropped.
3. **Element order.** Child elements keep the order they have in the input. The builder's
   generators write elements in XSD sequence order, which makes that the canonical order;
   the canonicalizer does not reorder elements.
4. **Namespace declarations.** For each ERN version there is a fixed table from DDEX
   namespace URIs to prefixes (for example `http://ddex.net/xml/ern/43` → `ern`,
   `http://ddex.net/xml/avs` → `avs`, and the XML Schema instance namespace → `xsi`). A
   declaration of a URI in the table is written with the table's prefix; other
   declarations keep their prefix (a clashing prefix gets a unique one). Element and
   attribute names are not rewritten, so a document that uses other prefixes for these
   namespaces must already use the table's prefixes (the builder's output does).
5. **Attribute order.** On each element, namespace declarations come first, sorted by
   prefix (the default namespace first), then the other attributes sorted by qualified
   name (byte order).
6. **Text.** Whitespace-only text between elements (indentation) is dropped. All other
   text is kept exactly as written: it is not trimmed, collapsed or line-end normalized
   beyond what an XML parser does, and a leaf element keeps its text even when it is only
   whitespace.
7. **Escaping.** In text, `&`, `<` and `>` are escaped, and a carriage return is written as
   `&#13;`. In attribute values, additionally `"` and `'`, and tab, line feed and carriage
   return as `&#9;`, `&#10;` and `&#13;`, so the values read back unchanged. Comments are
   written as they were.
8. **Layout.** Each element starts on its own line, indented by two spaces per level
   (`OutputConfig::indent_char` and `indent_width` do not apply to DB-C14N). An element with
   no content is written `<Name/>`; an element with only text has it inline
   (`<Name>text</Name>`); an element with child elements has each child on its own line and
   its end tag on its own line. Every line ends with a line feed, and there is no trailing
   whitespace.

## Determinism

The builder reads no clock or random source and uses no unordered iteration on the build
path (enforced by clippy lints). The same request and options produce byte-identical
output with the same crate version; repeated builds are tested to be byte-identical
(`packages/ddex-builder/tests/it/determinism_0_6.rs`).

## Example

```rust
use ddex_builder::DB_C14N;

let input = r#"<ern:NewReleaseMessage MessageSchemaVersionId="ern/43" xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader><MessageId>MSG001</MessageId></MessageHeader>
</ern:NewReleaseMessage>"#;

let canonical = DB_C14N::new().canonicalize(input)?;
assert_eq!(
    canonical,
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\" MessageSchemaVersionId=\"ern/43\">\n  \
     <MessageHeader>\n    \
     <MessageId>MSG001</MessageId>\n  \
     </MessageHeader>\n\
     </ern:NewReleaseMessage>\n"
);
# Ok::<(), ddex_builder::BuildError>(())
```
