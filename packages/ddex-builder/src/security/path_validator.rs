//! Cross-Platform Path Validation Module
//!
//! This module provides robust, platform-agnostic path validation that prevents:
//! - Directory traversal attacks (../, ..\, encoded variants)
//! - Absolute path access (/etc/passwd, C:\Windows\System32)
//! - UNC path attacks (\\server\share)
//! - URL-encoded traversal (%2e%2e/, %252e%252e/)
//! - Null byte injection (file.xml%00.txt)
//! - Symlink attacks and canonicalization issues
//! - Unicode normalization attacks
//! 
//! The validator works identically across Windows, Linux, and macOS by:
//! - Normalizing all paths to forward slashes internally
//! - Using platform-specific canonicalization when available
//! - Implementing whitelist-based validation
//! - Resolving symlinks and checking final destinations

use crate::error::BuildError;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf, Component};
use std::fs;
use once_cell::sync::Lazy;

/// Maximum allowed path length (cross-platform safe)
const MAX_PATH_LENGTH: usize = 260; // Windows MAX_PATH limit

/// Maximum path components depth
const MAX_PATH_DEPTH: usize = 32;

/// Directory traversal pattern detection
static DIRECTORY_TRAVERSAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\.\./|\.\.\x5c|/\.\./|\x5c\.\.\x5c)").unwrap()
});

/// URL-encoded traversal pattern detection
static ENCODED_TRAVERSAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(%2e%2e%2f|%2e%2e%5c|%252e%252e%252f|%252e%252e%255c)").unwrap()
});

/// Absolute path detection (Unix/Windows)
static ABSOLUTE_PATH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(^[a-zA-Z]:\x5c|^/|^\x5c\x5c)").unwrap()
});

/// Null byte and dangerous characters
static DANGEROUS_CHARS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\x00-\x1F\x7F-\x9F]|%00").unwrap()
});

/// Windows reserved filenames
static WINDOWS_RESERVED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\.|$)").unwrap()
});

/// Suspicious file extensions
static SUSPICIOUS_EXTENSIONS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(exe|bat|cmd|com|scr|pif|vbs|js|jar|dll|sys)$").unwrap()
});

/// Windows reserved device names
static WINDOWS_RESERVED_NAMES: Lazy<HashSet<&str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("CON"); set.insert("PRN"); set.insert("AUX"); set.insert("NUL");
    set.insert("COM1"); set.insert("COM2"); set.insert("COM3"); set.insert("COM4");
    set.insert("COM5"); set.insert("COM6"); set.insert("COM7"); set.insert("COM8"); set.insert("COM9");
    set.insert("LPT1"); set.insert("LPT2"); set.insert("LPT3"); set.insert("LPT4");
    set.insert("LPT5"); set.insert("LPT6"); set.insert("LPT7"); set.insert("LPT8"); set.insert("LPT9");
    set
});

/// Path validation configuration
#[derive(Debug, Clone)]
pub struct PathValidationConfig {
    /// Maximum allowed path length
    pub max_path_length: usize,
    /// Maximum path depth (number of components)
    pub max_path_depth: usize,
    /// Allowed base directories (whitelist)
    pub allowed_base_dirs: Vec<PathBuf>,
    /// Whether to allow relative paths outside allowed directories
    pub allow_relative_outside_base: bool,
    /// Whether to resolve and validate symlinks
    pub validate_symlinks: bool,
    /// Whether to check for file existence
    pub check_existence: bool,
    /// Additional allowed file extensions
    pub allowed_extensions: HashSet<String>,
    /// Whether to allow hidden files/directories
    pub allow_hidden: bool,
}

impl Default for PathValidationConfig {
    fn default() -> Self {
        let mut allowed_extensions = HashSet::new();
        allowed_extensions.insert("xml".to_string());
        allowed_extensions.insert("json".to_string());
        allowed_extensions.insert("txt".to_string());
        allowed_extensions.insert("csv".to_string());
        
        Self {
            max_path_length: MAX_PATH_LENGTH,
            max_path_depth: MAX_PATH_DEPTH,
            allowed_base_dirs: vec![
                PathBuf::from("data"),
                PathBuf::from("input"),
                PathBuf::from("output"), 
                PathBuf::from("temp"),
                PathBuf::from("."),
            ],
            allow_relative_outside_base: false,
            validate_symlinks: true,
            check_existence: false,
            allowed_extensions,
            allow_hidden: false,
        }
    }
}

/// Result of path validation
#[derive(Debug, Clone)]
pub struct ValidatedPath {
    /// Original input path
    pub original: String,
    /// Normalized path (forward slashes, no redundant components)
    pub normalized: PathBuf,
    /// Canonicalized path (if successful)
    pub canonical: Option<PathBuf>,
    /// Whether the path exists
    pub exists: bool,
    /// Detected security issues (warnings)
    pub warnings: Vec<String>,
}

/// Cross-platform path validator
#[derive(Debug, Clone)]
pub struct PathValidator {
    config: PathValidationConfig,
}

impl PathValidator {
    /// Create a new path validator with default configuration
    pub fn new() -> Self {
        Self {
            config: PathValidationConfig::default(),
        }
    }
    
    /// Create a new path validator with custom configuration
    pub fn with_config(config: PathValidationConfig) -> Self {
        Self { config }
    }
    
    /// Validate a path string for security issues
    pub fn validate(&self, path_str: &str) -> Result<ValidatedPath, BuildError> {
        // Input sanitization and initial checks
        let sanitized_input = self.sanitize_input(path_str)?;
        
        // Length check
        if sanitized_input.len() > self.config.max_path_length {
            return Err(BuildError::InputSanitization(
                format!("Path too long: {} > {}", sanitized_input.len(), self.config.max_path_length)
            ));
        }
        
        // Detect dangerous patterns
        self.detect_dangerous_patterns(&sanitized_input)?;
        
        // Normalize the path
        let normalized = self.normalize_path(&sanitized_input)?;
        
        // Validate path components  
        self.validate_components(&normalized)?;
        
        // Check against whitelist
        self.validate_against_whitelist(&normalized)?;
        
        // Handle canonicalization (platform-aware)
        let (canonical, exists) = self.safe_canonicalize(&normalized);
        
        // Validate symlinks if enabled
        if self.config.validate_symlinks {
            self.validate_symlinks(&normalized, &canonical)?;
        }
        
        // Check file existence if required
        if self.config.check_existence && !exists {
            return Err(BuildError::InputSanitization(
                "File does not exist".to_string()
            ));
        }
        
        let warnings = self.collect_warnings(&sanitized_input, &normalized);
        
        Ok(ValidatedPath {
            original: path_str.to_string(),
            normalized,
            canonical,
            exists,
            warnings,
        })
    }
    
    /// Sanitize input string and detect encoding attacks
    fn sanitize_input(&self, input: &str) -> Result<String, BuildError> {
        // Check for null bytes
        if input.contains('\0') {
            return Err(BuildError::InputSanitization(
                "Null byte detected in path".to_string()
            ));
        }
        
        // Decode URL encoding (but detect double-encoding attacks)
        let decoded = self.safe_url_decode(input)?;
        
        // Check for control characters
        if decoded.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            return Err(BuildError::InputSanitization(
                "Control characters detected in path".to_string()
            ));
        }
        
        // Normalize Unicode (detect normalization attacks)
        let normalized = self.normalize_unicode(&decoded)?;
        
        Ok(normalized)
    }
    
    /// Safe URL decoding that detects double-encoding attacks
    fn safe_url_decode(&self, input: &str) -> Result<String, BuildError> {
        let first_decode = urlencoding::decode(input)
            .map_err(|e| BuildError::InputSanitization(format!("URL decode error: {}", e)))?;
        
        // Check for double-encoding by attempting to decode again
        let second_decode = urlencoding::decode(&first_decode);
        if second_decode.is_ok() && second_decode.as_ref().unwrap() != &first_decode {
            return Err(BuildError::InputSanitization(
                "Double URL encoding detected (potential attack)".to_string()
            ));
        }
        
        Ok(first_decode.into_owned())
    }
    
    /// Normalize Unicode and detect normalization attacks
    fn normalize_unicode(&self, input: &str) -> Result<String, BuildError> {
        use unicode_normalization::UnicodeNormalization;
        
        let nfc = input.nfc().collect::<String>();
        let nfd = input.nfd().collect::<String>();
        let nfkc = input.nfkc().collect::<String>();
        let nfkd = input.nfkd().collect::<String>();
        
        // Check if normalization forms are different AND contain dangerous patterns
        // Only flag as normalization attack if forms differ significantly
        let forms_identical = nfc == nfd && nfd == nfkc && nfkc == nfkd;
        
        // If all forms are the same, this is not a normalization attack
        // Let the normal path validation handle it
        if !forms_identical {
            let forms = [&nfc, &nfd, &nfkc, &nfkd];
            let mut dangerous_forms = Vec::new();
            
            for (i, form) in forms.iter().enumerate() {
                if DIRECTORY_TRAVERSAL.is_match(form) || ENCODED_TRAVERSAL.is_match(form) || ABSOLUTE_PATH.is_match(form) || DANGEROUS_CHARS.is_match(form) {
                    dangerous_forms.push(match i {
                        0 => "NFC",
                        1 => "NFD", 
                        2 => "NFKC",
                        3 => "NFKD",
                        _ => unreachable!(),
                    });
                }
            }
            
            if !dangerous_forms.is_empty() {
                return Err(BuildError::InputSanitization(
                    format!("Unicode normalization attack detected in forms: {:?}", dangerous_forms)
                ));
            }
        }
        
        // Use NFC normalization
        Ok(nfc)
    }
    
    /// Detect dangerous patterns in the path
    fn detect_dangerous_patterns(&self, path: &str) -> Result<(), BuildError> {
        // Check directory traversal patterns
        if DIRECTORY_TRAVERSAL.is_match(path) {
            return Err(BuildError::InputSanitization(
                "Directory traversal pattern detected".to_string()
            ));
        }
        
        // Check encoded traversal patterns
        if ENCODED_TRAVERSAL.is_match(path) {
            return Err(BuildError::InputSanitization(
                "Encoded path traversal detected".to_string()
            ));
        }
        
        // Check for absolute paths
        if ABSOLUTE_PATH.is_match(path) {
            return Err(BuildError::InputSanitization(
                "Absolute path not allowed".to_string()
            ));
        }
        
        // Check for dangerous characters
        if DANGEROUS_CHARS.is_match(path) {
            return Err(BuildError::InputSanitization(
                "Dangerous characters detected".to_string()
            ));
        }
        
        // Check for suspicious filenames
        if let Some(filename) = Path::new(path).file_name().and_then(|s| s.to_str()) {
            if WINDOWS_RESERVED.is_match(filename) {
                return Err(BuildError::InputSanitization(
                    "Windows reserved filename detected".to_string()
                ));
            }
            
            if SUSPICIOUS_EXTENSIONS.is_match(filename) {
                return Err(BuildError::InputSanitization(
                    "Suspicious file extension detected".to_string()
                ));
            }
            
            // Check Windows reserved names
            let filename_upper = filename.to_uppercase();
            let base_name = filename_upper.split('.').next().unwrap_or(&filename_upper);
            if WINDOWS_RESERVED_NAMES.contains(base_name) {
                return Err(BuildError::InputSanitization(
                    "Windows reserved filename detected".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Normalize path to use forward slashes and remove redundant components
    fn normalize_path(&self, path: &str) -> Result<PathBuf, BuildError> {
        // Convert all separators to forward slashes for consistent processing
        let normalized_str = path.replace('\\', "/");
        
        // Split into components and filter out empty and current directory references
        let components: Vec<&str> = normalized_str
            .split('/')
            .filter(|c| !c.is_empty() && *c != ".")
            .collect();
        
        // Check depth
        if components.len() > self.config.max_path_depth {
            return Err(BuildError::InputSanitization(
                format!("Path too deep: {} > {}", components.len(), self.config.max_path_depth)
            ));
        }
        
        // Build normalized path
        let mut normalized = PathBuf::new();
        for component in components {
            // Reject parent directory references
            if component == ".." {
                return Err(BuildError::InputSanitization(
                    "Path traversal (..) detected".to_string()
                ));
            }
            
            normalized.push(component);
        }
        
        Ok(normalized)
    }
    
    /// Validate individual path components
    fn validate_components(&self, path: &Path) -> Result<(), BuildError> {
        for component in path.components() {
            match component {
                Component::Normal(name) => {
                    let name_str = name.to_string_lossy();
                    
                    // Check for hidden files/directories
                    if !self.config.allow_hidden && name_str.starts_with('.') {
                        return Err(BuildError::InputSanitization(
                            "Hidden files/directories not allowed".to_string()
                        ));
                    }
                    
                    // Check component length
                    if name_str.len() > 255 {
                        return Err(BuildError::InputSanitization(
                            "Path component too long".to_string()
                        ));
                    }
                    
                    // Check for dangerous characters in component  
                    if name_str.chars().any(|c| r#"<>:"|?*"#.contains(c)) {
                        return Err(BuildError::InputSanitization(
                            "Dangerous characters in path component".to_string()
                        ));
                    }
                }
                Component::ParentDir => {
                    return Err(BuildError::InputSanitization(
                        "Parent directory traversal detected".to_string()
                    ));
                }
                Component::RootDir => {
                    return Err(BuildError::InputSanitization(
                        "Root directory access not allowed".to_string()
                    ));
                }
                Component::Prefix(_) => {
                    return Err(BuildError::InputSanitization(
                        "Windows path prefix not allowed".to_string()
                    ));
                }
                Component::CurDir => {
                    // Already filtered out in normalize_path
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate path against whitelist of allowed base directories
    fn validate_against_whitelist(&self, path: &Path) -> Result<(), BuildError> {
        if self.config.allow_relative_outside_base && path.is_relative() {
            return Ok(()); // Allow any relative path
        }
        
        // Check if path starts with any allowed base directory
        for base_dir in &self.config.allowed_base_dirs {
            if path.starts_with(base_dir) || path == base_dir {
                return Ok(());
            }
            
            // Also check if the path is within the base directory when normalized
            if let Ok(canonical_base) = base_dir.canonicalize() {
                if let Ok(canonical_path) = path.canonicalize() {
                    if canonical_path.starts_with(canonical_base) {
                        return Ok(());
                    }
                }
            }
        }
        
        Err(BuildError::InputSanitization(
            "Path not within allowed directories".to_string()
        ))
    }
    
    /// Safely canonicalize path (handle platform differences)
    fn safe_canonicalize(&self, path: &Path) -> (Option<PathBuf>, bool) {
        let exists = path.exists();
        
        // Try to canonicalize if the path exists
        if exists {
            match path.canonicalize() {
                Ok(canonical) => (Some(canonical), true),
                Err(_) => (None, exists),
            }
        } else {
            // For non-existent paths, try to canonicalize the parent directory
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    match parent.canonicalize() {
                        Ok(canonical_parent) => {
                            if let Some(filename) = path.file_name() {
                                let canonical = canonical_parent.join(filename);
                                (Some(canonical), false)
                            } else {
                                (None, false)
                            }
                        }
                        Err(_) => (None, false),
                    }
                } else {
                    (None, false)
                }
            } else {
                (None, false)
            }
        }
    }
    
    /// Validate symlinks to prevent symlink attacks
    fn validate_symlinks(&self, normalized: &Path, canonical: &Option<PathBuf>) -> Result<(), BuildError> {
        if let Some(canonical_path) = canonical {
            // Check if the canonical path is different from the normalized path
            // This indicates the presence of symlinks
            if normalized != canonical_path {
                // Verify the canonical path is still within allowed directories
                self.validate_against_whitelist(canonical_path)?;
                
                // Check if the symlink target contains dangerous patterns
                if let Some(target_str) = canonical_path.to_str() {
                    if DIRECTORY_TRAVERSAL.is_match(target_str) || ENCODED_TRAVERSAL.is_match(target_str) || 
                       ABSOLUTE_PATH.is_match(target_str) || DANGEROUS_CHARS.is_match(target_str) {
                        return Err(BuildError::InputSanitization(
                            "Symlink target contains dangerous patterns".to_string()
                        ));
                    }
                }
                
                // Check for symlink loops (basic detection)
                if let Ok(metadata) = fs::symlink_metadata(normalized) {
                    if metadata.file_type().is_symlink() {
                        // This is a symlink, let's check for potential loops
                        let mut visited = HashSet::new();
                        let mut current = normalized.to_path_buf();
                        
                        while current.is_symlink() && visited.len() < 32 {
                            if visited.contains(&current) {
                                return Err(BuildError::InputSanitization(
                                    "Symlink loop detected".to_string()
                                ));
                            }
                            visited.insert(current.clone());
                            
                            match fs::read_link(&current) {
                                Ok(target) => {
                                    current = if target.is_absolute() {
                                        target
                                    } else {
                                        current.parent()
                                            .unwrap_or_else(|| Path::new("."))
                                            .join(target)
                                    };
                                }
                                Err(_) => break,
                            }
                        }
                        
                        if visited.len() >= 32 {
                            return Err(BuildError::InputSanitization(
                                "Symlink chain too long (potential loop)".to_string()
                            ));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Collect warnings about potentially suspicious but not necessarily dangerous patterns
    fn collect_warnings(&self, input: &str, normalized: &Path) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Warn about unusual characters
        if input.chars().any(|c| !c.is_ascii()) {
            warnings.push("Path contains non-ASCII characters".to_string());
        }
        
        // Warn about very long filenames
        if let Some(filename) = normalized.file_name().and_then(|s| s.to_str()) {
            if filename.len() > 100 {
                warnings.push("Very long filename".to_string());
            }
        }
        
        // Warn about deeply nested paths
        if normalized.components().count() > 8 {
            warnings.push("Deeply nested path".to_string());
        }
        
        // Warn about unusual extensions
        if let Some(extension) = normalized.extension().and_then(|s| s.to_str()) {
            if !self.config.allowed_extensions.contains(&extension.to_lowercase()) {
                warnings.push(format!("Unusual file extension: {}", extension));
            }
        }
        
        warnings
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &PathValidationConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: PathValidationConfig) {
        self.config = config;
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_basic_path_validation() {
        let validator = PathValidator::new();
        
        // Valid paths
        assert!(validator.validate("data/file.xml").is_ok());
        assert!(validator.validate("input/subdir/file.json").is_ok());
        assert!(validator.validate("./file.txt").is_ok());
        
        // Invalid paths
        assert!(validator.validate("../etc/passwd").is_err());
        assert!(validator.validate("/etc/passwd").is_err());
        assert!(validator.validate("C:\\Windows\\System32").is_err());
    }
    
    #[test]
    fn test_dangerous_patterns() {
        let validator = PathValidator::new();
        
        let dangerous_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "/etc/passwd",
            "/proc/self/environ",
            "C:\\Windows\\System32",
            "\\\\server\\share",
            "file%00.txt",
            "%2e%2e%2fpasswd",
            "%252e%252e%252fpasswd",
        ];
        
        for path in dangerous_paths {
            let result = validator.validate(path);
            assert!(result.is_err(), "Should reject dangerous path: {}", path);
        }
    }
    
    #[test]
    fn test_url_encoding_attacks() {
        let validator = PathValidator::new();
        
        let encoded_attacks = vec![
            "%2e%2e%2f",           // ../
            "%2e%2e%5c",           // ..\
            "%252e%252e%252f",     // Double-encoded ../
            "..%2f",               // ../ mixed
            "..%00",               // Null byte
        ];
        
        for attack in encoded_attacks {
            assert!(validator.validate(attack).is_err(), "Should block encoded attack: {}", attack);
        }
    }
    
    #[test]
    fn test_windows_reserved_names() {
        let validator = PathValidator::new();
        
        let reserved_names = vec![
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "LPT1", "LPT2",
            "con.txt", "prn.xml", "aux.json",
        ];
        
        for name in reserved_names {
            assert!(validator.validate(name).is_err(), "Should block reserved name: {}", name);
        }
    }
    
    #[test]
    fn test_path_normalization() {
        let validator = PathValidator::new();
        
        // Test that paths are normalized correctly
        let result = validator.validate("data//file.xml").unwrap();
        assert_eq!(result.normalized, Path::new("data/file.xml"));
        
        let result = validator.validate("data\\subdir\\file.json").unwrap();
        assert_eq!(result.normalized, Path::new("data/subdir/file.json"));
        
        let result = validator.validate("./data/./file.txt").unwrap();
        assert_eq!(result.normalized, Path::new("data/file.txt"));
    }
    
    #[test]
    fn test_whitelist_validation() {
        let mut config = PathValidationConfig::default();
        config.allowed_base_dirs = vec![PathBuf::from("allowed")];
        config.allow_relative_outside_base = false;
        
        let validator = PathValidator::with_config(config);
        
        assert!(validator.validate("allowed/file.xml").is_ok());
        assert!(validator.validate("disallowed/file.xml").is_err());
    }
    
    #[test]
    fn test_unicode_normalization() {
        let validator = PathValidator::new();
        
        // Test normal Unicode characters
        assert!(validator.validate("data/résumé.txt").is_ok());
        
        // The validator should handle Unicode normalization safely
        // This is a basic test - more sophisticated Unicode attacks would need specific test cases
    }
    
    #[test]
    fn test_length_limits() {
        let mut config = PathValidationConfig::default();
        config.max_path_length = 50;
        config.max_path_depth = 3;
        
        let validator = PathValidator::with_config(config);
        
        // Too long
        let long_path = "a/".repeat(30);
        assert!(validator.validate(&long_path).is_err());
        
        // Too deep
        let deep_path = "a/b/c/d/e/f/g.txt";
        assert!(validator.validate(deep_path).is_err());
    }
    
    #[test]
    fn test_file_extensions() {
        let mut config = PathValidationConfig::default();
        config.allowed_extensions = vec!["xml".to_string(), "json".to_string()].into_iter().collect();
        
        let validator = PathValidator::with_config(config);
        
        let result = validator.validate("data/file.xml").unwrap();
        assert!(result.warnings.is_empty());
        
        let result = validator.validate("data/file.exe").unwrap();
        assert!(result.warnings.iter().any(|w| w.contains("extension")));
    }
}