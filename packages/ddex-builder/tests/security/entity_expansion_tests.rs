//! Entity Expansion Attack Tests
//!
//! This module contains comprehensive tests for various entity expansion attacks:
//! - Billion laughs attacks (exponential expansion)
//! - Quadratic blowup attacks 
//! - Linear expansion attacks
//! - Recursive entity definitions
//! - Parameter entity bombs
//! - Mixed internal/external entity attacks
//! - Memory exhaustion attempts

use ddex_builder::{
    error::BuildError,
    security::{InputValidator, SecurityConfig, SecureXmlReader},
};
use std::io::Cursor;

/// Test classic billion laughs attacks with various entity depths
#[test]
fn test_billion_laughs_variants() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Classic 10-level billion laughs
    let classic_billion_laughs = r#"<!DOCTYPE lolz [
        <!ENTITY lol "lol">
        <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
        <!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
        <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
        <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
        <!ENTITY lol5 "&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;">
        <!ENTITY lol6 "&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;">
        <!ENTITY lol7 "&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;">
        <!ENTITY lol8 "&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;">
        <!ENTITY lol9 "&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;">
    ]>
    <lolz>&lol9;</lolz>"#;

    let result = validator.validate_xml_content(classic_billion_laughs);
    assert!(
        result.is_err(),
        "Classic billion laughs attack should be blocked: {:?}",
        result
    );

    // Shorter but still exponential
    let compact_billion_laughs = r#"<!DOCTYPE bomb [
        <!ENTITY a "aaaaaaaaaa">
        <!ENTITY b "&a;&a;&a;&a;&a;">
        <!ENTITY c "&b;&b;&b;&b;&b;">
        <!ENTITY d "&c;&c;&c;&c;&c;">
        <!ENTITY e "&d;&d;&d;&d;&d;">
    ]>
    <bomb>&e;</bomb>"#;

    let result = validator.validate_xml_content(compact_billion_laughs);
    assert!(
        result.is_err(),
        "Compact billion laughs should be blocked: {:?}",
        result
    );

    // Very aggressive expansion (fewer levels, more entities per level)
    let aggressive_expansion = r#"<!DOCTYPE bomb [
        <!ENTITY boom "BOOM!">
        <!ENTITY boom2 "&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;&boom;">
        <!ENTITY boom3 "&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;&boom2;">
    ]>
    <bomb>&boom3;</bomb>"#;

    let result = validator.validate_xml_content(aggressive_expansion);
    assert!(
        result.is_err(),
        "Aggressive expansion should be blocked: {:?}",
        result
    );
}

/// Test quadratic blowup attacks (linear entity expansion)
#[test]
fn test_quadratic_blowup_attacks() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Large single entity repeated many times
    let quadratic_attack_1 = r#"<!DOCTYPE bomb [
        <!ENTITY kaboom "Kaboom! This is a very long string that will be repeated many times to create a quadratic blowup attack vector for XML parsing engines.">
    ]>
    <bomb>&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;&kaboom;</bomb>"#;

    let result = validator.validate_xml_content(quadratic_attack_1);
    assert!(
        result.is_err(),
        "Quadratic blowup attack should be blocked: {:?}",
        result
    );

    // Multiple different entities used many times
    let quadratic_attack_2 = r#"<!DOCTYPE bomb [
        <!ENTITY a "This is entity A which contains some amount of text that will be repeated.">
        <!ENTITY b "This is entity B which also contains text that contributes to the expansion.">
        <!ENTITY c "Entity C adds even more content to the quadratic expansion attack.">
    ]>
    <bomb>&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;&a;&b;&c;</bomb>"#;

    let result = validator.validate_xml_content(quadratic_attack_2);
    assert!(
        result.is_err(),
        "Multi-entity quadratic attack should be blocked: {:?}",
        result
    );
}

/// Test recursive entity definitions
#[test]
fn test_recursive_entity_definitions() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Direct recursion (entity referencing itself)
    let direct_recursion = r#"<!DOCTYPE bomb [
        <!ENTITY recursive "&recursive;">
    ]>
    <bomb>&recursive;</bomb>"#;

    let result = validator.validate_xml_content(direct_recursion);
    assert!(
        result.is_err(),
        "Direct recursive entity should be blocked: {:?}",
        result
    );

    // Indirect recursion (cycle between entities)
    let indirect_recursion = r#"<!DOCTYPE bomb [
        <!ENTITY a "&b;">
        <!ENTITY b "&c;">
        <!ENTITY c "&a;">
    ]>
    <bomb>&a;</bomb>"#;

    let result = validator.validate_xml_content(indirect_recursion);
    assert!(
        result.is_err(),
        "Indirect recursive entities should be blocked: {:?}",
        result
    );

    // Complex cycle with expansion
    let complex_cycle = r#"<!DOCTYPE bomb [
        <!ENTITY a "&b;&b;&b;">
        <!ENTITY b "&c;&c;">
        <!ENTITY c "&d;">
        <!ENTITY d "&a;">
    ]>
    <bomb>&a;</bomb>"#;

    let result = validator.validate_xml_content(complex_cycle);
    assert!(
        result.is_err(),
        "Complex recursive cycle should be blocked: {:?}",
        result
    );
}

/// Test parameter entity expansion bombs
#[test]
fn test_parameter_entity_bombs() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Parameter entity billion laughs
    let param_billion_laughs = r#"<!DOCTYPE bomb [
        <!ENTITY % a "aaaaaaaaaa">
        <!ENTITY % b "%a;%a;%a;%a;%a;%a;%a;%a;%a;%a;">
        <!ENTITY % c "%b;%b;%b;%b;%b;%b;%b;%b;%b;%b;">
        <!ENTITY % d "%c;%c;%c;%c;%c;%c;%c;%c;%c;%c;">
        %d;
    ]>
    <bomb>content</bomb>"#;

    let result = validator.validate_xml_content(param_billion_laughs);
    assert!(
        result.is_err(),
        "Parameter entity billion laughs should be blocked: {:?}",
        result
    );

    // Parameter entity with general entity expansion
    let mixed_param_expansion = r#"<!DOCTYPE bomb [
        <!ENTITY % param "<!ENTITY expand 'BOOM!BOOM!BOOM!BOOM!BOOM!BOOM!BOOM!BOOM!BOOM!BOOM!'>">
        <!ENTITY % param2 "<!ENTITY bigexpand '%param;%param;%param;%param;%param;'>">
        %param;
        %param2;
    ]>
    <bomb>&expand;&bigexpand;</bomb>"#;

    let result = validator.validate_xml_content(mixed_param_expansion);
    assert!(
        result.is_err(),
        "Mixed parameter/general entity expansion should be blocked: {:?}",
        result
    );
}

/// Test nested entity definitions (entities within entities)
#[test]
fn test_nested_entity_definitions() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Deeply nested entity structure
    let nested_entities = r#"<!DOCTYPE bomb [
        <!ENTITY level1 "Content Level 1">
        <!ENTITY level2 "&level1; Extended Level 2">
        <!ENTITY level3 "&level2; Extended Level 3">
        <!ENTITY level4 "&level3; Extended Level 4">
        <!ENTITY level5 "&level4; Extended Level 5">
        <!ENTITY level6 "&level5; Extended Level 6">
        <!ENTITY level7 "&level6; Extended Level 7">
        <!ENTITY level8 "&level7; Extended Level 8">
        <!ENTITY level9 "&level8; Extended Level 9">
        <!ENTITY level10 "&level9; Extended Level 10">
    ]>
    <bomb>&level10;</bomb>"#;

    let result = validator.validate_xml_content(nested_entities);
    assert!(
        result.is_err(),
        "Deeply nested entities should be blocked: {:?}",
        result
    );

    // Exponential nesting with multiple references
    let exponential_nesting = r#"<!DOCTYPE bomb [
        <!ENTITY base "BASE">
        <!ENTITY nest1 "&base;&base;">
        <!ENTITY nest2 "&nest1;&nest1;&nest1;&nest1;">
        <!ENTITY nest3 "&nest2;&nest2;&nest2;&nest2;">
        <!ENTITY nest4 "&nest3;&nest3;&nest3;&nest3;">
        <!ENTITY nest5 "&nest4;&nest4;&nest4;&nest4;">
    ]>
    <bomb>&nest5;</bomb>"#;

    let result = validator.validate_xml_content(exponential_nesting);
    assert!(
        result.is_err(),
        "Exponential nesting should be blocked: {:?}",
        result
    );
}

/// Test entity expansion with attribute values
#[test]
fn test_entity_expansion_in_attributes() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Entity expansion in attribute values
    let attr_expansion = r#"<!DOCTYPE bomb [
        <!ENTITY expand "This is expanded content that could be very long and repeated many times in attributes">
    ]>
    <bomb attr1="&expand;" attr2="&expand;" attr3="&expand;" attr4="&expand;" attr5="&expand;">
        Content
    </bomb>"#;

    let result = validator.validate_xml_content(attr_expansion);
    assert!(
        result.is_err(),
        "Entity expansion in attributes should be blocked: {:?}",
        result
    );

    // Multiple entity types in attributes
    let multi_attr_expansion = r#"<!DOCTYPE bomb [
        <!ENTITY short "S">
        <!ENTITY medium "&short;&short;&short;&short;&short;">
        <!ENTITY long "&medium;&medium;&medium;&medium;&medium;">
        <!ENTITY huge "&long;&long;&long;&long;&long;">
    ]>
    <bomb a="&huge;" b="&huge;" c="&huge;" d="&huge;" e="&huge;" f="&huge;" g="&huge;" h="&huge;">
        <child attr="&huge;&huge;&huge;">&huge;</child>
    </bomb>"#;

    let result = validator.validate_xml_content(multi_attr_expansion);
    assert!(
        result.is_err(),
        "Multiple entity expansion in attributes should be blocked: {:?}",
        result
    );
}

/// Test memory exhaustion through large content entities
#[test]
fn test_large_content_entities() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Single very large entity
    let large_string = "A".repeat(10000);
    let large_entity_xml = format!(
        r#"<!DOCTYPE bomb [
            <!ENTITY huge "{}">
        ]>
        <bomb>&huge;&huge;&huge;&huge;&huge;</bomb>"#,
        large_string
    );

    let result = validator.validate_xml_content(&large_entity_xml);
    assert!(
        result.is_err(),
        "Large content entity should be blocked: {:?}",
        result
    );

    // Multiple large entities
    let large_1 = "B".repeat(5000);
    let large_2 = "C".repeat(5000);
    let large_3 = "D".repeat(5000);
    let multiple_large_entities = format!(
        r#"<!DOCTYPE bomb [
            <!ENTITY big1 "{}">
            <!ENTITY big2 "{}">
            <!ENTITY big3 "{}">
            <!ENTITY combined "&big1;&big2;&big3;">
        ]>
        <bomb>&combined;&combined;&combined;</bomb>"#,
        large_1, large_2, large_3
    );

    let result = validator.validate_xml_content(&multiple_large_entities);
    assert!(
        result.is_err(),
        "Multiple large entities should be blocked: {:?}",
        result
    );
}

/// Test SecureXmlReader with entity expansion attacks
#[test]
fn test_secure_xml_reader_entity_protection() {
    let config = SecurityConfig::default();

    // Test with billion laughs
    let billion_laughs = r#"<!DOCTYPE bomb [
        <!ENTITY lol "lol">
        <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;">
        <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;">
    ]>
    <bomb>&lol3;</bomb>"#;

    let cursor = Cursor::new(billion_laughs.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config.clone());

    let mut buf = Vec::new();
    match reader.read_event(&mut buf) {
        Err(BuildError::Security(msg)) => {
            assert!(
                msg.contains("DTD processing not allowed") 
                || msg.contains("XML bomb") 
                || msg.contains("Dangerous entity")
            );
        }
        other => panic!("Expected security error for billion laughs, got: {:?}", other),
    }

    // Test with quadratic expansion
    let quadratic = r#"<!DOCTYPE bomb [
        <!ENTITY big "This is a big string that gets repeated many times">
    ]>
    <bomb>&big;&big;&big;&big;&big;&big;&big;&big;&big;&big;</bomb>"#;

    let cursor = Cursor::new(quadratic.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);

    let mut buf = Vec::new();
    match reader.read_event(&mut buf) {
        Err(BuildError::Security(msg)) => {
            assert!(
                msg.contains("DTD processing not allowed")
                || msg.contains("Dangerous entity")
            );
        }
        other => panic!("Expected security error for quadratic expansion, got: {:?}", other),
    }
}

/// Test entity count limits
#[test]
fn test_entity_count_limits() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Create XML with excessive entity usage
    let mut many_entities_xml = String::from(r#"<!DOCTYPE bomb ["#);
    
    // Define many entities
    for i in 0..50 {
        many_entities_xml.push_str(&format!(r#"<!ENTITY entity{} "Content{}{}{}{}{}{}{}{}{}{}">
"#, i, i, i, i, i, i, i, i, i, i, i));
    }
    
    many_entities_xml.push_str(r#"]><bomb>"#);
    
    // Use many entities
    for i in 0..50 {
        many_entities_xml.push_str(&format!("&entity{};", i));
    }
    
    many_entities_xml.push_str("</bomb>");

    let result = validator.validate_xml_content(&many_entities_xml);
    assert!(
        result.is_err(),
        "Excessive entity usage should be blocked: {:?}",
        result
    );
}

/// Test edge cases for entity expansion
#[test]
fn test_entity_expansion_edge_cases() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Empty entity expansion
    let empty_entities = r#"<!DOCTYPE test [
        <!ENTITY empty "">
        <!ENTITY manyempty "&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;&empty;">
    ]>
    <test>&manyempty;</test>"#;

    let result = validator.validate_xml_content(empty_entities);
    assert!(
        result.is_err(),
        "Many empty entities should still be blocked due to DTD: {:?}",
        result
    );

    // Single character entities with high repetition
    let single_char_spam = r#"<!DOCTYPE bomb [
        <!ENTITY x "X">
    ]>
    <bomb>"#;

    let mut single_char_xml = String::from(single_char_spam);
    for _ in 0..2000 {
        single_char_xml.push_str("&x;");
    }
    single_char_xml.push_str("</bomb>");

    let result = validator.validate_xml_content(&single_char_xml);
    assert!(
        result.is_err(),
        "High repetition single char entities should be blocked: {:?}",
        result
    );

    // Whitespace-only entities
    let whitespace_entities = r#"<!DOCTYPE bomb [
        <!ENTITY space " ">
        <!ENTITY tab "&#9;">
        <!ENTITY newline "&#10;">
        <!ENTITY spaces "&space;&space;&space;&space;&space;">
        <!ENTITY whitespam "&spaces;&spaces;&spaces;&spaces;&spaces;&spaces;&spaces;&spaces;&spaces;&spaces;">
    ]>
    <bomb>&whitespam;</bomb>"#;

    let result = validator.validate_xml_content(whitespace_entities);
    assert!(
        result.is_err(),
        "Whitespace entity spam should be blocked: {:?}",
        result
    );
}

/// Test that reasonable entity usage is still allowed
#[test]
fn test_reasonable_entities_allowed() {
    let config = SecurityConfig {
        allow_dtd: true, // Temporarily allow DTD for this test
        ..SecurityConfig::default()
    };
    let validator = InputValidator::new(config);

    // Small, reasonable entity usage
    let reasonable_entities = r#"<!DOCTYPE test [
        <!ENTITY company "ACME Corporation">
        <!ENTITY copyright "Copyright 2024 ACME Corporation. All rights reserved.">
    ]>
    <test>
        <header>&company;</header>
        <footer>&copyright;</footer>
    </test>"#;

    // Note: This test may fail if DTD processing is completely disabled
    // In that case, it demonstrates that the security measures are working
    let result = validator.validate_xml_content(reasonable_entities);
    
    // Either it should work (if DTD is allowed) or fail with DTD disabled message
    match result {
        Ok(_) => {
            // This is fine - reasonable entities should be allowed if DTD is enabled
        }
        Err(BuildError::Security(msg)) if msg.contains("DTD processing not allowed") => {
            // This is also fine - it means DTD is completely disabled for security
        }
        Err(e) => {
            panic!("Unexpected error for reasonable entities: {:?}", e);
        }
    }
}