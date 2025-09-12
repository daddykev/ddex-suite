//! Error Sanitization Demo
//!
//! This example demonstrates how to use the error sanitization system
//! to prevent information disclosure while maintaining debugging capabilities.

use ddex_builder::security::{
    ErrorSanitizer, SanitizerConfig, ErrorMode, ErrorContext,
    sanitize_io_error, sanitize_parse_error, sanitize_build_error,
    sanitize_security_error,
};
use std::io::{Error, ErrorKind};

fn main() {
    println!("🔒 Error Sanitization Demo");
    println!("==========================\n");

    // Demo 1: Production mode sanitization
    demonstrate_production_mode();
    println!();

    // Demo 2: Development mode sanitization
    demonstrate_development_mode();
    println!();

    // Demo 3: Global sanitizer functions
    demonstrate_global_functions();
    println!();

    // Demo 4: Different error contexts
    demonstrate_error_contexts();
    println!();

    // Demo 5: Path redaction examples
    demonstrate_path_redaction();
}

fn demonstrate_production_mode() {
    println!("📋 Demo 1: Production Mode Sanitization");
    println!("----------------------------------------");
    
    let config = SanitizerConfig {
        mode: ErrorMode::Production,
        generate_correlation_ids: true,
        log_internal_details: false, // Don't spam logs in demo
        max_message_length: 150,
        include_error_codes: true,
    };
    
    let mut sanitizer = ErrorSanitizer::with_config(config);
    
    // Simulate various errors that could leak sensitive information
    let errors = vec![
        (
            "File Access Error",
            Error::new(ErrorKind::PermissionDenied, "Permission denied: /home/admin/secrets.txt"),
            ErrorContext::FileRead,
        ),
        (
            "Network Error",
            Error::new(ErrorKind::ConnectionRefused, "Connection refused to 192.168.1.100:8080"),
            ErrorContext::NetworkRequest,
        ),
        (
            "XML Parsing Error", 
            Error::new(ErrorKind::InvalidData, "Invalid XML at /usr/local/app/config.xml:42"),
            ErrorContext::XmlParsing,
        ),
        (
            "Memory Error",
            Error::new(ErrorKind::OutOfMemory, "Failed to allocate at address 0x7fff5fbff000"),
            ErrorContext::MemoryAllocation,
        ),
    ];
    
    for (name, error, context) in errors {
        let sanitized = sanitizer.sanitize(error, context);
        println!("  {}: {}", name, sanitized);
    }
}

fn demonstrate_development_mode() {
    println!("🛠️  Demo 2: Development Mode Sanitization");
    println!("------------------------------------------");
    
    let config = SanitizerConfig {
        mode: ErrorMode::Development,
        generate_correlation_ids: true,
        log_internal_details: false,
        max_message_length: 200,
        include_error_codes: true,
    };
    
    let mut sanitizer = ErrorSanitizer::with_config(config);
    
    // Same errors, but in development mode
    let file_error = Error::new(ErrorKind::NotFound, "Cannot find /Users/dev/project/data.xml");
    let sanitized = sanitizer.sanitize_io_error(file_error, ErrorContext::FileOpen);
    println!("  File Error (Dev Mode): {}", sanitized);
    
    let parse_error = Error::new(ErrorKind::InvalidData, "Malformed XML: unexpected token at line 15");
    let sanitized = sanitizer.sanitize_parse_error(parse_error);
    println!("  Parse Error (Dev Mode): {}", sanitized);
}

fn demonstrate_global_functions() {
    println!("🌐 Demo 3: Global Sanitizer Functions");
    println!("-------------------------------------");
    
    // These functions use a global sanitizer instance for convenience
    let io_error = Error::new(ErrorKind::PermissionDenied, "Access denied to /etc/shadow");
    let sanitized = sanitize_io_error(io_error, ErrorContext::FileRead);
    println!("  Global IO Error: {}", sanitized);
    
    let parse_error = Error::new(ErrorKind::InvalidData, "Invalid DDEX structure");
    let sanitized = sanitize_parse_error(parse_error);
    println!("  Global Parse Error: {}", sanitized);
    
    let build_error = Error::new(ErrorKind::Other, "Failed to generate XML");
    let sanitized = sanitize_build_error(build_error);
    println!("  Global Build Error: {}", sanitized);
    
    let security_error = Error::new(ErrorKind::PermissionDenied, "Entity validation failed");
    let sanitized = sanitize_security_error(security_error);
    println!("  Global Security Error: {}", sanitized);
}

fn demonstrate_error_contexts() {
    println!("📂 Demo 4: Different Error Contexts");
    println!("------------------------------------");
    
    let mut sanitizer = ErrorSanitizer::with_config(SanitizerConfig {
        mode: ErrorMode::Production,
        ..SanitizerConfig::default()
    });
    
    let base_error = Error::new(ErrorKind::Other, "Operation failed with sensitive details");
    
    let contexts = vec![
        ("File Operation", ErrorContext::FileOpen),
        ("XML Parsing", ErrorContext::XmlParsing),
        ("XML Building", ErrorContext::XmlBuilding),
        ("Security Check", ErrorContext::SecurityValidation),
        ("Authentication", ErrorContext::Authentication),
        ("Authorization", ErrorContext::Authorization),
    ];
    
    for (name, context) in contexts {
        // Clone the error since it's consumed by sanitize
        let error = Error::new(base_error.kind(), &base_error.to_string());
        let sanitized = sanitizer.sanitize(error, context);
        println!("  {} Context: {}", name, sanitized.message);
    }
}

fn demonstrate_path_redaction() {
    println!("🛡️  Demo 5: Path Redaction Examples");
    println!("-----------------------------------");
    
    let mut prod_sanitizer = ErrorSanitizer::with_config(SanitizerConfig {
        mode: ErrorMode::Production,
        ..SanitizerConfig::default()
    });
    
    let mut dev_sanitizer = ErrorSanitizer::with_config(SanitizerConfig {
        mode: ErrorMode::Development,
        ..SanitizerConfig::default()
    });
    
    let sensitive_paths = vec![
        "Failed to access /Users/admin/secrets.txt",
        "Cannot write to /home/user/.ssh/id_rsa",
        "Permission denied: C:\\Users\\Admin\\Documents\\passwords.xlsx",
        "Connection failed to https://api.internal.company.com/secrets",
        "Memory error at address 0x7fff5fbff000",
    ];
    
    for path_error in sensitive_paths {
        let prod_error = Error::new(ErrorKind::Other, path_error);
        let dev_error = Error::new(ErrorKind::Other, path_error);
        
        let prod_sanitized = prod_sanitizer.sanitize(prod_error, ErrorContext::FileOpen);
        let dev_sanitized = dev_sanitizer.sanitize(dev_error, ErrorContext::FileOpen);
        
        println!("  Original: {}", path_error);
        println!("  Production: {}", prod_sanitized.message);
        println!("  Development: {}", dev_sanitized.message);
        println!();
    }
}

// Helper to run this example
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_functions() {
        // Just make sure the demo functions don't panic
        demonstrate_production_mode();
        demonstrate_development_mode();
        demonstrate_global_functions();
        demonstrate_error_contexts();
        demonstrate_path_redaction();
    }
}