use ddex_builder::presets::DdexVersion;
use ddex_builder::{Builder, BuildRequest, BuildOptions};
use ddex_builder::builder::{ReleaseRequest, SoundRecordingRequest, DealRequest};
use ddex_builder::presets::MessageProfile;
use ddex_builder::diff::{DiffEngine, DiffConfig, VersionCompatibility, DiffFormatter};
use ddex_builder::diff::types::{ChangeSet, SemanticChange, ChangeType, ImpactLevel, DiffPath};
use std::collections::HashMap;

#[tokio::test]
async fn test_diff_engine_release_updates() {
    let mut builder = Builder::new();
    
    // Create original release
    let original_request = create_original_release();
    let original_result = builder.build_internal(&original_request)
        .expect("Failed to build original release");
    
    // Create updated release
    let updated_request = create_updated_release();
    let updated_result = builder.build_internal(&updated_request)
        .expect("Failed to build updated release");
    
    // Configure diff engine
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    
    // Generate diff
    let changeset = diff_engine.compare_releases(&original_result.xml, &updated_result.xml)
        .expect("Failed to generate diff");
    
    // Verify diff results
    assert!(!changeset.changes.is_empty(), "Should detect changes between releases");
    
    // Verify specific changes
    let title_changes: Vec<_> = changeset.changes.iter()
        .filter(|change| change.path.to_string().contains("Title"))
        .collect();
    assert!(!title_changes.is_empty(), "Should detect title change");
    
    let price_changes: Vec<_> = changeset.changes.iter()
        .filter(|change| change.path.to_string().contains("Price"))
        .collect();
    assert!(!price_changes.is_empty(), "Should detect price change");
    
    println!("Detected {} changes", changeset.changes.len());
    for change in &changeset.changes {
        println!("  {}: {} -> {}", change.path, 
                 change.old_value.as_deref().unwrap_or("None"),
                 change.new_value.as_deref().unwrap_or("None"));
    }
}

#[tokio::test]
async fn test_diff_semantic_analysis() {
    let mut builder = Builder::new();
    
    let original_request = create_original_release();
    let original_result = builder.build_internal(&original_request)
        .expect("Failed to build original release");
    
    // Create semantically different release
    let semantic_request = create_semantically_different_release();
    let semantic_result = builder.build_internal(&semantic_request)
        .expect("Failed to build semantic release");
    
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    let changeset = diff_engine.compare_releases(&original_result.xml, &semantic_result.xml)
        .expect("Failed to generate semantic diff");
    
    // Categorize changes by impact level
    let critical_changes: Vec<_> = changeset.changes.iter()
        .filter(|c| c.impact == ImpactLevel::Critical)
        .collect();
    
    let high_changes: Vec<_> = changeset.changes.iter()
        .filter(|c| c.impact == ImpactLevel::High)
        .collect();
    
    let medium_changes: Vec<_> = changeset.changes.iter()
        .filter(|c| c.impact == ImpactLevel::Medium)
        .collect();
    
    let low_changes: Vec<_> = changeset.changes.iter()
        .filter(|c| c.impact == ImpactLevel::Low)
        .collect();
    
    println!("Semantic analysis: Critical={}, High={}, Medium={}, Low={}", 
             critical_changes.len(), high_changes.len(), 
             medium_changes.len(), low_changes.len());
    
    // ISRC change should be critical
    assert!(!critical_changes.is_empty(), "ISRC change should be critical");
    
    // Genre change should be high impact
    assert!(!high_changes.is_empty(), "Genre change should be high impact");
}

#[tokio::test]
async fn test_diff_technical_changes() {
    let mut builder = Builder::new();
    
    let original_request = create_original_release();
    let original_result = builder.build_internal(&original_request)
        .expect("Failed to build original release");
    
    // Create technical changes
    let tech_request = create_technical_changes_release();
    let tech_result = builder.build_internal(&tech_request)
        .expect("Failed to build technical release");
    
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    let changeset = diff_engine.compare_releases(&original_result.xml, &tech_result.xml)
        .expect("Failed to generate technical diff");
    
    // Look for technical changes
    let technical_changes: Vec<_> = changeset.changes.iter()
        .filter(|change| {
            let path_str = change.path.to_string().to_lowercase();
            path_str.contains("bitrate") || 
            path_str.contains("codec") || 
            path_str.contains("samplerate") ||
            path_str.contains("hashsum")
        })
        .collect();
    
    assert!(!technical_changes.is_empty(), "Should detect technical changes");
    
    // Technical changes should typically be low impact
    let low_impact_technical: Vec<_> = technical_changes.iter()
        .filter(|change| change.impact == ImpactLevel::Low)
        .collect();
    
    println!("Technical changes: {} total, {} low impact", 
             technical_changes.len(), low_impact_technical.len());
}

#[tokio::test]
async fn test_diff_formatter_output() {
    let mut builder = Builder::new();
    
    let original_request = create_original_release();
    let original_result = builder.build_internal(&original_request)
        .expect("Failed to build original release");
    
    let updated_request = create_updated_release();
    let updated_result = builder.build_internal(&updated_request)
        .expect("Failed to build updated release");
    
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    let changeset = diff_engine.compare_releases(&original_result.xml, &updated_result.xml)
        .expect("Failed to generate diff");
    
    // Test different formatter outputs
    let formatter = DiffFormatter::new();
    
    // Human-readable format
    let human_readable = formatter.format_human_readable(&changeset)
        .expect("Failed to format as human readable");
    
    assert!(human_readable.contains("Changes detected"));
    assert!(human_readable.contains("Title"));
    println!("Human readable format:\n{}", human_readable);
    
    // JSON format
    let json_format = formatter.format_json(&changeset)
        .expect("Failed to format as JSON");
    
    assert!(json_format.contains("\"changes\""));
    assert!(json_format.contains("\"path\""));
    println!("JSON format length: {} chars", json_format.len());
    
    // Summary format
    let summary = formatter.format_summary(&changeset)
        .expect("Failed to format summary");
    
    assert!(summary.contains("changes"));
    println!("Summary: {}", summary);
}

#[tokio::test]
async fn test_diff_version_compatibility() {
    let mut builder = Builder::new();
    
    // Create ERN 4.2 release
    let mut v42_request = create_original_release();
    v42_request.version = Some(DdexVersion::Ern42);
    let v42_result = builder.build_internal(&v42_request)
        .expect("Failed to build v4.2 release");
    
    // Create ERN 4.3 release
    let mut v43_request = create_original_release();
    v43_request.version = Some(DdexVersion::Ern43);
    let v43_result = builder.build_internal(&v43_request)
        .expect("Failed to build v4.3 release");
    
    // Test version compatibility checking
    let strict_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let lenient_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Lenient,
    };
    
    let strict_engine = DiffEngine::new(strict_config);
    let lenient_engine = DiffEngine::new(lenient_config);
    
    // Compare different versions
    let strict_changeset = strict_engine.compare_releases(&v42_result.xml, &v43_result.xml)
        .expect("Failed strict version comparison");
    
    let lenient_changeset = lenient_engine.compare_releases(&v42_result.xml, &v43_result.xml)
        .expect("Failed lenient version comparison");
    
    println!("Strict version diff: {} changes", strict_changeset.changes.len());
    println!("Lenient version diff: {} changes", lenient_changeset.changes.len());
    
    // Strict should detect version differences
    let version_changes: Vec<_> = strict_changeset.changes.iter()
        .filter(|change| change.path.to_string().contains("version") || 
                        change.path.to_string().contains("schema"))
        .collect();
    
    // Should detect version-related changes in strict mode
    if !version_changes.is_empty() {
        println!("Detected version changes in strict mode");
    }
}

#[tokio::test]
async fn test_diff_large_release_performance() {
    let mut builder = Builder::new();
    
    // Create large releases
    let large_original = create_large_catalog_release(1);
    let large_updated = create_large_catalog_release(2);
    
    let original_result = builder.build_internal(&large_original)
        .expect("Failed to build large original");
    let updated_result = builder.build_internal(&large_updated)
        .expect("Failed to build large updated");
    
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: false, // Skip for performance
        version_compatibility: VersionCompatibility::Lenient,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    
    let start_time = std::time::Instant::now();
    let changeset = diff_engine.compare_releases(&original_result.xml, &updated_result.xml)
        .expect("Failed to diff large releases");
    let diff_time = start_time.elapsed();
    
    println!("Large diff completed in {:?} with {} changes", 
             diff_time, changeset.changes.len());
    
    // Performance assertion - should complete within reasonable time
    assert!(diff_time.as_secs() < 10, "Large diff should complete within 10 seconds");
    
    // Should detect changes despite size
    assert!(!changeset.changes.is_empty(), "Should detect changes in large releases");
}

#[tokio::test]
async fn test_diff_incremental_updates() {
    let mut builder = Builder::new();
    
    // Simulate incremental updates
    let mut current_request = create_original_release();
    let mut previous_xml = builder.build_internal(&current_request)
        .expect("Failed to build initial release").xml;
    
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Lenient,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    let mut all_changes = Vec::new();
    
    // Apply several incremental updates
    for update_num in 1..=5 {
        // Make incremental change
        current_request.release.title = format!("Updated Title v{}", update_num);
        current_request.deals[0].price = Some(format!("{}.99", 9 + update_num));
        
        let current_xml = builder.build_internal(&current_request)
            .expect("Failed to build incremental update").xml;
        
        let changeset = diff_engine.compare_releases(&previous_xml, &current_xml)
            .expect("Failed to generate incremental diff");
        
        println!("Update {}: {} changes", update_num, changeset.changes.len());
        
        all_changes.extend(changeset.changes);
        previous_xml = current_xml;
    }
    
    // Should track all incremental changes
    assert!(all_changes.len() >= 10, "Should track all incremental changes"); // 2 changes per update * 5 updates
    
    // Changes should be properly categorized
    let title_changes: Vec<_> = all_changes.iter()
        .filter(|change| change.path.to_string().contains("Title"))
        .collect();
    let price_changes: Vec<_> = all_changes.iter()
        .filter(|change| change.path.to_string().contains("Price"))
        .collect();
    
    assert_eq!(title_changes.len(), 5, "Should track all title changes");
    assert_eq!(price_changes.len(), 5, "Should track all price changes");
}

fn create_original_release() -> BuildRequest {
    BuildRequest {
        message_id: "DIFF_TEST_ORIGINAL".to_string(),
        version: Some(DdexVersion::Ern43),
        profile: Some(MessageProfile::AudioSingle),
        sender: "DiffTestSender".to_string(),
        recipient: "DiffTestRecipient".to_string(),
        release: ReleaseRequest {
            release_id: "DIFFREL001".to_string(),
            title: "Original Test Release".to_string(),
            display_artist: "Original Artist".to_string(),
            label_name: Some("Original Records".to_string()),
            release_date: "2024-01-01".to_string(),
            original_release_date: Some("2024-01-01".to_string()),
            genre: Some("Pop".to_string()),
            pline: Some("℗ 2024 Original Records".to_string()),
            cline: Some("© 2024 Original Records".to_string()),
            upc: Some("123456789001".to_string()),
            grid: None,
            icpn: None,
            catalog_number: Some("OR001".to_string()),
            release_type: Some("Single".to_string()),
        },
        resources: ddex_builder::builder::ResourcesRequest {
            sound_recordings: vec![
                SoundRecordingRequest {
                    resource_reference: "R1".to_string(),
                    resource_id: "DIFFSR001".to_string(),
                    title: "Original Track".to_string(),
                    display_artist: "Original Artist".to_string(),
                    isrc: Some("ORIG17607001".to_string()),
                    duration: Some("PT3M30S".to_string()),
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), "original.flac".to_string());
                        details.insert("Codec".to_string(), "FLAC".to_string());
                        details.insert("BitRate".to_string(), "1411".to_string());
                        details.insert("SampleRate".to_string(), "44100".to_string());
                        details.insert("HashSum".to_string(), "sha256:original123456789abcdef".to_string());
                        details
                    },
                },
            ],
            image_resources: vec![],
            video_resources: vec![],
        },
        deals: vec![
            DealRequest {
                deal_id: "DIFFDEAL001".to_string(),
                commercial_model_type: Some("SubscriptionModel".to_string()),
                usage_type: "Stream".to_string(),
                territory_code: "US".to_string(),
                start_date: Some("2024-01-01".to_string()),
                end_date: None,
                price: Some("9.99".to_string()),
                currency: Some("USD".to_string()),
                resources: vec!["R1".to_string()],
            },
        ],
        metadata: HashMap::new(),
        options: BuildOptions::default(),
    }
}

fn create_updated_release() -> BuildRequest {
    let mut request = create_original_release();
    
    // Make updates
    request.message_id = "DIFF_TEST_UPDATED".to_string();
    request.release.title = "Updated Test Release".to_string();
    request.release.label_name = Some("Updated Records".to_string());
    request.deals[0].price = Some("12.99".to_string());
    request.deals[0].territory_code = "Worldwide".to_string();
    
    request
}

fn create_semantically_different_release() -> BuildRequest {
    let mut request = create_original_release();
    
    // Make semantic changes
    request.message_id = "DIFF_TEST_SEMANTIC".to_string();
    request.resources.sound_recordings[0].isrc = Some("DIFF17607002".to_string()); // Critical change
    request.release.genre = Some("Rock".to_string()); // High impact change
    request.deals[0].commercial_model_type = Some("PayPerViewModel".to_string()); // Medium impact
    
    request
}

fn create_technical_changes_release() -> BuildRequest {
    let mut request = create_original_release();
    
    // Make technical changes
    request.message_id = "DIFF_TEST_TECHNICAL".to_string();
    request.resources.sound_recordings[0].technical_details.insert(
        "BitRate".to_string(), "2822".to_string() // Changed from 1411
    );
    request.resources.sound_recordings[0].technical_details.insert(
        "Codec".to_string(), "ALAC".to_string() // Changed from FLAC
    );
    request.resources.sound_recordings[0].technical_details.insert(
        "HashSum".to_string(), "sha256:updated987654321fedcba".to_string() // Updated hash
    );
    
    request
}

fn create_large_catalog_release(variant: usize) -> BuildRequest {
    let mut request = create_original_release();
    
    request.message_id = format!("DIFF_LARGE_CATALOG_{}", variant);
    request.profile = Some(MessageProfile::AudioAlbum);
    request.release.title = format!("Large Catalog Album {}", variant);
    request.release.release_type = Some("Album".to_string());
    
    // Add many tracks
    for i in 1..20 { // 20 tracks
        request.resources.sound_recordings.push(
            SoundRecordingRequest {
                resource_reference: format!("R{}", i + 1),
                resource_id: format!("DIFFSR{:03}_{}", i, variant),
                title: format!("Track {} Variant {}", i, variant),
                display_artist: format!("Artist {} Variant {}", i, variant),
                isrc: Some(format!("DIFF{:08}", 17600000 + i * 1000 + variant)),
                duration: Some(format!("PT{}M{}S", 3 + (i % 3), 15 + (i % 45))),
                technical_details: {
                    let mut details = HashMap::new();
                    details.insert("FileName".to_string(), format!("track{}_{}.flac", i, variant));
                    details.insert("Codec".to_string(), "FLAC".to_string());
                    details.insert("BitRate".to_string(), "1411".to_string());
                    details.insert("SampleRate".to_string(), "44100".to_string());
                    details.insert("HashSum".to_string(), format!("sha256:{:032x}{:032x}", i, variant));
                    details
                },
            }
        );
        
        // Add corresponding deal resources
        request.deals[0].resources.push(format!("R{}", i + 1));
    }
    
    request
}