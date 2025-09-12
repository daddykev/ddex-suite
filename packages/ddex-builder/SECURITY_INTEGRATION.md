# Error Sanitization Integration Guide

This guide shows how to integrate the error sanitization system across the DDEX Builder codebase to prevent information disclosure.

## Overview

The error sanitizer provides three key security features:

1. **Information Redaction**: Automatically removes sensitive data (file paths, IPs, memory addresses) from error messages
2. **Context-Aware Sanitization**: Different sanitization levels based on error context (file operations, XML parsing, security validation)
3. **Environment-Based Modes**: Production mode provides minimal information, development mode provides more details

## Basic Usage

### Using Global Functions (Recommended)

```rust
use ddex_builder::security::{
    sanitize_io_error, sanitize_parse_error, sanitize_build_error, 
    sanitize_security_error, ErrorContext
};
use std::io::{Error, ErrorKind};

// Before: Potential information disclosure
fn unsafe_file_operation() -> Result<String, String> {
    std::fs::read_to_string("/path/to/file")
        .map_err(|e| format!("Failed to read /path/to/file: {}", e))
}

// After: Secure error handling
fn safe_file_operation() -> Result<String, SanitizedError> {
    std::fs::read_to_string("/path/to/file")
        .map_err(|e| sanitize_io_error(e, ErrorContext::FileRead))
}
```

### Using Custom Sanitizer Instance

```rust
use ddex_builder::security::{ErrorSanitizer, SanitizerConfig, ErrorMode, ErrorContext};

fn create_production_sanitizer() -> ErrorSanitizer {
    let config = SanitizerConfig {
        mode: ErrorMode::Production,
        generate_correlation_ids: true,
        log_internal_details: true,
        max_message_length: 200,
        include_error_codes: true,
    };
    ErrorSanitizer::with_config(config)
}

fn handle_error_with_custom_sanitizer() {
    let mut sanitizer = create_production_sanitizer();
    
    match risky_operation() {
        Err(e) => {
            let sanitized = sanitizer.sanitize_security_error(e);
            eprintln!("Operation failed: {}", sanitized);
            // Log full details internally using correlation ID
            tracing::error!(correlation_id = %sanitized.correlation_id, "Security error occurred");
        }
        Ok(result) => println!("Success: {:?}", result),
    }
}
```

## Integration Patterns

### 1. File Operations

Replace all file operation error handling:

```rust
// Before
.map_err(|e| format!("Failed to open {}: {}", path, e))

// After  
.map_err(|e| sanitize_io_error(e, ErrorContext::FileOpen))
```

### 2. XML Parsing Errors

```rust
// Before
.map_err(|e| format!("XML parsing failed at {}: {}", location, e))

// After
.map_err(|e| sanitize_parse_error(e))
```

### 3. Build/Generation Errors

```rust
// Before
.map_err(|e| format!("Failed to generate XML: {}", e))

// After
.map_err(|e| sanitize_build_error(e))
```

### 4. Security Validation Errors

```rust
// Before
.map_err(|e| format!("Security validation failed: {}", e))

// After
.map_err(|e| sanitize_security_error(e))
```

## Error Context Guide

Choose the appropriate `ErrorContext` based on the operation:

- `ErrorContext::FileOpen` - File opening operations
- `ErrorContext::FileRead` - File reading operations  
- `ErrorContext::FileWrite` - File writing operations
- `ErrorContext::NetworkRequest` - Network operations
- `ErrorContext::XmlParsing` - XML parsing/validation
- `ErrorContext::XmlBuilding` - XML generation/building
- `ErrorContext::SecurityValidation` - Security checks
- `ErrorContext::EntityClassification` - Entity validation
- `ErrorContext::PathValidation` - Path validation
- `ErrorContext::MemoryAllocation` - Memory operations
- `ErrorContext::Authentication` - Authentication operations
- `ErrorContext::Authorization` - Authorization operations

## Environment Configuration

### Production Environment

```rust
let config = SanitizerConfig {
    mode: ErrorMode::Production,
    generate_correlation_ids: true,
    log_internal_details: true,   // Log internally only
    max_message_length: 150,      // Limit message length
    include_error_codes: true,    // Include error codes
};
```

### Development Environment

```rust
let config = SanitizerConfig {
    mode: ErrorMode::Development,
    generate_correlation_ids: true,
    log_internal_details: false,  // Reduce log noise
    max_message_length: 300,      // Allow longer messages
    include_error_codes: true,
};
```

### Testing Environment

```rust
let config = SanitizerConfig {
    mode: ErrorMode::Testing,
    generate_correlation_ids: false, // Reduce test noise
    log_internal_details: false,
    max_message_length: 500,         // Allow full messages
    include_error_codes: false,      // Focus on message content
};
```

## Advanced Features

### Custom Redaction Rules

```rust
let mut sanitizer = ErrorSanitizer::new();

// Add custom redaction rule for API keys
let api_key_rule = RedactionRule::new(
    "custom_api_keys",
    r"sk-[a-zA-Z0-9]{48}",  // Stripe-style API keys
    "sk-<redacted>",
    true,  // production
    true,  // development  
    false, // testing (allow real keys in tests)
).unwrap();

sanitizer.add_redaction_rule(api_key_rule);
```

### Error Correlation and Debugging

```rust
fn handle_error_with_correlation(sanitized: SanitizedError) {
    // Public response to user
    println!("Error: {}", sanitized);
    
    // Internal logging with full context
    tracing::error!(
        correlation_id = %sanitized.correlation_id,
        error_code = ?sanitized.code,
        context = ?sanitized.context,
        "Detailed error information logged for debugging"
    );
    
    // Optionally retrieve stored details later
    if let Some(details) = get_error_details(&sanitized.correlation_id) {
        tracing::debug!("Full error details: {}", details);
    }
}
```

### Implementing SecureError Trait

For custom error types:

```rust
use ddex_builder::security::{SecureError, ErrorLevel, ErrorContext};

#[derive(Debug)]
struct CustomError {
    message: String,
    sensitive_data: String,
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CustomError {}

impl SecureError for CustomError {
    fn public_message(&self) -> String {
        "Operation failed".to_string()
    }
    
    fn internal_message(&self) -> String {
        format!("{} (sensitive: {})", self.message, self.sensitive_data)
    }
    
    fn debug_message(&self) -> String {
        format!("{:?}", self)
    }
    
    fn error_level(&self) -> ErrorLevel {
        ErrorLevel::Internal
    }
    
    fn error_context(&self) -> ErrorContext {
        ErrorContext::XmlBuilding
    }
}
```

## Migration Strategy

### Phase 1: Core Error Paths
1. Update all file I/O error handling
2. Update XML parsing error handling  
3. Update security validation error handling

### Phase 2: Network and External Operations
1. Update network request error handling
2. Update database operation error handling
3. Update authentication/authorization errors

### Phase 3: Fine-Grained Context
1. Add specific error contexts for different operations
2. Implement custom redaction rules for domain-specific data
3. Add comprehensive logging and monitoring

### Phase 4: Testing and Validation
1. Test all error paths in different modes
2. Verify no sensitive information leaks in production mode
3. Ensure debugging information is available in development mode

## Best Practices

### Do's
- ✅ Use appropriate error contexts for different operations
- ✅ Initialize sanitizer early in application lifecycle
- ✅ Log correlation IDs for debugging
- ✅ Test error handling in all modes (production, development, testing)
- ✅ Use global functions for simple cases
- ✅ Configure appropriate message length limits
- ✅ Include error codes for programmatic handling

### Don'ts
- ❌ Include sensitive data in error messages before sanitization
- ❌ Log sanitized errors multiple times (causes correlation ID confusion)
- ❌ Use testing mode in production
- ❌ Ignore correlation IDs in error handling
- ❌ Override production mode sanitization for "debugging"
- ❌ Hardcode error messages instead of using context-aware sanitization
- ❌ Store correlation data indefinitely (implement cleanup)

## Testing Your Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ddex_builder::security::*;

    #[test]
    fn test_no_sensitive_data_leaks() {
        let config = SanitizerConfig {
            mode: ErrorMode::Production,
            ..SanitizerConfig::default()
        };
        
        let mut sanitizer = ErrorSanitizer::with_config(config);
        
        let sensitive_error = Error::new(
            ErrorKind::PermissionDenied, 
            "Cannot access /home/admin/secrets.txt containing API keys"
        );
        
        let sanitized = sanitizer.sanitize_io_error(sensitive_error, ErrorContext::FileRead);
        
        // Verify no sensitive information leaked
        assert_eq!(sanitized.message, "File operation failed");
        assert!(!sanitized.to_string().contains("/home/admin"));
        assert!(!sanitized.to_string().contains("secrets.txt"));
        assert!(!sanitized.to_string().contains("API keys"));
        
        // But should have useful information for debugging
        assert!(sanitized.code.is_some());
        assert!(!sanitized.correlation_id.is_empty());
    }
    
    #[test]
    fn test_development_mode_helpful() {
        let config = SanitizerConfig {
            mode: ErrorMode::Development,
            ..SanitizerConfig::default()
        };
        
        let mut sanitizer = ErrorSanitizer::with_config(config);
        
        let error = Error::new(ErrorKind::InvalidData, "Parse error at line 42");
        let sanitized = sanitizer.sanitize_parse_error(error);
        
        // Development mode should be more helpful
        assert!(sanitized.message.len() > "Operation failed".len());
        assert!(!sanitized.correlation_id.is_empty());
    }
}
```

This integration ensures that:
- Sensitive information is never leaked to users in production
- Developers get helpful error information during development
- Security teams can correlate errors using correlation IDs
- All error paths are consistently secured across the codebase