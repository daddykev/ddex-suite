// examples/fast_streaming_usage.rs
//! Example demonstrating how to use the FastStreamingParser for high-performance DDEX parsing

use ddex_parser::parser::security::SecurityConfig;
use ddex_parser::streaming::{create_fast_parser, FastStreamingParser, StreamingConfig};
use std::io::{BufReader, Cursor};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FastStreamingParser Usage Example ===\n");

    // Example 1: Parse with default fast configuration
    example_default_fast_parsing()?;

    // Example 2: Parse with custom configuration
    example_custom_configuration()?;

    // Example 3: Parse large file with progress tracking
    example_large_file_parsing()?;

    // Example 4: Performance comparison
    example_performance_comparison()?;

    Ok(())
}

/// Example 1: Parse with default fast configuration for maximum performance
fn example_default_fast_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Default Fast Parser Configuration");
    println!("   - Relaxed security for maximum speed");
    println!("   - 64KB buffer, 512KB chunks");
    println!("   - Zero-copy parsing optimizations");

    let sample_xml = generate_sample_xml(100); // 100 releases
    let cursor = Cursor::new(sample_xml.as_bytes());
    let mut reader = BufReader::new(cursor);

    let mut parser = create_fast_parser();
    let start = Instant::now();

    let iterator = parser.parse_streaming(&mut reader, None)?;
    let elements: Vec<_> = iterator.collect();
    let elapsed = start.elapsed();

    // Get stats after consuming iterator
    let stats = parser.get_stats();

    println!("   Results:");
    println!("   - Elements parsed: {}", elements.len());
    println!(
        "   - Data size: {:.2} KB",
        stats.total_bytes as f64 / 1024.0
    );
    println!("   - Parse time: {:?}", elapsed);
    println!("   - Throughput: {:.2} MB/s", stats.throughput_mbps);
    println!("   - Elements/sec: {:.0}", stats.elements_per_second);
    println!();

    Ok(())
}

/// Example 2: Parse with custom configuration
fn example_custom_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Custom Parser Configuration");
    println!("   - Strict security validation");
    println!("   - Large 256KB buffer for better I/O");
    println!("   - Progress tracking enabled");

    let config = StreamingConfig {
        security: SecurityConfig::strict(), // Strict validation
        buffer_size: 256 * 1024,            // 256KB buffer
        max_memory: 500 * 1024 * 1024,      // 500MB memory limit
        chunk_size: 1024,                   // 1MB chunks
        enable_progress: true,              // Enable progress callbacks
        progress_interval: 100 * 1024,      // Progress every 100KB
    };

    let sample_xml = generate_sample_xml(500); // Larger dataset
    let cursor = Cursor::new(sample_xml.as_bytes());
    let mut reader = BufReader::new(cursor);

    let mut parser = FastStreamingParser::new(config);
    let start = Instant::now();

    // Parse with progress callback
    let progress_callback = Box::new(|progress: ddex_parser::streaming::StreamingProgress| {
        println!(
            "   Progress: {:.1}KB processed, {} elements found, depth: {}",
            progress.bytes_processed as f64 / 1024.0,
            progress.elements_parsed,
            progress.current_depth
        );
    });

    let iterator = parser.parse_streaming(&mut reader, Some(progress_callback))?;

    let releases: Vec<_> = iterator
        .filter(|el| {
            matches!(
                el.element_type,
                ddex_parser::streaming::FastElementType::Release
            )
        })
        .collect();
    let elapsed = start.elapsed();

    // Get stats after consuming iterator
    let stats = parser.get_stats();

    println!("   Results:");
    println!("   - Total elements: {}", stats.total_elements);
    println!("   - Releases found: {}", releases.len());
    println!(
        "   - Data size: {:.2} MB",
        stats.total_bytes as f64 / (1024.0 * 1024.0)
    );
    println!("   - Parse time: {:?}", elapsed);
    println!("   - Throughput: {:.2} MB/s", stats.throughput_mbps);
    println!();

    Ok(())
}

/// Example 3: Parse large file with memory management
fn example_large_file_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Large File Parsing Example");
    println!("   - Memory-bounded parsing");
    println!("   - Incremental element processing");

    // Create a large synthetic dataset
    let large_xml = generate_sample_xml(2000); // 2000 releases ~= several MB
    let total_size = large_xml.len();

    let config = StreamingConfig {
        security: SecurityConfig::relaxed(),
        buffer_size: 128 * 1024,      // 128KB buffer
        max_memory: 50 * 1024 * 1024, // 50MB memory limit
        chunk_size: 512,              // 512KB chunks
        enable_progress: true,
        progress_interval: 1024 * 1024, // Progress every 1MB
    };

    let cursor = Cursor::new(large_xml.as_bytes());
    let mut reader = BufReader::new(cursor);
    let mut parser = FastStreamingParser::new(config);

    println!(
        "   Processing {:.2} MB of XML data...",
        total_size as f64 / (1024.0 * 1024.0)
    );

    let start = Instant::now();
    let mut total_releases = 0;
    let mut total_elements = 0;

    let total_size_owned = total_size; // Copy the value
    let progress_callback = Box::new(move |progress: ddex_parser::streaming::StreamingProgress| {
        let percent = if total_size_owned > 0 {
            (progress.bytes_processed as f64 / total_size_owned as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "   [{:5.1}%] {:.1}MB processed, {} elements, {:.1}MB memory",
            percent,
            progress.bytes_processed as f64 / (1024.0 * 1024.0),
            progress.elements_parsed,
            progress.memory_usage as f64 / (1024.0 * 1024.0)
        );
    });

    let iterator = parser.parse_streaming(&mut reader, Some(progress_callback))?;

    // Process elements in chunks to manage memory
    for element in iterator {
        match element.element_type {
            ddex_parser::streaming::FastElementType::Release => {
                total_releases += 1;
                // Process release element here
                // println!("Processing release at position {}", element.position);
            }
            _ => {
                total_elements += 1;
            }
        }
    }

    let elapsed = start.elapsed();

    // Get stats after consuming iterator
    let stats = parser.get_stats();

    println!("   Results:");
    println!("   - Total releases processed: {}", total_releases);
    println!("   - Other elements: {}", total_elements);
    println!("   - Processing time: {:?}", elapsed);
    println!("   - Peak memory: {:.2} MB", stats.peak_memory_mb);
    println!("   - Average throughput: {:.2} MB/s", stats.throughput_mbps);
    println!();

    Ok(())
}

/// Example 4: Performance comparison with different configurations
fn example_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Performance Comparison");

    let test_xml = generate_sample_xml(1000);
    let data_size_mb = test_xml.len() as f64 / (1024.0 * 1024.0);

    println!("   Testing with {:.2} MB of XML data", data_size_mb);

    // Test configurations
    let configs = vec![
        (
            "Relaxed/Fast",
            StreamingConfig {
                security: SecurityConfig::relaxed(),
                buffer_size: 64 * 1024,
                chunk_size: 512,
                ..Default::default()
            },
        ),
        (
            "Strict/Secure",
            StreamingConfig {
                security: SecurityConfig::strict(),
                buffer_size: 32 * 1024,
                chunk_size: 256,
                ..Default::default()
            },
        ),
        (
            "Large Buffer",
            StreamingConfig {
                security: SecurityConfig::relaxed(),
                buffer_size: 512 * 1024, // 512KB
                chunk_size: 2048,        // 2MB chunks
                ..Default::default()
            },
        ),
    ];

    for (name, config) in configs {
        let cursor = Cursor::new(test_xml.as_bytes());
        let mut reader = BufReader::new(cursor);
        let mut parser = FastStreamingParser::new(config);

        let start = Instant::now();
        let iterator = parser.parse_streaming(&mut reader, None)?;
        let elements: Vec<_> = iterator.collect();
        let elapsed = start.elapsed();

        // Get stats after consuming iterator
        let stats = parser.get_stats();

        println!(
            "   {:<15} | {:>6} elements | {:>6.0}ms | {:>6.2}MB/s | {:>8.0} elem/s",
            name,
            elements.len(),
            elapsed.as_millis(),
            stats.throughput_mbps,
            stats.elements_per_second
        );
    }

    println!();
    println!("Note: Performance results are from debug builds.");
    println!("      Release builds typically achieve 5-10x better throughput.");
    println!("      Target of 280+ MB/s is achievable with:");
    println!("      - Release build optimizations (-O3)");
    println!("      - Larger datasets (>10MB) for better amortization");
    println!("      - SIMD CPU instructions on modern hardware");

    Ok(())
}

/// Generate sample DDEX XML for testing
fn generate_sample_xml(num_releases: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <ern:MessageHeader>
        <ern:MessageId>FAST_STREAMING_EXAMPLE</ern:MessageId>
        <ern:MessageSender>
            <ern:PartyName>Fast Parser Demo</ern:PartyName>
        </ern:MessageSender>
        <ern:MessageRecipient>
            <ern:PartyName>Performance Test</ern:PartyName>
        </ern:MessageRecipient>
        <ern:MessageCreatedDateTime>2024-01-15T10:30:00Z</ern:MessageCreatedDateTime>
    </ern:MessageHeader>
    <ern:ReleaseList>"#,
    );

    for i in 0..num_releases {
        xml.push_str(&format!(
            r#"
        <ern:Release>
            <ern:ReleaseId>REL{:06}</ern:ReleaseId>
            <ern:ReleaseReference>R{:06}</ern:ReleaseReference>
            <ern:ReferenceTitle>
                <ern:TitleText>Fast Streaming Test Release {}</ern:TitleText>
            </ern:ReferenceTitle>
            <ern:ReleaseType>Album</ern:ReleaseType>
            <ern:ReleaseDetailsByTerritory>
                <ern:TerritoryCode>Worldwide</ern:TerritoryCode>
                <ern:DisplayArtist>
                    <ern:PartyName>Test Artist {}</ern:PartyName>
                </ern:DisplayArtist>
                <ern:ReleaseDate>2024-01-15</ern:ReleaseDate>
            </ern:ReleaseDetailsByTerritory>
        </ern:Release>"#,
            i, i, i, i
        ));
    }

    xml.push_str("</ern:ReleaseList></ern:NewReleaseMessage>");
    xml
}
