// docs/VERSION_COMPATIBILITY.md
# DDEX ERN Version Compatibility Matrix

## Supported Versions

| Version | Status | Namespace | Features |
|---------|--------|-----------|----------|
| ERN 3.8.2 | ✅ Full Support | http://ddex.net/xml/ern/382 | Legacy format, basic structure |
| ERN 4.2 | ✅ Full Support | http://ddex.net/xml/ern/42 | Standard format, audit trail |
| ERN 4.3 | ✅ Full Support | http://ddex.net/xml/ern/43 | Latest, resource groups, chapters |

## Key Differences

### MessageHeader Changes

| Element | 3.8.2 | 4.2 | 4.3 |
|---------|-------|-----|-----|
| MessageThreadId | Required | Optional | Optional |
| MessageType | String | Enum | Extended Enum |
| MessageAuditTrail | ❌ | ✅ | ✅ |
| Profile | ❌ | ❌ | ✅ |
| PartyName | Single String | Array of LocalizedString | Array of LocalizedString |
| PartyId | Single String | Array of Identifier | Array of Identifier |

### DealTerms Evolution

| Element | 3.8.2 | 4.2 | 4.3 |
|---------|-------|-----|-----|
| DealReference | ❌ | ✅ | ✅ |
| CommercialModelType | Single | Array | Extended Array |
| PreOrderDate | ❌ | ❌ | ✅ |
| InstantGratificationDate | ❌ | ❌ | ✅ |
| PriceTier | ❌ | ✅ | ✅ |

### Resource Structure

| Feature | 3.8.2 | 4.2 | 4.3 |
|---------|-------|-----|-----|
| TechnicalInstantiation | ❌ | ✅ | ✅ |
| ResourceGroup | ❌ | ❌ | ✅ |
| ChapterInformation | ❌ | ❌ | ✅ |
| Title | String Array | LocalizedString Array | LocalizedString Array |

## Migration Paths

### Upgrading from 3.8.2 to 4.2
- MessageHeader: Convert single PartyName/Id to arrays
- DealTerms: Convert single CommercialModelType to array
- Resources: Add TechnicalInstantiation wrapper
- Add placeholder MessageAuditTrail

### Upgrading from 4.2 to 4.3
- Add Profile to MessageHeader if applicable
- Extend DealTerms with pre-order dates
- Group resources if chapters exist
- Add ChapterInformation for long-form content

### Downgrading (with data loss)
- 4.3 to 4.2: Lose Profile, PreOrderDates, ResourceGroups
- 4.2 to 3.8.2: Lose MessageAuditTrail, TechnicalInstantiation, multiple IDs

## Vendor Quirks

### Known Issues by Vendor

| Vendor | Version | Issue | Workaround |
|--------|---------|-------|------------|
| Vendor A | 3.8.2 | Missing MessageThreadId | Generate UUID |
| Vendor B | 4.2 | Empty MessageAuditTrail | Allow empty, warn |
| Vendor C | 4.3 | Invalid territory codes | Map to standard codes |
| Various | All | Mixed namespaces | Normalize to primary |
| Various | 3.8.2 | UTF-8 BOM | Strip before parsing |

## Auto-Detection Logic