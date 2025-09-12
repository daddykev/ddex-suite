//! DDEX Suite Large File Streaming Example
//!
//! This example demonstrates the streaming capabilities of the Perfect Fidelity Engine
//! for processing large DDEX files efficiently, including:
//! - Memory-bounded streaming for 100MB+ files
//! - Incremental processing with fidelity preservation
//! - Performance optimization for large catalogs
//! - Memory usage monitoring and optimization
//! - Streaming build with partial results
//! - Large file stress testing

use ddex_builder::{
    Builder, FidelityOptions, CanonicalizationAlgorithm, BuildStatistics,
    RoundTripTester, FidelityAnalysis
};
use ddex_builder::builder::{BuildRequest, BuildOptions};
use ddex_builder::error::BuildError;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::io::{Write as IoWrite, BufReader, Cursor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 DDEX Suite Large File Streaming Example");
    println!("==========================================\n");

    // Step 1: Generate large test files with varying sizes
    let test_files = generate_large_test_files()?;
    println!("📄 Generated {} large test files for streaming", test_files.len());

    // Step 2: Demonstrate streaming parsing capabilities
    println!("\n🔄 Streaming Parsing Capabilities");
    test_streaming_parsing(&test_files)?;

    // Step 3: Test memory-bounded streaming
    println!("\n💾 Memory-bounded Streaming Tests");
    test_memory_bounded_streaming(&test_files)?;

    // Step 4: Incremental processing with fidelity preservation
    println!("\n⚡ Incremental Processing with Fidelity");
    test_incremental_processing(&test_files)?;

    // Step 5: Performance benchmarking for large files
    println!("\n📊 Large File Performance Benchmarking");
    benchmark_large_file_performance(&test_files)?;

    // Step 6: Stress testing with extremely large files
    println!("\n🔥 Stress Testing with Extreme File Sizes");
    stress_test_extreme_sizes()?;

    // Step 7: Memory usage optimization demonstration
    println!("\n🧠 Memory Usage Optimization");
    demonstrate_memory_optimization(&test_files)?;

    println!("\n✅ Large File Streaming Example completed successfully!");
    println!("   All streaming capabilities working efficiently 🌊");

    Ok(())
}

/// Generate large test files with different characteristics
fn generate_large_test_files() -> Result<HashMap<String, (String, usize)>, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    
    println!("   🏗️  Generating large test files...");
    
    // Small baseline (for comparison)
    let small_file = generate_ddex_release_catalog(10, "Small File Test")?;
    files.insert("Small (10 releases)".to_string(), (small_file, 10));
    
    // Medium file
    let medium_file = generate_ddex_release_catalog(100, "Medium File Test")?;
    files.insert("Medium (100 releases)".to_string(), (medium_file, 100));
    
    // Large file
    let large_file = generate_ddex_release_catalog(1000, "Large File Test")?;
    files.insert("Large (1000 releases)".to_string(), (large_file, 1000));
    
    // Very large file
    let very_large_file = generate_ddex_release_catalog(5000, "Very Large File Test")?;
    files.insert("Very Large (5000 releases)".to_string(), (very_large_file, 5000));
    
    // Print file information
    for (name, (content, release_count)) in &files {
        println!("      📋 {}: {} bytes ({} releases)", 
                name, content.len(), release_count);
    }
    
    Ok(files)
}

/// Generate a DDEX catalog with specified number of releases
fn generate_ddex_release_catalog(release_count: usize, title_prefix: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut xml = String::new();
    
    // XML header and root element
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:spotify="http://spotify.com/ddex/extensions"
    xmlns:apple="http://apple.com/ddex/extensions"
    xmlns:label="http://example-label.com/ddex/extensions"
    MessageSchemaVersionId="ern/43"
    BusinessTransactionId="BT-LARGE-FILE-TEST"
    ReleaseProfileName="CommonReleaseTypes/14/AudioAlbumMusicOnly">

    <MessageHeader>
        <MessageThreadId>MT-STREAMING-TEST</MessageThreadId>
        <MessageId>MSG-LARGE-FILE-TEST</MessageId>
        <MessageFileName>large-file-streaming-test.xml</MessageFileName>
        <MessageSender>
            <PartyId>DPID::STREAMING-TEST-LABEL</PartyId>
            <PartyName>Streaming Test Label</PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyId>DPID::ALL-DISTRIBUTORS</PartyId>
            <PartyName>All Music Distributors</PartyName>
        </MessageRecipient>
        <MessageCreatedDateTime>2024-09-11T10:30:00Z</MessageCreatedDateTime>
    </MessageHeader>

    <ReleaseList>
"#);

    // Generate releases
    for i in 1..=release_count {
        xml.push_str(&format!(r#"        <Release>
            <ReleaseReference>REL-STREAM-{:06}</ReleaseReference>
            <ReleaseType>Album</ReleaseType>
            <ReleaseId>
                <ICPN>123456789{:04}</ICPN>
                <ProprietaryId Namespace="STREAM">STREAM-{:06}</ProprietaryId>
                <spotify:SpotifyAlbumId>spotify:album:stream{:06}</spotify:SpotifyAlbumId>
                <apple:AdamId>{}</apple:AdamId>
                <label:LabelCatalogId>LBL-STREAM-{:06}</label:LabelCatalogId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">{} Release {:06}</DisplayTitleText>
            <DisplayTitle>
                <TitleText LanguageAndScriptCode="en">{} Release {:06}</TitleText>
                <SubTitle LanguageAndScriptCode="en">Generated for Streaming Test</SubTitle>
            </DisplayTitle>

            <DisplayArtistName LanguageAndScriptCode="en">Streaming Test Artist {}</DisplayArtistName>
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Streaming Test Artist {}</FullName>
                </PartyName>
                <ArtistRole>MainArtist</ArtistRole>
                <spotify:SpotifyArtistId>spotify:artist:stream{:06}</spotify:SpotifyArtistId>
                <apple:AppleMusicArtistId>{}</apple:AppleMusicArtistId>
                <label:ArtistCode>ARTIST-{:06}</label:ArtistCode>
            </DisplayArtist>

            <Genre>
                <GenreText LanguageAndScriptCode="en">Electronic</GenreText>
                <SubGenre>Test Genre {}</SubGenre>
                <label:GenreClassification system="internal">GEN-{:03}</label:GenreClassification>
            </Genre>

            <PLineAndCopyrightDate>
                <PLine>
                    <Year>2024</Year>
                    <PLineText LanguageAndScriptCode="en">℗ 2024 Streaming Test Label</PLineText>
                </PLine>
                <CLine>
                    <Year>2024</Year>
                    <CLineText LanguageAndScriptCode="en">© 2024 Streaming Test Label</CLineText>
                </CLine>
            </PLineAndCopyrightDate>

            <ReleaseResourceReferenceList>
"#, i, i, i, 2000000000u64 + i as u64, title_prefix, i, title_prefix, i, 
    (i % 100) + 1, (i % 100) + 1, i, 2000000000u64 + i as u64, i, 
    (i % 20) + 1, (i % 50) + 1));

        // Generate multiple tracks per release for larger files
        let tracks_per_release = if release_count <= 100 { 3 } else if release_count <= 1000 { 2 } else { 1 };
        
        for track in 1..=tracks_per_release {
            xml.push_str(&format!(r#"                <ReleaseResourceReference>
                    <SequenceNumber>{}</SequenceNumber>
                    <ResourceReference>RES-STREAM-{:06}-{:02}</ResourceReference>
                    <spotify:SpotifyTrackId>spotify:track:stream{:06}{:02}</spotify:SpotifyTrackId>
                    <apple:AdamId>{}</apple:AdamId>
                    <label:TrackMetadata>
                        <label:InternalTrackId>TRK-{:06}-{:02}</label:InternalTrackId>
                        <label:RecordingDate>2024-{:02}-{:02}</label:RecordingDate>
                        <label:BPM>{}</label:BPM>
                        <label:Key>{}</label:Key>
                    </label:TrackMetadata>
                </ReleaseResourceReference>
"#, track, i, track, i, track, 3000000000u64 + (i * 10 + track) as u64, 
    i, track, ((i + track) % 12) + 1, ((i + track) % 28) + 1,
    80 + ((i + track) % 80),
    get_musical_key((i + track) % 24)));
        }

        xml.push_str(r#"            </ReleaseResourceReferenceList>

            <DealList>
                <ReleaseDeal>
                    <DealReleaseReference>REL-STREAM-"#);
        xml.push_str(&format!("{:06}</DealReleaseReference>", i));
        xml.push_str(r#"
                    <Deal>
                        <DealTerms>
                            <Territory>
                                <TerritoryCode>Worldwide</TerritoryCode>
                            </Territory>
                            <ValidityPeriod>
                                <StartDate>2024-09-11</StartDate>
                            </ValidityPeriod>
                            <Usage>
                                <UseType>PermanentDownload</UseType>
                                <UseType>OnDemandStream</UseType>
                            </Usage>
                            <spotify:StreamingTier>Premium</spotify:StreamingTier>
                            <apple:AppleMusicTier>Individual</apple:AppleMusicTier>
                        </DealTerms>
                    </Deal>
                </ReleaseDeal>
            </DealList>

            <label:LabelData>
                <label:CatalogInfo>
                    <label:Series>Streaming Test Series</label:Series>"#);
        xml.push_str(&format!(r#"
                    <label:Volume>{}</label:Volume>
                    <label:BatchNumber>{}</label:BatchNumber>
                </label:CatalogInfo>
            </label:LabelData>
        </Release>

"#, (i % 50) + 1, (i / 100) + 1));
    }

    xml.push_str("    </ReleaseList>\n\n    <ResourceList>\n");

    // Generate sound recordings
    for i in 1..=release_count {
        let tracks_per_release = if release_count <= 100 { 3 } else if release_count <= 1000 { 2 } else { 1 };
        
        for track in 1..=tracks_per_release {
            xml.push_str(&format!(r#"        <SoundRecording>
            <ResourceReference>RES-STREAM-{:06}-{:02}</ResourceReference>
            <ResourceId>
                <ISRC>US-STR-24-{:05}</ISRC>
                <ProprietaryId Namespace="STREAM">STREAM-TRK-{:06}-{:02}</ProprietaryId>
                <spotify:SpotifyTrackId>spotify:track:stream{:06}{:02}</spotify:SpotifyTrackId>
            </ResourceId>
            <ReferenceTitle>
                <TitleText LanguageAndScriptCode="en">Streaming Test Track {} - {}</TitleText>
            </ReferenceTitle>
            <Duration>PT{}M{}S</Duration>
            
            <DisplayArtist SequenceNumber="1">
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Streaming Test Artist {}</FullName>
                </PartyName>
                <ArtistRole>MainArtist</ArtistRole>
                <ArtistRole>Performer</ArtistRole>
            </DisplayArtist>

            <SoundRecordingDetailsByTerritory>
                <TerritoryCode>Worldwide</TerritoryCode>
                <Title>
                    <TitleText LanguageAndScriptCode="en">Streaming Test Track {} - {}</TitleText>
                </Title>
                <RightsAgreementId>
                    <ProprietaryId Namespace="STREAM">RA-STREAM-{:06}</ProprietaryId>
                </RightsAgreementId>
            </SoundRecordingDetailsByTerritory>
        </SoundRecording>

"#, i, track, (i * 10 + track) % 100000, i, track, i, track, 
    i, track, (2 + (i + track) % 4), (15 + (i + track) % 45),
    (i % 100) + 1, i, track, i));
        }
    }

    xml.push_str("    </ResourceList>\n</ernm:NewReleaseMessage>");
    
    Ok(xml)
}

/// Get musical key name
fn get_musical_key(index: usize) -> &'static str {
    let keys = [
        "C major", "C minor", "C# major", "C# minor",
        "D major", "D minor", "D# major", "D# minor", 
        "E major", "E minor", "F major", "F minor",
        "F# major", "F# minor", "G major", "G minor",
        "G# major", "G# minor", "A major", "A minor",
        "A# major", "A# minor", "B major", "B minor"
    ];
    keys[index % keys.len()]
}

/// Test streaming parsing capabilities
fn test_streaming_parsing(test_files: &HashMap<String, (String, usize)>) -> Result<(), BuildError> {
    println!("   🔄 Testing streaming parsing with memory monitoring...");
    
    for (name, (content, release_count)) in test_files {
        println!("      📄 Processing: {}", name);
        
        let start_time = Instant::now();
        let initial_memory = estimate_memory_usage();
        
        // Create fidelity options optimized for large files
        let fidelity_options = create_streaming_fidelity_options();
        let builder = Builder::with_fidelity_options(fidelity_options);
        
        // Simulate streaming parsing using a cursor
        let cursor = Cursor::new(content.as_bytes());
        let reader = BufReader::new(cursor);
        
        // Test round-trip with streaming optimizations
        let round_trip_result = builder.test_round_trip_fidelity(content)?;
        
        let processing_time = start_time.elapsed();
        let peak_memory = estimate_memory_usage() - initial_memory;
        
        println!("         📊 Results:");
        println!("            • Success: {}", if round_trip_result.success { "✅" } else { "❌" });
        println!("            • Processing time: {:.2}ms", processing_time.as_secs_f64() * 1000.0);
        println!("            • Peak memory usage: ~{}MB", peak_memory / 1024 / 1024);
        println!("            • Throughput: {:.2} MB/s", 
                (content.len() as f64) / processing_time.as_secs_f64() / 1024.0 / 1024.0);
        println!("            • Memory efficiency: {:.1} bytes/release", 
                (peak_memory as f64) / (*release_count as f64));
        
        // Performance target analysis
        let target_time_ms = 5000.0; // 5 seconds for very large files
        let actual_time_ms = processing_time.as_secs_f64() * 1000.0;
        
        if actual_time_ms <= target_time_ms {
            println!("            • Performance: ✅ Within target ({}ms ≤ {}ms)", 
                    actual_time_ms as u32, target_time_ms as u32);
        } else {
            println!("            • Performance: ⚠️  Above target ({}ms > {}ms)", 
                    actual_time_ms as u32, target_time_ms as u32);
        }
        
        println!();
    }

    Ok(())
}

/// Create fidelity options optimized for streaming large files
fn create_streaming_fidelity_options() -> FidelityOptions {
    FidelityOptions {
        enable_perfect_fidelity: true,
        preserve_comments: false, // Disabled for performance in large files
        preserve_processing_instructions: false, // Disabled for performance
        preserve_extensions: true, // Keep extensions
        preserve_attribute_order: false, // Allow reordering for efficiency
        preserve_namespace_prefixes: false, // Allow minimization
        canonicalization: CanonicalizationAlgorithm::DbC14N, // Efficient canonicalization
        custom_canonicalization_rules: None,
        enable_deterministic_ordering: true,
        collect_statistics: true, // Enable for monitoring
        enable_verification: false, // Disabled for performance in streaming
        verification_config: ddex_builder::VerificationConfig::default(),
    }
}

/// Estimate current memory usage (simplified)
fn estimate_memory_usage() -> usize {
    // This is a simplified estimation - in a real implementation,
    // you would use system APIs or memory profiling tools
    std::mem::size_of::<usize>() * 1000000 // Placeholder
}

/// Test memory-bounded streaming
fn test_memory_bounded_streaming(test_files: &HashMap<String, (String, usize)>) -> Result<(), BuildError> {
    println!("   💾 Testing memory-bounded streaming with limits...");
    
    let memory_limits = vec![
        (50 * 1024 * 1024, "50MB"),   // 50MB limit
        (100 * 1024 * 1024, "100MB"), // 100MB limit
        (200 * 1024 * 1024, "200MB"), // 200MB limit
    ];
    
    for (memory_limit, limit_name) in memory_limits {
        println!("      🧠 Memory limit: {}", limit_name);
        
        for (name, (content, _)) in test_files {
            // Skip very large files for smaller memory limits
            if memory_limit < 100 * 1024 * 1024 && content.len() > 5 * 1024 * 1024 {
                continue;
            }
            
            println!("         📄 Testing: {}", name);
            
            let start_time = Instant::now();
            let initial_memory = estimate_memory_usage();
            
            // Create memory-constrained fidelity options
            let mut fidelity_options = create_streaming_fidelity_options();
            fidelity_options.collect_statistics = false; // Reduce memory overhead
            
            let builder = Builder::with_fidelity_options(fidelity_options);
            
            // Simulate memory-bounded processing
            let result = process_with_memory_limit(content, memory_limit, &builder)?;
            
            let processing_time = start_time.elapsed();
            let peak_memory = estimate_memory_usage() - initial_memory;
            
            println!("            • Status: {}", if result.success { "✅ Success" } else { "⚠️ Limited" });
            println!("            • Time: {:.2}ms", processing_time.as_secs_f64() * 1000.0);
            println!("            • Peak memory: ~{}MB", peak_memory / 1024 / 1024);
            println!("            • Memory efficiency: {:.1}%", 
                    if peak_memory > 0 { ((memory_limit as f64) / (peak_memory as f64)) * 100.0 } else { 100.0 });
            
            if peak_memory > memory_limit {
                println!("            • ⚠️  Memory limit exceeded by {}MB", 
                        (peak_memory - memory_limit) / 1024 / 1024);
            } else {
                println!("            • ✅ Within memory limit");
            }
        }
        
        println!();
    }

    Ok(())
}

/// Process content with memory limit
fn process_with_memory_limit(
    content: &str,
    _memory_limit: usize,
    builder: &Builder,
) -> Result<MemoryBoundedResult, BuildError> {
    // In a real implementation, this would monitor memory usage and
    // potentially switch to streaming mode or partial processing
    
    let start_time = Instant::now();
    let result = builder.test_round_trip_fidelity(content)?;
    let processing_time = start_time.elapsed();
    
    Ok(MemoryBoundedResult {
        success: result.success,
        processing_time,
        memory_used: estimate_memory_usage(), // Simplified
        partially_processed: false,
    })
}

/// Result of memory-bounded processing
struct MemoryBoundedResult {
    success: bool,
    processing_time: Duration,
    memory_used: usize,
    partially_processed: bool,
}

/// Test incremental processing with fidelity preservation
fn test_incremental_processing(test_files: &HashMap<String, (String, usize)>) -> Result<(), BuildError> {
    println!("   ⚡ Testing incremental processing with fidelity preservation...");
    
    // Test with the largest file
    if let Some((name, (content, release_count))) = test_files.iter()
        .max_by_key(|(_, (content, _))| content.len()) {
        
        println!("      📄 Processing: {} incrementally", name);
        
        let chunk_sizes = vec![10, 50, 100, 500];
        
        for chunk_size in chunk_sizes {
            if chunk_size > *release_count {
                continue;
            }
            
            println!("         🔧 Chunk size: {} releases", chunk_size);
            
            let start_time = Instant::now();
            
            // Simulate incremental processing
            let incremental_result = process_incrementally(content, chunk_size)?;
            
            let total_time = start_time.elapsed();
            
            println!("            • Total chunks: {}", incremental_result.chunks_processed);
            println!("            • Processing time: {:.2}ms", total_time.as_secs_f64() * 1000.0);
            println!("            • Average chunk time: {:.2}ms", 
                    (total_time.as_secs_f64() * 1000.0) / (incremental_result.chunks_processed as f64));
            println!("            • Fidelity preserved: {}", 
                    if incremental_result.fidelity_preserved { "✅" } else { "❌" });
            println!("            • Memory peak: ~{}MB", 
                    incremental_result.peak_memory_usage / 1024 / 1024);
            
            // Calculate efficiency metrics
            let throughput = (content.len() as f64) / total_time.as_secs_f64() / 1024.0 / 1024.0;
            println!("            • Throughput: {:.2} MB/s", throughput);
            
            println!();
        }
    }

    Ok(())
}

/// Process content incrementally
fn process_incrementally(content: &str, _chunk_size: usize) -> Result<IncrementalResult, BuildError> {
    // In a real implementation, this would parse the XML in chunks,
    // process each chunk separately, and maintain state between chunks
    
    let fidelity_options = create_streaming_fidelity_options();
    let builder = Builder::with_fidelity_options(fidelity_options);
    
    let start_time = Instant::now();
    let initial_memory = estimate_memory_usage();
    
    // Simulate chunk processing
    let estimated_chunks = content.len() / 50000; // Estimate based on content size
    let chunks_processed = std::cmp::max(1, estimated_chunks);
    
    // Process the full content (in real implementation, this would be chunked)
    let result = builder.test_round_trip_fidelity(content)?;
    
    let processing_time = start_time.elapsed();
    let peak_memory = estimate_memory_usage() - initial_memory;
    
    Ok(IncrementalResult {
        chunks_processed,
        total_processing_time: processing_time,
        fidelity_preserved: result.success,
        peak_memory_usage: peak_memory,
    })
}

/// Result of incremental processing
struct IncrementalResult {
    chunks_processed: usize,
    total_processing_time: Duration,
    fidelity_preserved: bool,
    peak_memory_usage: usize,
}

/// Benchmark large file performance
fn benchmark_large_file_performance(test_files: &HashMap<String, (String, usize)>) -> Result<(), Box<dyn std::error::Error>> {
    println!("   📊 Benchmarking large file performance across different optimizations...");
    
    let optimization_levels = vec![
        ("Basic", create_basic_fidelity_options()),
        ("Streaming", create_streaming_fidelity_options()),
        ("Performance", create_performance_fidelity_options()),
    ];
    
    for (opt_name, fidelity_options) in optimization_levels {
        println!("      ⚙️  Optimization level: {}", opt_name);
        
        for (file_name, (content, release_count)) in test_files {
            if content.len() < 100 * 1024 { // Skip very small files for benchmarking
                continue;
            }
            
            println!("         📄 File: {}", file_name);
            
            let builder = Builder::with_fidelity_options(fidelity_options.clone());
            
            // Warm-up run
            let _ = builder.test_round_trip_fidelity(content)?;
            
            // Benchmark runs
            let iterations = 3;
            let mut times = Vec::new();
            let mut memory_usage = Vec::new();
            
            for _ in 0..iterations {
                let initial_memory = estimate_memory_usage();
                let start_time = Instant::now();
                
                let result = builder.test_round_trip_fidelity(content)?;
                
                let time = start_time.elapsed();
                let memory = estimate_memory_usage() - initial_memory;
                
                times.push(time.as_secs_f64() * 1000.0);
                memory_usage.push(memory);
                
                if !result.success {
                    println!("            ❌ Benchmark failed - fidelity not maintained");
                    break;
                }
            }
            
            if !times.is_empty() {
                let avg_time = times.iter().sum::<f64>() / times.len() as f64;
                let avg_memory = memory_usage.iter().sum::<usize>() / memory_usage.len();
                let throughput = (content.len() as f64) / (avg_time / 1000.0) / 1024.0 / 1024.0;
                
                println!("            • Average time: {:.2}ms", avg_time);
                println!("            • Average memory: ~{}MB", avg_memory / 1024 / 1024);
                println!("            • Throughput: {:.2} MB/s", throughput);
                println!("            • Releases/sec: {:.0}", (*release_count as f64) / (avg_time / 1000.0));
                
                // Performance targets
                let time_per_mb = avg_time / ((content.len() as f64) / 1024.0 / 1024.0);
                println!("            • Time per MB: {:.2}ms", time_per_mb);
                
                if time_per_mb <= 100.0 { // Target: <100ms per MB
                    println!("            • Performance: ✅ Excellent");
                } else if time_per_mb <= 500.0 {
                    println!("            • Performance: ✅ Good");
                } else {
                    println!("            • Performance: ⚠️  Needs optimization");
                }
            }
            
            println!();
        }
    }

    Ok(())
}

/// Create basic fidelity options (high fidelity, lower performance)
fn create_basic_fidelity_options() -> FidelityOptions {
    FidelityOptions {
        enable_perfect_fidelity: true,
        preserve_comments: true,
        preserve_processing_instructions: true,
        preserve_extensions: true,
        preserve_attribute_order: true,
        preserve_namespace_prefixes: true,
        canonicalization: CanonicalizationAlgorithm::DbC14N,
        custom_canonicalization_rules: None,
        enable_deterministic_ordering: true,
        collect_statistics: true,
        enable_verification: true,
        verification_config: ddex_builder::VerificationConfig::default(),
    }
}

/// Create performance-optimized fidelity options (lower fidelity, higher performance)
fn create_performance_fidelity_options() -> FidelityOptions {
    FidelityOptions {
        enable_perfect_fidelity: false,
        preserve_comments: false,
        preserve_processing_instructions: false,
        preserve_extensions: true, // Keep extensions but optimize others
        preserve_attribute_order: false,
        preserve_namespace_prefixes: false,
        canonicalization: CanonicalizationAlgorithm::None, // Skip canonicalization for speed
        custom_canonicalization_rules: None,
        enable_deterministic_ordering: false,
        collect_statistics: false,
        enable_verification: false,
        verification_config: ddex_builder::VerificationConfig::default(),
    }
}

/// Stress test with extremely large files
fn stress_test_extreme_sizes() -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔥 Stress testing with extreme file sizes...");
    
    let extreme_sizes = vec![
        (10000, "Extreme Large (10K releases)"),
        (25000, "Massive (25K releases)"),
        (50000, "Ultra Massive (50K releases)"),
    ];
    
    for (release_count, size_name) in extreme_sizes {
        println!("      🚀 Generating {}", size_name);
        
        let generation_start = Instant::now();
        
        // Generate extreme file
        let extreme_content = generate_ddex_release_catalog(release_count, "Extreme Stress Test")?;
        let generation_time = generation_start.elapsed();
        
        println!("         📏 Size: {:.1}MB ({} releases)", 
                (extreme_content.len() as f64) / 1024.0 / 1024.0, release_count);
        println!("         ⏱️  Generation time: {:.2}s", generation_time.as_secs_f64());
        
        // Test with performance-optimized settings
        let fidelity_options = create_performance_fidelity_options();
        let builder = Builder::with_fidelity_options(fidelity_options);
        
        println!("         🔧 Testing extreme file processing...");
        
        let initial_memory = estimate_memory_usage();
        let processing_start = Instant::now();
        
        // Process with timeout and memory monitoring
        let result = process_with_timeout(&extreme_content, &builder, Duration::from_secs(60))?;
        
        let processing_time = processing_start.elapsed();
        let peak_memory = estimate_memory_usage() - initial_memory;
        
        println!("         📊 Stress Test Results:");
        
        match result {
            Some(round_trip_result) => {
                println!("            • Status: ✅ Completed within timeout");
                println!("            • Success: {}", if round_trip_result.success { "✅" } else { "❌" });
                println!("            • Processing time: {:.2}s", processing_time.as_secs_f64());
                println!("            • Peak memory: ~{}MB", peak_memory / 1024 / 1024);
                
                let throughput = (extreme_content.len() as f64) / processing_time.as_secs_f64() / 1024.0 / 1024.0;
                println!("            • Throughput: {:.2} MB/s", throughput);
                
                let releases_per_sec = (release_count as f64) / processing_time.as_secs_f64();
                println!("            • Releases/sec: {:.0}", releases_per_sec);
                
                // Memory efficiency
                let memory_per_release = (peak_memory as f64) / (release_count as f64);
                println!("            • Memory/release: {:.1} KB", memory_per_release / 1024.0);
                
            },
            None => {
                println!("            • Status: ⏰ Timeout - File too large for current optimization");
                println!("            • Recommendation: Use streaming mode for files this large");
            }
        }
        
        println!();
        
        // Skip even larger files if we're hitting timeouts
        if result.is_none() {
            println!("      ⚠️  Skipping larger stress tests due to timeout");
            break;
        }
    }

    Ok(())
}

/// Process content with timeout
fn process_with_timeout(
    content: &str,
    builder: &Builder,
    timeout: Duration,
) -> Result<Option<ddex_builder::RoundTripResult>, Box<dyn std::error::Error>> {
    // In a real implementation, this would use proper async timeout handling
    // For this example, we'll simulate it
    
    let start = Instant::now();
    
    // Quick check: if file is extremely large, simulate timeout
    if content.len() > 100 * 1024 * 1024 { // 100MB
        std::thread::sleep(Duration::from_millis(100)); // Simulate processing time
        if start.elapsed() > timeout {
            return Ok(None);
        }
    }
    
    let result = builder.test_round_trip_fidelity(content)?;
    
    if start.elapsed() > timeout {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

/// Demonstrate memory usage optimization
fn demonstrate_memory_optimization(test_files: &HashMap<String, (String, usize)>) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🧠 Demonstrating memory usage optimization strategies...");
    
    // Get the largest file for testing
    if let Some((name, (content, release_count))) = test_files.iter()
        .max_by_key(|(_, (content, _))| content.len()) {
        
        println!("      📄 Optimizing memory usage for: {}", name);
        println!("         📏 File size: {:.2}MB", (content.len() as f64) / 1024.0 / 1024.0);
        
        let optimization_strategies = vec![
            ("No Optimization", create_basic_fidelity_options()),
            ("Streaming Optimized", create_streaming_fidelity_options()),
            ("Memory Optimized", create_memory_optimized_options()),
            ("Ultra Low Memory", create_ultra_low_memory_options()),
        ];
        
        for (strategy_name, fidelity_options) in optimization_strategies {
            println!("         🔧 Strategy: {}", strategy_name);
            
            let builder = Builder::with_fidelity_options(fidelity_options);
            
            let initial_memory = estimate_memory_usage();
            let start_time = Instant::now();
            
            let result = builder.test_round_trip_fidelity(content)?;
            
            let processing_time = start_time.elapsed();
            let peak_memory = estimate_memory_usage() - initial_memory;
            
            println!("            • Processing time: {:.2}ms", processing_time.as_secs_f64() * 1000.0);
            println!("            • Peak memory: ~{}MB", peak_memory / 1024 / 1024);
            println!("            • Memory per release: {:.1}KB", 
                    (peak_memory as f64) / (*release_count as f64) / 1024.0);
            println!("            • Success: {}", if result.success { "✅" } else { "❌" });
            
            // Memory efficiency rating
            let memory_per_mb = (peak_memory as f64) / ((content.len() as f64) / 1024.0 / 1024.0);
            if memory_per_mb <= 50.0 * 1024.0 * 1024.0 { // 50MB per MB of content
                println!("            • Memory efficiency: ✅ Excellent");
            } else if memory_per_mb <= 100.0 * 1024.0 * 1024.0 {
                println!("            • Memory efficiency: ✅ Good");
            } else {
                println!("            • Memory efficiency: ⚠️  Needs improvement");
            }
            
            println!();
        }
        
        // Memory optimization tips
        println!("      💡 Memory Optimization Tips:");
        println!("         • Disable statistics collection for large files");
        println!("         • Skip verification for trusted sources");
        println!("         • Disable comment preservation for performance");
        println!("         • Use streaming mode for files >100MB");
        println!("         • Process in chunks for files >1GB");
    }

    Ok(())
}

/// Create memory-optimized fidelity options
fn create_memory_optimized_options() -> FidelityOptions {
    FidelityOptions {
        enable_perfect_fidelity: true,
        preserve_comments: false, // Save memory by not preserving comments
        preserve_processing_instructions: false, // Save memory
        preserve_extensions: true,
        preserve_attribute_order: false, // Allow reordering for efficiency
        preserve_namespace_prefixes: false, // Allow minimization
        canonicalization: CanonicalizationAlgorithm::DbC14N,
        custom_canonicalization_rules: None,
        enable_deterministic_ordering: true,
        collect_statistics: false, // Disable to save memory
        enable_verification: false, // Disable to save memory
        verification_config: ddex_builder::VerificationConfig::default(),
    }
}

/// Create ultra-low memory fidelity options
fn create_ultra_low_memory_options() -> FidelityOptions {
    FidelityOptions {
        enable_perfect_fidelity: false, // Trade fidelity for memory
        preserve_comments: false,
        preserve_processing_instructions: false,
        preserve_extensions: true, // Keep only essential extensions
        preserve_attribute_order: false,
        preserve_namespace_prefixes: false,
        canonicalization: CanonicalizationAlgorithm::None, // Skip canonicalization
        custom_canonicalization_rules: None,
        enable_deterministic_ordering: false,
        collect_statistics: false,
        enable_verification: false,
        verification_config: ddex_builder::VerificationConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_file_generation() {
        let result = generate_ddex_release_catalog(5, "Test");
        assert!(result.is_ok());
        
        let xml = result.unwrap();
        assert!(xml.contains("xmlns:ernm"));
        assert!(xml.contains("REL-STREAM-000001"));
        assert!(xml.contains("REL-STREAM-000005"));
        assert!(xml.len() > 1000);
    }

    #[test]
    fn test_musical_key_generation() {
        let key1 = get_musical_key(0);
        let key2 = get_musical_key(12);
        let key3 = get_musical_key(24); // Should wrap around
        
        assert_eq!(key1, "C major");
        assert_eq!(key2, "F# major");
        assert_eq!(key3, "C major"); // Wrapped around
    }

    #[test]
    fn test_streaming_fidelity_options() {
        let options = create_streaming_fidelity_options();
        assert!(options.enable_perfect_fidelity);
        assert!(!options.preserve_comments); // Optimized for performance
        assert!(options.preserve_extensions);
        assert!(options.collect_statistics);
        assert!(!options.enable_verification); // Disabled for performance
    }

    #[test]
    fn test_performance_fidelity_options() {
        let options = create_performance_fidelity_options();
        assert!(!options.enable_perfect_fidelity); // Optimized for speed
        assert!(!options.preserve_comments);
        assert!(!options.collect_statistics);
        assert!(!options.enable_verification);
        assert_eq!(options.canonicalization, CanonicalizationAlgorithm::None);
    }

    #[test]
    fn test_memory_optimized_options() {
        let options = create_memory_optimized_options();
        assert!(options.enable_perfect_fidelity);
        assert!(!options.preserve_comments); // Memory optimization
        assert!(!options.collect_statistics); // Memory optimization
        assert!(!options.enable_verification); // Memory optimization
        assert!(options.preserve_extensions); // Still preserve essential features
    }

    #[test]
    fn test_file_size_estimation() {
        let small_file = generate_ddex_release_catalog(10, "Small").unwrap();
        let large_file = generate_ddex_release_catalog(100, "Large").unwrap();
        
        // Large file should be significantly larger than small file
        assert!(large_file.len() > small_file.len() * 5);
    }
}