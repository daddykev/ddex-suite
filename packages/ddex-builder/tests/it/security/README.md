# DDEX Builder Security Test Suite

This directory contains comprehensive security tests for XXE (XML External Entity) attack prevention and other security measures in the DDEX Builder.

## Overview

The security test suite implements comprehensive tests for various types of XML security attacks, with a focus on XXE (XML External Entity) prevention. The tests are organized into several modules, each covering different aspects of security:

## Test Modules

### 1. XXE Prevention Tests (`xxe_prevention_tests.rs`)
- **External DTD references** - Tests blocking of SYSTEM/PUBLIC DTD references
- **Billion laughs attacks** - Tests prevention of exponential entity expansion
- **Parameter entity attacks** - Tests blocking of parameter entity file disclosure
- **Network request prevention** - Tests blocking of HTTP/HTTPS/FTP requests
- **Mixed attack vectors** - Tests combinations of different attack types
- **Edge cases** - Tests various evasion techniques and boundary conditions

### 2. Entity Expansion Tests (`entity_expansion_tests.rs`) 
- **Classic billion laughs** - Tests various forms of exponential entity expansion
- **Quadratic blowup attacks** - Tests linear entity expansion attacks
- **Recursive entity definitions** - Tests circular entity references
- **Parameter entity bombs** - Tests parameter entity expansion attacks
- **Memory exhaustion** - Tests protection against large content entities
- **Entity count limits** - Tests enforcement of entity usage limits

### 3. Parameter Entity Tests (`parameter_entity_tests.rs`)
- **File disclosure attempts** - Tests blocking of file:// URI access
- **Network request attacks** - Tests blocking of HTTP/HTTPS requests
- **Nested parameter entities** - Tests complex nested entity structures  
- **Injection attacks** - Tests SQL/command injection via entities
- **Parameter entity loops** - Tests circular parameter entity references
- **Out-of-band (OOB) exfiltration** - Tests data exfiltration attempts

### 4. Malicious Payload Tests (`malicious_payload_tests.rs`)
- **XML injection** - Tests injection of malicious XML in DDEX fields
- **Script injection** - Tests XSS and script injection attempts
- **SQL injection** - Tests SQL injection in metadata fields  
- **Path traversal** - Tests directory traversal attempts
- **Command injection** - Tests OS command injection
- **CDATA abuse** - Tests malicious content in CDATA sections
- **Encoding attacks** - Tests various encoding manipulation attacks
- **Unicode attacks** - Tests Unicode normalization vulnerabilities

### 5. Builder Security Tests (`builder_security_tests.rs`)
- **Input validation** - Tests validation during the build process
- **Output sanitization** - Tests safe handling of generated XML
- **Memory protection** - Tests protection against memory exhaustion
- **Rate limiting** - Tests DoS protection mechanisms
- **Secure logging** - Tests that logs don't leak sensitive information
- **Temporary file handling** - Tests secure temp file management

### 6. Integration Tests (`integration_tests.rs`)
- **End-to-end security** - Tests complete security pipeline
- **Concurrent security** - Tests security under concurrent load
- **Performance impact** - Tests that security doesn't create DoS vulnerabilities
- **Error handling** - Tests secure error messages
- **Configuration testing** - Tests different security configurations
- **Attack simulation** - Tests comprehensive attack campaigns

## Security Test Infrastructure

### Test Utilities (`mod.rs`)
The module provides helper functions and utilities:

- `test_security_config()` - Default security configuration for tests
- `restrictive_security_config()` - High-security configuration
- `permissive_security_config()` - Permissive configuration for edge case testing
- `assert_xxe_blocked()` - Helper to verify XXE attacks are blocked
- `assert_valid_xml_allowed()` - Helper to verify valid XML is not blocked
- `generate_xxe_payloads()` - Generator for common XXE attack payloads
- `generate_valid_xml_payloads()` - Generator for valid XML test cases

## Security Principles Tested

The test suite validates the following security principles:

### 1. Defense in Depth
- Multiple layers of security validation
- Input validation, content inspection, and output sanitization
- Protection at both XML parsing and content validation levels

### 2. Fail-Safe Defaults
- DTD processing disabled by default
- External entity resolution disabled
- Conservative size and complexity limits

### 3. Input Validation
- All input is validated and sanitized
- Rejection of malicious patterns and structures
- Size and complexity limits enforced

### 4. Output Encoding
- All output is properly encoded for safety
- Sensitive data detection and redaction
- Malformed output rejection

### 5. Least Privilege
- Minimal permissions and capabilities
- No unnecessary file system or network access
- Restricted processing contexts

### 6. Error Handling
- Secure error messages without information disclosure
- No sensitive data in error messages or logs
- Consistent error handling across all components

## Attack Vectors Covered

### XXE (XML External Entity) Attacks
- **File Disclosure**: `file:///etc/passwd`, `file:///C:/Windows/system32/config/sam`
- **Network Requests**: `http://attacker.com/steal`, `https://evil.com/exfil`
- **Protocol Abuse**: `ftp://`, `gopher://`, `ldap://`, `jar://`
- **Parameter Entities**: Complex nested parameter entity attacks
- **Blind XXE**: Time-based and error-based blind attacks

### Entity Expansion Attacks
- **Billion Laughs**: Classic exponential expansion attacks
- **Quadratic Blowup**: Linear expansion with high repetition
- **XML Bombs**: Memory exhaustion through entity expansion
- **Recursive Entities**: Circular entity references

### Injection Attacks
- **XML Injection**: Malicious XML in data fields
- **Script Injection**: XSS attempts in XML content
- **SQL Injection**: Database injection through XML data
- **Command Injection**: OS command injection attempts
- **Path Traversal**: Directory traversal attacks

### Denial of Service Attacks
- **Memory Exhaustion**: Large document attacks
- **CPU Exhaustion**: Complex parsing attacks  
- **Infinite Loops**: Recursive structure attacks
- **Rate Limiting**: High-volume request attacks

## Running the Tests

### Run All Security Tests
```bash
cargo test --test security_integration_test
```

### Run Specific Test Modules  
```bash
cargo test xxe_prevention
cargo test entity_expansion
cargo test parameter_entity
cargo test malicious_payload
cargo test builder_security
```

### Run Individual Security Tests
```bash
cargo test test_xxe_prevention_basic
cargo test test_billion_laughs_prevention  
cargo test test_parameter_entity_file_disclosure
```

### Run with Specific Security Configuration
```bash
cargo test security_config_variations
```

## Test Results and Coverage

The security test suite provides comprehensive coverage of XML security vulnerabilities:

### ✅ Currently Passing Tests
- Basic XXE prevention
- SecureXmlReader protection
- Security configuration limits
- URL validation
- Normal operation preservation
- Performance under attack load

### ⚠️ Tests Revealing Issues
- Path validation (needs platform-specific handling)
- Error message formatting (needs consistent security messages)
- String sanitization (needs proper dangerous content handling)
- Entity handling (needs refined detection of valid vs malicious entities)

## Security Implementation Status

### ✅ Implemented Security Measures
- **DTD Processing Disabled**: All DTD processing is disabled by default
- **Entity Detection**: Pattern-based detection of dangerous entities
- **Size Limits**: Input size restrictions to prevent DoS
- **Depth Limits**: XML nesting depth restrictions
- **Rate Limiting**: Request rate limiting to prevent abuse
- **Input Validation**: Content validation and sanitization
- **Output Sanitization**: Safe output generation

### 🔄 Areas for Improvement
- **Platform-specific Path Validation**: Better handling of Windows vs Unix paths
- **Error Message Standardization**: Consistent security error messages
- **Entity Classification**: More precise detection of malicious vs valid entities
- **Content Sanitization**: Proper sanitization vs rejection strategies
- **Performance Optimization**: Minimize security overhead

## Contributing to Security Tests

When adding new security tests:

1. **Follow the modular structure** - Add tests to the appropriate module
2. **Use helper functions** - Leverage existing test utilities
3. **Test both positive and negative cases** - Ensure valid content isn't blocked
4. **Include edge cases** - Test boundary conditions and evasion attempts
5. **Document attack vectors** - Clearly explain what each test covers
6. **Performance consideration** - Ensure tests don't create DoS vulnerabilities

## Security Test Philosophy

The security test suite follows a "security by verification" approach:

1. **Assume Attacks Will Happen** - Test common and novel attack vectors
2. **Verify Defense Mechanisms** - Ensure security measures actually work
3. **Test Edge Cases** - Verify security at boundaries and corner cases
4. **Performance Awareness** - Ensure security doesn't create new vulnerabilities
5. **Layered Defense** - Test multiple security layers working together
6. **Real-World Relevance** - Focus on actual attack techniques used in the wild

## References

- [OWASP XML Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/XML_Security_Cheat_Sheet.html)
- [XML External Entity (XXE) Processing](https://owasp.org/www-community/vulnerabilities/XML_External_Entity_(XXE)_Processing)
- [XML Bomb](https://en.wikipedia.org/wiki/Billion_laughs_attack)
- [DDEX Security Best Practices](https://ddex.net/security)

This security test suite ensures that the DDEX Builder is resilient against XML-based attacks and provides a secure foundation for DDEX XML processing.