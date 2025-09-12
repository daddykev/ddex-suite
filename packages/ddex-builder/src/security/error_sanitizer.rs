//! Error message sanitization system for preventing information disclosure
//!
//! This module provides a comprehensive error sanitization system that ensures
//! sensitive information is not leaked through error messages while maintaining
//! useful debugging capabilities for developers.

use std::collections::HashMap;
use std::fmt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{error, warn, debug};
use uuid::Uuid;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Operating mode for error sanitization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorMode {
    /// Production mode - maximum sanitization, minimal information disclosure
    Production,
    /// Development mode - balanced sanitization, more details for debugging
    Development,
    /// Testing mode - minimal sanitization, full details for test validation
    Testing,
}

/// Error classification levels for secure error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLevel {
    /// Safe for external users - no sensitive information
    Public,
    /// For internal logging only - may contain sensitive details
    Internal,
    /// Development only - full details, stripped in release builds
    Debug,
}

/// Context information for error classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorContext {
    FileOpen,
    FileRead,
    FileWrite,
    NetworkRequest,
    XmlParsing,
    XmlBuilding,
    SecurityValidation,
    EntityClassification,
    PathValidation,
    MemoryAllocation,
    DatabaseConnection,
    Authentication,
    Authorization,
}

/// Trait for secure error handling with multiple disclosure levels
pub trait SecureError: fmt::Display + fmt::Debug {
    /// Get the public-safe error message
    fn public_message(&self) -> String;
    
    /// Get the internal error message for logging
    fn internal_message(&self) -> String;
    
    /// Get the debug error message (development only)
    fn debug_message(&self) -> String;
    
    /// Get the error classification level
    fn error_level(&self) -> ErrorLevel;
    
    /// Get the error context
    fn error_context(&self) -> ErrorContext;
    
    /// Generate a unique error ID for correlation
    fn error_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

/// Rule for redacting sensitive information from error messages
#[derive(Debug, Clone)]
pub struct RedactionRule {
    /// Name of the rule for identification
    pub name: String,
    /// Regex pattern to match sensitive data
    pub pattern: Regex,
    /// Replacement text (may include capture groups)
    pub replacement: String,
    /// Whether this rule applies in production mode
    pub production: bool,
    /// Whether this rule applies in development mode
    pub development: bool,
    /// Whether this rule applies in testing mode
    pub testing: bool,
}

impl RedactionRule {
    /// Create a new redaction rule
    pub fn new(
        name: &str,
        pattern: &str,
        replacement: &str,
        production: bool,
        development: bool,
        testing: bool,
    ) -> Result<Self, regex::Error> {
        Ok(RedactionRule {
            name: name.to_string(),
            pattern: Regex::new(pattern)?,
            replacement: replacement.to_string(),
            production,
            development,
            testing,
        })
    }
    
    /// Check if this rule applies in the given mode
    pub fn applies_to_mode(&self, mode: ErrorMode) -> bool {
        match mode {
            ErrorMode::Production => self.production,
            ErrorMode::Development => self.development,
            ErrorMode::Testing => self.testing,
        }
    }
    
    /// Apply this rule to a message
    pub fn apply(&self, message: &str) -> String {
        self.pattern.replace_all(message, self.replacement.as_str()).to_string()
    }
}

/// Configuration for error sanitization behavior
#[derive(Debug, Clone)]
pub struct SanitizerConfig {
    /// Operating mode
    pub mode: ErrorMode,
    /// Whether to generate correlation IDs
    pub generate_correlation_ids: bool,
    /// Whether to log internal details
    pub log_internal_details: bool,
    /// Maximum error message length
    pub max_message_length: usize,
    /// Whether to include error codes
    pub include_error_codes: bool,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        SanitizerConfig {
            mode: if cfg!(debug_assertions) {
                ErrorMode::Development
            } else {
                ErrorMode::Production
            },
            generate_correlation_ids: true,
            log_internal_details: true,
            max_message_length: 256,
            include_error_codes: true,
        }
    }
}

/// Main error sanitization engine
pub struct ErrorSanitizer {
    config: SanitizerConfig,
    redaction_rules: Vec<RedactionRule>,
    error_code_map: HashMap<ErrorContext, &'static str>,
    correlation_store: HashMap<String, String>,
}

/// Sanitized error result with correlation ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedError {
    /// Correlation ID for internal tracking
    pub correlation_id: String,
    /// Public-safe error message
    pub message: String,
    /// Error code for programmatic handling
    pub code: Option<String>,
    /// Additional context that's safe to expose
    pub context: Option<String>,
}

impl fmt::Display for SanitizedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "[{}] {}", code, self.message)?;
        } else {
            write!(f, "{}", self.message)?;
        }
        
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        
        write!(f, " [ID: {}]", &self.correlation_id[0..8])
    }
}

/// Pre-defined redaction rules for common sensitive data patterns
static DEFAULT_REDACTION_RULES: Lazy<Vec<RedactionRule>> = Lazy::new(|| {
    let mut rules = Vec::new();
    
    // File system paths - most aggressive in production
    if let Ok(rule) = RedactionRule::new(
        "filesystem_paths",
        r"(/[^/\s]+)+(/[^/\s]*\.[^/\s]+)?|([A-Z]:\\[^\\]+\\[^\\]*)",
        "<file path>",
        true,  // production
        false, // development
        false, // testing
    ) {
        rules.push(rule);
    }
    
    // Development-friendly path redaction (keep filename)
    if let Ok(rule) = RedactionRule::new(
        "filesystem_paths_dev",
        r"(/[^/\s]+)+/([^/\s]*\.[^/\s]+)|([A-Z]:\\[^\\]+\\[^\\]*)\\([^\\]*)",
        "<path>/$2$4",
        false, // production
        true,  // development
        false, // testing
    ) {
        rules.push(rule);
    }
    
    // IP addresses
    if let Ok(rule) = RedactionRule::new(
        "ip_addresses",
        r"\b(?:\d{1,3}\.){3}\d{1,3}\b|\b[0-9a-fA-F]{1,4}(?::[0-9a-fA-F]{1,4}){7}\b",
        "<ip address>",
        true,  // production
        true,  // development
        false, // testing
    ) {
        rules.push(rule);
    }
    
    // Hostnames and URLs
    if let Ok(rule) = RedactionRule::new(
        "hostnames",
        r"https?://[^\s/$.?#].[^\s]*|[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:[/\s]|$)",
        "<hostname>",
        true,  // production
        true,  // development
        false, // testing
    ) {
        rules.push(rule);
    }
    
    // Memory addresses
    if let Ok(rule) = RedactionRule::new(
        "memory_addresses",
        r"0x[0-9a-fA-F]+|[0-9a-fA-F]{8,16}",
        "<memory address>",
        true,  // production
        true,  // development
        false, // testing
    ) {
        rules.push(rule);
    }
    
    // Stack traces (line numbers and function names)
    if let Ok(rule) = RedactionRule::new(
        "stack_traces",
        r"at [^:]+:\d+:\d+|in `[^`]+`",
        "<stack trace>",
        true,  // production
        false, // development
        false, // testing
    ) {
        rules.push(rule);
    }
    
    // API keys and tokens (basic patterns)
    if let Ok(rule) = RedactionRule::new(
        "api_keys",
        r"(?i)(api_?key|token|secret|password|auth)[\s]*[:=][\s]*[a-zA-Z0-9\-_]{16,}",
        "$1=<redacted>",
        true, // production
        true, // development
        true, // testing (even in testing, don't leak real keys)
    ) {
        rules.push(rule);
    }
    
    // User-specific paths (home directories)
    if let Ok(rule) = RedactionRule::new(
        "user_paths",
        r"/Users/[^/\s]+|/home/[^/\s]+|C:\\Users\\[^\\\\]+",
        "<user directory>",
        true,  // production
        true,  // development
        false, // testing
    ) {
        rules.push(rule);
    }
    
    // Database connection strings
    if let Ok(rule) = RedactionRule::new(
        "db_connections",
        r"(?i)(mysql|postgres|mongodb)://[^@\s]+@[^/\s]+/[^\s]*",
        "$1://<connection>",
        true, // production
        true, // development
        true, // testing
    ) {
        rules.push(rule);
    }
    
    rules
});

impl ErrorSanitizer {
    /// Create a new error sanitizer with default configuration
    pub fn new() -> Self {
        Self::with_config(SanitizerConfig::default())
    }
    
    /// Create a new error sanitizer with custom configuration
    pub fn with_config(config: SanitizerConfig) -> Self {
        let error_code_map = Self::create_error_code_map();
        
        ErrorSanitizer {
            config,
            redaction_rules: DEFAULT_REDACTION_RULES.clone(),
            error_code_map,
            correlation_store: HashMap::new(),
        }
    }
    
    /// Add a custom redaction rule
    pub fn add_redaction_rule(&mut self, rule: RedactionRule) {
        self.redaction_rules.push(rule);
    }
    
    /// Sanitize an error message based on context and mode
    pub fn sanitize<E>(&mut self, error: E, context: ErrorContext) -> SanitizedError 
    where
        E: std::error::Error + fmt::Display + fmt::Debug,
    {
        let correlation_id = if self.config.generate_correlation_ids {
            Uuid::new_v4().to_string()
        } else {
            "none".to_string()
        };
        
        // Get the raw error message
        let raw_message = error.to_string();
        let debug_message = format!("{:?}", error);
        
        // Log full details internally if enabled
        if self.config.log_internal_details {
            error!(
                correlation_id = %correlation_id,
                context = ?context,
                raw_message = %raw_message,
                debug_info = %debug_message,
                "Internal error details"
            );
            
            // Store full details for potential debugging
            if self.config.generate_correlation_ids {
                self.correlation_store.insert(
                    correlation_id.clone(),
                    format!("Context: {:?}, Error: {}, Debug: {}", context, raw_message, debug_message)
                );
            }
        }
        
        // Apply sanitization based on mode and context
        let sanitized_message = self.apply_sanitization(&raw_message, context);
        
        // Truncate if too long
        let final_message = if sanitized_message.len() > self.config.max_message_length {
            format!("{}...", &sanitized_message[0..self.config.max_message_length.saturating_sub(3)])
        } else {
            sanitized_message
        };
        
        // Get error code
        let error_code = if self.config.include_error_codes {
            self.error_code_map.get(&context).map(|&s| s.to_string())
        } else {
            None
        };
        
        SanitizedError {
            correlation_id,
            message: final_message,
            code: error_code,
            context: Some(self.get_safe_context_description(context)),
        }
    }
    
    /// Apply sanitization rules to a message
    fn apply_sanitization(&self, message: &str, context: ErrorContext) -> String {
        let mut sanitized = message.to_string();
        
        // Apply context-specific sanitization first
        sanitized = self.apply_context_specific_sanitization(sanitized, context);
        
        // Apply general redaction rules
        for rule in &self.redaction_rules {
            if rule.applies_to_mode(self.config.mode) {
                sanitized = rule.apply(&sanitized);
            }
        }
        
        sanitized
    }
    
    /// Apply context-specific sanitization logic
    fn apply_context_specific_sanitization(&self, message: String, context: ErrorContext) -> String {
        match (context, self.config.mode) {
            (ErrorContext::FileOpen | ErrorContext::FileRead | ErrorContext::FileWrite, ErrorMode::Production) => {
                "File operation failed".to_string()
            },
            (ErrorContext::FileOpen | ErrorContext::FileRead | ErrorContext::FileWrite, ErrorMode::Development) => {
                // Keep operation type but redact full paths
                let operation = match context {
                    ErrorContext::FileOpen => "open",
                    ErrorContext::FileRead => "read",
                    ErrorContext::FileWrite => "write",
                    _ => "access",
                };
                format!("Failed to {} file", operation)
            },
            (ErrorContext::NetworkRequest, ErrorMode::Production) => {
                "Network operation failed".to_string()
            },
            (ErrorContext::XmlParsing, ErrorMode::Production) => {
                "Invalid XML structure".to_string()
            },
            (ErrorContext::XmlBuilding, ErrorMode::Production) => {
                "XML generation failed".to_string()
            },
            (ErrorContext::SecurityValidation, ErrorMode::Production) => {
                "Security validation failed".to_string()
            },
            (ErrorContext::EntityClassification, ErrorMode::Production) => {
                "Entity validation failed".to_string()
            },
            (ErrorContext::PathValidation, ErrorMode::Production) => {
                "Path validation failed".to_string()
            },
            (ErrorContext::MemoryAllocation, ErrorMode::Production) => {
                "Memory allocation failed".to_string()
            },
            (ErrorContext::DatabaseConnection, ErrorMode::Production) => {
                "Database connection failed".to_string()
            },
            (ErrorContext::Authentication, ErrorMode::Production) => {
                "Authentication failed".to_string()
            },
            (ErrorContext::Authorization, ErrorMode::Production) => {
                "Access denied".to_string()
            },
            // In development and testing modes, allow more detail
            _ => message,
        }
    }
    
    /// Create error code mapping
    fn create_error_code_map() -> HashMap<ErrorContext, &'static str> {
        let mut map = HashMap::new();
        map.insert(ErrorContext::FileOpen, "E1001");
        map.insert(ErrorContext::FileRead, "E1002");
        map.insert(ErrorContext::FileWrite, "E1003");
        map.insert(ErrorContext::NetworkRequest, "E2001");
        map.insert(ErrorContext::XmlParsing, "E3001");
        map.insert(ErrorContext::XmlBuilding, "E3002");
        map.insert(ErrorContext::SecurityValidation, "E4001");
        map.insert(ErrorContext::EntityClassification, "E4002");
        map.insert(ErrorContext::PathValidation, "E4003");
        map.insert(ErrorContext::MemoryAllocation, "E5001");
        map.insert(ErrorContext::DatabaseConnection, "E6001");
        map.insert(ErrorContext::Authentication, "E7001");
        map.insert(ErrorContext::Authorization, "E7002");
        map
    }
    
    /// Get a safe description of the error context
    fn get_safe_context_description(&self, context: ErrorContext) -> String {
        match context {
            ErrorContext::FileOpen => "file access".to_string(),
            ErrorContext::FileRead => "file reading".to_string(),
            ErrorContext::FileWrite => "file writing".to_string(),
            ErrorContext::NetworkRequest => "network operation".to_string(),
            ErrorContext::XmlParsing => "XML parsing".to_string(),
            ErrorContext::XmlBuilding => "XML generation".to_string(),
            ErrorContext::SecurityValidation => "security check".to_string(),
            ErrorContext::EntityClassification => "entity validation".to_string(),
            ErrorContext::PathValidation => "path validation".to_string(),
            ErrorContext::MemoryAllocation => "memory management".to_string(),
            ErrorContext::DatabaseConnection => "database access".to_string(),
            ErrorContext::Authentication => "authentication".to_string(),
            ErrorContext::Authorization => "authorization".to_string(),
        }
    }
    
    /// Retrieve stored error details by correlation ID (for debugging)
    pub fn get_error_details(&self, correlation_id: &str) -> Option<&String> {
        self.correlation_store.get(correlation_id)
    }
    
    /// Clear stored error details (for memory management)
    pub fn clear_error_store(&mut self) {
        self.correlation_store.clear();
    }
    
    /// Get statistics about sanitization
    pub fn get_statistics(&self) -> SanitizerStatistics {
        SanitizerStatistics {
            mode: self.config.mode,
            active_rules: self.redaction_rules.iter()
                .filter(|r| r.applies_to_mode(self.config.mode))
                .count(),
            stored_errors: self.correlation_store.len(),
        }
    }
}

/// Statistics about the error sanitizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizerStatistics {
    pub mode: ErrorMode,
    pub active_rules: usize,
    pub stored_errors: usize,
}

impl Default for ErrorSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common error types
impl ErrorSanitizer {
    /// Sanitize an I/O error
    pub fn sanitize_io_error<E>(&mut self, error: E, context: ErrorContext) -> SanitizedError
    where
        E: std::error::Error + fmt::Display + fmt::Debug,
    {
        self.sanitize(error, context)
    }
    
    /// Sanitize a parsing error
    pub fn sanitize_parse_error<E>(&mut self, error: E) -> SanitizedError
    where
        E: std::error::Error + fmt::Display + fmt::Debug,
    {
        self.sanitize(error, ErrorContext::XmlParsing)
    }
    
    /// Sanitize a build error
    pub fn sanitize_build_error<E>(&mut self, error: E) -> SanitizedError
    where
        E: std::error::Error + fmt::Display + fmt::Debug,
    {
        self.sanitize(error, ErrorContext::XmlBuilding)
    }
    
    /// Sanitize a security error
    pub fn sanitize_security_error<E>(&mut self, error: E) -> SanitizedError
    where
        E: std::error::Error + fmt::Display + fmt::Debug,
    {
        self.sanitize(error, ErrorContext::SecurityValidation)
    }
}

/// Global error sanitizer instance - thread-safe and no unsafe code required
static GLOBAL_SANITIZER: Lazy<Mutex<ErrorSanitizer>> = Lazy::new(|| {
    Mutex::new(ErrorSanitizer::with_config(SanitizerConfig::default()))
});

/// Initialize the global error sanitizer with custom configuration
pub fn init_global_sanitizer(config: SanitizerConfig) {
    // Replace the default sanitizer with one using the provided config
    *GLOBAL_SANITIZER.lock().unwrap() = ErrorSanitizer::with_config(config);
}

/// Get access to the global error sanitizer
pub fn with_global_sanitizer<F, R>(f: F) -> R
where
    F: FnOnce(&mut ErrorSanitizer) -> R,
{
    let mut sanitizer = GLOBAL_SANITIZER.lock().unwrap();
    f(&mut *sanitizer)
}

/// Quick sanitization functions using global sanitizer
pub fn sanitize_error<E>(error: E, context: ErrorContext) -> SanitizedError
where
    E: std::error::Error + fmt::Display + fmt::Debug,
{
    with_global_sanitizer(|sanitizer| sanitizer.sanitize(error, context))
}

pub fn sanitize_io_error<E>(error: E, context: ErrorContext) -> SanitizedError
where
    E: std::error::Error + fmt::Display + fmt::Debug,
{
    with_global_sanitizer(|sanitizer| sanitizer.sanitize_io_error(error, context))
}

pub fn sanitize_parse_error<E>(error: E) -> SanitizedError
where
    E: std::error::Error + fmt::Display + fmt::Debug,
{
    with_global_sanitizer(|sanitizer| sanitizer.sanitize_parse_error(error))
}

pub fn sanitize_build_error<E>(error: E) -> SanitizedError
where
    E: std::error::Error + fmt::Display + fmt::Debug,
{
    with_global_sanitizer(|sanitizer| sanitizer.sanitize_build_error(error))
}

pub fn sanitize_security_error<E>(error: E) -> SanitizedError
where
    E: std::error::Error + fmt::Display + fmt::Debug,
{
    with_global_sanitizer(|sanitizer| sanitizer.sanitize_security_error(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn test_secure_error_trait() {
        struct TestError {
            message: String,
            context: ErrorContext,
        }

        impl fmt::Display for TestError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.message)
            }
        }

        impl fmt::Debug for TestError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "TestError {{ message: {:?}, context: {:?} }}", self.message, self.context)
            }
        }

        impl std::error::Error for TestError {}

        impl SecureError for TestError {
            fn public_message(&self) -> String {
                "Operation failed".to_string()
            }

            fn internal_message(&self) -> String {
                self.message.clone()
            }

            fn debug_message(&self) -> String {
                format!("{:?}", self)
            }

            fn error_level(&self) -> ErrorLevel {
                ErrorLevel::Internal
            }

            fn error_context(&self) -> ErrorContext {
                self.context
            }
        }

        let error = TestError {
            message: "Detailed error with /path/to/file.txt".to_string(),
            context: ErrorContext::FileRead,
        };

        assert_eq!(error.public_message(), "Operation failed");
        assert!(error.internal_message().contains("/path/to/file.txt"));
        assert_eq!(error.error_level(), ErrorLevel::Internal);
        assert_eq!(error.error_context(), ErrorContext::FileRead);
    }

    #[test]
    fn test_redaction_rules() {
        let rule = RedactionRule::new(
            "test_paths",
            r"/[^/\s]+/[^/\s]+",
            "<redacted path>",
            true, true, false
        ).unwrap();

        let message = "Failed to open /home/user/secret.txt";
        let redacted = rule.apply(message);
        assert_eq!(redacted, "Failed to open <redacted path>/secret.txt");

        assert!(rule.applies_to_mode(ErrorMode::Production));
        assert!(rule.applies_to_mode(ErrorMode::Development));
        assert!(!rule.applies_to_mode(ErrorMode::Testing));
    }

    #[test]
    fn test_error_sanitizer_production_mode() {
        let config = SanitizerConfig {
            mode: ErrorMode::Production,
            generate_correlation_ids: true,
            log_internal_details: false, // Don't spam logs in tests
            max_message_length: 100,
            include_error_codes: true,
        };

        let mut sanitizer = ErrorSanitizer::with_config(config);
        let io_error = Error::new(ErrorKind::NotFound, "File not found: /home/user/secrets.txt");
        
        let sanitized = sanitizer.sanitize_io_error(io_error, ErrorContext::FileOpen);
        
        assert_eq!(sanitized.message, "File operation failed");
        assert_eq!(sanitized.code, Some("E1001".to_string()));
        assert!(sanitized.context.is_some());
        assert!(!sanitized.correlation_id.is_empty());
    }

    #[test]
    fn test_error_sanitizer_development_mode() {
        let config = SanitizerConfig {
            mode: ErrorMode::Development,
            generate_correlation_ids: true,
            log_internal_details: false,
            max_message_length: 200,
            include_error_codes: true,
        };

        let mut sanitizer = ErrorSanitizer::with_config(config);
        let io_error = Error::new(ErrorKind::PermissionDenied, "Permission denied: /etc/shadow");
        
        let sanitized = sanitizer.sanitize_io_error(io_error, ErrorContext::FileRead);
        
        // Should be more descriptive in development mode
        assert!(sanitized.message.contains("file"));
        assert_eq!(sanitized.code, Some("E1002".to_string()));
        assert!(sanitized.context.is_some());
    }

    #[test]
    fn test_path_redaction() {
        let mut sanitizer = ErrorSanitizer::with_config(SanitizerConfig {
            mode: ErrorMode::Production,
            ..SanitizerConfig::default()
        });

        let error = Error::new(ErrorKind::NotFound, "Cannot find /Users/john/Documents/secret.pdf");
        let sanitized = sanitizer.sanitize_io_error(error, ErrorContext::FileOpen);
        
        // In production mode, should get generic message
        assert_eq!(sanitized.message, "File operation failed");
    }

    #[test]
    fn test_ip_address_redaction() {
        let rule = RedactionRule::new(
            "test_ips",
            r"\b(?:\d{1,3}\.){3}\d{1,3}\b",
            "<ip>",
            true, true, true
        ).unwrap();

        let message = "Connection failed to 192.168.1.1:8080";
        let redacted = rule.apply(message);
        assert_eq!(redacted, "Connection failed to <ip>:8080");
    }

    #[test]
    fn test_memory_address_redaction() {
        let rule = RedactionRule::new(
            "test_memory",
            r"0x[0-9a-fA-F]+",
            "<addr>",
            true, true, false
        ).unwrap();

        let message = "Segfault at address 0x7fff5fbff000";
        let redacted = rule.apply(message);
        assert_eq!(redacted, "Segfault at address <addr>");
    }

    #[test]
    fn test_api_key_redaction() {
        let rule = RedactionRule::new(
            "test_keys",
            r"(?i)(api_?key|token)[\s]*[:=][\s]*[a-zA-Z0-9\-_]{16,}",
            "$1=<redacted>",
            true, true, true
        ).unwrap();

        let message = r#"Authentication failed: api_key="sk_test_123456789abcdefghij""#;
        let redacted = rule.apply(message);
        assert!(redacted.contains("api_key=<redacted>"));
        assert!(!redacted.contains("sk_test_123456789abcdefghij"));
    }

    #[test]
    fn test_context_specific_sanitization() {
        let mut sanitizer = ErrorSanitizer::with_config(SanitizerConfig {
            mode: ErrorMode::Production,
            ..SanitizerConfig::default()
        });

        // Test different contexts
        let contexts = vec![
            (ErrorContext::XmlParsing, "Invalid XML structure"),
            (ErrorContext::XmlBuilding, "XML generation failed"),
            (ErrorContext::SecurityValidation, "Security validation failed"),
            (ErrorContext::Authentication, "Authentication failed"),
            (ErrorContext::Authorization, "Access denied"),
        ];

        for (context, expected) in contexts {
            let error = Error::new(ErrorKind::InvalidInput, "Detailed error message with /path/to/file.txt");
            let sanitized = sanitizer.sanitize_io_error(error, context);
            assert_eq!(sanitized.message, expected);
        }
    }

    #[test]
    fn test_message_length_truncation() {
        let config = SanitizerConfig {
            mode: ErrorMode::Testing, // Allow full message to test truncation
            max_message_length: 20,
            ..SanitizerConfig::default()
        };

        let mut sanitizer = ErrorSanitizer::with_config(config);
        let long_error = Error::new(ErrorKind::Other, "This is a very long error message that should be truncated.");
        
        let sanitized = sanitizer.sanitize_io_error(long_error, ErrorContext::FileRead);
        assert!(sanitized.message.len() <= 20);
        assert!(sanitized.message.ends_with("..."));
    }

    #[test]
    fn test_correlation_id_generation() {
        let mut sanitizer = ErrorSanitizer::with_config(SanitizerConfig {
            generate_correlation_ids: true,
            ..SanitizerConfig::default()
        });

        let error1 = Error::new(ErrorKind::NotFound, "Error 1");
        let error2 = Error::new(ErrorKind::NotFound, "Error 2");

        let sanitized1 = sanitizer.sanitize_io_error(error1, ErrorContext::FileOpen);
        let sanitized2 = sanitizer.sanitize_io_error(error2, ErrorContext::FileOpen);

        assert_ne!(sanitized1.correlation_id, sanitized2.correlation_id);
        assert!(!sanitized1.correlation_id.is_empty());
        assert!(!sanitized2.correlation_id.is_empty());
    }

    #[test]
    fn test_error_codes() {
        let sanitizer = ErrorSanitizer::new();
        let stats = sanitizer.get_statistics();
        
        assert_eq!(stats.mode, if cfg!(debug_assertions) { ErrorMode::Development } else { ErrorMode::Production });
        assert!(stats.active_rules > 0);
        assert_eq!(stats.stored_errors, 0);
    }

    #[test]
    fn test_global_sanitizer() {
        let error = Error::new(ErrorKind::PermissionDenied, "Access denied to /secret/file.txt");
        let sanitized = sanitize_io_error(error, ErrorContext::FileRead);
        
        assert!(!sanitized.correlation_id.is_empty());
        assert!(!sanitized.message.is_empty());
        assert!(sanitized.code.is_some());
    }
}