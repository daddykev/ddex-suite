//! API Security Features for DDEX Builder
//! 
//! This module provides comprehensive security features for API boundaries,
//! including FFI validation, WASM security headers, and batch operation protection.

use crate::error::BuildError;
use crate::security::{SecurityConfig, RateLimiter, OutputSanitizer};
use indexmap::{IndexMap, IndexSet};
use std::time::{Duration, Instant};

/// API security manager for coordinating security features
#[derive(Debug)]
pub struct ApiSecurityManager {
    rate_limiter: RateLimiter,
    output_sanitizer: OutputSanitizer,
    batch_monitor: BatchOperationMonitor,
    ffi_validator: FfiValidator,
    config: SecurityConfig,
}

impl ApiSecurityManager {
    /// Create new API security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            rate_limiter: RateLimiter::new(config.clone()),
            output_sanitizer: OutputSanitizer::new(config.clone()),
            batch_monitor: BatchOperationMonitor::new(config.clone()),
            ffi_validator: FfiValidator::new(config.clone()),
            config,
        }
    }
    
    /// Validate API request before processing
    pub fn validate_request(&mut self, operation: &str, identifier: &str, payload_size: usize) -> Result<(), BuildError> {
        // Check rate limits
        self.rate_limiter.check_rate_limit(identifier)?;
        
        // Validate payload size
        if payload_size > self.config.max_xml_size {
            return Err(BuildError::Security(
                format!("Payload too large: {} bytes", payload_size)
            ));
        }
        
        // Track batch operations
        self.batch_monitor.track_operation(identifier, operation)?;
        
        Ok(())
    }
    
    /// Sanitize API response before returning
    pub fn sanitize_response(&self, response: &str) -> Result<String, BuildError> {
        self.output_sanitizer.sanitize_xml_output(response)
    }
    
    /// Validate FFI boundary data
    pub fn validate_ffi_input(&self, data: &[u8], expected_type: FfiDataType) -> Result<(), BuildError> {
        self.ffi_validator.validate_input(data, expected_type)
    }
    
    /// Get security headers for WASM builds
    pub fn get_wasm_security_headers(&self) -> IndexMap<String, String> {
        let mut headers = IndexMap::new();
        
        // Content Security Policy
        headers.insert(
            "Content-Security-Policy".to_string(),
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string(),
        );
        
        // Prevent MIME type sniffing
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        
        // Frame options
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        
        // XSS Protection
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        
        // Referrer policy
        headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());
        
        // Permissions policy
        headers.insert(
            "Permissions-Policy".to_string(),
            "camera=(), microphone=(), location=(), interest-cohort=()".to_string(),
        );
        
        headers
    }
    
    /// Create secure error response (without internal details)
    pub fn create_secure_error_response(&self, error: &BuildError, request_id: &str) -> String {
        let sanitized_message = match error {
            BuildError::Security(_) => "Security validation failed",
            BuildError::InvalidFormat { .. } => "Invalid input format",
            BuildError::Validation(..) => "Validation error",
            BuildError::Io(_) => "I/O operation failed",
            _ => "Internal error occurred",
        };
        
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        
        format!(
            r#"{{"error": "{}", "request_id": "{}", "timestamp": "{}"}}"#,
            sanitized_message, request_id, timestamp
        )
    }
}

/// Monitor batch operations to prevent resource exhaustion
#[derive(Debug)]
pub struct BatchOperationMonitor {
    operations: IndexMap<String, Vec<OperationRecord>>,
    config: SecurityConfig,
}

#[derive(Debug, Clone)]
struct OperationRecord {
    operation: String,
    timestamp: Instant,
    resource_usage: usize,
}

impl BatchOperationMonitor {
    /// Create new batch operation monitor
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            operations: IndexMap::new(),
            config,
        }
    }
    
    /// Track a batch operation
    pub fn track_operation(&mut self, identifier: &str, operation: &str) -> Result<(), BuildError> {
        let now = Instant::now();
        let records = self.operations.entry(identifier.to_string()).or_default();
        
        // Clean up old records
        records.retain(|record| now.duration_since(record.timestamp) <= Duration::from_secs(60));
        
        // Check batch limits
        if records.len() >= self.config.max_requests_per_minute as usize {
            return Err(BuildError::Security(
                "Batch operation limit exceeded".to_string()
            ));
        }
        
        // Add new record
        records.push(OperationRecord {
            operation: operation.to_string(),
            timestamp: now,
            resource_usage: 1, // Could be made more sophisticated
        });
        
        Ok(())
    }
    
    /// Get operation statistics
    pub fn get_stats(&self, identifier: &str) -> Option<BatchStats> {
        let records = self.operations.get(identifier)?;
        let now = Instant::now();
        
        let recent_records: Vec<_> = records
            .iter()
            .filter(|r| now.duration_since(r.timestamp) <= Duration::from_secs(60))
            .collect();
        
        Some(BatchStats {
            total_operations: recent_records.len(),
            unique_operations: recent_records
                .iter()
                .map(|r| r.operation.as_str())
                .collect::<IndexSet<_>>()
                .len(),
            time_window_seconds: 60,
        })
    }
}

/// Statistics for batch operations
#[derive(Debug, Clone)]
pub struct BatchStats {
    pub total_operations: usize,
    pub unique_operations: usize,
    pub time_window_seconds: u64,
}

/// FFI boundary validator
#[derive(Debug)]
pub struct FfiValidator {
    config: SecurityConfig,
}

/// Expected data types for FFI validation
#[derive(Debug, Clone, Copy)]
pub enum FfiDataType {
    Xml,
    Json,
    Binary,
    Utf8String,
}

impl FfiValidator {
    /// Create new FFI validator
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
    
    /// Validate FFI input data
    pub fn validate_input(&self, data: &[u8], expected_type: FfiDataType) -> Result<(), BuildError> {
        // Check size limits
        if data.len() > self.config.max_xml_size {
            return Err(BuildError::Security(
                format!("FFI input too large: {} bytes", data.len())
            ));
        }
        
        // Validate data format
        match expected_type {
            FfiDataType::Utf8String => {
                std::str::from_utf8(data).map_err(|_| {
                    BuildError::Security("Invalid UTF-8 in FFI input".to_string())
                })?;
            }
            FfiDataType::Xml => {
                let xml_str = std::str::from_utf8(data).map_err(|_| {
                    BuildError::Security("Invalid UTF-8 in XML FFI input".to_string())
                })?;
                self.validate_xml_structure(xml_str)?;
            }
            FfiDataType::Json => {
                let json_str = std::str::from_utf8(data).map_err(|_| {
                    BuildError::Security("Invalid UTF-8 in JSON FFI input".to_string())
                })?;
                serde_json::from_str::<serde_json::Value>(json_str).map_err(|_| {
                    BuildError::Security("Invalid JSON in FFI input".to_string())
                })?;
            }
            FfiDataType::Binary => {
                // For binary data, just check size - no format validation needed
            }
        }
        
        Ok(())
    }
    
    /// Validate XML structure in FFI input
    fn validate_xml_structure(&self, xml: &str) -> Result<(), BuildError> {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().expand_empty_elements = false;
        
        let mut buf = Vec::new();
        let mut depth: i32 = 0;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(_)) => {
                    depth += 1;
                    if depth > 100 { // Reasonable depth limit for FFI
                        return Err(BuildError::Security(
                            "XML depth limit exceeded in FFI input".to_string()
                        ));
                    }
                }
                Ok(quick_xml::events::Event::End(_)) => {
                    depth = depth.saturating_sub(1);
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => {}
                Err(e) => {
                    return Err(BuildError::Security(
                        format!("Invalid XML structure in FFI input: {}", e)
                    ));
                }
            }
            buf.clear();
        }
        
        Ok(())
    }
}

/// API security configuration specifically for different API boundaries
#[derive(Debug, Clone)]
pub struct ApiSecurityConfig {
    /// Enable API security features
    pub enabled: bool,
    /// Maximum concurrent requests per client
    pub max_concurrent_requests: u32,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Enable detailed error messages (disable in production)
    pub detailed_errors: bool,
    /// Enable CORS headers for WASM
    pub enable_cors: bool,
    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,
}

impl Default for ApiSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_requests: 10,
            request_timeout_seconds: 30,
            detailed_errors: false, // Secure default
            enable_cors: false,
            allowed_origins: vec!["https://localhost".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_security_manager() {
        let config = SecurityConfig::default();
        let mut manager = ApiSecurityManager::new(config);
        
        // Test valid request
        assert!(manager.validate_request("parse", "user1", 1000).is_ok());
        
        // Test oversized payload (should exceed max_xml_size of 100MB)
        let result = manager.validate_request("parse", "user1", 200_000_000);
        assert!(result.is_err(), "Expected oversized payload to be rejected, but got: {:?}", result);
    }
    
    #[test]
    fn test_batch_operation_monitor() {
        let config = SecurityConfig {
            max_requests_per_minute: 3,
            ..SecurityConfig::default()
        };
        let mut monitor = BatchOperationMonitor::new(config);
        
        // First few operations should succeed
        assert!(monitor.track_operation("user1", "parse").is_ok());
        assert!(monitor.track_operation("user1", "build").is_ok());
        assert!(monitor.track_operation("user1", "validate").is_ok());
        
        // Fourth operation should fail
        assert!(monitor.track_operation("user1", "parse").is_err());
        
        // Different user should work
        assert!(monitor.track_operation("user2", "parse").is_ok());
        
        // Check stats
        let stats = monitor.get_stats("user1").unwrap();
        assert_eq!(stats.total_operations, 3);
        assert_eq!(stats.unique_operations, 3);
    }
    
    #[test]
    fn test_ffi_validator() {
        let config = SecurityConfig::default();
        let validator = FfiValidator::new(config);
        
        // Test valid UTF-8 string
        let valid_string = "Hello, world!".as_bytes();
        assert!(validator.validate_input(valid_string, FfiDataType::Utf8String).is_ok());
        
        // Test valid XML
        let valid_xml = "<root><child>content</child></root>".as_bytes();
        assert!(validator.validate_input(valid_xml, FfiDataType::Xml).is_ok());
        
        // Test valid JSON
        let valid_json = r#"{"key": "value"}"#.as_bytes();
        assert!(validator.validate_input(valid_json, FfiDataType::Json).is_ok());
        
        // Test invalid UTF-8
        let invalid_utf8 = &[0xff, 0xfe, 0xfd];
        assert!(validator.validate_input(invalid_utf8, FfiDataType::Utf8String).is_err());
        
        // Test invalid JSON
        let invalid_json = "{broken json".as_bytes();
        assert!(validator.validate_input(invalid_json, FfiDataType::Json).is_err());
    }
    
    #[test]
    fn test_wasm_security_headers() {
        let config = SecurityConfig::default();
        let manager = ApiSecurityManager::new(config);
        
        let headers = manager.get_wasm_security_headers();
        
        // Check that essential security headers are present
        assert!(headers.contains_key("Content-Security-Policy"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-XSS-Protection"));
        
        // Check header values
        assert_eq!(headers.get("X-Content-Type-Options").unwrap(), "nosniff");
        assert_eq!(headers.get("X-Frame-Options").unwrap(), "DENY");
    }
    
    #[test]
    fn test_secure_error_response() {
        let config = SecurityConfig::default();
        let manager = ApiSecurityManager::new(config);
        
        let error = BuildError::Security("Internal security details".to_string());
        let response = manager.create_secure_error_response(&error, "req-123");
        
        // Should not contain internal details
        assert!(!response.contains("Internal security details"));
        assert!(response.contains("Security validation failed"));
        assert!(response.contains("req-123"));
        assert!(response.contains("error"));
    }
}