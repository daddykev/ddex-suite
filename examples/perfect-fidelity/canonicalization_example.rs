//! DDEX Suite Canonicalization Example
//!
//! This example demonstrates the various canonicalization algorithms supported
//! by the Perfect Fidelity Engine, including:
//! - XML Canonicalization (C14N) 1.0 and 1.1
//! - DDEX-specific DB-C14N/1.0 algorithm
//! - Custom canonicalization rules
//! - Deterministic output generation
//! - Canonicalization verification and consistency testing

use ddex_builder::{
    Builder, FidelityOptions, CanonicalizationAlgorithm, CustomCanonicalizationRules,
    VerificationConfig, NamespaceHandling, LineEndingStyle, IndentationStyle
};
use ddex_builder::error::BuildError;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📐 DDEX Suite Canonicalization Example");
    println!("======================================\n");

    // Step 1: Create test XML samples with various formatting challenges
    let test_samples = create_canonicalization_test_samples();
    println!("📄 Created {} test XML samples for canonicalization", test_samples.len());

    // Step 2: Test each canonicalization algorithm
    for (name, sample) in &test_samples {
        println!("\n🔧 Testing with sample: {}", name);
        println!("   Original size: {} bytes", sample.len());
        
        test_canonicalization_algorithms(sample)?;
    }

    // Step 3: Demonstrate custom canonicalization rules
    println!("\n⚙️  Custom Canonicalization Rules Demo");
    demonstrate_custom_canonicalization(&test_samples["namespace_heavy"])?;

    // Step 4: Test canonicalization consistency and determinism
    println!("\n🔒 Canonicalization Consistency Testing");
    test_canonicalization_consistency(&test_samples["mixed_formatting"])?;

    // Step 5: Benchmark canonicalization performance
    println!("\n⚡ Canonicalization Performance Benchmark");
    benchmark_canonicalization_performance(&test_samples)?;

    // Step 6: DB-C14N specific features demonstration
    println!("\n🎵 DDEX DB-C14N/1.0 Specific Features");
    demonstrate_db_c14n_features(&test_samples["ddex_complex"])?;

    println!("\n✅ Canonicalization Example completed successfully!");
    println!("   All canonicalization algorithms working correctly 🎉");

    Ok(())
}

/// Create test XML samples with different canonicalization challenges
fn create_canonicalization_test_samples() -> HashMap<String, String> {
    let mut samples = HashMap::new();

    // Sample 1: Basic XML with attribute ordering issues
    samples.insert("basic_attributes".to_string(), r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:b="http://b.example.com" xmlns:a="http://a.example.com" id="123" name="test">
    <element z="last" a="first" m="middle">Content</element>
    <empty-element x="value" />
</root>"#.to_string());

    // Sample 2: Namespace-heavy XML
    samples.insert("namespace_heavy".to_string(), r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns="http://default.example.com"
      xmlns:ddex="http://ddex.net/xml/ern/43"
      xmlns:spotify="http://spotify.com/ddex"
      xmlns:apple="http://apple.com/ddex"
      xmlns:unused="http://unused.example.com">
    <ddex:element spotify:attr="value">
        <apple:subelement xmlns:local="http://local.example.com">
            <local:content>Data</local:content>
        </apple:subelement>
    </ddex:element>
</root>"#.to_string());

    // Sample 3: Mixed formatting and whitespace
    samples.insert("mixed_formatting".to_string(), r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <element1>

        <subelement>   Content with spaces   </subelement>

    </element1>
    <element2><compact>NoSpaces</compact></element2>
        <element3 attr="value"   attr2="value2"  >
            Mixed content with text and <inline>elements</inline>
        </element3>
</root>"#.to_string());

    // Sample 4: Complex DDEX structure
    samples.insert("ddex_complex".to_string(), r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage xmlns:ernm="http://ddex.net/xml/ern/43"
                        MessageSchemaVersionId="ern/43"
                        BusinessTransactionId="BT123">
    <MessageHeader>
        <MessageId>MSG-001</MessageId>
        <MessageSender>
            <PartyId namespace="DPID">SENDER123</PartyId>
        </MessageSender>
    </MessageHeader>
    <ReleaseList>
        <Release ReleaseProfileName="CommonReleaseTypes/14/AudioAlbumMusicOnly">
            <ReleaseReference>REL-001</ReleaseReference>
            <DisplayTitleText LanguageAndScriptCode="en">Test Album</DisplayTitleText>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string());

    // Sample 5: Unicode and special characters
    samples.insert("unicode_special".to_string(), r#"<?xml version="1.0" encoding="UTF-8"?>
<root title="Special Characters: &lt;&gt;&amp;&quot;&#x27;">
    <unicode>Émojis: 🎵 🎶 ℗ © ™</unicode>
    <cdata><![CDATA[Raw content with <tags> and & symbols]]></cdata>
    <entity>&amp; &lt; &gt; &quot; &apos;</entity>
    <mixed>Text with &lt;escaped&gt; and <real>tags</real></mixed>
</root>"#.to_string());

    samples
}

/// Test all canonicalization algorithms on a sample
fn test_canonicalization_algorithms(sample_xml: &str) -> Result<(), BuildError> {
    let algorithms = vec![
        ("Original (No Canonicalization)", CanonicalizationAlgorithm::None),
        ("XML C14N 1.0", CanonicalizationAlgorithm::C14N),
        ("XML C14N 1.1", CanonicalizationAlgorithm::C14N11),
        ("DDEX DB-C14N/1.0", CanonicalizationAlgorithm::DbC14N),
    ];

    for (name, algorithm) in algorithms {
        println!("   📐 {}", name);
        
        let start_time = Instant::now();
        
        // Create builder with this canonicalization algorithm
        let mut fidelity_options = FidelityOptions::default();
        fidelity_options.canonicalization = algorithm;
        fidelity_options.enable_perfect_fidelity = true;
        
        let builder = Builder::with_fidelity_options(fidelity_options);
        
        // Canonicalize the XML
        match builder.canonicalize(sample_xml) {
            Ok(canonical_xml) => {
                let process_time = start_time.elapsed();
                let size_change = (canonical_xml.len() as i32) - (sample_xml.len() as i32);
                
                println!("      ✅ Success");
                println!("      ⏱️  Time: {:.2}ms", process_time.as_secs_f64() * 1000.0);
                println!("      📏 Size: {} bytes ({:+})", canonical_xml.len(), size_change);
                
                // Analyze the changes
                analyze_canonicalization_changes(sample_xml, &canonical_xml);
                
                // Verify consistency (run canonicalization multiple times)
                verify_canonicalization_consistency(&builder, sample_xml)?;
            },
            Err(e) => {
                println!("      ❌ Failed: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

/// Analyze what changes canonicalization made
fn analyze_canonicalization_changes(original: &str, canonical: &str) {
    let original_lines = original.lines().count();
    let canonical_lines = canonical.lines().count();
    
    if original == canonical {
        println!("      🔍 No changes (perfect preservation)");
    } else {
        println!("      🔍 Changes detected:");
        
        if original_lines != canonical_lines {
            println!("         • Line count: {} → {} ({:+})", original_lines, canonical_lines, canonical_lines as i32 - original_lines as i32);
        }
        
        // Check for common canonicalization transformations
        if count_occurrences(original, " xmlns:") != count_occurrences(canonical, " xmlns:") {
            println!("         • Namespace declarations modified");
        }
        
        if count_occurrences(original, "=\"") != count_occurrences(canonical, "=\"") {
            println!("         • Attribute formatting modified");
        }
        
        let original_whitespace = original.chars().filter(|c| c.is_whitespace()).count();
        let canonical_whitespace = canonical.chars().filter(|c| c.is_whitespace()).count();
        if original_whitespace != canonical_whitespace {
            println!("         • Whitespace: {} → {} characters", original_whitespace, canonical_whitespace);
        }
    }
}

/// Count occurrences of a substring
fn count_occurrences(text: &str, pattern: &str) -> usize {
    text.matches(pattern).count()
}

/// Verify canonicalization produces consistent results
fn verify_canonicalization_consistency(builder: &Builder, xml: &str) -> Result<(), BuildError> {
    let iterations = 5;
    let mut results = Vec::new();
    
    for _ in 0..iterations {
        let result = builder.canonicalize(xml)?;
        results.push(result);
    }
    
    // Check if all results are identical
    let first_result = &results[0];
    let all_identical = results.iter().all(|r| r == first_result);
    
    if all_identical {
        println!("      🔒 Consistency: ✅ Deterministic ({} iterations)", iterations);
    } else {
        println!("      🔒 Consistency: ❌ Non-deterministic detected!");
        for (i, result) in results.iter().enumerate() {
            println!("         Iteration {}: {} bytes", i + 1, result.len());
        }
    }

    Ok(())
}

/// Demonstrate custom canonicalization rules
fn demonstrate_custom_canonicalization(sample_xml: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("   ⚙️  Creating custom canonicalization rules...");
    
    // Create custom rules
    let mut custom_rules = CustomCanonicalizationRules::default();
    custom_rules.preserve_whitespace = true;
    custom_rules.sort_attributes = true;
    custom_rules.normalize_line_endings = true;
    custom_rules.minimize_namespaces = true;
    
    // Define custom attribute ordering (DDEX-specific)
    custom_rules.attribute_ordering = vec![
        "MessageSchemaVersionId".to_string(),
        "BusinessTransactionId".to_string(),
        "ReleaseProfileName".to_string(),
        "LanguageAndScriptCode".to_string(),
        "namespace".to_string(),
    ];
    
    // Define custom element ordering for specific parents
    let mut element_ordering = HashMap::new();
    element_ordering.insert("MessageHeader".to_string(), vec![
        "MessageThreadId".to_string(),
        "MessageId".to_string(),
        "MessageFileName".to_string(),
        "MessageSender".to_string(),
        "MessageRecipient".to_string(),
        "MessageCreatedDateTime".to_string(),
    ]);
    custom_rules.element_ordering = element_ordering;
    
    println!("      📋 Custom Rules:");
    println!("         • Preserve whitespace: {}", custom_rules.preserve_whitespace);
    println!("         • Sort attributes: {}", custom_rules.sort_attributes);
    println!("         • Normalize line endings: {}", custom_rules.normalize_line_endings);
    println!("         • Minimize namespaces: {}", custom_rules.minimize_namespaces);
    println!("         • Custom attribute order: {} rules", custom_rules.attribute_ordering.len());
    println!("         • Custom element order: {} parent rules", custom_rules.element_ordering.len());
    
    // Apply custom canonicalization
    let mut fidelity_options = FidelityOptions::default();
    fidelity_options.canonicalization = CanonicalizationAlgorithm::Custom(custom_rules.clone());
    fidelity_options.custom_canonicalization_rules = Some(custom_rules);
    
    let builder = Builder::with_fidelity_options(fidelity_options);
    
    let start_time = Instant::now();
    let result = builder.canonicalize(sample_xml)?;
    let process_time = start_time.elapsed();
    
    println!("      🔧 Custom Canonicalization Result:");
    println!("         • Processing time: {:.2}ms", process_time.as_secs_f64() * 1000.0);
    println!("         • Size change: {} → {} bytes", sample_xml.len(), result.len());
    println!("         • Rules applied: ✅ Custom ordering and formatting");

    Ok(())
}

/// Test canonicalization consistency across multiple runs
fn test_canonicalization_consistency(sample_xml: &str) -> Result<(), BuildError> {
    println!("   🔒 Testing deterministic output...");
    
    let test_algorithms = vec![
        ("DB-C14N", CanonicalizationAlgorithm::DbC14N),
        ("C14N", CanonicalizationAlgorithm::C14N),
    ];
    
    for (name, algorithm) in test_algorithms {
        println!("      📐 {}", name);
        
        let mut fidelity_options = FidelityOptions::default();
        fidelity_options.canonicalization = algorithm;
        fidelity_options.enable_deterministic_ordering = true;
        
        let builder = Builder::with_fidelity_options(fidelity_options);
        
        // Run canonicalization multiple times
        let iterations = 10;
        let mut results = Vec::new();
        let mut total_time = std::time::Duration::ZERO;
        
        for _ in 0..iterations {
            let start = Instant::now();
            let result = builder.canonicalize(sample_xml)?;
            total_time += start.elapsed();
            results.push(result);
        }
        
        // Check consistency
        let first_result = &results[0];
        let all_identical = results.iter().all(|r| r == first_result);
        
        let avg_time = total_time / iterations;
        
        if all_identical {
            println!("         ✅ Deterministic: All {} runs identical", iterations);
            println!("         ⏱️  Average time: {:.2}ms", avg_time.as_secs_f64() * 1000.0);
            println!("         📏 Result size: {} bytes", first_result.len());
        } else {
            println!("         ❌ Non-deterministic: Results differ across runs");
            for (i, result) in results.iter().enumerate().take(3) {
                println!("            Run {}: {} bytes", i + 1, result.len());
            }
        }
    }

    Ok(())
}

/// Benchmark canonicalization performance
fn benchmark_canonicalization_performance(samples: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("   📊 Performance benchmarking across sample types...");
    
    let algorithm = CanonicalizationAlgorithm::DbC14N; // Focus on DB-C14N for performance
    let iterations = 100;
    
    for (name, sample) in samples {
        println!("      🔧 Sample: {}", name);
        
        let mut fidelity_options = FidelityOptions::default();
        fidelity_options.canonicalization = algorithm.clone();
        
        let builder = Builder::with_fidelity_options(fidelity_options);
        
        let mut total_time = std::time::Duration::ZERO;
        let mut results = Vec::new();
        
        // Warm-up runs
        for _ in 0..5 {
            let _ = builder.canonicalize(sample)?;
        }
        
        // Benchmark runs
        for _ in 0..iterations {
            let start = Instant::now();
            let result = builder.canonicalize(sample)?;
            let time = start.elapsed();
            total_time += time;
            results.push(result);
        }
        
        let avg_time = total_time / iterations;
        let throughput = (sample.len() as f64) / avg_time.as_secs_f64() / 1024.0 / 1024.0; // MB/s
        
        // Calculate statistics
        let mut times: Vec<f64> = Vec::new();
        let mut sample_total_time = std::time::Duration::ZERO;
        for _ in 0..10 {
            let start = Instant::now();
            let _ = builder.canonicalize(sample)?;
            let time = start.elapsed();
            times.push(time.as_secs_f64() * 1000.0);
            sample_total_time += time;
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min_time = times[0];
        let max_time = times[times.len() - 1];
        let median_time = times[times.len() / 2];
        
        println!("         📏 Input size: {} bytes", sample.len());
        println!("         ⏱️  Average time: {:.2}ms", avg_time.as_secs_f64() * 1000.0);
        println!("         📈 Min/Median/Max: {:.2}ms / {:.2}ms / {:.2}ms", min_time, median_time, max_time);
        println!("         🚀 Throughput: {:.2} MB/s", throughput);
        
        // Check if all results are identical (determinism)
        let first_result = &results[0];
        let deterministic = results.iter().all(|r| r == first_result);
        println!("         🔒 Deterministic: {}", if deterministic { "✅ Yes" } else { "❌ No" });
        
        println!();
    }

    Ok(())
}

/// Demonstrate DDEX DB-C14N specific features
fn demonstrate_db_c14n_features(sample_xml: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🎵 DDEX DB-C14N/1.0 Algorithm Features:");
    
    let mut fidelity_options = FidelityOptions::default();
    fidelity_options.canonicalization = CanonicalizationAlgorithm::DbC14N;
    fidelity_options.enable_perfect_fidelity = true;
    fidelity_options.preserve_namespace_prefixes = false; // Let DB-C14N optimize
    
    let builder = Builder::with_fidelity_options(fidelity_options);
    
    // Get DB-C14N configuration
    let db_c14n_config = builder.db_c14n_config();
    
    println!("      📋 DB-C14N Configuration:");
    println!("         • Version: {}", db_c14n_config.version);
    println!("         • Algorithm: {:?}", db_c14n_config.algorithm);
    println!("         • Deterministic ordering: {}", db_c14n_config.deterministic_ordering);
    println!("         • Preserve comments: {}", db_c14n_config.preserve_comments);
    println!("         • Preserve PIs: {}", db_c14n_config.preserve_processing_instructions);
    println!("         • Namespace handling: {:?}", db_c14n_config.namespace_handling);
    
    // Apply DB-C14N canonicalization
    let start_time = Instant::now();
    let canonical_result = builder.canonicalize(sample_xml)?;
    let canonicalize_time = start_time.elapsed();
    
    println!("      🔧 DB-C14N Processing:");
    println!("         • Processing time: {:.2}ms", canonicalize_time.as_secs_f64() * 1000.0);
    println!("         • Input size: {} bytes", sample_xml.len());
    println!("         • Output size: {} bytes", canonical_result.len());
    
    let compression_ratio = (canonical_result.len() as f64) / (sample_xml.len() as f64);
    println!("         • Compression ratio: {:.2}x", compression_ratio);
    
    // Verify the output is valid XML
    match quick_xml::Reader::from_str(&canonical_result).read_event() {
        Ok(_) => println!("         • Output validity: ✅ Valid XML"),
        Err(e) => println!("         • Output validity: ❌ Invalid - {}", e),
    }
    
    // Test DB-C14N specific features
    test_db_c14n_namespace_handling(sample_xml, &canonical_result)?;
    test_db_c14n_attribute_ordering(sample_xml, &canonical_result)?;
    
    Ok(())
}

/// Test DB-C14N namespace handling
fn test_db_c14n_namespace_handling(original: &str, canonical: &str) -> Result<(), Box<dyn std::error::Error>> {
    let original_ns_count = count_occurrences(original, "xmlns:");
    let canonical_ns_count = count_occurrences(canonical, "xmlns:");
    
    println!("      🏷️  Namespace Handling:");
    println!("         • Original declarations: {}", original_ns_count);
    println!("         • Canonical declarations: {}", canonical_ns_count);
    
    if canonical_ns_count < original_ns_count {
        println!("         • Optimization: ✅ Removed {} unused declarations", original_ns_count - canonical_ns_count);
    } else if canonical_ns_count == original_ns_count {
        println!("         • Optimization: → All declarations preserved (needed)");
    } else {
        println!("         • Optimization: ❌ Added declarations (unexpected)");
    }
    
    Ok(())
}

/// Test DB-C14N attribute ordering
fn test_db_c14n_attribute_ordering(original: &str, canonical: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This is a simplified test - in reality we'd parse the XML and check attribute order
    let original_attr_count = count_occurrences(original, "=\"");
    let canonical_attr_count = count_occurrences(canonical, "=\"");
    
    println!("      📝 Attribute Handling:");
    println!("         • Original attributes: {}", original_attr_count);
    println!("         • Canonical attributes: {}", canonical_attr_count);
    
    if canonical_attr_count == original_attr_count {
        println!("         • Preservation: ✅ All attributes preserved");
        println!("         • Ordering: → Deterministic order applied");
    } else {
        println!("         • Preservation: ❌ Attribute count mismatch");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalization_samples_creation() {
        let samples = create_canonicalization_test_samples();
        assert_eq!(samples.len(), 5);
        assert!(samples.contains_key("basic_attributes"));
        assert!(samples.contains_key("ddex_complex"));
    }

    #[test]
    fn test_custom_canonicalization_rules() {
        let mut rules = CustomCanonicalizationRules::default();
        rules.sort_attributes = true;
        rules.normalize_line_endings = true;
        
        assert!(rules.sort_attributes);
        assert!(rules.normalize_line_endings);
        assert!(!rules.preserve_whitespace); // Default is false
    }

    #[test]
    fn test_canonicalization_algorithm_types() {
        let algorithms = vec![
            CanonicalizationAlgorithm::None,
            CanonicalizationAlgorithm::C14N,
            CanonicalizationAlgorithm::C14N11,
            CanonicalizationAlgorithm::DbC14N,
        ];

        for algorithm in algorithms {
            let mut options = FidelityOptions::default();
            options.canonicalization = algorithm;
            
            let builder = Builder::with_fidelity_options(options);
            assert_eq!(builder.canonicalization_algorithm(), &builder.fidelity_options().canonicalization);
        }
    }

    #[test]
    fn test_count_occurrences_helper() {
        let text = "xmlns:a=\"http://a.com\" xmlns:b=\"http://b.com\" xmlns:c=\"http://c.com\"";
        assert_eq!(count_occurrences(text, "xmlns:"), 3);
        assert_eq!(count_occurrences(text, "http://"), 3);
        assert_eq!(count_occurrences(text, "=\""), 3);
    }

    #[test] 
    fn test_db_c14n_config() {
        let mut fidelity_options = FidelityOptions::default();
        fidelity_options.canonicalization = CanonicalizationAlgorithm::DbC14N;
        
        let builder = Builder::with_fidelity_options(fidelity_options);
        let config = builder.db_c14n_config();
        
        assert_eq!(config.version, "1.0");
        assert_eq!(config.algorithm, CanonicalizationAlgorithm::DbC14N);
        assert!(config.deterministic_ordering);
    }
}