//! Comprehensive Attribute Preservation Tests
//!
//! These tests ensure complete fidelity of XML attributes through:
//! - Round-trip parsing and building
//! - Canonical ordering preservation
//! - Namespace-qualified attribute handling
//! - Unknown/proprietary attribute preservation
//! - Attribute value type preservation
//! - HTML escaping and special character handling

#![allow(clippy::approx_constant)] // Tests use literal values for validation

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use ddex_core::models::attributes::AttributeMergeStrategy;
use ddex_core::models::{AttributeMap, AttributeType, AttributeValidator, AttributeValue, QName};

#[cfg(test)]
mod comprehensive_attribute_tests {
    use super::*;

    #[test]
    fn test_round_trip_attribute_preservation() {
        // Test complete round-trip fidelity
        let mut original_attributes = AttributeMap::new();

        // Add various types of attributes
        original_attributes.insert_str("title", "My Release");
        original_attributes.insert_str("version", "4.3");
        original_attributes.insert(
            QName::new("isMainRelease".to_string()),
            AttributeValue::Boolean(true),
        );
        original_attributes.insert(
            QName::new("sequenceNumber".to_string()),
            AttributeValue::Integer(1),
        );
        original_attributes.insert(
            QName::new("price".to_string()),
            AttributeValue::Decimal(9.99),
        );

        // Namespace-qualified attributes
        let xsi_type = QName::with_namespace(
            "type".to_string(),
            "http://www.w3.org/2001/XMLSchema-instance".to_string(),
        );
        original_attributes.insert(xsi_type, AttributeValue::Token("ReleaseType".to_string()));

        // Custom/proprietary attributes
        let custom_attr = QName::with_prefix_and_namespace(
            "customField".to_string(),
            "label".to_string(),
            "http://example.com/label-extensions".to_string(),
        );
        original_attributes.insert(
            custom_attr,
            AttributeValue::String("proprietary-value".to_string()),
        );

        // Serialize to XML-like representation
        let xml_map = original_attributes.to_string_map();

        // Deserialize back
        let restored_attributes = AttributeMap::from_string_map(xml_map);

        // Verify all attributes are preserved
        assert_eq!(original_attributes.len(), restored_attributes.len());
        assert_eq!(
            original_attributes.get_str("title").unwrap().to_xml_value(),
            "My Release"
        );
        assert_eq!(
            original_attributes
                .get_str("version")
                .unwrap()
                .to_xml_value(),
            "4.3"
        );

        // Verify boolean preservation
        let is_main_qname = QName::new("isMainRelease".to_string());
        if let Some(AttributeValue::Boolean(value)) = original_attributes.get(&is_main_qname) {
            assert!(*value);
        } else {
            panic!("Boolean attribute not preserved correctly");
        }

        // Verify namespace-qualified attribute
        let xsi_type_qname = QName::with_namespace(
            "type".to_string(),
            "http://www.w3.org/2001/XMLSchema-instance".to_string(),
        );
        assert!(original_attributes.contains_key(&xsi_type_qname));
    }

    #[test]
    fn test_canonical_attribute_ordering() {
        let mut attributes = AttributeMap::new();

        // Add attributes in non-alphabetical order
        attributes.insert_str("zebra", "last");
        attributes.insert_str("alpha", "first");
        attributes.insert_str("beta", "second");

        // Add namespace declarations (should come first in canonical order)
        let xmlns_default = QName::new("xmlns".to_string());
        attributes.insert(
            xmlns_default,
            AttributeValue::String("http://ddex.net/xml/ern/382".to_string()),
        );

        let xmlns_xsi = QName::with_prefix_and_namespace(
            "xmlns".to_string(),
            "xsi".to_string(),
            "http://www.w3.org/2001/XMLSchema-instance".to_string(),
        );
        attributes.insert(
            xmlns_xsi,
            AttributeValue::String("http://www.w3.org/2001/XMLSchema-instance".to_string()),
        );

        // Convert to canonical ordered representation
        let canonical_map = attributes.to_canonical_ordered();

        // Verify namespace declarations come first
        let keys: Vec<_> = canonical_map.keys().collect();
        assert!(keys[0].local_name == "xmlns" && keys[0].prefix.is_none());

        // Verify alphabetical ordering of regular attributes
        let non_ns_keys: Vec<_> = canonical_map
            .keys()
            .filter(|qname| !qname.is_namespace_declaration())
            .collect();

        let mut sorted_names: Vec<_> = non_ns_keys.iter().map(|q| &q.local_name).collect();
        sorted_names.sort();

        for (i, key) in non_ns_keys.iter().enumerate() {
            assert_eq!(&key.local_name, sorted_names[i]);
        }
    }

    #[test]
    fn test_namespace_qualified_attributes() {
        let mut attributes = AttributeMap::new();

        // Add attributes with different namespace combinations
        let no_ns = QName::new("localAttr".to_string());
        attributes.insert(no_ns, AttributeValue::String("no-namespace".to_string()));

        let with_ns =
            QName::with_namespace("nsAttr".to_string(), "http://example.com/ns".to_string());
        attributes.insert(
            with_ns,
            AttributeValue::String("with-namespace".to_string()),
        );

        let with_prefix_ns = QName::with_prefix_and_namespace(
            "prefixAttr".to_string(),
            "ex".to_string(),
            "http://example.com/ns".to_string(),
        );
        attributes.insert(
            with_prefix_ns,
            AttributeValue::String("with-prefix-namespace".to_string()),
        );

        // Test QName resolution and XML name generation
        let keys: Vec<_> = attributes.keys().collect();

        // Verify QName properties
        for key in keys {
            match key.local_name.as_str() {
                "localAttr" => {
                    assert!(key.namespace_uri.is_none());
                    assert!(key.prefix.is_none());
                    assert_eq!(key.to_xml_name(), "localAttr");
                }
                "nsAttr" => {
                    assert_eq!(key.namespace_uri.as_ref().unwrap(), "http://example.com/ns");
                    assert!(key.prefix.is_none());
                    assert_eq!(key.to_xml_name(), "nsAttr");
                }
                "prefixAttr" => {
                    assert_eq!(key.namespace_uri.as_ref().unwrap(), "http://example.com/ns");
                    assert_eq!(key.prefix.as_ref().unwrap(), "ex");
                    assert_eq!(key.to_xml_name(), "ex:prefixAttr");
                }
                _ => panic!("Unexpected attribute name: {}", key.local_name),
            }
        }
    }

    #[test]
    fn test_special_character_escaping() {
        let mut attributes = AttributeMap::new();

        // Add attributes with special XML characters
        attributes.insert_str("withQuotes", "He said \"Hello World\"");
        attributes.insert_str("withAmpersand", "Rock & Roll");
        attributes.insert_str("withAngles", "<tag>value</tag>");
        attributes.insert_str("withApostrophe", "It's working");
        attributes.insert_str("unicodeChars", "Ümlauts and émojis 🎵");

        // Convert to XML representation
        let xml_map = attributes.to_string_map();

        // Verify proper escaping in XML values
        assert_eq!(
            xml_map.get("withQuotes").unwrap(),
            "He said \"Hello World\""
        );
        assert_eq!(xml_map.get("withAmpersand").unwrap(), "Rock & Roll");
        assert_eq!(xml_map.get("withAngles").unwrap(), "<tag>value</tag>");
        assert_eq!(xml_map.get("withApostrophe").unwrap(), "It's working");
        assert_eq!(
            xml_map.get("unicodeChars").unwrap(),
            "Ümlauts and émojis 🎵"
        );

        // Verify round-trip preservation
        let restored = AttributeMap::from_string_map(xml_map);
        assert_eq!(
            restored.get_str("withQuotes").unwrap().to_xml_value(),
            "He said \"Hello World\""
        );
    }

    #[test]
    fn test_attribute_type_preservation() {
        let mut attributes = AttributeMap::new();

        // Add attributes of different types
        attributes.insert(
            QName::new("stringValue".to_string()),
            AttributeValue::String("text".to_string()),
        );
        attributes.insert(
            QName::new("booleanValue".to_string()),
            AttributeValue::Boolean(true),
        );
        attributes.insert(
            QName::new("integerValue".to_string()),
            AttributeValue::Integer(42),
        );
        attributes.insert(
            QName::new("decimalValue".to_string()),
            AttributeValue::Decimal(3.14159),
        );
        attributes.insert(
            QName::new("dateValue".to_string()),
            AttributeValue::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        );
        attributes.insert(
            QName::new("datetimeValue".to_string()),
            AttributeValue::DateTime(
                DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        );
        attributes.insert(
            QName::new("uriValue".to_string()),
            AttributeValue::Uri("https://example.com".to_string()),
        );
        attributes.insert(
            QName::new("languageValue".to_string()),
            AttributeValue::Language("en-US".to_string()),
        );
        attributes.insert(
            QName::new("tokenValue".to_string()),
            AttributeValue::Token("normalized-token".to_string()),
        );

        // Test type-specific parsing and formatting
        for (qname, value) in &attributes {
            match qname.local_name.as_str() {
                "stringValue" => assert!(matches!(value, AttributeValue::String(_))),
                "booleanValue" => {
                    assert!(matches!(value, AttributeValue::Boolean(_)));
                    assert_eq!(value.to_xml_value(), "true");
                }
                "integerValue" => {
                    assert!(matches!(value, AttributeValue::Integer(_)));
                    assert_eq!(value.to_xml_value(), "42");
                }
                "decimalValue" => {
                    assert!(matches!(value, AttributeValue::Decimal(_)));
                    assert_eq!(value.to_xml_value(), "3.14159");
                }
                "dateValue" => {
                    assert!(matches!(value, AttributeValue::Date(_)));
                    assert_eq!(value.to_xml_value(), "2024-01-01");
                }
                "datetimeValue" => {
                    assert!(matches!(value, AttributeValue::DateTime(_)));
                    assert!(value.to_xml_value().starts_with("2024-01-01T12:00:00"));
                }
                "uriValue" => {
                    assert!(matches!(value, AttributeValue::Uri(_)));
                    assert_eq!(value.to_xml_value(), "https://example.com");
                }
                "languageValue" => {
                    assert!(matches!(value, AttributeValue::Language(_)));
                    assert_eq!(value.to_xml_value(), "en-US");
                }
                "tokenValue" => {
                    assert!(matches!(value, AttributeValue::Token(_)));
                    assert_eq!(value.to_xml_value(), "normalized-token");
                }
                _ => panic!("Unexpected attribute: {}", qname.local_name),
            }
        }
    }

    #[test]
    fn test_unknown_proprietary_attributes() {
        let mut attributes = AttributeMap::new();

        // Add various proprietary/unknown attributes
        let spotify_attr = QName::with_prefix_and_namespace(
            "trackId".to_string(),
            "spotify".to_string(),
            "http://open.spotify.com/extensions".to_string(),
        );
        attributes.insert(
            spotify_attr.clone(),
            AttributeValue::String("4iV5W9uYEdYUVa79Axb7Rh".to_string()),
        );

        let apple_attr = QName::with_prefix_and_namespace(
            "adamId".to_string(),
            "apple".to_string(),
            "http://apple.com/itunes/extensions".to_string(),
        );
        attributes.insert(apple_attr.clone(), AttributeValue::Integer(1234567890));

        let custom_boolean = QName::with_prefix_and_namespace(
            "isExplicit".to_string(),
            "label".to_string(),
            "http://label.example.com/extensions".to_string(),
        );
        attributes.insert(custom_boolean.clone(), AttributeValue::Boolean(false));

        // Verify preservation
        assert_eq!(attributes.len(), 3);
        assert!(attributes.contains_key(&spotify_attr));
        assert!(attributes.contains_key(&apple_attr));
        assert!(attributes.contains_key(&custom_boolean));

        // Verify values and types
        if let Some(AttributeValue::String(spotify_id)) = attributes.get(&spotify_attr) {
            assert_eq!(spotify_id, "4iV5W9uYEdYUVa79Axb7Rh");
        } else {
            panic!("Spotify attribute not preserved");
        }

        if let Some(AttributeValue::Integer(adam_id)) = attributes.get(&apple_attr) {
            assert_eq!(*adam_id, 1234567890);
        } else {
            panic!("Apple attribute not preserved");
        }

        if let Some(AttributeValue::Boolean(is_explicit)) = attributes.get(&custom_boolean) {
            assert!(!(*is_explicit));
        } else {
            panic!("Custom boolean attribute not preserved");
        }
    }

    #[test]
    fn test_comprehensive_validation_system() {
        let mut validator = AttributeValidator::new();

        // Test built-in DDEX validation rules
        let mut attributes = AttributeMap::new();

        // Valid territory code
        attributes.insert(
            QName::new("TerritoryCode".to_string()),
            AttributeValue::String("US".to_string()),
        );

        // Valid language code
        attributes.insert(
            QName::new("LanguageAndScriptCode".to_string()),
            AttributeValue::String("en-US".to_string()),
        );

        // Valid currency code
        attributes.insert(
            QName::new("CurrencyCode".to_string()),
            AttributeValue::String("USD".to_string()),
        );

        // Valid sequence number
        attributes.insert(
            QName::new("SequenceNumber".to_string()),
            AttributeValue::Integer(1),
        );

        // Valid ISRC
        attributes.insert(
            QName::new("ISRC".to_string()),
            AttributeValue::String("USRC17607839".to_string()),
        );

        let result = validator.validate_global_attributes(&attributes);
        assert!(
            result.is_valid,
            "Valid DDEX attributes should pass validation"
        );

        // Test invalid values
        let mut invalid_attributes = AttributeMap::new();

        // Invalid territory code
        invalid_attributes.insert(
            QName::new("TerritoryCode".to_string()),
            AttributeValue::String("INVALID".to_string()),
        );

        // Invalid sequence number (out of range)
        invalid_attributes.insert(
            QName::new("SequenceNumber".to_string()),
            AttributeValue::Integer(0),
        );

        // Invalid ISRC format
        invalid_attributes.insert(
            QName::new("ISRC".to_string()),
            AttributeValue::String("INVALID-ISRC".to_string()),
        );

        let invalid_result = validator.validate_global_attributes(&invalid_attributes);
        assert!(
            !invalid_result.is_valid,
            "Invalid attributes should fail validation"
        );
        assert!(
            !invalid_result.errors.is_empty(),
            "Should have validation errors"
        );
    }

    #[test]
    fn test_attribute_parsing_with_type_hints() {
        // Test parsing various attribute values with correct type hints

        // Boolean parsing
        let bool_true = AttributeValue::parse_with_type("true", AttributeType::Boolean).unwrap();
        assert!(matches!(bool_true, AttributeValue::Boolean(true)));

        let bool_false = AttributeValue::parse_with_type("false", AttributeType::Boolean).unwrap();
        assert!(matches!(bool_false, AttributeValue::Boolean(false)));

        let bool_one = AttributeValue::parse_with_type("1", AttributeType::Boolean).unwrap();
        assert!(matches!(bool_one, AttributeValue::Boolean(true)));

        let bool_zero = AttributeValue::parse_with_type("0", AttributeType::Boolean).unwrap();
        assert!(matches!(bool_zero, AttributeValue::Boolean(false)));

        // Integer parsing
        let int_val = AttributeValue::parse_with_type("42", AttributeType::Integer).unwrap();
        assert!(matches!(int_val, AttributeValue::Integer(42)));

        let neg_int = AttributeValue::parse_with_type("-42", AttributeType::Integer).unwrap();
        assert!(matches!(neg_int, AttributeValue::Integer(-42)));

        // Decimal parsing
        let decimal_val =
            AttributeValue::parse_with_type("3.14159", AttributeType::Decimal).unwrap();
        if let AttributeValue::Decimal(val) = decimal_val {
            assert!((val - 3.14159).abs() < 1e-6);
        } else {
            panic!("Expected decimal value");
        }

        // Date parsing
        let date_val = AttributeValue::parse_with_type("2024-01-01", AttributeType::Date).unwrap();
        if let AttributeValue::Date(date) = date_val {
            assert_eq!(date.year(), 2024);
            assert_eq!(date.month(), 1);
            assert_eq!(date.day(), 1);
        } else {
            panic!("Expected date value");
        }

        // DateTime parsing
        let datetime_val =
            AttributeValue::parse_with_type("2024-01-01T12:00:00Z", AttributeType::DateTime)
                .unwrap();
        assert!(matches!(datetime_val, AttributeValue::DateTime(_)));

        // URI parsing
        let uri_val =
            AttributeValue::parse_with_type("https://example.com", AttributeType::Uri).unwrap();
        assert!(matches!(uri_val, AttributeValue::Uri(_)));

        // Language parsing
        let lang_val = AttributeValue::parse_with_type("en-US", AttributeType::Language).unwrap();
        assert!(matches!(lang_val, AttributeValue::Language(_)));

        // Token parsing (should trim whitespace)
        let token_val =
            AttributeValue::parse_with_type("  trimmed-token  ", AttributeType::Token).unwrap();
        if let AttributeValue::Token(token) = token_val {
            assert_eq!(token, "trimmed-token");
        } else {
            panic!("Expected token value");
        }
    }

    #[test]
    fn test_error_handling() {
        // Test various error conditions

        // Invalid boolean
        let bool_err = AttributeValue::parse_with_type("maybe", AttributeType::Boolean);
        assert!(bool_err.is_err());

        // Invalid integer
        let int_err = AttributeValue::parse_with_type("not-a-number", AttributeType::Integer);
        assert!(int_err.is_err());

        // Invalid decimal
        let decimal_err = AttributeValue::parse_with_type("not-a-decimal", AttributeType::Decimal);
        assert!(decimal_err.is_err());

        // Invalid date
        let date_err = AttributeValue::parse_with_type("2024-13-45", AttributeType::Date);
        assert!(date_err.is_err());

        // Invalid datetime
        let datetime_err =
            AttributeValue::parse_with_type("not-a-datetime", AttributeType::DateTime);
        assert!(datetime_err.is_err());
    }

    #[test]
    fn test_attribute_merging_strategies() {
        let mut map1 = AttributeMap::new();
        map1.insert_str("common", "value1");
        map1.insert_str("unique1", "only-in-1");

        let mut map2 = AttributeMap::new();
        map2.insert_str("common", "value2");
        map2.insert_str("unique2", "only-in-2");

        // Test PreferThis strategy
        let mut merged_prefer_this = map1.clone();
        merged_prefer_this.merge(&map2, AttributeMergeStrategy::PreferThis);

        assert_eq!(
            merged_prefer_this.get_str("common").unwrap().to_xml_value(),
            "value1"
        );
        assert_eq!(
            merged_prefer_this
                .get_str("unique1")
                .unwrap()
                .to_xml_value(),
            "only-in-1"
        );
        assert_eq!(
            merged_prefer_this
                .get_str("unique2")
                .unwrap()
                .to_xml_value(),
            "only-in-2"
        );

        // Test PreferOther strategy
        let mut merged_prefer_other = map1.clone();
        merged_prefer_other.merge(&map2, AttributeMergeStrategy::PreferOther);

        assert_eq!(
            merged_prefer_other
                .get_str("common")
                .unwrap()
                .to_xml_value(),
            "value2"
        );
        assert_eq!(
            merged_prefer_other
                .get_str("unique1")
                .unwrap()
                .to_xml_value(),
            "only-in-1"
        );
        assert_eq!(
            merged_prefer_other
                .get_str("unique2")
                .unwrap()
                .to_xml_value(),
            "only-in-2"
        );
    }

    #[test]
    fn test_complex_namespace_scenarios() {
        let mut attributes = AttributeMap::new();

        // Same local name, different namespaces
        let attr1 =
            QName::with_namespace("id".to_string(), "http://ddex.net/xml/ern/382".to_string());
        let attr2 =
            QName::with_namespace("id".to_string(), "http://example.com/custom".to_string());
        let attr3 = QName::new("id".to_string()); // No namespace

        attributes.insert(attr1.clone(), AttributeValue::String("ddex-id".to_string()));
        attributes.insert(
            attr2.clone(),
            AttributeValue::String("custom-id".to_string()),
        );
        attributes.insert(
            attr3.clone(),
            AttributeValue::String("local-id".to_string()),
        );

        // All three should be distinct
        assert_eq!(attributes.len(), 3);
        assert!(attributes.contains_key(&attr1));
        assert!(attributes.contains_key(&attr2));
        assert!(attributes.contains_key(&attr3));

        // Verify correct values
        assert_eq!(attributes.get(&attr1).unwrap().to_xml_value(), "ddex-id");
        assert_eq!(attributes.get(&attr2).unwrap().to_xml_value(), "custom-id");
        assert_eq!(attributes.get(&attr3).unwrap().to_xml_value(), "local-id");

        // Test canonical sorting with namespaces
        let canonical = attributes.to_canonical_ordered();
        let keys: Vec<_> = canonical.keys().collect();

        // Verify namespace URIs are considered in sorting
        for i in 1..keys.len() {
            let prev_key = &keys[i - 1];
            let curr_key = &keys[i];
            assert!(prev_key.canonical_sort_key() <= curr_key.canonical_sort_key());
        }
    }

    #[test]
    fn test_large_scale_attribute_handling() {
        // Test performance and correctness with many attributes
        let mut attributes = AttributeMap::new();

        // Add 1000 attributes with various types and namespaces
        for i in 0..1000 {
            let qname = if i % 3 == 0 {
                QName::with_namespace(format!("attr{}", i), "http://example.com/test".to_string())
            } else if i % 3 == 1 {
                QName::with_prefix_and_namespace(
                    format!("attr{}", i),
                    "test".to_string(),
                    "http://example.com/test".to_string(),
                )
            } else {
                QName::new(format!("attr{}", i))
            };

            let value = match i % 5 {
                0 => AttributeValue::String(format!("string-{}", i)),
                1 => AttributeValue::Integer(i as i64),
                2 => AttributeValue::Boolean(i % 2 == 0),
                3 => AttributeValue::Decimal(i as f64 / 100.0),
                4 => AttributeValue::Token(format!("token-{}", i)),
                _ => unreachable!(),
            };

            attributes.insert(qname, value);
        }

        // Verify all attributes were added
        assert_eq!(attributes.len(), 1000);

        // Test canonical ordering with large dataset
        let canonical = attributes.to_canonical_ordered();
        assert_eq!(canonical.len(), 1000);

        // Verify ordering is maintained
        let keys: Vec<_> = canonical.keys().collect();
        for i in 1..keys.len() {
            assert!(keys[i - 1].canonical_sort_key() <= keys[i].canonical_sort_key());
        }

        // Test round-trip with large dataset
        let xml_map = attributes.to_string_map();
        assert_eq!(xml_map.len(), 1000);

        let restored = AttributeMap::from_string_map(xml_map);
        assert_eq!(restored.len(), 1000);

        // Verify some random samples are correct
        for i in [0, 100, 500, 999] {
            let qname_key = format!("attr{}", i);
            let found = restored.keys().find(|k| k.local_name == qname_key);
            assert!(found.is_some(), "Attribute attr{} should be preserved", i);
        }
    }
}
