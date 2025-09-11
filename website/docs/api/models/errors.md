# Error Models

Error models define the structure and types of errors that can occur during DDEX parsing, validation, and building operations.

## Core Error Types

### DDEXError

Base error class for all DDEX-related errors:

```typescript
class DDEXError extends Error {
  code: string;
  severity: ErrorSeverity;
  path?: string;
  context?: ErrorContext;
  recoverable: boolean;
  timestamp: Date;
}
```

### ErrorSeverity

Classification of error severity:

```typescript
enum ErrorSeverity {
  FATAL = 'fatal',       // Operation cannot continue
  ERROR = 'error',       // Significant problem, operation may fail
  WARNING = 'warning',   // Potential issue, operation can continue
  INFO = 'info'         // Informational message
}
```

### ErrorContext

Additional context for error debugging:

```typescript
interface ErrorContext {
  operation: string;          // 'parse', 'build', 'validate'
  version?: string;           // DDEX version being processed
  messageType?: string;       // Type of DDEX message
  fileSize?: number;          // Size of input file
  memoryUsage?: number;       // Memory usage at error time
  processingTime?: number;    // Time elapsed before error
  stackTrace?: string[];      // Detailed stack trace
}
```

## Parser Errors

### ParseError

Errors occurring during XML parsing:

```typescript
class ParseError extends DDEXError {
  xmlLine?: number;
  xmlColumn?: number;
  xmlPosition?: number;
  expectedElements?: string[];
}
```

### Common Parser Error Codes

```typescript
enum ParseErrorCodes {
  INVALID_XML = 'PARSE_INVALID_XML',
  SCHEMA_VIOLATION = 'PARSE_SCHEMA_VIOLATION', 
  UNSUPPORTED_VERSION = 'PARSE_UNSUPPORTED_VERSION',
  MISSING_HEADER = 'PARSE_MISSING_HEADER',
  INVALID_ENCODING = 'PARSE_INVALID_ENCODING',
  XXE_DETECTED = 'PARSE_XXE_DETECTED',
  MEMORY_LIMIT_EXCEEDED = 'PARSE_MEMORY_LIMIT',
  TIMEOUT_EXCEEDED = 'PARSE_TIMEOUT'
}
```

### Parser Error Examples

```typescript
// XML structure error
const xmlError = new ParseError('Invalid XML structure at line 45', {
  code: ParseErrorCodes.INVALID_XML,
  severity: ErrorSeverity.FATAL,
  xmlLine: 45,
  xmlColumn: 23,
  recoverable: false
});

// Schema validation error  
const schemaError = new ParseError('Required element MessageId missing', {
  code: ParseErrorCodes.SCHEMA_VIOLATION,
  severity: ErrorSeverity.ERROR,
  path: 'MessageHeader.MessageId',
  expectedElements: ['MessageId'],
  recoverable: false
});
```

## Builder Errors

### BuildError

Errors during DDEX XML generation:

```typescript
class BuildError extends DDEXError {
  validationErrors?: ValidationError[];
  templateErrors?: TemplateError[];
  canonicalizationErrors?: string[];
}
```

### Common Builder Error Codes

```typescript
enum BuildErrorCodes {
  VALIDATION_FAILED = 'BUILD_VALIDATION_FAILED',
  MISSING_REQUIRED_DATA = 'BUILD_MISSING_REQUIRED_DATA',
  INVALID_DATA_TYPE = 'BUILD_INVALID_DATA_TYPE',
  REFERENCE_NOT_FOUND = 'BUILD_REFERENCE_NOT_FOUND',
  TEMPLATE_ERROR = 'BUILD_TEMPLATE_ERROR',
  CANONICALIZATION_FAILED = 'BUILD_CANONICALIZATION_FAILED',
  OUTPUT_TOO_LARGE = 'BUILD_OUTPUT_TOO_LARGE'
}
```

### Builder Error Examples

```typescript
// Validation failure
const validationError = new BuildError('Build validation failed', {
  code: BuildErrorCodes.VALIDATION_FAILED,
  severity: ErrorSeverity.ERROR,
  validationErrors: [
    {
      path: 'releases[0].title',
      message: 'Title is required',
      code: 'MISSING_REQUIRED_FIELD'
    }
  ],
  recoverable: true
});

// Reference resolution error
const referenceError = new BuildError('Resource reference not found', {
  code: BuildErrorCodes.REFERENCE_NOT_FOUND,
  severity: ErrorSeverity.ERROR,
  path: 'releases[0].resourceReferences[2]',
  context: {
    operation: 'build',
    version: '4.3'
  },
  recoverable: false
});
```

## Validation Errors

### ValidationError

Detailed validation error information:

```typescript
interface ValidationError {
  path: string;              // JSON path to invalid field
  message: string;           // Human-readable error message  
  code: string;             // Machine-readable error code
  severity: ErrorSeverity;  // Error severity level
  value?: any;              // The invalid value
  expectedType?: string;    // Expected data type
  expectedFormat?: string;  // Expected format pattern
  suggestions?: string[];   // Possible corrections
  ruleId?: string;         // Validation rule that failed
}
```

### Common Validation Error Codes

```typescript
enum ValidationErrorCodes {
  // Required fields
  MISSING_REQUIRED_FIELD = 'VALIDATION_MISSING_REQUIRED',
  EMPTY_REQUIRED_FIELD = 'VALIDATION_EMPTY_REQUIRED',
  
  // Data types
  INVALID_TYPE = 'VALIDATION_INVALID_TYPE',
  INVALID_FORMAT = 'VALIDATION_INVALID_FORMAT',
  INVALID_LENGTH = 'VALIDATION_INVALID_LENGTH',
  
  // Business rules
  DUPLICATE_IDENTIFIER = 'VALIDATION_DUPLICATE_ID',
  INVALID_REFERENCE = 'VALIDATION_INVALID_REFERENCE',
  DATE_INCONSISTENCY = 'VALIDATION_DATE_INCONSISTENCY',
  
  // Format-specific
  INVALID_ISRC = 'VALIDATION_INVALID_ISRC',
  INVALID_UPC = 'VALIDATION_INVALID_UPC',
  INVALID_DURATION = 'VALIDATION_INVALID_DURATION',
  INVALID_TERRITORY = 'VALIDATION_INVALID_TERRITORY'
}
```

## System Errors

### SystemError

Infrastructure and system-level errors:

```typescript
class SystemError extends DDEXError {
  systemInfo?: {
    platform: string;
    nodeVersion?: string;
    memoryAvailable: number;
    diskSpace: number;
  };
}
```

### Common System Error Codes

```typescript
enum SystemErrorCodes {
  OUT_OF_MEMORY = 'SYSTEM_OUT_OF_MEMORY',
  DISK_FULL = 'SYSTEM_DISK_FULL',
  FILE_NOT_FOUND = 'SYSTEM_FILE_NOT_FOUND',
  PERMISSION_DENIED = 'SYSTEM_PERMISSION_DENIED',
  NETWORK_ERROR = 'SYSTEM_NETWORK_ERROR',
  TIMEOUT = 'SYSTEM_TIMEOUT'
}
```

## Error Handling Patterns

### Error Collection

Collect multiple errors for batch processing:

```typescript
class ErrorCollector {
  private errors: DDEXError[] = [];
  
  add(error: DDEXError): void {
    this.errors.push(error);
  }
  
  addValidationError(path: string, message: string, code: string): void {
    this.errors.push(new ValidationError(message, {
      code,
      path,
      severity: ErrorSeverity.ERROR
    }));
  }
  
  hasErrors(): boolean {
    return this.errors.some(e => e.severity === ErrorSeverity.ERROR || e.severity === ErrorSeverity.FATAL);
  }
  
  getErrors(): DDEXError[] {
    return this.errors.filter(e => e.severity === ErrorSeverity.ERROR || e.severity === ErrorSeverity.FATAL);
  }
  
  getWarnings(): DDEXError[] {
    return this.errors.filter(e => e.severity === ErrorSeverity.WARNING);
  }
  
  getAll(): DDEXError[] {
    return [...this.errors];
  }
}
```

### Error Recovery

Attempt to recover from errors when possible:

```typescript
class ErrorRecovery {
  static canRecover(error: DDEXError): boolean {
    return error.recoverable && error.severity !== ErrorSeverity.FATAL;
  }
  
  static suggestFix(error: ValidationError): string[] {
    const suggestions: string[] = [];
    
    switch (error.code) {
      case ValidationErrorCodes.MISSING_REQUIRED_FIELD:
        suggestions.push(`Add required field: ${error.path}`);
        break;
        
      case ValidationErrorCodes.INVALID_ISRC:
        suggestions.push('Use format: CC-XXX-YY-NNNNN (e.g., US-ABC-12-34567)');
        break;
        
      case ValidationErrorCodes.INVALID_DURATION:
        suggestions.push('Use ISO 8601 duration format: PT3M45S for 3:45');
        break;
        
      default:
        suggestions.push('Check documentation for correct format');
    }
    
    return suggestions;
  }
}
```

## Usage Examples

### Basic Error Handling

```typescript
import { DDEXParser, ParseError } from 'ddex-parser';

try {
  const parser = new DDEXParser();
  const result = await parser.parse(xmlContent);
} catch (error) {
  if (error instanceof ParseError) {
    console.error(`Parse error at line ${error.xmlLine}: ${error.message}`);
    console.error(`Error code: ${error.code}`);
    
    if (error.recoverable) {
      console.log('This error might be recoverable');
    }
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

### Validation Error Handling

```typescript
import { DDEXBuilder, BuildError, ValidationError } from 'ddex-builder';

try {
  const builder = new DDEXBuilder();
  const xml = await builder.build(releaseData);
} catch (error) {
  if (error instanceof BuildError && error.validationErrors) {
    console.error('Validation errors:');
    
    error.validationErrors.forEach((validationError: ValidationError) => {
      console.error(`- ${validationError.path}: ${validationError.message}`);
      
      if (validationError.suggestions) {
        console.log(`  Suggestions: ${validationError.suggestions.join(', ')}`);
      }
    });
  }
}
```

### Error Logging and Monitoring

```typescript
class ErrorLogger {
  static log(error: DDEXError): void {
    const logEntry = {
      timestamp: error.timestamp,
      code: error.code,
      severity: error.severity,
      message: error.message,
      path: error.path,
      context: error.context,
      recoverable: error.recoverable
    };
    
    // Log to appropriate system based on severity
    switch (error.severity) {
      case ErrorSeverity.FATAL:
      case ErrorSeverity.ERROR:
        console.error(JSON.stringify(logEntry));
        // Send to error monitoring service
        break;
        
      case ErrorSeverity.WARNING:
        console.warn(JSON.stringify(logEntry));
        break;
        
      case ErrorSeverity.INFO:
        console.info(JSON.stringify(logEntry));
        break;
    }
  }
}
```

## See Also

- [Parser API](../parser/) - Parser error handling
- [Builder API](../builder/) - Builder error handling
- [Error Handling Guide](../../guides/error-handling) - Comprehensive error handling strategies
- [Validation API](../builder/validation) - Validation error details