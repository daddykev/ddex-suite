# Schema Versions

DDEX schema versions supported by DDEX Suite.

## Supported Versions

| Version | Release Date | Support Level | Recommended Use |
|---------|--------------|---------------|-----------------|
| **ERN 3.8.2** | 2014 | Full | Legacy systems |
| **ERN 4.2** | 2020 | Full | Stable production |
| **ERN 4.3** | 2022 | Full | New implementations |
| **ERN 4.4** | 2024 | Planned | Future releases |

## Version Comparison

### ERN 3.8.2 (Legacy)
**Strengths:**
- Mature and stable
- Widely supported
- Well-documented

**Limitations:**
- Limited metadata fields
- Older XML structure
- Missing modern features

```xml
<!-- ERN 3.8.2 Message Structure -->
<NewReleaseMessage MessageSchemaVersionId="ern/382"
                   xmlns="http://ddex.net/xml/ern/382">
  <MessageHeader>
    <MessageThreadId>ABC123</MessageThreadId>
    <MessageId>MSG001</MessageId>
    <!-- ... -->
  </MessageHeader>
  <UpdateIndicator>OriginalMessage</UpdateIndicator>
  <CatalogTransfer>
    <CatalogTransferType>FullCatalog</CatalogTransferType>
  </CatalogTransfer>
  <!-- ... -->
</NewReleaseMessage>
```

### ERN 4.2 (Stable)
**New Features:**
- Enhanced metadata support
- Improved territory handling
- Better validation rules
- Additional content types

**Key Improvements:**
- More flexible release structures
- Enhanced track metadata
- Better resource management
- Improved contributor roles

```xml
<!-- ERN 4.2 Message Structure -->
<NewReleaseMessage MessageSchemaVersionId="ern/42"
                   xmlns="http://ddex.net/xml/ern/42">
  <MessageHeader>
    <MessageId>MSG001</MessageId>
    <MessageSender>
      <PartyId>SENDER123</PartyId>
    </MessageSender>
    <!-- Enhanced header structure -->
  </MessageHeader>
  <!-- Improved body structure -->
</NewReleaseMessage>
```

### ERN 4.3 (Current)
**Latest Features:**
- Advanced metadata fields
- Enhanced validation
- Better streaming support
- Improved partner compatibility

**Key Enhancements:**
- Richer metadata model
- Better classification systems
- Enhanced territory management
- Improved resource references

```xml
<!-- ERN 4.3 Message Structure -->
<NewReleaseMessage MessageSchemaVersionId="ern/43"
                   xmlns="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>MSG001</MessageId>
    <MessageSender>
      <PartyId>SENDER123</PartyId>
      <PartyName>
        <FullName>Sender Name</FullName>
      </PartyName>
    </MessageSender>
    <!-- Latest header enhancements -->
  </MessageHeader>
  <!-- Most current body structure -->
</NewReleaseMessage>
```

## Migration Between Versions

### 3.8.2 → 4.2 Migration

```typescript
// Key changes when upgrading from 3.8.2 to 4.2
const migrationMapping = {
  // Header changes
  messageHeader: {
    '3.8.2': 'MessageThreadId',
    '4.2': 'MessageId' // Simplified identifier
  },
  
  // Release structure changes
  releaseData: {
    '3.8.2': 'ReleaseDetailsByTerritory/TerritoryCode',
    '4.2': 'ReleaseDetailsByTerritory/ExcludedTerritoryCode' // More flexible
  },
  
  // Resource references
  resources: {
    '3.8.2': 'ResourceReference',
    '4.2': 'ResourceReference/@ReleaseResourceReference' // Enhanced referencing
  }
};
```

### 4.2 → 4.3 Migration

```python
# Key differences between 4.2 and 4.3
VERSION_DIFFERENCES = {
    '4.2_to_4.3': {
        'enhanced_metadata': [
            'AdditionalTitle',
            'EnhancedGenre',
            'ExtendedResourceMetadata'
        ],
        'new_elements': [
            'AlternativeReleaseId',
            'EnhancedContributorRole',
            'DetailedRightsInformation'
        ],
        'improved_validation': [
            'StricterISRCValidation',
            'EnhancedTerritoryCodeCheck',
            'BetterDateFormatValidation'
        ]
    }
}
```

## Version Detection

```typescript
// Automatically detect DDEX version
export function detectDDEXVersion(xmlContent: string): string | null {
  const versionPatterns = {
    '3.8.2': /MessageSchemaVersionId="ern\/382"/,
    '4.2': /MessageSchemaVersionId="ern\/42"/,
    '4.3': /MessageSchemaVersionId="ern\/43"/
  };
  
  for (const [version, pattern] of Object.entries(versionPatterns)) {
    if (pattern.test(xmlContent)) {
      return version;
    }
  }
  
  // Try namespace detection as fallback
  const namespacePatterns = {
    '3.8.2': /xmlns="http:\/\/ddex\.net\/xml\/ern\/382"/,
    '4.2': /xmlns="http:\/\/ddex\.net\/xml\/ern\/42"/,
    '4.3': /xmlns="http:\/\/ddex\.net\/xml\/ern\/43"/
  };
  
  for (const [version, pattern] of Object.entries(namespacePatterns)) {
    if (pattern.test(xmlContent)) {
      return version;
    }
  }
  
  return null;
}
```

## Version-Specific Features

### ERN 3.8.2 Features
- Basic release metadata
- Standard contributor roles
- Simple territory handling
- Basic resource references
- Limited genre classification

### ERN 4.2 Features
- Enhanced release metadata
- Extended contributor roles
- Flexible territory management
- Improved resource handling
- Better genre classification
- Additional identifier support

### ERN 4.3 Features
- Advanced metadata model
- Rich contributor information
- Complex territory rules
- Enhanced resource management
- Comprehensive genre system
- Multiple identifier types
- Streaming-specific metadata

## Partner Support Matrix

| Partner | ERN 3.8.2 | ERN 4.2 | ERN 4.3 | Preferred Version |
|---------|-----------|---------|---------|-------------------|
| **Spotify** | ✅ | ✅ | ✅ | 4.3 |
| **Apple Music** | ✅ | ✅ | ✅ | 4.2/4.3 |
| **YouTube Music** | ✅ | ✅ | ✅ | 4.3 |
| **Amazon Music** | ✅ | ✅ | ✅ | 4.2/4.3 |
| **Deezer** | ✅ | ✅ | ✅ | 4.2 |
| **Tidal** | ✅ | ✅ | ✅ | 4.2/4.3 |

## Version Selection Guidelines

### Choose ERN 3.8.2 when:
- Working with legacy systems
- Partner specifically requires 3.8.2
- Minimal metadata requirements
- Existing 3.8.2 workflows

### Choose ERN 4.2 when:
- Need enhanced metadata
- Working with modern partners
- Require better territory handling
- Want improved validation

### Choose ERN 4.3 when:
- Building new implementations
- Need latest features
- Want best partner compatibility
- Require advanced metadata

## Validation Differences

```python
# Version-specific validation rules
VALIDATION_RULES = {
    '3.8.2': {
        'isrc_format': r'^[A-Z]{2}[A-Z0-9]{3}[0-9]{7}$',
        'required_fields': ['MessageId', 'ReleaseId', 'Title'],
        'territory_handling': 'basic'
    },
    '4.2': {
        'isrc_format': r'^[A-Z]{2}[A-Z0-9]{3}[0-9]{7}$',
        'required_fields': ['MessageId', 'ReleaseId', 'Title', 'Artist'],
        'territory_handling': 'enhanced',
        'additional_validations': ['contributor_roles', 'resource_types']
    },
    '4.3': {
        'isrc_format': r'^[A-Z]{2}[A-Z0-9]{3}[0-9]{7}$',
        'required_fields': ['MessageId', 'ReleaseId', 'Title', 'Artist'],
        'territory_handling': 'advanced',
        'additional_validations': [
            'contributor_roles', 
            'resource_types', 
            'metadata_consistency',
            'streaming_requirements'
        ]
    }
}
```

## Best Practices

1. **Use Latest Stable**: Generally prefer ERN 4.3 for new projects
2. **Check Partner Requirements**: Verify partner-specific version requirements
3. **Plan Migration Path**: Consider upgrade path when choosing versions
4. **Test Thoroughly**: Validate with target partners before production
5. **Document Version Choice**: Document why specific version was chosen
6. **Stay Updated**: Monitor new version releases and features
7. **Maintain Compatibility**: Consider supporting multiple versions if needed

## Future Versions

### ERN 4.4 (Upcoming)
**Expected Features:**
- Enhanced streaming metadata
- Better podcast support
- Improved classical music metadata
- Additional content types
- Enhanced rights management

**Timeline:**
- Specification: 2024 Q2
- Implementation: 2024 Q3
- DDEX Suite Support: 2024 Q4