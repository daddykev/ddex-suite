# DDEX Parser Error Contract

## Overview

The DDEX Parser uses a unified error contract across all language bindings to ensure consistent error handling. All errors crossing the FFI boundary are transformed into `FFIError` structures.

## Error Structure

### Core Fields
- `code`: Machine-readable error code (e.g., "XML_PARSE_ERROR")
- `message`: Human-readable error message
- `location`: Optional location information
- `severity`: Error severity level (Error, Warning, Info)
- `hint`: Optional suggestion for fixing the error
- `category`: Error category for grouping

### Error Categories

| Category | Description | Example Codes |
|----------|-------------|---------------|
| XmlParsing | XML syntax or structure errors | XML_PARSE_ERROR, MALFORMED_XML |
| InvalidVersion | Unsupported DDEX version | INVALID_VERSION, UNKNOWN_VERSION |
| UnresolvedReference | Reference integrity issues | UNRESOLVED_REFERENCE, CIRCULAR_REF |
| SecurityViolation | Security limit exceeded | SECURITY_VIOLATION, XXE_ATTEMPT |
| IoError | File I/O issues | IO_ERROR, FILE_NOT_FOUND |
| Timeout | Operation timeout | PARSE_TIMEOUT, STREAM_TIMEOUT |

## Language Bindings

### JavaScript/TypeScript
```typescript
class DDEXParseError extends Error {
  code: string;
  location?: {
    path: string;
    line: number;
    column: number;
    byteOffset?: number;
  };
  severity: 'error' | 'warning' | 'info';
  category: ErrorCategory;
  hint?: string;
}
```

### Python
```python
class DDEXParseError(Exception):
    def __init__(self, code: str, message: str, location: Optional[ErrorLocation] = None):
        self.code = code
        self.message = message
        self.location = location
        self.severity = severity
        self.category = category
        self.hint = hint
```

### Rust (Native)
```rust
pub enum ParseError {
    XmlError { message: String, location: ErrorLocation },
    InvalidVersion { version: String },
    UnresolvedReference { reference: String, location: ErrorLocation },
    SecurityViolation { limit: String },
    Io(std::io::Error),
    Timeout { seconds: u64 },
}
```

## Error Codes

### XML Parsing Errors
- `XML_PARSE_ERROR`: General XML parsing error
- `MALFORMED_XML`: XML is not well-formed
- `UNEXPECTED_EOF`: Unexpected end of file
- `INVALID_ENCODING`: Character encoding issue

### Version Errors
- `INVALID_VERSION`: Version not supported
- `VERSION_MISMATCH`: Version conflicts in document
- `NO_VERSION_FOUND`: Cannot detect version

### Reference Errors
- `UNRESOLVED_REFERENCE`: Reference target not found
- `CIRCULAR_REFERENCE`: Circular reference detected
- `DUPLICATE_REFERENCE`: Duplicate reference ID

### Security Errors
- `SECURITY_VIOLATION`: General security violation
- `XXE_ATTEMPT`: External entity expansion attempt
- `MAX_DEPTH_EXCEEDED`: Maximum nesting depth exceeded
- `MAX_SIZE_EXCEEDED`: Maximum size limit exceeded

### System Errors
- `IO_ERROR`: File I/O error
- `PARSE_TIMEOUT`: Parsing timeout
- `OUT_OF_MEMORY`: Memory limit exceeded

## Error Handling Best Practices

1. **Always check error category first** - Handle different categories differently
2. **Use error codes for programmatic handling** - Don't parse messages
3. **Display hints to users** - They provide actionable guidance
4. **Log full error context** - Including location for debugging
5. **Handle warnings gracefully** - They shouldn't stop processing

## Examples

### Handling in JavaScript
```javascript
try {
  const result = await parser.parse(xmlContent);
} catch (error) {
  if (error instanceof DDEXParseError) {
    switch (error.category) {
      case 'XmlParsing':
        console.error(`XML error at line ${error.location?.line}: ${error.message}`);
        break;
      case 'SecurityViolation':
        console.error('Security issue detected. File may be malicious.');
        break;
      default:
        console.error(`Parse error: ${error.message}`);
    }
    
    if (error.hint) {
      console.log(`Hint: ${error.hint}`);
    }
  }
}
```

### Handling in Python
```python
try:
    result = parser.parse(xml_content)
except DDEXParseError as e:
    if e.category == ErrorCategory.XML_PARSING:
        print(f"XML error at {e.location}: {e.message}")
    elif e.category == ErrorCategory.SECURITY_VIOLATION:
        print("Security violation - file may be malicious")
    
    if e.hint:
        print(f"Suggestion: {e.hint}")
```