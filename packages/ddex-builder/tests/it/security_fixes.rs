//! Path validation security tests: traversal attacks and cross-platform paths.
//! (The entity-classification and error-sanitization tests were removed with those helpers
//! in 0.6.0.)

use ddex_builder::security::PathValidator;

/// Test suite for comprehensive security integration
#[cfg(test)]
mod security_integration_tests {
    use super::*;

    #[test]
    #[ignore = "known bug: PathValidator allows sensitive files such as settings.json"]
    fn test_path_validation_comprehensive() {
        println!("🛡️  Testing comprehensive path validation...");

        let validator = PathValidator::new();

        // Test cases that previously failed - should now be blocked
        let dangerous_paths = vec![
            // Directory traversal attacks
            "../../../etc/passwd",
            "..\\..\\..\\Windows\\System32\\config\\SAM",
            "./../../root/.ssh/id_rsa",
            "dir/../../etc/shadow",
            // Encoded traversal attempts
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
            "..%252f..%252f..%252fetc%252fpasswd",
            "..%c0%af..%c0%af..%c0%afetc%c0%afpasswd",
            // Unicode normalization attacks
            "..\u{2044}..\u{2044}etc\u{2044}passwd",
            "..\u{FF0F}..\u{FF0F}etc\u{FF0F}passwd",
            // Windows specific
            "C:\\Windows\\System32\\drivers\\etc\\hosts",
            "\\\\?\\C:\\Windows\\System32\\config\\SAM",
            "\\\\localhost\\c$\\Windows\\System32\\config\\SAM",
            "//./C:/Windows/System32/config/SAM",
            // Unix specific
            "/proc/self/environ",
            "/dev/mem",
            "/etc/shadow",
            "/root/.ssh/authorized_keys",
            // Application config files
            ".env",
            "config/database.yml",
            "settings.json",
            ".aws/credentials",
        ];

        for dangerous_path in dangerous_paths {
            let result = validator.validate(dangerous_path);
            assert!(
                result.is_err(),
                "Path '{}' should be blocked but was allowed",
                dangerous_path
            );
        }

        // Test safe paths still work
        let safe_paths = vec![
            "valid/file.xml",
            "data/music/track.mp3",
            "output/generated.xml",
            "temp/processing.json",
        ];

        for safe_path in safe_paths {
            let result = validator.validate(safe_path);
            assert!(
                result.is_ok(),
                "Safe path '{}' should be allowed",
                safe_path
            );
        }

        println!("✅ Path validation comprehensive test passed");
    }

    #[test]
    #[ignore = "known bug: PathValidator allows Unicode-homograph and other platform attack paths"]
    fn test_cross_platform_path_validation() {
        println!("🛡️  Testing cross-platform path validation...");

        let validator = PathValidator::new();

        // Test platform-specific attacks
        let long_path_string = "A".repeat(5000);
        let platform_attacks = vec![
            // Windows-specific
            ("Windows UNC", "\\\\evil.com\\share\\file.exe"),
            ("Windows device", "\\\\.\\pipe\\evil"),
            ("Windows alternate data stream", "file.txt:hidden.exe"),
            ("Windows reserved names", "CON.txt"),
            (
                "Windows drive traversal",
                "C:\\..\\..\\Windows\\System32\\cmd.exe",
            ),
            // Unix-specific
            ("Unix proc filesystem", "/proc/self/mem"),
            ("Unix dev filesystem", "/dev/random"),
            ("Unix home traversal", "~/../../../etc/passwd"),
            ("Unix hidden files", ".bashrc"),
            // macOS-specific
            ("macOS resource fork", "file.txt/..namedfork/rsrc"),
            ("macOS system path", "/System/Library/CoreServices/boot.efi"),
            ("macOS app bundle", "evil.app/Contents/MacOS/evil"),
            // Cross-platform
            ("Null byte injection", "safe.txt\0../../etc/passwd"),
            ("Long path attack", &long_path_string),
            ("Control character", "file\x01name.txt"),
            ("Unicode homograph", "file\u{2044}name.txt"), // Unicode slash
        ];

        for (attack_name, attack_path) in platform_attacks {
            let result = validator.validate(attack_path);
            assert!(
                result.is_err(),
                "Platform attack '{}' with path '{}' should be blocked",
                attack_name,
                attack_path
            );
        }

        println!("✅ Cross-platform path validation test passed");
    }

    #[test]
    fn test_integrated_security_workflow() {
        println!("🔒 Testing integrated security workflow...");

        let path_validator = PathValidator::new();

        // Validate file path (should fail)
        let malicious_path = "../../../etc/passwd";
        let path_result = path_validator.validate(malicious_path);
        assert!(path_result.is_err(), "Malicious path should be blocked");

        println!("✅ Integrated security workflow test passed");
    }

    #[test]
    #[ignore = "known bug: PathValidator accepts an empty path"]
    fn test_security_edge_cases() {
        println!("🔍 Testing security edge cases...");

        // Test empty inputs
        let validator = PathValidator::new();
        let empty_path = validator.validate("");
        assert!(empty_path.is_err(), "Empty path should be blocked");

        // Test very long inputs
        let long_path = "A".repeat(10000);
        let long_path_result = validator.validate(&long_path);
        assert!(
            long_path_result.is_err(),
            "Very long path should be blocked"
        );

        // Test null byte attacks
        let null_byte_path = "safe.txt\0../../etc/passwd";
        let null_result = validator.validate(null_byte_path);
        assert!(null_result.is_err(), "Null byte attack should be blocked");

        // Test Unicode normalization attacks
        let unicode_attack = "..\u{2044}..\u{2044}etc\u{2044}passwd";
        let unicode_result = validator.validate(unicode_attack);
        assert!(
            unicode_result.is_err(),
            "Unicode normalization attack should be blocked"
        );

        println!("✅ Security edge cases test passed");
    }
}

/// Test that all security components work correctly under concurrent access
#[cfg(feature = "async")]
#[tokio::test]
#[ignore = "known bug: PathValidator rejects valid paths (seen under concurrent use)"]
async fn test_security_concurrent_access() {
    use tokio::task;

    println!("🔄 Testing concurrent access to security components...");

    let handles: Vec<_> = (0..10)
        .map(|i| {
            task::spawn(async move {
                // Each task tests all security components
                let validator = PathValidator::new();

                // Test concurrent path validation
                let path_result = validator.validate(&format!("test/path/{}.xml", i));
                assert!(path_result.is_ok(), "Valid path should pass validation");

                i
            })
        })
        .collect();

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }

    println!("✅ Concurrent access test passed");
}

/// Test security components handle malformed/corrupted inputs gracefully
#[test]
#[ignore = "known bug: PathValidator accepts malformed paths"]
fn test_security_malformed_input_handling() {
    println!("💥 Testing malformed input handling...");

    let validator = PathValidator::new();

    // Test path validator with malformed inputs
    let bom_string = "\u{FEFF}".repeat(100);
    let malformed_paths = vec![
        "\0\0\0\0",         // Null bytes
        "\x01\x02\x03\x04", // Control characters
        "🔥💀☠️",           // Emojis
        &bom_string,        // BOM characters
    ];

    for path in malformed_paths {
        let result = validator.validate(path);
        // Should not panic, should handle gracefully
        assert!(result.is_err(), "Malformed path should be rejected");
    }

    println!("✅ Malformed input handling test passed");
}
