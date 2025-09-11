# Identifier Types

Complete reference for standard identifiers, codes, and unique keys used throughout DDEX messages.

## Overview

DDEX employs various standardized identifier systems to ensure global uniqueness and enable cross-platform content identification. These identifiers serve as the foundation for content tracking, rights management, and commercial operations.

## Standard Identifier Categories

### Audio Content Identifiers

| Identifier | Full Name | Scope | Format | Example |
|------------|-----------|-------|--------|---------|
| ISRC | International Standard Recording Code | Sound recordings | 12 characters | `USRC17607839` |
| ISWC | International Standard Musical Work Code | Musical works | Variable | `T-010.015.001-1` |
| GRID | Global Release Identifier | Releases | 18 characters | `A1-2425G-ABC1234002-M` |

### Product Identifiers

| Identifier | Full Name | Scope | Format | Example |
|------------|-----------|-------|--------|---------|
| UPC | Universal Product Code | Products (North America) | 12 digits | `123456789012` |
| EAN | European Article Number | Products (International) | 13 digits | `1234567890123` |
| GTIN | Global Trade Item Number | Products (Global) | 8/12/13/14 digits | `01234567890128` |

### Party Identifiers

| Identifier | Full Name | Scope | Format | Example |
|------------|-----------|-------|--------|---------|
| DPID | DDEX Party Identifier | DDEX parties | 21 characters | `PADPIDA2014101001U` |
| ISNI | International Standard Name Identifier | People/organizations | 16 digits | `0000 0001 2280 7671` |
| IPI | Interested Parties Information | Rights holders | 9-11 digits | `00052210040` |

## ISRC (International Standard Recording Code)

### Format Structure

```
[Country Code][Registrant][Year][Serial]
   2 chars      3 chars   2 digits  5 digits
```

#### Components
- **Country Code**: ISO 3166-1 alpha-2 (e.g., US, GB, DE)
- **Registrant**: Alphanumeric code assigned to registrant
- **Year**: Last 2 digits of registration year
- **Serial**: Sequential number for recordings

#### Examples and Validation

```typescript
interface ISRC {
  value: string;           // Full ISRC code
  countryCode: string;     // 2-character country
  registrant: string;      // 3-character registrant
  year: string;           // 2-digit year
  serial: string;         // 5-digit serial
}

// Validation regex
const ISRC_PATTERN = /^[A-Z]{2}[A-Z0-9]{3}\d{7}$/;

// Examples
const validISRCs = [
  'USRC17607839',  // US registrant RC1, year 17, serial 607839
  'GBUM71505078',  // GB registrant UM7, year 15, serial 05078
  'DEAR81000001'   // DE registrant AR8, year 10, serial 00001
];

function validateISRC(isrc: string): boolean {
  return ISRC_PATTERN.test(isrc);
}

function parseISRC(isrc: string): ISRC | null {
  if (!validateISRC(isrc)) return null;
  
  return {
    value: isrc,
    countryCode: isrc.substring(0, 2),
    registrant: isrc.substring(2, 5),
    year: isrc.substring(5, 7),
    serial: isrc.substring(7, 12)
  };
}
```

## UPC/EAN Product Codes

### UPC-A Format (12 digits)

```
[System][Manufacturer][Product][Check]
  1 digit    5 digits     5 digits  1 digit
```

### EAN-13 Format (13 digits)

```
[Country][Manufacturer][Product][Check]
 2-3 digits   4-5 digits   3-5 digits  1 digit
```

#### Validation and Calculation

```typescript
interface ProductCode {
  value: string;
  type: 'UPC' | 'EAN';
  isValid: boolean;
  checkDigit: number;
}

function calculateUPCCheckDigit(code: string): number {
  const digits = code.substring(0, 11).split('').map(Number);
  const sum = digits.reduce((acc, digit, index) => {
    return acc + digit * (index % 2 === 0 ? 1 : 3);
  }, 0);
  return (10 - (sum % 10)) % 10;
}

function calculateEANCheckDigit(code: string): number {
  const digits = code.substring(0, 12).split('').map(Number);
  const sum = digits.reduce((acc, digit, index) => {
    return acc + digit * (index % 2 === 0 ? 1 : 3);
  }, 0);
  return (10 - (sum % 10)) % 10;
}

function validateProductCode(code: string): ProductCode {
  const cleanCode = code.replace(/\D/g, '');
  
  if (cleanCode.length === 12) {
    const checkDigit = calculateUPCCheckDigit(cleanCode);
    const providedCheck = parseInt(cleanCode.charAt(11));
    
    return {
      value: cleanCode,
      type: 'UPC',
      isValid: checkDigit === providedCheck,
      checkDigit
    };
  } else if (cleanCode.length === 13) {
    const checkDigit = calculateEANCheckDigit(cleanCode);
    const providedCheck = parseInt(cleanCode.charAt(12));
    
    return {
      value: cleanCode,
      type: 'EAN',
      isValid: checkDigit === providedCheck,
      checkDigit
    };
  }
  
  return {
    value: cleanCode,
    type: cleanCode.length === 12 ? 'UPC' : 'EAN',
    isValid: false,
    checkDigit: -1
  };
}
```

## DPID (DDEX Party Identifier)

### Format Structure

```
[Prefix][Country][Type][Date][Sequence][Check]
   P        AD       P     IDA    2014      10    10    01    U
```

#### Components
- **Prefix**: Always "P" 
- **Authority**: "AD" for DDEX
- **Type**: "P" for Party, "S" for Service
- **Prefix Code**: "IDA" 
- **Registration Date**: YYYYMMDD
- **Sequence**: Sequential number
- **Check Character**: Calculated check

#### DPID Examples

```typescript
interface DPID {
  value: string;
  authority: string;
  type: 'Party' | 'Service';
  registrationDate: Date;
  sequence: string;
  checkCharacter: string;
}

// Examples
const validDPIDs = [
  'PADPIDA2014101001U',  // Party registered 2014-10-10, sequence 01
  'PADPIDA2015032515G',  // Party registered 2015-03-25, sequence 15
  'PADSIDA2016081203M'   // Service registered 2016-08-12, sequence 03
];

function parseDPID(dpid: string): DPID | null {
  const pattern = /^P(AD)(P|S)(IDA)(\d{8})(\d{2})([A-Z])$/;
  const match = dpid.match(pattern);
  
  if (!match) return null;
  
  const [, authority, type, prefix, dateStr, sequence, check] = match;
  const year = parseInt(dateStr.substring(0, 4));
  const month = parseInt(dateStr.substring(4, 6)) - 1;
  const day = parseInt(dateStr.substring(6, 8));
  
  return {
    value: dpid,
    authority,
    type: type === 'P' ? 'Party' : 'Service',
    registrationDate: new Date(year, month, day),
    sequence,
    checkCharacter: check
  };
}
```

## ISNI (International Standard Name Identifier)

### Format and Structure

```
[Block 1][Block 2][Block 3][Block 4]
  4 digits  4 digits  4 digits  4 digits (with check digit)
```

#### ISNI Validation

```typescript
interface ISNI {
  value: string;          // Full ISNI
  formatted: string;      // With spaces: "0000 0001 2280 7671"
  checkDigit: string;     // Last character (digit or X)
  isValid: boolean;
}

function validateISNI(isni: string): ISNI {
  // Remove spaces and normalize
  const cleaned = isni.replace(/\s/g, '');
  const digits = cleaned.substring(0, 15);
  const providedCheck = cleaned.charAt(15);
  
  // Calculate check digit
  let total = 0;
  for (let i = 0; i < 15; i++) {
    total = (total + parseInt(digits.charAt(i))) * 2;
  }
  
  const remainder = total % 11;
  const calculatedCheck = remainder === 1 ? 'X' : ((12 - remainder) % 11).toString();
  
  return {
    value: cleaned,
    formatted: cleaned.replace(/(\d{4})(\d{4})(\d{4})(\d{4})/, '$1 $2 $3 $4'),
    checkDigit: calculatedCheck,
    isValid: providedCheck === calculatedCheck
  };
}

// Examples
const validISNIs = [
  '0000000123456789',  // Numeric check digit
  '000000012345678X',  // X check digit for remainder 1
  '0000 0001 2280 7671' // Formatted with spaces
];
```

## GRid (Global Release Identifier)

### Format Structure

```
[Scheme][Issuer][Release][Check]
   A1      2425G     ABC1234002    M
```

#### Components
- **Scheme**: "A1" for current scheme
- **Issuer**: 5-character issuer code  
- **Release**: 10-character release identifier
- **Check**: Single check character

#### GRid Examples

```typescript
interface GRid {
  value: string;
  scheme: string;
  issuer: string;
  release: string;
  checkCharacter: string;
}

function parseGRid(grid: string): GRid | null {
  // Remove hyphens if present
  const cleaned = grid.replace(/-/g, '');
  
  if (cleaned.length !== 18) return null;
  
  return {
    value: cleaned,
    scheme: cleaned.substring(0, 2),
    issuer: cleaned.substring(2, 7),
    release: cleaned.substring(7, 17),
    checkCharacter: cleaned.substring(17, 18)
  };
}

// Format with standard hyphens
function formatGRid(grid: string): string {
  const cleaned = grid.replace(/-/g, '');
  return `${cleaned.substring(0, 2)}-${cleaned.substring(2, 7)}-${cleaned.substring(7, 17)}-${cleaned.substring(17, 18)}`;
}

// Examples
const validGRids = [
  'A1-2425G-ABC1234002-M',
  'A1-LABEL-REL0000001-7',
  'A1-DIST1-XYZ9876543-K'
];
```

## Proprietary Identifiers

### Namespace System

```typescript
interface ProprietaryId {
  namespace: string;      // Organization identifier
  value: string;         // Value within namespace
  schemeURI?: string;    // Optional scheme URI
}

// Common proprietary namespace examples
const proprietaryExamples = [
  {
    namespace: 'Spotify',
    value: 'spotify:track:4iV5W9uYEdYUVa79Axb7Rh',
    schemeURI: 'https://open.spotify.com'
  },
  {
    namespace: 'AppleMusic',
    value: '1234567890',
    schemeURI: 'https://music.apple.com'
  },
  {
    namespace: 'YouTubeMusic',
    value: 'MPLAyb_mkp1c',
    schemeURI: 'https://music.youtube.com'
  },
  {
    namespace: 'InternalLabel',
    value: 'LABEL-2024-001-TRACK-05'
  }
];
```

## Identifier Mapping and Resolution

### Cross-Platform Mapping

```typescript
interface IdentifierMapping {
  primaryId: string;          // Primary identifier (usually ISRC)
  platformMappings: Map<string, string>;
  lastUpdated: Date;
  confidence: number;         // 0-1 confidence score
}

class IdentifierResolver {
  private mappings = new Map<string, IdentifierMapping>();
  
  addMapping(primaryId: string, platform: string, platformId: string): void {
    let mapping = this.mappings.get(primaryId);
    if (!mapping) {
      mapping = {
        primaryId,
        platformMappings: new Map(),
        lastUpdated: new Date(),
        confidence: 1.0
      };
      this.mappings.set(primaryId, mapping);
    }
    
    mapping.platformMappings.set(platform, platformId);
    mapping.lastUpdated = new Date();
  }
  
  resolveToISRC(platformId: string, platform: string): string | null {
    for (const [isrc, mapping] of this.mappings) {
      if (mapping.platformMappings.get(platform) === platformId) {
        return isrc;
      }
    }
    return null;
  }
  
  resolveToPlatform(isrc: string, platform: string): string | null {
    const mapping = this.mappings.get(isrc);
    return mapping?.platformMappings.get(platform) || null;
  }
}
```

## Identifier Generation

### ISRC Generation

```typescript
class ISRCGenerator {
  private countryCode: string;
  private registrant: string;
  private currentYear: number;
  private serialCounter: number = 1;
  
  constructor(countryCode: string, registrant: string) {
    this.countryCode = countryCode.toUpperCase();
    this.registrant = registrant.toUpperCase();
    this.currentYear = new Date().getFullYear();
  }
  
  generateISRC(): string {
    const year = (this.currentYear % 100).toString().padStart(2, '0');
    const serial = this.serialCounter.toString().padStart(5, '0');
    this.serialCounter++;
    
    return `${this.countryCode}${this.registrant}${year}${serial}`;
  }
  
  generateBatch(count: number): string[] {
    return Array.from({ length: count }, () => this.generateISRC());
  }
}

// Usage
const generator = new ISRCGenerator('US', 'RC1');
const newISRCs = generator.generateBatch(10);
// ['USRC24000001', 'USRC24000002', ...]
```

### UPC Generation

```typescript
class UPCGenerator {
  private manufacturerPrefix: string;
  private productCounter: number = 1;
  
  constructor(manufacturerPrefix: string) {
    if (manufacturerPrefix.length !== 6) {
      throw new Error('Manufacturer prefix must be 6 digits');
    }
    this.manufacturerPrefix = manufacturerPrefix;
  }
  
  generateUPC(): string {
    const productCode = this.productCounter.toString().padStart(5, '0');
    this.productCounter++;
    
    const partial = `0${this.manufacturerPrefix}${productCode}`;
    const checkDigit = calculateUPCCheckDigit(partial);
    
    return `${partial}${checkDigit}`;
  }
}

// Usage  
const upcGen = new UPCGenerator('123456');
const newUPC = upcGen.generateUPC(); // '012345600001X'
```

## Validation Integration

### Multi-Identifier Validation

```typescript
interface ValidationResult {
  identifier: string;
  type: string;
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

class IdentifierValidator {
  validateAll(identifiers: Record<string, string>): ValidationResult[] {
    const results: ValidationResult[] = [];
    
    for (const [type, value] of Object.entries(identifiers)) {
      switch (type.toLowerCase()) {
        case 'isrc':
          results.push(this.validateISRC(value));
          break;
        case 'upc':
        case 'ean':
          results.push(this.validateProductCode(value));
          break;
        case 'dpid':
          results.push(this.validateDPID(value));
          break;
        case 'isni':
          results.push(this.validateISNI(value));
          break;
        case 'grid':
          results.push(this.validateGRid(value));
          break;
        default:
          results.push({
            identifier: value,
            type,
            isValid: true, // Assume proprietary IDs are valid
            errors: [],
            warnings: [`Unknown identifier type: ${type}`]
          });
      }
    }
    
    return results;
  }
  
  private validateISRC(isrc: string): ValidationResult {
    const isValid = validateISRC(isrc);
    return {
      identifier: isrc,
      type: 'ISRC',
      isValid,
      errors: isValid ? [] : ['Invalid ISRC format'],
      warnings: []
    };
  }
  
  // Additional validation methods...
}
```

## Best Practices

### Identifier Management

1. **Consistency**: Use consistent identifier formats across all systems
2. **Validation**: Always validate identifiers before storing or transmitting
3. **Normalization**: Store identifiers in normalized format (uppercase, no spaces)
4. **Mapping**: Maintain cross-platform identifier mappings
5. **Generation**: Use proper generation algorithms for new identifiers

### Data Quality

1. **Verification**: Cross-reference identifiers with authoritative sources
2. **Deduplication**: Detect and resolve duplicate identifier assignments
3. **Monitoring**: Monitor for identifier format changes or new standards
4. **Backup Systems**: Maintain fallback identification methods
5. **Documentation**: Document custom identifier schemes thoroughly

### Performance Optimization

1. **Indexing**: Index identifier fields for fast lookups
2. **Caching**: Cache frequently accessed identifier mappings
3. **Batch Processing**: Process identifier operations in batches
4. **Normalization**: Pre-normalize identifiers during ingestion
5. **Validation**: Use efficient validation algorithms for high-volume operations