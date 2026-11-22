//! Basic tests for the error sanitizer to ensure it works correctly

use ddex_builder::security::{
    sanitize_build_error, sanitize_io_error, sanitize_parse_error, ErrorContext, ErrorMode,
    ErrorSanitizer, SanitizerConfig,
};
use std::io::{Error, ErrorKind};

#[test]
fn test_basic_sanitizer_functionality() {
    let mut sanitizer = ErrorSanitizer::new();

    let io_error = Error::new(ErrorKind::NotFound, "File not found: /path/to/secret.txt");
    let sanitized = sanitizer.sanitize_io_error(io_error, ErrorContext::FileOpen);

    // Should have a correlation ID
    assert!(!sanitized.correlation_id.is_empty());

    // Should have a sanitized message
    assert!(!sanitized.message.is_empty());

    // Should have an error code
    assert!(sanitized.code.is_some());

    // Should have context info
    assert!(sanitized.context.is_some());
}

#[test]
fn test_production_mode_sanitization() {
    let config = SanitizerConfig {
        mode: ErrorMode::Production,
        generate_correlation_ids: true,
        log_internal_details: false, // Don't spam logs in tests
        max_message_length: 100,
        include_error_codes: true,
    };

    let mut sanitizer = ErrorSanitizer::with_config(config);

    // Test file operation error
    let file_error = Error::new(
        ErrorKind::PermissionDenied,
        "Cannot access /home/user/secrets.txt",
    );
    let sanitized = sanitizer.sanitize_io_error(file_error, ErrorContext::FileRead);

    // In production mode, should get generic file operation message
    assert_eq!(sanitized.message, "File operation failed");
    assert_eq!(sanitized.code, Some("E1002".to_string()));
}

#[test]
fn test_global_sanitizer_functions() {
    // Test the global convenience functions
    let io_error = Error::new(ErrorKind::NotFound, "File not found");
    let sanitized = sanitize_io_error(io_error, ErrorContext::FileOpen);

    assert!(!sanitized.correlation_id.is_empty());
    assert!(!sanitized.message.is_empty());

    // Test parse error
    let parse_error = Error::new(ErrorKind::InvalidData, "Invalid XML");
    let sanitized = sanitize_parse_error(parse_error);

    assert!(!sanitized.correlation_id.is_empty());
    assert!(!sanitized.message.is_empty());

    // Test build error
    let build_error = Error::new(ErrorKind::Other, "Build failed");
    let sanitized = sanitize_build_error(build_error);

    assert!(!sanitized.correlation_id.is_empty());
    assert!(!sanitized.message.is_empty());
}

#[test]
fn test_error_display_format() {
    let mut sanitizer = ErrorSanitizer::new();
    let error = Error::new(ErrorKind::Other, "Test error");

    let sanitized = sanitizer.sanitize_io_error(error, ErrorContext::FileOpen);

    // Test the Display implementation
    let display_string = format!("{}", sanitized);
    assert!(display_string.contains(&sanitized.message));
    assert!(display_string.contains("ID:"));

    if let Some(code) = &sanitized.code {
        assert!(display_string.contains(code));
    }
}
