# Builder Validation API

The validation system ensures DDEX XML output complies with schema requirements, business rules, and partner specifications before generation.

## Validation Classes

### DDEXValidator

Main validation engine for DDEX Builder:

```typescript
class DDEXValidator {
  constructor(options?: ValidationOptions);
  validate(data: any, version?: string): ValidationResult;
  validateAsync(data: any, version?: string): Promise<ValidationResult>;
  addCustomRule(rule: ValidationRule): void;
  removeCustomRule(ruleName: string): boolean;
  getAvailableRules(): ValidationRule[];
}
```

### ValidationOptions

Configuration for the validation engine:

```typescript
interface ValidationOptions {
  strictMode?: boolean;           // Strict schema compliance
  allowPartnerExtensions?: boolean; // Allow partner-specific extensions
  customRules?: ValidationRule[];   // Custom validation rules
  skipOptionalFields?: boolean;     // Skip validation of optional fields
  maxErrors?: number;              // Maximum errors before stopping
  enableWarnings?: boolean;        // Include warnings in results
}
```

## Validation Rules

### Built-in Rules

The validator includes comprehensive built-in rules:

```typescript
enum BuiltInRules {
  // Schema validation
  SCHEMA_COMPLIANCE = 'schemaCompliance',
  REQUIRED_FIELDS = 'requiredFields',
  DATA_TYPES = 'dataTypes',
  
  // Business logic
  UNIQUE_IDENTIFIERS = 'uniqueIdentifiers',
  REFERENCE_INTEGRITY = 'referenceIntegrity',
  DATE_CONSISTENCY = 'dateConsistency',
  
  // Format validation
  ISRC_FORMAT = 'isrcFormat',
  UPC_FORMAT = 'upcFormat',
  PARTY_IDENTIFIERS = 'partyIdentifiers',
  
  // Technical requirements
  DURATION_FORMAT = 'durationFormat',
  TERRITORY_CODES = 'territoryCodes',
  LANGUAGE_CODES = 'languageCodes'
}
```

### Custom Validation Rules

Create custom rules for specific business requirements:

```typescript
interface ValidationRule {
  name: string;
  description: string;
  severity: 'error' | 'warning' | 'info';
  validate: (data: any, context: ValidationContext) => ValidationError[];
}

// Example custom rule
const customISRCRule: ValidationRule = {
  name: 'customISRCValidation',
  description: 'Validates ISRC format for our catalog',
  severity: 'error',
  validate: (data, context) => {
    const errors: ValidationError[] = [];
    
    data.resources?.soundRecordings?.forEach((recording, index) => {
      const isrc = recording.soundRecordingId?.isrc;
      if (isrc && !isrc.match(/^[A-Z]{2}[A-Z0-9]{3}\d{7}$/)) {
        errors.push({
          path: `resources.soundRecordings[${index}].soundRecordingId.isrc`,
          message: 'ISRC must follow format: CC-XXX-YY-NNNNN',
          code: 'INVALID_ISRC_FORMAT',
          severity: 'error',
          value: isrc
        });
      }
    });
    
    return errors;
  }
};
```

## Validation Results

### ValidationResult

Complete validation outcome:

```typescript
interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationError[];
  info: ValidationError[];
  summary: ValidationSummary;
  metadata: ValidationMetadata;
}
```

### ValidationError

Individual validation issue:

```typescript
interface ValidationError {
  path: string;           // JSON path to the invalid field
  message: string;        // Human-readable error message
  code: string;          // Machine-readable error code
  severity: 'error' | 'warning' | 'info';
  value?: any;           // The invalid value
  expectedType?: string; // Expected data type
  suggestions?: string[]; // Possible fixes
}
```

### ValidationSummary

High-level validation statistics:

```typescript
interface ValidationSummary {
  totalChecks: number;
  errorCount: number;
  warningCount: number;
  infoCount: number;
  rulesCoverage: number;  // Percentage of rules that ran
  validationTime: number; // Time taken in milliseconds
}
```

## Usage Examples

### Basic Validation

```typescript
import { DDEXBuilder, DDEXValidator } from 'ddex-builder';

const validator = new DDEXValidator({
  strictMode: true,
  enableWarnings: true
});

const releaseData = {
  releaseId: 'R123456789',
  title: 'Example Album',
  // ... other release data
};

const result = validator.validate(releaseData, '4.3');

if (!result.isValid) {
  console.error(`Validation failed with ${result.errors.length} errors:`);
  result.errors.forEach(error => {
    console.error(`- ${error.path}: ${error.message}`);
  });
} else {
  console.log('Validation passed!');
}
```

### Async Validation with Custom Rules

```typescript
const validator = new DDEXValidator();

// Add custom rules
validator.addCustomRule(customISRCRule);
validator.addCustomRule(labelCodeValidationRule);

// Validate asynchronously
const result = await validator.validateAsync(releaseData, '4.3');

// Process results
if (result.warnings.length > 0) {
  console.warn('Validation warnings:');
  result.warnings.forEach(warning => {
    console.warn(`- ${warning.path}: ${warning.message}`);
  });
}
```

### Integration with Builder

```typescript
const builder = new DDEXBuilder({
  validation: {
    enabled: true,
    strictMode: false,
    customRules: [customISRCRule],
    maxErrors: 50
  }
});

try {
  // Builder automatically validates before building
  const xml = await builder.build(releaseData);
  console.log('Build successful!');
} catch (error) {
  if (error.validationErrors) {
    console.error('Validation errors prevented build:');
    error.validationErrors.forEach(err => {
      console.error(`- ${err.path}: ${err.message}`);
    });
  }
}
```

## Partner-Specific Validation

### Preset Validation Rules

```typescript
import { DDEXValidator, PartnerPresets } from 'ddex-builder';

// Use Spotify-specific validation
const spotifyValidator = new DDEXValidator({
  preset: PartnerPresets.SPOTIFY,
  strictMode: true
});

// Custom partner validation
const customPartnerRules = {
  maxTrackCount: 500,
  requiredArtworkDimensions: { width: 1400, height: 1400 },
  requiredPreviewClips: true,
  allowedTerritories: ['US', 'CA', 'MX']
};

const partnerValidator = new DDEXValidator({
  customRules: createPartnerRules(customPartnerRules)
});
```

### Multi-Partner Validation

```typescript
const multiValidator = new DDEXValidator();

// Validate against multiple partner requirements
const partners = ['spotify', 'apple', 'youtube'];
const results = {};

for (const partner of partners) {
  const preset = PartnerPresets[partner.toUpperCase()];
  const validator = new DDEXValidator({ preset });
  results[partner] = validator.validate(releaseData);
}

// Find common issues
const commonErrors = findCommonValidationIssues(results);
console.log('Issues affecting multiple partners:', commonErrors);
```

## Performance Optimization

### Validation Caching

```typescript
const validator = new DDEXValidator({
  cache: {
    enabled: true,
    maxSize: 1000,    // Cache up to 1000 validation results
    ttl: 3600000      // 1 hour cache lifetime
  }
});

// Subsequent validations of identical data will use cache
const result1 = validator.validate(releaseData);
const result2 = validator.validate(releaseData); // Uses cache
```

### Selective Validation

```typescript
// Only validate specific sections
const partialResult = validator.validate(releaseData, '4.3', {
  sections: ['releases', 'resources'],
  skipRules: ['territoryValidation', 'dealValidation']
});

// Validate only changed fields
const incrementalResult = validator.validateIncremental(
  originalData,
  changedData,
  changedPaths
);
```

## Error Recovery

### Validation with Auto-Fix

```typescript
const validator = new DDEXValidator({
  autoFix: {
    enabled: true,
    maxFixes: 10,
    allowedFixes: [
      'trimWhitespace',
      'formatDates', 
      'normalizeIdentifiers'
    ]
  }
});

const result = validator.validate(releaseData);

if (result.autoFixed) {
  console.log(`Auto-fixed ${result.autoFixed.length} issues:`);
  result.autoFixed.forEach(fix => {
    console.log(`- ${fix.path}: ${fix.description}`);
  });
}
```

### Graceful Degradation

```typescript
const validator = new DDEXValidator({
  errorHandling: {
    strategy: 'continue',    // Continue validation after errors
    maxErrors: 100,          // Stop after 100 errors
    reportPartialResults: true
  }
});

const result = validator.validate(problematicData);

// Even with errors, get partial validation results
if (result.partialResults) {
  console.log(`Validated ${result.partialResults.coverage}% of data`);
}
```

## See Also

- [Builder API Reference](./index.md) - Main builder documentation  
- [TypeScript API](./typescript) - TypeScript-specific validation
- [Python API](./python) - Python validation examples
- [Error Handling Guide](../../guides/error-handling) - Error handling strategies