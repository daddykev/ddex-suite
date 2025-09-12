//! Comprehensive Path Validation Tests
//!
//! This module contains exhaustive tests for the cross-platform path validator,
//! covering all major attack vectors and platform-specific edge cases.

use ddex_builder::security::path_validator::{PathValidator, PathValidationConfig};
use std::collections::HashSet;
use std::path::PathBuf;

/// Test basic path validation functionality
#[test]
fn test_basic_path_validation() {
    let validator = PathValidator::new();
    
    // Valid relative paths within allowed directories
    let valid_paths = vec![
        "data/file.xml",
        "input/subdir/file.json", 
        "output/results.txt",
        "temp/processing.csv",
        "file.xml",
        "subdir/file.json",
        "./file.txt",
        "data/deep/nested/path/file.xml",
    ];
    
    for path in valid_paths {
        let result = validator.validate(path);
        assert!(result.is_ok(), "Valid path should be accepted: {} -> {:?}", path, result);
        
        if let Ok(validated) = result {
            assert_eq!(validated.original, path);
            assert!(!validated.normalized.is_absolute());
        }
    }
    
    // Invalid paths that should be rejected
    let invalid_paths = vec![
        "../etc/passwd",              // Parent directory traversal
        "/etc/passwd",                // Absolute Unix path
        "C:\\Windows\\System32",      // Absolute Windows path
        "\\\\server\\share",          // UNC path
        "file\0.txt",                 // Null byte
        "",                           // Empty path
        "    ",                       // Whitespace only
        "data/../../../etc/passwd",   // Mixed traversal
    ];
    
    for path in invalid_paths {
        let result = validator.validate(path);
        assert!(result.is_err(), "Invalid path should be rejected: {}", path);
    }
}

/// Test directory traversal attack prevention
#[test] 
fn test_directory_traversal_prevention() {
    let validator = PathValidator::new();
    
    let traversal_attacks = vec![
        // Unix-style traversal
        "../etc/passwd",
        "../../etc/passwd", 
        "../../../etc/passwd",
        "../../../../etc/passwd",
        "data/../etc/passwd",
        "data/../../etc/passwd",
        "./../../etc/passwd",
        
        // Windows-style traversal
        "..\\windows\\system32\\config\\sam",
        "..\\..\\windows\\system32\\config\\sam",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "data\\..\\..\\windows\\system32",
        
        // Mixed separators
        "../windows\\system32",
        "..\\etc/passwd",
        "data/../windows\\system32",
        
        // URL-encoded traversal
        "%2e%2e%2f",                  // ../
        "%2e%2e%5c",                  // ..\
        "%2e%2e/",                    // ../ mixed
        "%2e%2e\\",                   // ..\ mixed
        "..%2f",                      // ../ mixed
        "..%5c",                      // ..\ mixed
        
        // Double URL-encoded
        "%252e%252e%252f",            // Double-encoded ../
        "%252e%252e%255c",            // Double-encoded ..\
        
        // Unicode variations
        "\u{002e}\u{002e}/",          // Unicode ../
        "\u{ff0e}\u{ff0e}/",          // Fullwidth dots
        
        // Null byte injection
        "../etc/passwd%00",
        "..%00/etc/passwd",
        "%2e%2e%00",
        
        // Space variations
        ".. /etc/passwd",
        " ../etc/passwd",
        "../ etc/passwd",
    ];
    
    for attack in traversal_attacks {
        let result = validator.validate(attack);
        assert!(result.is_err(), "Directory traversal should be blocked: {}", attack);
        
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("traversal") || 
                error_msg.contains("dangerous") ||
                error_msg.contains("pattern") ||
                error_msg.contains("encoding") ||
                error_msg.contains("null"),
                "Error should mention traversal/dangerous pattern: {} -> {}", attack, error_msg
            );
        }
    }
}

/// Test absolute path prevention
#[test]
fn test_absolute_path_prevention() {
    let validator = PathValidator::new();
    
    let absolute_paths = vec![
        // Unix absolute paths
        "/etc/passwd",
        "/proc/self/environ", 
        "/sys/kernel/version",
        "/dev/null",
        "/root/.ssh/id_rsa",
        "/tmp/malicious.txt",
        "/var/log/system.log",
        "/usr/bin/sh",
        "/sbin/init",
        "/boot/vmlinuz",
        "/home/user/.bashrc",
        "/home/user/.config/secret",
        
        // Windows absolute paths
        "C:\\",
        "C:\\Windows\\",
        "C:\\Windows\\System32",
        "C:\\Windows\\System32\\config\\sam",
        "D:\\data\\sensitive.txt",
        "E:\\backup\\passwords.txt",
        "F:\\system\\critical.dll",
        
        // UNC paths
        "\\\\server\\share",
        "\\\\server\\share\\file.txt",
        "\\\\localhost\\c$\\windows", 
        "\\\\127.0.0.1\\admin$",
        "\\\\?\\c:\\windows\\system32",
        "\\\\?\\UNC\\server\\share",
        
        // Windows device paths
        "\\\\.\\COM1",
        "\\\\.\\LPT1", 
        "\\\\.\\PRN",
        "\\\\.\\AUX",
        "\\\\.\\NUL",
        "\\Device\\HarddiskVolume1",
        "\\DosDevices\\C:",
        
        // Cygwin paths
        "/cygdrive/c/windows",
        "/cygdrive/d/data",
        
        // WSL paths  
        "/mnt/c/Windows",
        "/mnt/d/Data",
    ];
    
    for path in absolute_paths {
        let result = validator.validate(path);
        assert!(result.is_err(), "Absolute path should be blocked: {}", path);
    }
}

/// Test Windows reserved filename prevention
#[test]
fn test_windows_reserved_names() {
    let validator = PathValidator::new();
    
    let reserved_names = vec![
        // Basic reserved names
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        
        // With extensions
        "CON.txt", "PRN.xml", "AUX.json", "NUL.csv",
        "COM1.exe", "COM2.bat", "COM3.doc",
        "LPT1.log", "LPT2.tmp", 
        
        // Case variations
        "con", "prn", "aux", "nul",
        "Con", "Prn", "Aux", "Nul",
        "CON", "PRN", "AUX", "NUL",
        "com1", "Com1", "COM1",
        "lpt1", "Lpt1", "LPT1",
        
        // In subdirectories
        "data/CON.txt",
        "input/PRN.xml", 
        "output/AUX.json",
        "temp/COM1.log",
        "subdir/LPT1.tmp",
    ];
    
    for name in reserved_names {
        let result = validator.validate(name);
        assert!(result.is_err(), "Windows reserved name should be blocked: {}", name);
    }
}

/// Test URL encoding attack prevention
#[test]
fn test_url_encoding_attacks() {
    let validator = PathValidator::new();
    
    let encoded_attacks = vec![
        // Single URL encoding
        "%2e%2e%2f",                  // ../
        "%2e%2e%5c",                  // ..\
        "%2f%2e%2e%2f",              // /../
        "%5c%2e%2e%5c",              // \..\
        "%2e%2e%2fetc%2fpasswd",     // ../etc/passwd
        "%2e%2e%5cwindows%5csystem32", // ..\windows\system32
        
        // Double URL encoding
        "%252e%252e%252f",            // Double-encoded ../
        "%252e%252e%255c",            // Double-encoded ..\
        "%252f%252e%252e%252f",      // Double-encoded /../
        "%252e%252e%252fetc%252fpasswd", // Double-encoded ../etc/passwd
        
        // Triple URL encoding
        "%25252e%25252e%25252f",      // Triple-encoded ../
        
        // Mixed encoding
        "..%2f",                      // ../ with encoded slash
        "..%5c",                      // ..\ with encoded backslash
        "%2e%2e/",                    // Encoded dots with normal slash
        "%2e%2e\\",                   // Encoded dots with normal backslash
        ".%2e/",                      // Mixed encoding
        
        // Null byte injection
        "%00",                        // Null byte
        "file.txt%00",               // Null byte after filename
        "file.txt%00.exe",           // Null byte with extension
        "%2e%2e%2f%00",              // Traversal with null byte
        "..%00/etc/passwd",          // Mixed null byte injection
        
        // Other dangerous characters
        "%3c",                        // <
        "%3e",                        // >
        "%22",                        // "
        "%7c",                        // |
        "%3f",                        // ?
        "%2a",                        // *
        "%3a",                        // :
    ];
    
    for attack in encoded_attacks {
        let result = validator.validate(attack);
        assert!(result.is_err(), "URL encoding attack should be blocked: {}", attack);
        
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("encoding") || 
                error_msg.contains("dangerous") ||
                error_msg.contains("null") ||
                error_msg.contains("pattern"),
                "Error should mention encoding/dangerous pattern: {} -> {}", attack, error_msg
            );
        }
    }
}

/// Test Unicode normalization attack prevention  
#[test]
fn test_unicode_normalization_attacks() {
    let validator = PathValidator::new();
    
    let unicode_attacks = vec![
        // Fullwidth characters (looks like ../）
        "\u{ff0e}\u{ff0e}\u{ff0f}",           // ．．／
        "\u{ff0e}\u{ff0e}\u{ff3c}",           // ．．＼
        
        // Combining characters that could normalize to dangerous patterns
        "\u{002e}\u{0300}\u{002e}\u{0300}\u{002f}", // e. with combining grave accent
        
        // Right-to-left override attacks
        "\u{202e}../",                         // RLO + ../
        "\u{202d}../",                         // LRO + ../  
        
        // Zero-width characters
        ".\u{200b}./",                         // Zero-width space
        ".\u{200c}./",                         // Zero-width non-joiner
        ".\u{200d}./",                         // Zero-width joiner
        ".\u{feff}./",                         // Zero-width no-break space
        
        // Homograph attacks (visually similar characters)
        ".\u{ff0e}/",                          // Fullwidth dot
        "\u{2024}\u{2024}/",                  // One dot leader (looks like ..)
        "\u{fe52}\u{fe52}/",                  // Small dot
        
        // Mixed scripts that could be confusing
        "data/\u{0440}\u{0430}\u{0455}\u{0455}\u{0461}\u{043e}\u{0440}\u{0434}", // Cyrillic "password"
    ];
    
    for attack in unicode_attacks {
        let result = validator.validate(attack);
        // Some Unicode might be allowed but should generate warnings
        // Others should be blocked if they normalize to dangerous patterns
        match result {
            Ok(validated) => {
                // If allowed, check for warnings about Unicode usage
                if validated.original.chars().any(|c| !c.is_ascii()) {
                    assert!(
                        validated.warnings.iter().any(|w| w.contains("ASCII")),
                        "Unicode path should generate warning: {}", attack
                    );
                }
            }
            Err(_) => {
                // Blocking Unicode attacks is also acceptable
            }
        }
    }
}

/// Test suspicious filename pattern detection
#[test]
fn test_suspicious_filename_patterns() {
    let validator = PathValidator::new();
    
    let suspicious_files = vec![
        // Executable files
        "malware.exe",
        "script.bat",
        "command.cmd", 
        "program.com",
        "screen.scr",
        "installer.pif",
        "virus.vbs",
        "exploit.js",
        "payload.jar",
        "library.dll",
        "driver.sys",
        "config.ini",
        "install.inf",
        "settings.reg",
        
        // Script files
        "shell.sh",
        "perl.pl", 
        "python.py",
        "ruby.rb",
        "web.php",
        "active.asp",
        "server.aspx",
        "java.jsp",
        
        // Temporary/backup files
        "data.tmp",
        "file.temp",
        "backup.log",
        "old.bak",
        "original.backup",
        "previous.old",
        
        // Hidden/system files (if not allowed)
        ".htaccess",
        ".htpasswd",
        ".env",
        ".git",
        ".svn",
        ".DS_Store",
        "thumbs.db",
        "desktop.ini",
        
        // Files with only dots
        ".",
        "..",
        "...",
        "....",
        
        // Files with only spaces
        " ",
        "  ",
        "   ",
    ];
    
    for file in suspicious_files {
        let result = validator.validate(file);
        assert!(result.is_err(), "Suspicious file should be blocked: {}", file);
    }
}

/// Test path length and depth limits
#[test]
fn test_path_length_and_depth_limits() {
    let mut config = PathValidationConfig::default();
    config.max_path_length = 100;
    config.max_path_depth = 5;
    
    let validator = PathValidator::with_config(config);
    
    // Test path length limit
    let long_path = "a".repeat(101);
    assert!(validator.validate(&long_path).is_err());
    
    let ok_length_path = "a".repeat(50);
    assert!(validator.validate(&ok_length_path).is_ok());
    
    // Test path depth limit  
    let deep_path = "a/b/c/d/e/f/g.txt"; // 7 components
    assert!(validator.validate(deep_path).is_err());
    
    let ok_depth_path = "a/b/c/d.txt"; // 4 components
    assert!(validator.validate(ok_depth_path).is_ok());
    
    // Test component length limit
    let long_component = format!("data/{}.txt", "a".repeat(256));
    assert!(validator.validate(&long_component).is_err());
}

/// Test whitelist validation
#[test]
fn test_whitelist_validation() {
    let mut config = PathValidationConfig::default();
    config.allowed_base_dirs = vec![
        PathBuf::from("allowed"),
        PathBuf::from("safe"),
        PathBuf::from("data/subdir"),
    ];
    config.allow_relative_outside_base = false;
    
    let validator = PathValidator::with_config(config);
    
    // Allowed paths
    let allowed_paths = vec![
        "allowed/file.xml",
        "allowed/sub/file.json",
        "safe/data.txt",
        "data/subdir/file.csv",
    ];
    
    for path in allowed_paths {
        let result = validator.validate(path);
        assert!(result.is_ok(), "Whitelisted path should be allowed: {}", path);
    }
    
    // Disallowed paths
    let disallowed_paths = vec![
        "disallowed/file.xml",
        "danger/file.json", 
        "data/other/file.txt",  // Not in data/subdir
        "random/file.csv",
    ];
    
    for path in disallowed_paths {
        let result = validator.validate(path);
        assert!(result.is_err(), "Non-whitelisted path should be blocked: {}", path);
    }
}

/// Test file extension validation
#[test]
fn test_file_extension_validation() {
    let mut config = PathValidationConfig::default();
    config.allowed_extensions = ["xml", "json", "txt"].iter().map(|s| s.to_string()).collect();
    
    let validator = PathValidator::with_config(config);
    
    // Test allowed extensions (should not generate warnings)
    let allowed_files = vec![
        "data/file.xml",
        "input/data.json",
        "output/result.txt",
    ];
    
    for file in allowed_files {
        let result = validator.validate(file).unwrap();
        let ext_warnings: Vec<_> = result.warnings.iter()
            .filter(|w| w.contains("extension"))
            .collect();
        assert!(ext_warnings.is_empty(), "Allowed extension should not generate warning: {}", file);
    }
    
    // Test disallowed extensions (should generate warnings) 
    let disallowed_files = vec![
        "data/file.exe",
        "input/script.js",
        "output/data.bin",
    ];
    
    for file in disallowed_files {
        let result = validator.validate(file);
        match result {
            Ok(validated) => {
                assert!(
                    validated.warnings.iter().any(|w| w.contains("extension")),
                    "Disallowed extension should generate warning: {}", file
                );
            }
            Err(_) => {
                // Also acceptable if the file is completely blocked
            }
        }
    }
}

/// Test hidden file handling
#[test]
fn test_hidden_file_handling() {
    // Test with hidden files disabled (default)
    let validator = PathValidator::new();
    
    let hidden_files = vec![
        ".hidden",
        ".config",
        ".ssh",
        "data/.htaccess",
        "input/.env",
        ".git/config",
        ".svn/entries",
    ];
    
    for file in hidden_files {
        let result = validator.validate(file);
        assert!(result.is_err(), "Hidden file should be blocked when not allowed: {}", file);
    }
    
    // Test with hidden files enabled
    let mut config = PathValidationConfig::default();
    config.allow_hidden = true;
    let validator_allow_hidden = PathValidator::with_config(config);
    
    for file in ["data/.config", "input/.env"] {
        let result = validator_allow_hidden.validate(file);
        assert!(result.is_ok(), "Hidden file should be allowed when enabled: {}", file);
    }
}

/// Test path normalization
#[test]
fn test_path_normalization() {
    let validator = PathValidator::new();
    
    let test_cases = vec![
        // (input, expected_normalized)
        ("data/file.xml", "data/file.xml"),
        ("data//file.xml", "data/file.xml"),        // Double slashes
        ("data///file.xml", "data/file.xml"),       // Triple slashes
        ("data\\file.xml", "data/file.xml"),        // Backslashes to forward
        ("data\\\\file.xml", "data/file.xml"),      // Double backslashes
        ("data\\/file.xml", "data/file.xml"),       // Mixed slashes
        ("./data/file.xml", "data/file.xml"),       // Current directory
        ("data/./file.xml", "data/file.xml"),       // Current directory in middle
        ("data/sub/./file.xml", "data/sub/file.xml"), // Current directory at end
    ];
    
    for (input, expected) in test_cases {
        let result = validator.validate(input).unwrap();
        assert_eq!(
            result.normalized,
            PathBuf::from(expected),
            "Path normalization failed: {} -> {} (expected {})", 
            input, 
            result.normalized.display(), 
            expected
        );
    }
}

/// Test dangerous character detection
#[test]
fn test_dangerous_character_detection() {
    let validator = PathValidator::new();
    
    let dangerous_chars = vec![
        "file<.txt",          // Less than
        "file>.txt",          // Greater than  
        "file:.txt",          // Colon (Windows)
        "file\".txt",         // Quote
        "file|.txt",          // Pipe
        "file?.txt",          // Question mark
        "file*.txt",          // Asterisk
        "file\x00.txt",       // Null byte
        "file\x01.txt",       // Control character
        "file\x1F.txt",       // Control character
        "file\x7F.txt",       // DEL character
    ];
    
    for path in dangerous_chars {
        let result = validator.validate(path);
        assert!(result.is_err(), "Dangerous character should be blocked: {:?}", path);
    }
}

/// Test platform-specific edge cases
#[test]
fn test_platform_edge_cases() {
    let validator = PathValidator::new();
    
    // Windows-specific edge cases
    let windows_edge_cases = vec![
        "C:",                         // Drive letter only
        "C:file.txt",                // Relative to drive
        "\\",                        // Single backslash
        "\\file.txt",               // Leading backslash
        "file.txt\\",               // Trailing backslash
        "file.txt.",                // Trailing dot
        "file.txt ",                // Trailing space
        " file.txt",                // Leading space
    ];
    
    for case in windows_edge_cases {
        let result = validator.validate(case);
        // Most of these should be rejected
        assert!(result.is_err(), "Windows edge case should be handled: {}", case);
    }
    
    // Unix-specific edge cases
    let unix_edge_cases = vec![
        "/",                         // Root only
        "//",                        // Double slash root
        "///",                       // Triple slash root
        "/file.txt",                // Absolute path
        "~/.bashrc",                // Home directory
        "$HOME/file.txt",           // Environment variable
    ];
    
    for case in unix_edge_cases {
        let result = validator.validate(case);
        assert!(result.is_err(), "Unix edge case should be handled: {}", case);
    }
}

/// Test mixed attack scenarios
#[test]
fn test_mixed_attack_scenarios() {
    let validator = PathValidator::new();
    
    let mixed_attacks = vec![
        // Traversal + encoding
        "%2e%2e%2f../etc/passwd",
        "../%2e%2e/passwd",
        
        // Traversal + null byte
        "../etc/passwd%00.txt",
        "..%00/../etc/passwd",
        
        // Absolute + traversal
        "/etc/../passwd",
        "C:\\Windows\\..\\System32",
        
        // UNC + traversal  
        "\\\\server\\../share",
        
        // Reserved name + traversal
        "../CON",
        "..\\PRN.txt",
        
        // Unicode + traversal
        "\u{ff0e}\u{ff0e}/../etc/passwd",
        
        // Multiple encoding layers
        "%252e%252e%252f%2e%2e/passwd",
        
        // Case mixing + traversal
        "../ETC/PASSWD",
        "..\\WINDOWS\\system32",
    ];
    
    for attack in mixed_attacks {
        let result = validator.validate(attack);
        assert!(result.is_err(), "Mixed attack should be blocked: {}", attack);
    }
}

/// Test error message quality
#[test]
fn test_error_message_quality() {
    let validator = PathValidator::new();
    
    let test_cases = vec![
        ("../etc/passwd", &["traversal", "dangerous", "pattern"]),
        ("/etc/passwd", &["dangerous", "pattern", "root"]), 
        ("file%00.txt", &["null", "dangerous"]),
        ("%2e%2e%2f", &["encoding", "dangerous", "traversal"]),
        ("CON.txt", &["reserved", "filename", "dangerous"]),
        ("C:\\Windows", &["dangerous", "pattern"]),
        ("\\\\server\\share", &["dangerous", "pattern"]),
    ];
    
    for (input, expected_keywords) in test_cases {
        let result = validator.validate(input);
        assert!(result.is_err(), "Should be blocked: {}", input);
        
        if let Err(e) = result {
            let error_msg = format!("{}", e).to_lowercase();
            let matching_keywords: Vec<_> = expected_keywords.iter()
                .filter(|&keyword| error_msg.contains(keyword))
                .collect();
            
            assert!(
                !matching_keywords.is_empty(),
                "Error message should contain at least one expected keyword. Input: {}, Error: {}, Expected: {:?}",
                input, error_msg, expected_keywords
            );
        }
    }
}

/// Test warning generation
#[test]
fn test_warning_generation() {
    let validator = PathValidator::new();
    
    let test_cases = vec![
        // (input, expected_warning_keywords)
        ("data/résumé.txt", &["ASCII"]),                    // Non-ASCII chars
        ("data/verylongfilenamethatexceedsreasonablelimits.txt", &["filename"]), // Long filename
        ("a/b/c/d/e/f/g/h/i.txt", &["nested"]),           // Deep nesting
        ("data/file.exe", &["extension"]),                 // Unusual extension
    ];
    
    for (input, expected_keywords) in test_cases {
        let result = validator.validate(input);
        
        match result {
            Ok(validated) => {
                let has_expected_warning = expected_keywords.iter().any(|&keyword| {
                    validated.warnings.iter().any(|w| w.to_lowercase().contains(keyword))
                });
                
                assert!(
                    has_expected_warning,
                    "Should generate warning for: {} (expected keywords: {:?}, actual warnings: {:?})",
                    input, expected_keywords, validated.warnings
                );
            }
            Err(_) => {
                // If the path is blocked entirely, that's also acceptable
            }
        }
    }
}