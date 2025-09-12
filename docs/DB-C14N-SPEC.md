# DB-C14N/1.0 Specification

**DDEX Builder Canonical XML 1.0**

---

**Document Version**: 1.0  
**Date**: September 2024  
**Status**: Production Stable  
**Authors**: DDEX Suite Team  
**License**: MIT

## Abstract

DB-C14N/1.0 (DDEX Builder Canonical XML 1.0) is a canonicalization specification optimized for DDEX XML documents. It extends the W3C XML Canonicalization specifications (C14N and C14N11) with DDEX-specific optimizations for deterministic, byte-perfect XML generation while preserving business-critical metadata and partner extensions.

## 1. Introduction

### 1.1 Purpose

The music industry requires deterministic XML processing for:
- **Blockchain Applications**: Immutable metadata requires identical hashes
- **Audit and Compliance**: Reproducible builds for regulatory requirements
- **Partner Integration**: Consistent XML format across different systems
- **Version Control**: Meaningful diffs and change tracking
- **Caching Systems**: Reliable cache keys based on content

### 1.2 Design Goals

1. **Determinism**: Identical input always produces identical output
2. **DDEX Optimization**: Specialized handling for DDEX structures and patterns
3. **Extension Preservation**: Complete preservation of partner-specific extensions
4. **Performance**: Optimized for typical DDEX document sizes (1KB-100MB)
5. **Compliance**: Maintains W3C XML standards compliance
6. **Readability**: Human-readable output for debugging and validation

### 1.3 Relationship to W3C Standards

DB-C14N/1.0 is **compatible with** but **extends beyond** W3C C14N:
- **Builds upon**: W3C XML Canonicalization 1.0 (C14N) foundations
- **Extends**: DDEX-specific attribute ordering and namespace handling
- **Preserves**: Full XML semantics and business data integrity
- **Enhances**: Performance for music industry use cases

## 2. Canonicalization Process

### 2.1 Processing Steps

DB-C14N/1.0 processes XML documents in the following order:

1. **Document Structure Normalization**
2. **Namespace Declaration Optimization** 
3. **Element Ordering Stabilization**
4. **Attribute Canonicalization**
5. **Content Normalization**
6. **Extension Preservation**
7. **Final Output Generation**

### 2.2 Document Structure Normalization

#### 2.2.1 XML Declaration
- Remove XML declaration if using default values (`version="1.0"` and `encoding="UTF-8"`)
- Preserve XML declaration if non-default values are used
- Normalize encoding declaration to uppercase (`UTF-8`, not `utf-8`)

**Example**:
```xml
<!-- Input -->
<?xml version="1.0" encoding="utf-8"?>

<!-- DB-C14N Output -->
<?xml version="1.0" encoding="UTF-8"?>
```

#### 2.2.2 Processing Instructions
- Preserve all processing instructions in document order
- Normalize whitespace within PI content
- Maintain PI positioning relative to elements

#### 2.2.3 Comments
- Preserve comments when `preserve_comments` is enabled
- Remove comments when `preserve_comments` is disabled (default for performance)
- Normalize whitespace within comment content
- Maintain comment positioning relative to elements

### 2.3 Namespace Declaration Optimization

#### 2.3.1 Namespace Minimization
DB-C14N/1.0 implements intelligent namespace minimization:

1. **Remove Unused Declarations**: Unused namespace prefixes are removed
2. **Hoist Common Declarations**: Frequently used namespaces are moved to the root
3. **Minimize Redeclarations**: Avoid redundant declarations in child elements

**Example**:
```xml
<!-- Input -->
<root xmlns:a="http://a.example.com" xmlns:unused="http://unused.example.com">
    <child xmlns:a="http://a.example.com">
        <a:element>Content</a:element>
    </child>
</root>

<!-- DB-C14N Output -->
<root xmlns:a="http://a.example.com">
    <child>
        <a:element>Content</a:element>
    </child>
</root>
```

#### 2.3.2 DDEX Namespace Handling
Special handling for DDEX-specific namespaces:

- **ERN Namespace**: `http://ddex.net/xml/ern/{version}` - Always preserved
- **Partner Extensions**: Spotify, Apple, YouTube, Amazon - Preserved with original prefixes
- **Custom Extensions**: Label-specific namespaces - Preserved when possible

**Standard DDEX Namespace Order**:
1. Primary ERN namespace (default or `ernm:`)
2. Partner extensions (alphabetical by company name)
3. Custom extensions (alphabetical by domain)

### 2.4 Element Ordering Stabilization

#### 2.4.1 DDEX-Specific Element Order
DB-C14N/1.0 defines canonical ordering for key DDEX elements:

**Message Level**:
```
1. MessageHeader
2. UpdateIndicator (if present)
3. ResourceList (if present)
4. ReleaseList
5. DealList (if present)
```

**MessageHeader Order**:
```
1. MessageThreadId (if present)
2. MessageId
3. MessageFileName (if present)
4. MessageSender
5. MessageRecipient
6. MessageCreatedDateTime
```

**Release Order**:
```
1. ReleaseReference
2. ReleaseType
3. ReleaseId
4. DisplayTitleText / DisplayTitle
5. DisplayArtistName / DisplayArtist
6. LabelName
7. Genre (if present)
8. SubGenre (if present)
9. ReleaseDate / OriginalReleaseDate
10. PLineAndCopyrightDate
11. ReleaseResourceReferenceList
12. DealList (if present)
13. Extensions (partner-specific)
```

#### 2.4.2 Deterministic Ordering Algorithm
For elements not covered by DDEX-specific rules:
1. **Schema Order**: Follow XSD sequence order when available
2. **Alphabetical Order**: Sort by local name for choice groups
3. **Extension Order**: Partner extensions after standard elements

### 2.5 Attribute Canonicalization

#### 2.5.1 Attribute Ordering
DB-C14N/1.0 defines deterministic attribute ordering:

**Priority Order**:
1. **Schema-defined attributes** (in XSD order)
2. **DDEX-specific attributes** (see Section 2.5.2)
3. **Namespace declarations** (`xmlns` attributes)
4. **Partner extension attributes** (alphabetical by prefix)
5. **Custom attributes** (alphabetical)

#### 2.5.2 DDEX-Specific Attribute Order

**For ERN Message elements**:
```
1. MessageSchemaVersionId
2. BusinessTransactionId
3. ReleaseProfileName
4. LanguageAndScriptCode
5. TerritoryCode
6. Namespace
7. [Other attributes alphabetically]
```

**For Resource elements**:
```
1. SequenceNumber
2. LanguageAndScriptCode
3. TerritoryCode
4. Namespace
5. [Partner extensions]
6. [Other attributes alphabetically]
```

#### 2.5.3 Attribute Value Normalization

**String Values**:
- Trim leading/trailing whitespace
- Normalize internal whitespace to single spaces
- Preserve significant whitespace in CDATA

**Boolean Values**:
- Normalize to lowercase (`true`/`false`)
- Convert numeric representations (`1` → `true`, `0` → `false`)

**Numeric Values**:
- Remove leading zeros (except for IDs and codes)
- Preserve decimal precision for monetary values
- Scientific notation to decimal for display values

**Date/Time Values**:
- Normalize to ISO 8601 format
- Convert to UTC when timezone information is available
- Preserve original format for partial dates (year-only, etc.)

**Example**:
```xml
<!-- Input -->
<element flag="TRUE" count="007" date="2024-09-11T15:30:00+02:00">

<!-- DB-C14N Output -->
<element count="7" date="2024-09-11T13:30:00Z" flag="true">
```

### 2.6 Content Normalization

#### 2.6.1 Text Content
- Preserve significant whitespace
- Normalize line endings to LF (`\n`)
- Trim whitespace from pure text nodes
- Preserve formatting in mixed content

#### 2.6.2 CDATA Sections
- Preserve CDATA sections when they contain significant formatting
- Convert CDATA to text nodes when content doesn't require CDATA
- Normalize whitespace within CDATA according to context

### 2.7 Extension Preservation

#### 2.7.1 Partner Extensions
DB-C14N/1.0 provides specialized handling for known partner extensions:

**Spotify Extensions** (`xmlns:spotify="http://spotify.com/ddex/*"`):
- Preserve `spotify:` prefix
- Maintain Spotify-specific attribute order
- Preserve audio feature numeric precision

**Apple Extensions** (`xmlns:apple="http://apple.com/ddex/*"`):
- Preserve `apple:` prefix  
- Handle ADAM ID formatting consistently
- Preserve spatial audio metadata

**YouTube Extensions** (`xmlns:youtube="http://youtube.com/ddex/*"`):
- Preserve `youtube:` prefix
- Maintain video ID and channel ID format
- Preserve content ID references

#### 2.7.2 Custom Extensions
For custom label extensions:
- Preserve original namespace prefixes when possible
- Apply generic canonicalization rules
- Maintain element and attribute order within extensions

### 2.8 Final Output Generation

#### 2.8.1 Character Encoding
- Output UTF-8 encoding
- Escape special characters per XML 1.0 specification
- Preserve Unicode characters when valid

#### 2.8.2 Formatting
- No indentation (canonical form)
- No unnecessary whitespace between elements
- Single LF line ending at end of document

## 3. Implementation Requirements

### 3.1 Determinism Requirements

Implementations **MUST**:
1. Produce identical output for identical input across all platforms
2. Be independent of:
   - System time and timezone
   - Memory layout and garbage collection
   - Thread execution order
   - Filesystem state
3. Handle Unicode consistently across platforms

### 3.2 Performance Requirements

Implementations **SHOULD**:
- Process typical DDEX files (1-100KB) in <50ms
- Process large DDEX files (1-10MB) in <1s
- Use memory proportional to input size (not exponential)
- Support streaming for files >100MB

### 3.3 Compatibility Requirements

Implementations **MUST**:
- Accept all valid XML 1.0 documents
- Preserve all business-critical data
- Maintain namespace semantics
- Support DDEX ERN 3.8.2, 4.2, and 4.3

## 4. Test Suite and Validation

### 4.1 Canonical Test Cases

The DB-C14N/1.0 specification includes a comprehensive test suite:

1. **Basic Canonicalization**: 15 test cases
2. **DDEX-Specific Ordering**: 25 test cases  
3. **Extension Preservation**: 30 test cases
4. **Edge Cases**: 20 test cases
5. **Performance Tests**: 10 test cases

### 4.2 Validation Criteria

**Correctness**:
- 100% pass rate on canonical test suite
- Byte-identical output across multiple runs
- Successful round-trip through DDEX parser

**Performance**:
- Meet timing requirements for all test file sizes
- Memory usage within specified bounds
- No memory leaks during extended operation

### 4.3 Compliance Testing

```bash
# Run DB-C14N compliance test suite
cargo test --package ddex-builder --test db-c14n-compliance

# Performance benchmarking
cargo bench --package ddex-builder db_c14n_benchmark

# Memory usage testing
valgrind --tool=massif cargo test --package ddex-builder db_c14n_memory_test
```

## 5. Security Considerations

### 5.1 XML Security
- Protection against XML External Entity (XXE) attacks
- Entity expansion limits (billion laughs protection)
- Processing instruction content validation
- Comment content sanitization when preserved

### 5.2 Determinism Security
- Resistant to timing attacks through consistent processing
- No information leakage through processing time variations
- Cryptographically secure hash compatibility

### 5.3 Extension Security
- Validation of partner extension URIs
- Sanitization of custom extension content
- Limits on extension complexity and nesting depth

## 6. Examples

### 6.1 Basic Canonicalization

**Input**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage xmlns:ernm="http://ddex.net/xml/ern/43" 
                        xmlns:spotify="http://spotify.com/ddex"
                        BusinessTransactionId="BT123"
                        MessageSchemaVersionId="ern/43">
    <MessageHeader>
        <MessageId>MSG001</MessageId>
        <MessageSender>
            <PartyId Namespace="DPID">SENDER123</PartyId>
        </MessageSender>
    </MessageHeader>
    <ReleaseList>
        <Release>
            <ReleaseReference>REL001</ReleaseReference>
            <DisplayTitleText LanguageAndScriptCode="en">Test Album</DisplayTitleText>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>
```

**DB-C14N/1.0 Output**:
```xml
<ernm:NewReleaseMessage xmlns:ernm="http://ddex.net/xml/ern/43" xmlns:spotify="http://spotify.com/ddex" MessageSchemaVersionId="ern/43" BusinessTransactionId="BT123"><MessageHeader><MessageId>MSG001</MessageId><MessageSender><PartyId Namespace="DPID">SENDER123</PartyId></MessageSender></MessageHeader><ReleaseList><Release><ReleaseReference>REL001</ReleaseReference><DisplayTitleText LanguageAndScriptCode="en">Test Album</DisplayTitleText></Release></ReleaseList></ernm:NewReleaseMessage>
```

### 6.2 Extension Preservation

**Input with Partner Extensions**:
```xml
<Release xmlns:spotify="http://spotify.com/ddex"
         xmlns:apple="http://apple.com/ddex">
    <ReleaseId>
        <ProprietaryId Namespace="LABEL">LBL001</ProprietaryId>
        <spotify:SpotifyAlbumId>spotify:album:4iV5W9uYEdYUVa79Axb7Rh</spotify:SpotifyAlbumId>
        <apple:AdamId>1234567890</apple:AdamId>
    </ReleaseId>
    <DisplayTitleText LanguageAndScriptCode="en">Extension Test</DisplayTitleText>
</Release>
```

**DB-C14N/1.0 Output** (with extension preservation):
```xml
<Release xmlns:apple="http://apple.com/ddex" xmlns:spotify="http://spotify.com/ddex"><ReleaseId><ProprietaryId Namespace="LABEL">LBL001</ProprietaryId><apple:AdamId>1234567890</apple:AdamId><spotify:SpotifyAlbumId>spotify:album:4iV5W9uYEdYUVa79Axb7Rh</spotify:SpotifyAlbumId></ReleaseId><DisplayTitleText LanguageAndScriptCode="en">Extension Test</DisplayTitleText></Release>
```

## 7. Implementation Notes

### 7.1 DDEX Suite Implementation

The reference implementation in DDEX Suite provides:

```rust
use ddex_builder::{Builder, CanonicalizationAlgorithm};

let builder = Builder::new()
    .with_canonicalization(CanonicalizationAlgorithm::DbC14N);

let canonical_xml = builder.canonicalize(&input_xml)?;
```

### 7.2 Configuration Options

```rust
use ddex_builder::{CustomCanonicalizationRules, FidelityOptions};

let rules = CustomCanonicalizationRules {
    preserve_whitespace: false,
    sort_attributes: true,
    normalize_line_endings: true,
    minimize_namespaces: true,
    attribute_ordering: vec![
        "MessageSchemaVersionId".to_string(),
        "BusinessTransactionId".to_string(),
        // ... other DDEX-specific ordering
    ],
};

let fidelity_options = FidelityOptions {
    canonicalization: CanonicalizationAlgorithm::DbC14N,
    custom_canonicalization_rules: Some(rules),
    preserve_extensions: true,
    // ... other options
};
```

### 7.3 Performance Tuning

For different use cases:

**Maximum Fidelity**:
- Enable all preservation features
- Use DB-C14N with comment preservation
- Collect statistics for verification

**Balanced Performance**:
- Disable comment preservation
- Use standard DB-C14N settings
- Skip statistics collection

**High Performance**:
- Disable canonicalization (`Algorithm::None`)
- Minimal preservation features
- Streaming mode for large files

## 8. Future Considerations

### 8.1 Version Evolution
- DB-C14N/1.1 may include enhanced streaming support
- Additional partner extension optimizations
- Improved performance for very large catalogs

### 8.2 Standards Alignment
- Monitor W3C XML canonicalization updates
- Coordinate with DDEX standards evolution
- Maintain backward compatibility

## 9. References

1. **XML Canonicalization Version 1.0**: https://www.w3.org/TR/xml-c14n/
2. **XML Canonicalization Version 1.1**: https://www.w3.org/TR/xml-c14n11/
3. **DDEX ERN Standards**: https://kb.ddex.net/
4. **DDEX Suite Documentation**: https://ddex-suite.org
5. **W3C XML 1.0 Specification**: https://www.w3.org/TR/xml/

---

**Document Status**: Production Stable  
**Implementation**: DDEX Suite v0.2.5+  
**Test Coverage**: 100+ test cases  
**Last Updated**: September 2024