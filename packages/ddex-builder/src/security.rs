//! Security module for DDEX Builder
//! 
//! This module provides comprehensive security measures including:
//! - XXE (XML External Entity) attack prevention
//! - Input validation and sanitization
//! - Path traversal prevention
//! - Size limits and rate limiting
//! - Safe XML parsing configuration

use crate::error::BuildError;
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use url::Url;
use once_cell::sync::Lazy;

/// Maximum allowed size for XML input (100MB)
const MAX_XML_SIZE: usize = 100 * 1024 * 1024;

/// Maximum allowed size for JSON input (50MB)
const MAX_JSON_SIZE: usize = 50 * 1024 * 1024;

/// Maximum allowed size for any string field (1MB)
const MAX_STRING_SIZE: usize = 1024 * 1024;

/// Maximum nesting depth for XML elements
const MAX_XML_DEPTH: usize = 100;

/// Maximum number of XML attributes per element
const MAX_ATTRIBUTES_PER_ELEMENT: usize = 100;

/// Maximum number of child elements
const MAX_CHILD_ELEMENTS: usize = 10000;

/// Rate limiting configuration
const MAX_REQUESTS_PER_MINUTE: u32 = 100;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Dangerous XML entity patterns
static DANGEROUS_ENTITY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<!ENTITY\s+[^>]*>|&[a-zA-Z_][a-zA-Z0-9._-]*;").unwrap()
});

/// External reference patterns
static EXTERNAL_REF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(SYSTEM|PUBLIC)\s+['"][^'"]*['"]"#).unwrap()
});

/// Potentially dangerous file path patterns
static DANGEROUS_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.\./|\\\.\\\|/etc/|/proc/|/sys/|/dev/|/tmp/|C:\\|\\\\").unwrap()
});

/// SQL injection patterns
static SQL_INJECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(union|select|insert|update|delete|drop|exec|script|javascript|vbscript|onload|onerror)").unwrap()
});

/// XML bomb pattern detection
static XML_BOMB_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<!ENTITY\s+\w+\s+['"](&\w+;)+['"]"#).unwrap()
});

/// Security configuration for XML parsing
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum XML input size
    pub max_xml_size: usize,
    /// Maximum JSON input size  
    pub max_json_size: usize,
    /// Maximum string field size
    pub max_string_size: usize,
    /// Maximum XML nesting depth
    pub max_xml_depth: usize,
    /// Maximum attributes per element
    pub max_attributes_per_element: usize,
    /// Maximum child elements
    pub max_child_elements: usize,
    /// Whether to allow external entities
    pub allow_external_entities: bool,
    /// Whether to allow DTD processing
    pub allow_dtd: bool,
    /// Rate limiting enabled
    pub rate_limiting_enabled: bool,
    /// Maximum requests per minute
    pub max_requests_per_minute: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_xml_size: MAX_XML_SIZE,
            max_json_size: MAX_JSON_SIZE,
            max_string_size: MAX_STRING_SIZE,
            max_xml_depth: MAX_XML_DEPTH,
            max_attributes_per_element: MAX_ATTRIBUTES_PER_ELEMENT,
            max_child_elements: MAX_CHILD_ELEMENTS,
            allow_external_entities: false, // CRITICAL: Never allow external entities
            allow_dtd: false, // CRITICAL: Never allow DTD processing
            rate_limiting_enabled: true,
            max_requests_per_minute: MAX_REQUESTS_PER_MINUTE,
        }
    }
}

/// Secure XML reader with XXE protection
pub struct SecureXmlReader<R: BufRead> {
    reader: Reader<R>,
    config: SecurityConfig,
    current_depth: usize,
    element_count: usize,
    start_time: Instant,
}

impl<R: BufRead> SecureXmlReader<R> {
    /// Create a new secure XML reader
    pub fn new(reader: R, config: SecurityConfig) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        
        // Configure reader for security
        xml_reader.config_mut().check_comments = false;
        xml_reader.config_mut().check_end_names = true;
        xml_reader.config_mut().trim_text_start = true;
        xml_reader.config_mut().trim_text_end = true;
        xml_reader.config_mut().expand_empty_elements = false;
        
        Self {
            reader: xml_reader,
            config,
            current_depth: 0,
            element_count: 0,
            start_time: Instant::now(),
        }
    }
    
    /// Read the next event with security checks
    pub fn read_event<'a>(&mut self, buf: &'a mut Vec<u8>) -> Result<Event<'a>, BuildError> {
        // Check for timeout to prevent DoS
        if self.start_time.elapsed() > Duration::from_secs(30) {
            return Err(BuildError::Security("XML processing timeout".to_string()));
        }
        
        let event = self.reader.read_event_into(buf)
            .map_err(|e| BuildError::Security(format!("XML parsing error: {}", e)))?;
        
        match &event {
            Event::Start(_) => {
                self.current_depth += 1;
                self.element_count += 1;
                
                // Check depth limit
                if self.current_depth > self.config.max_xml_depth {
                    return Err(BuildError::Security(
                        format!("XML nesting too deep: {} > {}", 
                               self.current_depth, 
                               self.config.max_xml_depth)
                    ));
                }
                
                // Check element count limit
                if self.element_count > self.config.max_child_elements {
                    return Err(BuildError::Security(
                        format!("Too many XML elements: {} > {}", 
                               self.element_count, 
                               self.config.max_child_elements)
                    ));
                }
            }
            Event::End(_) => {
                self.current_depth = self.current_depth.saturating_sub(1);
            }
            Event::DocType(dt) => {
                if !self.config.allow_dtd {
                    return Err(BuildError::Security("DTD processing not allowed".to_string()));
                }
                
                // Check for dangerous DTD content
                let dtd_str = String::from_utf8_lossy(dt.as_ref());
                if DANGEROUS_ENTITY_REGEX.is_match(&dtd_str) {
                    return Err(BuildError::Security("Dangerous entity detected in DTD".to_string()));
                }
                
                if EXTERNAL_REF_REGEX.is_match(&dtd_str) {
                    return Err(BuildError::Security("External reference detected in DTD".to_string()));
                }
                
                if XML_BOMB_REGEX.is_match(&dtd_str) {
                    return Err(BuildError::Security("Potential XML bomb detected".to_string()));
                }
            }
            _ => {}
        }
        
        Ok(event)
    }
    
    /// Get the underlying reader
    pub fn into_inner(self) -> Reader<R> {
        self.reader
    }
}

/// Input validator for various data types
pub struct InputValidator {
    config: SecurityConfig,
}

impl InputValidator {
    /// Create a new input validator
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
    
    /// Validate and sanitize a string input
    pub fn validate_string(&self, input: &str, field_name: &str) -> Result<String, BuildError> {
        // Check size limit
        if input.len() > self.config.max_string_size {
            return Err(BuildError::InputSanitization(
                format!("String too long for field '{}': {} > {}", 
                       field_name, 
                       input.len(), 
                       self.config.max_string_size)
            ));
        }
        
        // Check for null bytes
        if input.contains('\0') {
            return Err(BuildError::InputSanitization(
                format!("Null byte detected in field '{}'", field_name)
            ));
        }
        
        // Check for potential injection attacks
        if SQL_INJECTION_REGEX.is_match(input) {
            return Err(BuildError::InputSanitization(
                format!("Potential injection attack detected in field '{}'", field_name)
            ));
        }
        
        // Check for dangerous entity references
        if DANGEROUS_ENTITY_REGEX.is_match(input) {
            return Err(BuildError::InputSanitization(
                format!("Dangerous entity reference detected in field '{}'", field_name)
            ));
        }
        
        // Normalize whitespace and control characters
        let sanitized = input
            .chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
            .collect::<String>()
            .trim()
            .to_string();
        
        Ok(sanitized)
    }
    
    /// Validate a file path for safety
    pub fn validate_path(&self, path: &str) -> Result<PathBuf, BuildError> {
        // Check for dangerous path patterns
        if DANGEROUS_PATH_REGEX.is_match(path) {
            return Err(BuildError::InputSanitization(
                format!("Dangerous path pattern detected: {}", path)
            ));
        }
        
        // Convert to PathBuf and canonicalize
        let path_buf = PathBuf::from(path);
        
        // Reject absolute paths
        if path_buf.is_absolute() {
            return Err(BuildError::InputSanitization(
                "Absolute paths not allowed".to_string()
            ));
        }
        
        // Check for path traversal
        for component in path_buf.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(BuildError::InputSanitization(
                        "Path traversal attempt detected".to_string()
                    ));
                }
                std::path::Component::RootDir => {
                    return Err(BuildError::InputSanitization(
                        "Root directory access not allowed".to_string()
                    ));
                }
                _ => {}
            }
        }
        
        Ok(path_buf)
    }
    
    /// Validate a URL for safety
    pub fn validate_url(&self, url_str: &str) -> Result<Url, BuildError> {
        // Parse URL
        let url = Url::parse(url_str)
            .map_err(|e| BuildError::InputSanitization(format!("Invalid URL: {}", e)))?;
        
        // Only allow safe schemes
        match url.scheme() {
            "http" | "https" => {}
            _ => {
                return Err(BuildError::InputSanitization(
                    format!("Unsafe URL scheme: {}", url.scheme())
                ));
            }
        }
        
        // Reject localhost and private IPs
        if let Some(host_str) = url.host_str() {
            if host_str == "localhost" 
                || host_str == "127.0.0.1" 
                || host_str == "::1"
                || host_str.starts_with("192.168.")
                || host_str.starts_with("10.")
                || host_str.starts_with("172.") {
                
                return Err(BuildError::InputSanitization(
                    "Private or local URLs not allowed".to_string()
                ));
            }
        }
        
        Ok(url)
    }
    
    /// Validate XML content for security
    pub fn validate_xml_content(&self, xml: &str) -> Result<(), BuildError> {
        // Check size
        if xml.len() > self.config.max_xml_size {
            return Err(BuildError::InputSanitization(
                format!("XML too large: {} > {}", xml.len(), self.config.max_xml_size)
            ));
        }
        
        // Check for XXE patterns
        if DANGEROUS_ENTITY_REGEX.is_match(xml) {
            return Err(BuildError::Security("Potential XXE attack detected".to_string()));
        }
        
        if EXTERNAL_REF_REGEX.is_match(xml) {
            return Err(BuildError::Security("External reference detected".to_string()));
        }
        
        if XML_BOMB_REGEX.is_match(xml) {
            return Err(BuildError::Security("Potential XML bomb detected".to_string()));
        }
        
        // Check for excessive entity expansion
        let entity_count = xml.matches("&").count();
        if entity_count > 1000 {
            return Err(BuildError::Security("Excessive entity usage detected".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate JSON content for security
    pub fn validate_json_content(&self, json: &str) -> Result<(), BuildError> {
        // Check size
        if json.len() > self.config.max_json_size {
            return Err(BuildError::InputSanitization(
                format!("JSON too large: {} > {}", json.len(), self.config.max_json_size)
            ));
        }
        
        // Check for potential injection
        if SQL_INJECTION_REGEX.is_match(json) {
            return Err(BuildError::InputSanitization("Potential injection in JSON".to_string()));
        }
        
        // Basic JSON structure validation
        let depth = json.chars().fold((0i32, 0i32), |(max_depth, current_depth), c| {
            match c {
                '{' | '[' => (max_depth.max(current_depth + 1), current_depth + 1),
                '}' | ']' => (max_depth, current_depth.saturating_sub(1)),
                _ => (max_depth, current_depth),
            }
        }).0;
        
        if depth > self.config.max_xml_depth as i32 {
            return Err(BuildError::InputSanitization(
                format!("JSON nesting too deep: {}", depth)
            ));
        }
        
        Ok(())
    }
}

/// Rate limiter for API endpoints
#[derive(Debug)]
pub struct RateLimiter {
    requests: indexmap::IndexMap<String, Vec<Instant>>,
    config: SecurityConfig,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            requests: indexmap::IndexMap::new(),
            config,
        }
    }
    
    /// Check if request is allowed for given identifier
    pub fn check_rate_limit(&mut self, identifier: &str) -> Result<(), BuildError> {
        if !self.config.rate_limiting_enabled {
            return Ok(());
        }
        
        let now = Instant::now();
        let requests = self.requests.entry(identifier.to_string()).or_default();
        
        // Remove old requests outside the window
        requests.retain(|&req_time| now.duration_since(req_time) <= RATE_LIMIT_WINDOW);
        
        // Check if limit exceeded
        if requests.len() >= self.config.max_requests_per_minute as usize {
            return Err(BuildError::Security(
                format!("Rate limit exceeded for {}", identifier)
            ));
        }
        
        // Add current request
        requests.push(now);
        
        Ok(())
    }
    
    /// Clean up old entries periodically
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        
        self.requests.retain(|_, requests| {
            requests.retain(|&req_time| now.duration_since(req_time) <= RATE_LIMIT_WINDOW);
            !requests.is_empty()
        });
    }
}

/// Output safety and sanitization
#[derive(Debug)]
pub struct OutputSanitizer {
    config: SecurityConfig,
}

impl OutputSanitizer {
    /// Create new output sanitizer
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
    
    /// Sanitize XML output for safety
    pub fn sanitize_xml_output(&self, xml: &str) -> Result<String, BuildError> {
        // Check for potentially sensitive data patterns
        self.check_for_sensitive_data(xml)?;
        
        // Validate the XML structure first (before escaping)
        self.validate_xml_structure(xml)?;
        
        // Ensure proper XML escaping
        let sanitized = self.escape_xml_entities(xml);
        
        Ok(sanitized)
    }
    
    /// Check for sensitive data patterns in output
    fn check_for_sensitive_data(&self, content: &str) -> Result<(), BuildError> {
        // Check for common patterns that shouldn't be in output
        let sensitive_patterns = [
            r"<password[^>]*>[^<]+</password>",
            r"<secret[^>]*>[^<]+</secret>", 
            r"<key[^>]*>[^<]+</key>",
            r"<token[^>]*>[^<]+</token>",
            r"password\s*[:=]\s*[^\s<]+",
            r"secret\s*[:=]\s*[^\s<]+", 
            r"key\s*[:=]\s*[^\s<]+",
            r"token\s*[:=]\s*[^\s<]+",
            r"[A-Za-z0-9+/]{40,}={0,2}", // Base64 encoded data
        ];
        
        for pattern in &sensitive_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(content) {
                    return Err(BuildError::Security(
                        "Potential sensitive data detected in output".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Escape XML entities properly
    fn escape_xml_entities(&self, xml: &str) -> String {
        html_escape::encode_text(xml).to_string()
    }
    
    /// Validate XML structure is well-formed
    fn validate_xml_structure(&self, xml: &str) -> Result<(), BuildError> {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().expand_empty_elements = false;
        reader.config_mut().trim_text(true);
        
        let mut buf = Vec::new();
        let mut depth = 0;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(_)) => {
                    depth += 1;
                    if depth > MAX_XML_DEPTH {
                        return Err(BuildError::Security(
                            "XML depth limit exceeded in output".to_string()
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
                        format!("Invalid XML structure in output: {}", e)
                    ));
                }
            }
            buf.clear();
        }
        
        Ok(())
    }
    
    /// Generate secure log messages (without sensitive details)
    pub fn create_secure_log_message(&self, operation: &str, success: bool, details: Option<&str>) -> String {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let status = if success { "SUCCESS" } else { "FAILED" };
        
        match details {
            Some(detail) if detail.len() < 100 => {
                // Only include short, non-sensitive details
                let sanitized_detail = self.sanitize_log_detail(detail);
                format!("[{}] {} - {}: {}", timestamp, operation, status, sanitized_detail)
            }
            _ => {
                format!("[{}] {} - {}", timestamp, operation, status)
            }
        }
    }
    
    /// Sanitize log details to remove sensitive information
    fn sanitize_log_detail(&self, detail: &str) -> String {
        // Remove potential sensitive patterns from log messages
        let sensitive_patterns = [
            (r"password\s*[:=]\s*[^\s]+", "password=[REDACTED]"),
            (r"secret\s*[:=]\s*[^\s]+", "secret=[REDACTED]"),
            (r"key\s*[:=]\s*[^\s]+", "key=[REDACTED]"),
            (r"token\s*[:=]\s*[^\s]+", "token=[REDACTED]"),
        ];
        
        let mut sanitized = detail.to_string();
        for (pattern, replacement) in &sensitive_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                sanitized = regex.replace_all(&sanitized, *replacement).to_string();
            }
        }
        
        // Truncate if too long
        if sanitized.len() > 200 {
            sanitized.truncate(197);
            sanitized.push_str("...");
        }
        
        sanitized
    }
}

/// Secure temporary file handling
pub struct SecureTempFile {
    path: PathBuf,
    file: std::fs::File,
}

impl SecureTempFile {
    /// Create a secure temporary file
    pub fn new() -> Result<Self, BuildError> {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        
        let temp_dir = std::env::temp_dir();
        let file_name = format!("ddex_builder_{}", uuid::Uuid::new_v4());
        let path = temp_dir.join(file_name);
        
        // Create file with restricted permissions (owner read/write only)
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .mode(0o600) // Only owner can read/write
            .open(&path)
            .map_err(|e| BuildError::Io(format!("Failed to create secure temp file: {}", e)))?;
        
        Ok(Self { path, file })
    }
    
    /// Get the file reference
    pub fn file(&mut self) -> &mut std::fs::File {
        &mut self.file
    }
    
    /// Get the path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SecureTempFile {
    fn drop(&mut self) {
        // Securely delete the file
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Security utilities
pub mod utils {
    
    /// Sanitize filename for safe storage
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .take(255) // Limit filename length
            .collect::<String>()
            .replace("..", "") // Remove path traversal attempts
    }
    
    /// Generate secure random ID
    pub fn generate_secure_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    /// Constant-time string comparison to prevent timing attacks
    pub fn constant_time_compare(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }
        
        result == 0
    }
    
    /// Hash sensitive data for logging (truncated SHA-256)
    pub fn hash_for_logging(data: &str) -> String {
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(data.as_bytes());
        format!("{:.8}", hex::encode(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_input_validation() {
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // Test valid string
        assert!(validator.validate_string("Valid input", "test").is_ok());
        
        // Test string with null byte
        assert!(validator.validate_string("Invalid\0input", "test").is_err());
        
        // Test SQL injection attempt
        assert!(validator.validate_string("'; DROP TABLE users; --", "test").is_err());
        
        // Test dangerous entity
        assert!(validator.validate_string("&dangerous;", "test").is_err());
    }
    
    #[test]
    fn test_path_validation() {
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // Test valid path
        assert!(validator.validate_path("safe/path/file.xml").is_ok());
        
        // Test path traversal
        assert!(validator.validate_path("../../../etc/passwd").is_err());
        
        // Test absolute path
        assert!(validator.validate_path("/etc/passwd").is_err());
    }
    
    #[test]
    fn test_xml_security() {
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // Test safe XML
        assert!(validator.validate_xml_content("<root><child>content</child></root>").is_ok());
        
        // Test XXE attempt
        assert!(validator.validate_xml_content(
            "<!DOCTYPE test [<!ENTITY xxe SYSTEM 'file:///etc/passwd'>]><root>&xxe;</root>"
        ).is_err());
        
        // Test XML bomb
        assert!(validator.validate_xml_content(
            "<!DOCTYPE bomb [<!ENTITY a '&b;&b;'><!ENTITY b '&c;&c;'><!ENTITY c 'boom'>]><root>&a;</root>"
        ).is_err());
    }
    
    #[test]
    fn test_secure_xml_reader() {
        let config = SecurityConfig::default();
        let xml = b"<root><child>content</child></root>";
        let cursor = Cursor::new(xml);
        let mut reader = SecureXmlReader::new(cursor, config);
        
        // Should be able to read valid XML
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(_) => {
                    buf.clear();
                    continue;
                }
                Err(e) => panic!("Unexpected error: {}", e),
            }
        }
    }
    
    #[test]
    fn test_rate_limiter() {
        let config = SecurityConfig {
            rate_limiting_enabled: true,
            max_requests_per_minute: 2,
            ..SecurityConfig::default()
        };
        let mut limiter = RateLimiter::new(config);
        
        // First two requests should succeed
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        
        // Third request should fail
        assert!(limiter.check_rate_limit("user1").is_err());
        
        // Different user should work
        assert!(limiter.check_rate_limit("user2").is_ok());
    }
    
    #[test]
    fn test_url_validation() {
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // Test valid URL
        assert!(validator.validate_url("https://example.com/path").is_ok());
        
        // Test private IP
        assert!(validator.validate_url("http://192.168.1.1/").is_err());
        
        // Test localhost
        assert!(validator.validate_url("http://localhost:8080/").is_err());
        
        // Test unsafe scheme
        assert!(validator.validate_url("file:///etc/passwd").is_err());
    }
    
    #[test]
    fn test_output_sanitizer() {
        let config = SecurityConfig::default();
        let sanitizer = OutputSanitizer::new(config);
        
        // Test safe XML output
        let safe_xml = "<root><child>content</child></root>";
        assert!(sanitizer.sanitize_xml_output(safe_xml).is_ok());
        
        // Test XML with potential sensitive data
        let sensitive_xml = "<root><password>secret123</password></root>";
        let result = sanitizer.sanitize_xml_output(sensitive_xml);
        assert!(result.is_err(), "Expected sensitive data to be detected, but got: {:?}", result);
        
        // Test malformed XML (should fail XML structure validation after escaping)
        let malformed_xml = "<root><child>content</child><"; // Incomplete tag
        let result = sanitizer.sanitize_xml_output(malformed_xml);
        assert!(result.is_err(), "Expected malformed XML to be rejected, but got: {:?}", result);
    }
    
    #[test]
    fn test_secure_logging() {
        let config = SecurityConfig::default();
        let sanitizer = OutputSanitizer::new(config);
        
        // Test secure log message creation
        let log_msg = sanitizer.create_secure_log_message("BUILD", true, Some("file.xml"));
        assert!(log_msg.contains("BUILD"));
        assert!(log_msg.contains("SUCCESS"));
        assert!(log_msg.contains("file.xml"));
        
        // Test sensitive data redaction
        let sensitive_detail = "password=secret123 key=abc";
        let log_msg = sanitizer.create_secure_log_message("LOGIN", false, Some(sensitive_detail));
        assert!(log_msg.contains("[REDACTED]"));
        assert!(!log_msg.contains("secret123"));
        assert!(!log_msg.contains("abc"));
    }
    
    #[test]
    fn test_security_utils() {
        // Test filename sanitization
        let clean_name = utils::sanitize_filename("../../../etc/passwd");
        assert!(!clean_name.contains(".."));
        assert!(!clean_name.contains("/"));
        
        // Test secure ID generation
        let id1 = utils::generate_secure_id();
        let id2 = utils::generate_secure_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID length
        
        // Test constant-time comparison
        assert!(utils::constant_time_compare("test", "test"));
        assert!(!utils::constant_time_compare("test", "other"));
        assert!(!utils::constant_time_compare("test", "testing"));
        
        // Test hash for logging
        let hash = utils::hash_for_logging("sensitive_data");
        assert_eq!(hash.len(), 8);
        assert!(!hash.contains("sensitive"));
    }
}