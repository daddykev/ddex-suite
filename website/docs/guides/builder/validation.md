# Validation

The DDEX Builder provides comprehensive validation to ensure your XML output meets DDEX specifications and partner requirements.

## Overview

Validation occurs at multiple stages:
- **Preflight validation**: Before building begins
- **Structure validation**: During XML construction  
- **Schema validation**: Against DDEX XSD schemas
- **Partner validation**: Using partner-specific rules

## Preflight Validation

Validate your data before building to catch errors early:

```typescript
import { DDEXBuilder, ValidationLevel } from 'ddex-builder';

const builder = new DDEXBuilder();
const data = { /* your release data */ };

// Comprehensive validation
const validation = await builder.validate(data, {
  level: ValidationLevel.STRICT,
  includeWarnings: true,
  partnerPreset: 'spotify'
});

if (!validation.isValid) {
  console.log('Validation errors:');
  validation.errors.forEach(error => {
    console.log(`- ${error.path}: ${error.message}`);
  });
  
  console.log('Warnings:');
  validation.warnings.forEach(warning => {
    console.log(`- ${warning.path}: ${warning.message}`);
  });
}
```

## Validation Levels

### Basic Validation
```python
from ddex_builder import DDEXBuilder, ValidationLevel

builder = DDEXBuilder()

# Basic validation - required fields only
result = builder.validate(data, level=ValidationLevel.BASIC)
```

### Standard Validation
```typescript
// Standard validation - DDEX specification compliance
const result = await builder.validate(data, {
  level: ValidationLevel.STANDARD,
  version: '4.3'
});
```

### Strict Validation
```python
# Strict validation - all rules + best practices
result = builder.validate(data, 
    level=ValidationLevel.STRICT,
    enforce_best_practices=True
)
```

## Partner-Specific Validation

Validate against partner requirements:

```typescript
// Spotify validation
const spotifyValidation = await builder.validate(data, {
  partnerPreset: 'spotify',
  includeMetadataChecks: true
});

// YouTube Music validation  
const youtubeValidation = await builder.validate(data, {
  partnerPreset: 'youtube',
  requireTerritoryInfo: true
});

// Apple Music validation
const appleValidation = await builder.validate(data, {
  partnerPreset: 'apple',
  enforceIsrcFormat: true
});
```

## Custom Validation Rules

Add your own validation logic:

```python
from ddex_builder import DDEXBuilder, ValidationRule

class CustomISRCRule(ValidationRule):
    def validate(self, data):
        errors = []
        for track in data.get('tracks', []):
            isrc = track.get('isrc')
            if isrc and not self.is_valid_isrc(isrc):
                errors.append({
                    'path': f'tracks[{track["id"]}].isrc',
                    'message': f'Invalid ISRC format: {isrc}',
                    'severity': 'error'
                })
        return errors
    
    def is_valid_isrc(self, isrc):
        # Your custom ISRC validation logic
        return len(isrc) == 12 and isrc[:2].isalpha()

builder = DDEXBuilder()
builder.add_validation_rule(CustomISRCRule())

result = builder.validate(data)
```

## Field-Level Validation

### Required Fields
```typescript
const validation = await builder.validate(data);

// Check specific field requirements
if (validation.missingRequired.length > 0) {
  console.log('Missing required fields:');
  validation.missingRequired.forEach(field => {
    console.log(`- ${field.path}: ${field.description}`);
  });
}
```

### Data Type Validation
```python
# Automatic type checking
data = {
    'title': 123,  # Should be string
    'duration': 'invalid',  # Should be number
    'release_date': '2023-13-45'  # Invalid date
}

result = builder.validate(data)
for error in result.type_errors:
    print(f"{error.path}: expected {error.expected_type}, got {error.actual_type}")
```

### Format Validation
```rust
use ddex_builder::{DDEXBuilder, ValidationOptions};

let options = ValidationOptions {
    validate_isrc_format: true,
    validate_upc_format: true,
    validate_territory_codes: true,
    validate_language_codes: true,
    validate_currencies: true,
};

let result = builder.validate_with_options(&data, options)?;
```

## Territory and Language Validation

```typescript
const validation = await builder.validate(data, {
  validateTerritories: true,
  validateLanguages: true,
  allowedTerritories: ['US', 'GB', 'DE'], // Restrict territories
  requireLanguageForLyrics: true
});

// Check territory-specific errors
validation.territoryErrors.forEach(error => {
  console.log(`Territory error: ${error.territory} - ${error.message}`);
});
```

## Validation During Build

Enable validation during the build process:

```python
builder = DDEXBuilder(
    validate_on_build=True,
    validation_level=ValidationLevel.STANDARD,
    fail_on_warnings=False
)

try:
    xml = builder.build(data)
    print("Build successful with validation")
except ValidationError as e:
    print(f"Validation failed: {e.message}")
    for error in e.errors:
        print(f"  - {error.path}: {error.message}")
```

## Batch Validation

Validate multiple releases efficiently:

```typescript
const releases = [/* array of release data */];

const batchValidation = await builder.validateBatch(releases, {
  parallel: true,
  stopOnFirstError: false,
  includeValidReleases: true
});

console.log(`Valid: ${batchValidation.valid.length}`);
console.log(`Invalid: ${batchValidation.invalid.length}`);

// Process valid releases
const validXml = await builder.buildBatch(batchValidation.valid);
```

## Error Categories

### Critical Errors
- Missing required fields
- Invalid data types
- Schema violations
- Partner requirement violations

### Warnings
- Missing recommended fields
- Suboptimal formats
- Best practice violations
- Performance concerns

### Info Messages
- Suggestions for improvement
- Alternative approaches
- Additional metadata opportunities

## Validation Reports

Generate detailed validation reports:

```python
validation = builder.validate(data, include_suggestions=True)

# Generate HTML report
report_html = validation.to_html_report()
with open('validation_report.html', 'w') as f:
    f.write(report_html)

# Generate JSON report for API integration
report_json = validation.to_json()

# Generate CSV for spreadsheet analysis
report_csv = validation.to_csv()
```

## Configuration

Customize validation behavior:

```typescript
const builder = new DDEXBuilder({
  validation: {
    enabledRules: [
      'required-fields',
      'data-types', 
      'isrc-format',
      'territory-codes'
    ],
    customRules: [
      new MyCustomRule(),
      new AnotherRule()
    ],
    errorThreshold: 10,  // Stop after 10 errors
    warningThreshold: 50, // Stop after 50 warnings
    treatWarningsAsErrors: false
  }
});
```

## Performance Considerations

Validation performance tips:

1. **Use appropriate validation levels** - Basic for development, Strict for production
2. **Enable parallel validation** for batch processing
3. **Cache validation results** for repeated builds
4. **Disable unused rules** to improve performance
5. **Use incremental validation** for large datasets

## Best Practices

1. **Validate early and often** - Don't wait until build time
2. **Use partner presets** when targeting specific platforms
3. **Implement custom rules** for organization-specific requirements
4. **Monitor validation metrics** to identify common issues
5. **Provide clear error messages** to help users fix issues
6. **Version your validation rules** along with your data schemas

## Common Validation Patterns

### Pre-submission Validation
```typescript
async function validateForSubmission(releaseData, targetPartner) {
  const validation = await builder.validate(releaseData, {
    level: ValidationLevel.STRICT,
    partnerPreset: targetPartner,
    includeMetadataQuality: true
  });
  
  if (!validation.isValid) {
    throw new Error(`Release not ready for ${targetPartner}: ${validation.summary}`);
  }
  
  return validation;
}
```

### Automated Quality Checks
```python
def automated_quality_check(releases):
    builder = DDEXBuilder()
    results = []
    
    for release in releases:
        validation = builder.validate(release, 
            level=ValidationLevel.STANDARD,
            include_quality_score=True
        )
        
        results.append({
            'id': release['id'],
            'valid': validation.is_valid,
            'quality_score': validation.quality_score,
            'recommendations': validation.recommendations
        })
    
    return results
```