//! Diff Comparison Example
//! 
//! This example demonstrates how to use the diff engine to compare DDEX releases,
//! track changes, and generate detailed reports for release updates.

use std::error::Error;

// Mock diff engine (in real implementation, these would be in the main library)
mod diff_engine {
    use super::*;
    
    #[derive(Clone)]
    pub struct DiffConfig {
        pub ignore_timestamps: bool,
        pub ignore_message_ids: bool,
        pub semantic_analysis: bool,
        pub include_technical_changes: bool,
        pub version_compatibility: VersionCompatibility,
    }
    
    #[derive(Clone)]
    pub enum VersionCompatibility {
        Strict,
        Lenient,
    }
    
    pub struct DiffEngine {
        config: DiffConfig,
    }
    
    #[derive(Debug)]
    pub struct ChangeSet {
        pub changes: Vec<SemanticChange>,
        pub summary: String,
        pub impact_assessment: ImpactAssessment,
    }
    
    #[derive(Debug)]
    pub struct SemanticChange {
        pub path: String,
        pub change_type: ChangeType,
        pub old_value: Option<String>,
        pub new_value: Option<String>,
        pub impact: ImpactLevel,
        pub description: String,
    }
    
    #[derive(Debug, PartialEq)]
    pub enum ChangeType {
        Added,
        Modified,
        Removed,
        Moved,
    }
    
    #[derive(Debug, PartialEq)]
    pub enum ImpactLevel {
        Low,
        Medium,
        High,
        Critical,
    }
    
    #[derive(Debug)]
    pub struct ImpactAssessment {
        pub overall_impact: ImpactLevel,
        pub breaking_changes: usize,
        pub compatibility_score: f64,
        pub recommendations: Vec<String>,
    }
    
    pub struct DiffFormatter;
    
    impl DiffEngine {
        pub fn new(config: DiffConfig) -> Self {
            Self { config }
        }
        
        pub fn compare_releases(&self, original_xml: &str, updated_xml: &str) -> Result<ChangeSet, Box<dyn Error>> {
            let mut changes = Vec::new();
            
            // Simulate diff analysis
            changes.extend(self.analyze_metadata_changes(original_xml, updated_xml));
            changes.extend(self.analyze_track_changes(original_xml, updated_xml));
            changes.extend(self.analyze_deal_changes(original_xml, updated_xml));
            
            if self.config.include_technical_changes {
                changes.extend(self.analyze_technical_changes(original_xml, updated_xml));
            }
            
            let impact_assessment = self.assess_overall_impact(&changes);
            let summary = self.generate_summary(&changes, &impact_assessment);
            
            Ok(ChangeSet {
                changes,
                summary,
                impact_assessment,
            })
        }
        
        fn analyze_metadata_changes(&self, original: &str, updated: &str) -> Vec<SemanticChange> {
            let mut changes = Vec::new();
            
            // Check title changes
            if let (Some(old_title), Some(new_title)) = (
                self.extract_element(original, "Title"),
                self.extract_element(updated, "Title")
            ) {
                if old_title != new_title {
                    changes.push(SemanticChange {
                        path: "Release/Title".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_title),
                        new_value: Some(new_title),
                        impact: ImpactLevel::High,
                        description: "Release title changed".to_string(),
                    });
                }
            }
            
            // Check artist changes
            if let (Some(old_artist), Some(new_artist)) = (
                self.extract_element(original, "DisplayArtist"),
                self.extract_element(updated, "DisplayArtist")
            ) {
                if old_artist != new_artist {
                    changes.push(SemanticChange {
                        path: "Release/DisplayArtist".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_artist),
                        new_value: Some(new_artist),
                        impact: ImpactLevel::Critical,
                        description: "Artist name changed - may affect recognition".to_string(),
                    });
                }
            }
            
            // Check release date changes
            if let (Some(old_date), Some(new_date)) = (
                self.extract_element(original, "ReleaseDate"),
                self.extract_element(updated, "ReleaseDate")
            ) {
                if old_date != new_date {
                    changes.push(SemanticChange {
                        path: "Release/ReleaseDate".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_date),
                        new_value: Some(new_date),
                        impact: ImpactLevel::Medium,
                        description: "Release date modified".to_string(),
                    });
                }
            }
            
            changes
        }
        
        fn analyze_track_changes(&self, original: &str, updated: &str) -> Vec<SemanticChange> {
            let mut changes = Vec::new();
            
            let original_track_count = original.matches("<SoundRecording>").count();
            let updated_track_count = updated.matches("<SoundRecording>").count();
            
            if original_track_count != updated_track_count {
                changes.push(SemanticChange {
                    path: "Resources/SoundRecordings".to_string(),
                    change_type: if updated_track_count > original_track_count {
                        ChangeType::Added
                    } else {
                        ChangeType::Removed
                    },
                    old_value: Some(original_track_count.to_string()),
                    new_value: Some(updated_track_count.to_string()),
                    impact: ImpactLevel::High,
                    description: format!("Track count changed from {} to {}", 
                                       original_track_count, updated_track_count),
                });
            }
            
            // Check for ISRC changes
            if original.contains("ISRC>") && updated.contains("ISRC>") {
                // Simulate ISRC comparison
                if self.extract_element(original, "ISRC") != self.extract_element(updated, "ISRC") {
                    changes.push(SemanticChange {
                        path: "SoundRecording/ISRC".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: self.extract_element(original, "ISRC"),
                        new_value: self.extract_element(updated, "ISRC"),
                        impact: ImpactLevel::Critical,
                        description: "ISRC code changed - critical for tracking".to_string(),
                    });
                }
            }
            
            changes
        }
        
        fn analyze_deal_changes(&self, original: &str, updated: &str) -> Vec<SemanticChange> {
            let mut changes = Vec::new();
            
            // Check territory changes
            if let (Some(old_territory), Some(new_territory)) = (
                self.extract_element(original, "TerritoryCode"),
                self.extract_element(updated, "TerritoryCode")
            ) {
                if old_territory != new_territory {
                    changes.push(SemanticChange {
                        path: "Deal/TerritoryCode".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_territory),
                        new_value: Some(new_territory),
                        impact: ImpactLevel::Medium,
                        description: "Distribution territory changed".to_string(),
                    });
                }
            }
            
            // Check pricing changes
            if let (Some(old_price), Some(new_price)) = (
                self.extract_element(original, "Price"),
                self.extract_element(updated, "Price")
            ) {
                if old_price != new_price {
                    changes.push(SemanticChange {
                        path: "Deal/Price".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_price),
                        new_value: Some(new_price),
                        impact: ImpactLevel::Low,
                        description: "Price updated".to_string(),
                    });
                }
            }
            
            changes
        }
        
        fn analyze_technical_changes(&self, original: &str, updated: &str) -> Vec<SemanticChange> {
            let mut changes = Vec::new();
            
            // Check audio quality changes
            if let (Some(old_bitrate), Some(new_bitrate)) = (
                self.extract_element(original, "BitRate"),
                self.extract_element(updated, "BitRate")
            ) {
                if old_bitrate != new_bitrate {
                    changes.push(SemanticChange {
                        path: "TechnicalDetails/BitRate".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_bitrate),
                        new_value: Some(new_bitrate),
                        impact: ImpactLevel::Low,
                        description: "Audio bit rate changed".to_string(),
                    });
                }
            }
            
            // Check codec changes
            if let (Some(old_codec), Some(new_codec)) = (
                self.extract_element(original, "Codec"),
                self.extract_element(updated, "Codec")
            ) {
                if old_codec != new_codec {
                    changes.push(SemanticChange {
                        path: "TechnicalDetails/Codec".to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_codec),
                        new_value: Some(new_codec),
                        impact: ImpactLevel::Medium,
                        description: "Audio codec format changed".to_string(),
                    });
                }
            }
            
            changes
        }
        
        fn extract_element(&self, xml: &str, element_name: &str) -> Option<String> {
            let start_tag = format!("<{}>", element_name);
            let end_tag = format!("</{}>", element_name);
            
            if let Some(start) = xml.find(&start_tag) {
                if let Some(end) = xml.find(&end_tag) {
                    let content_start = start + start_tag.len();
                    if content_start < end {
                        return Some(xml[content_start..end].to_string());
                    }
                }
            }
            None
        }
        
        fn assess_overall_impact(&self, changes: &[SemanticChange]) -> ImpactAssessment {
            let critical_count = changes.iter().filter(|c| c.impact == ImpactLevel::Critical).count();
            let high_count = changes.iter().filter(|c| c.impact == ImpactLevel::High).count();
            let medium_count = changes.iter().filter(|c| c.impact == ImpactLevel::Medium).count();
            let low_count = changes.iter().filter(|c| c.impact == ImpactLevel::Low).count();
            
            let overall_impact = if critical_count > 0 {
                ImpactLevel::Critical
            } else if high_count > 0 {
                ImpactLevel::High
            } else if medium_count > 0 {
                ImpactLevel::Medium
            } else {
                ImpactLevel::Low
            };
            
            let breaking_changes = critical_count + high_count;
            let non_breaking_changes = medium_count + low_count;
            let total_changes = changes.len();
            let compatibility_score = if total_changes > 0 {
                (non_breaking_changes as f64 / total_changes as f64) * 100.0
            } else {
                100.0
            };
            
            let mut recommendations = Vec::new();
            
            if critical_count > 0 {
                recommendations.push("Critical changes detected - verify downstream systems".to_string());
            }
            if high_count > 0 {
                recommendations.push("High impact changes may affect user experience".to_string());
            }
            if breaking_changes > total_changes / 2 {
                recommendations.push("Consider incremental rollout due to significant changes".to_string());
            }
            if compatibility_score < 70.0 {
                recommendations.push("Low compatibility - thorough testing recommended".to_string());
            }
            
            ImpactAssessment {
                overall_impact,
                breaking_changes,
                compatibility_score,
                recommendations,
            }
        }
        
        fn generate_summary(&self, changes: &[SemanticChange], impact: &ImpactAssessment) -> String {
            format!(
                "Found {} changes with {} impact. {} breaking changes detected. Compatibility score: {:.1}%",
                changes.len(),
                match impact.overall_impact {
                    ImpactLevel::Critical => "CRITICAL",
                    ImpactLevel::High => "HIGH",
                    ImpactLevel::Medium => "MEDIUM",
                    ImpactLevel::Low => "LOW",
                },
                impact.breaking_changes,
                impact.compatibility_score
            )
        }
    }
    
    impl DiffFormatter {
        pub fn new() -> Self {
            Self
        }
        
        pub fn format_human_readable(&self, changeset: &ChangeSet) -> Result<String, Box<dyn Error>> {
            let mut output = String::new();
            
            output.push_str("📊 DDEX Release Comparison Report\n");
            output.push_str("================================\n\n");
            
            output.push_str(&format!("📋 Summary: {}\n\n", changeset.summary));
            
            output.push_str("🔍 Changes Detected:\n");
            for (i, change) in changeset.changes.iter().enumerate() {
                let impact_icon = match change.impact {
                    ImpactLevel::Critical => "🚨",
                    ImpactLevel::High => "⚠️",
                    ImpactLevel::Medium => "📝",
                    ImpactLevel::Low => "ℹ️",
                };
                
                output.push_str(&format!(
                    "{}. {} {} ({})\n   Path: {}\n   Change: {} → {}\n   Description: {}\n\n",
                    i + 1,
                    impact_icon,
                    match change.change_type {
                        ChangeType::Added => "ADDED",
                        ChangeType::Modified => "MODIFIED",
                        ChangeType::Removed => "REMOVED",
                        ChangeType::Moved => "MOVED",
                    },
                    match change.impact {
                        ImpactLevel::Critical => "CRITICAL",
                        ImpactLevel::High => "HIGH",
                        ImpactLevel::Medium => "MEDIUM",
                        ImpactLevel::Low => "LOW",
                    },
                    change.path,
                    change.old_value.as_deref().unwrap_or("None"),
                    change.new_value.as_deref().unwrap_or("None"),
                    change.description
                ));
            }
            
            output.push_str("📈 Impact Assessment:\n");
            output.push_str(&format!("   Overall Impact: {:?}\n", changeset.impact_assessment.overall_impact));
            output.push_str(&format!("   Breaking Changes: {}\n", changeset.impact_assessment.breaking_changes));
            output.push_str(&format!("   Compatibility Score: {:.1}%\n\n", changeset.impact_assessment.compatibility_score));
            
            if !changeset.impact_assessment.recommendations.is_empty() {
                output.push_str("💡 Recommendations:\n");
                for rec in &changeset.impact_assessment.recommendations {
                    output.push_str(&format!("   • {}\n", rec));
                }
            }
            
            Ok(output)
        }
        
        pub fn format_json(&self, changeset: &ChangeSet) -> Result<String, Box<dyn Error>> {
            // Simplified JSON format for example
            Ok(format!(
                r#"{{
  "summary": "{}",
  "changes_count": {},
  "breaking_changes": {},
  "compatibility_score": {},
  "overall_impact": "{:?}",
  "changes": [{}]
}}"#,
                changeset.summary,
                changeset.changes.len(),
                changeset.impact_assessment.breaking_changes,
                changeset.impact_assessment.compatibility_score,
                changeset.impact_assessment.overall_impact,
                changeset.changes.iter()
                    .enumerate()
                    .map(|(i, c)| format!(r#"    {{
      "id": {},
      "path": "{}",
      "type": "{:?}",
      "impact": "{:?}",
      "description": "{}"
    }}"#, i, c.path, c.change_type, c.impact, c.description))
                    .collect::<Vec<_>>()
                    .join(",\n")
            ))
        }
        
        pub fn format_summary(&self, changeset: &ChangeSet) -> Result<String, Box<dyn Error>> {
            Ok(format!("DDEX Diff: {} changes, {} breaking, {:.1}% compatible", 
                     changeset.changes.len(),
                     changeset.impact_assessment.breaking_changes,
                     changeset.impact_assessment.compatibility_score))
        }
    }
}

use diff_engine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 DDEX Builder - Diff Comparison Example");
    println!("Comparing DDEX releases and analyzing changes...\n");
    
    // Create original and updated releases
    println!("📝 Creating original release...");
    let original_release = create_original_release();
    println!("  Title: {}", original_release.title);
    println!("  Artist: {}", original_release.artist);
    println!("  Tracks: {}", original_release.tracks.len());
    
    println!("\n📝 Creating updated release...");
    let updated_release = create_updated_release();
    println!("  Title: {}", updated_release.title);
    println!("  Artist: {}", updated_release.artist);
    println!("  Tracks: {}", updated_release.tracks.len());
    
    // Generate XML for both releases (simplified for example)
    let original_xml = generate_mock_xml(&original_release);
    let updated_xml = generate_mock_xml(&updated_release);
    
    // Configure diff engine
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    
    println!("\n🔍 Analyzing differences...");
    
    // Perform diff analysis
    let changeset = diff_engine.compare_releases(&original_xml, &updated_xml)?;
    
    println!("✅ Diff analysis completed");
    println!("📊 Found {} changes", changeset.changes.len());
    
    // Format and display results
    let formatter = DiffFormatter::new();
    
    println!("\n{}", "=".repeat(60));
    println!("📄 HUMAN-READABLE REPORT");
    println!("{}", "=".repeat(60));
    let human_report = formatter.format_human_readable(&changeset)?;
    println!("{}", human_report);
    
    // Save detailed JSON report
    let json_report = formatter.format_json(&changeset)?;
    std::fs::write("diff_report.json", &json_report)?;
    println!("💾 Detailed JSON report saved to: diff_report.json");
    
    // Display summary
    let summary = formatter.format_summary(&changeset)?;
    println!("\n📋 Summary: {}", summary);
    
    // Demonstrate different analysis scenarios
    println!("\n🎯 Analysis Scenarios:");
    demonstrate_version_compatibility_analysis(&original_xml, &updated_xml)?;
    demonstrate_incremental_diff_tracking().await?;
    demonstrate_impact_assessment(&changeset);
    
    Ok(())
}

// Mock release structure for example
#[derive(Clone)]
struct MockRelease {
    title: String,
    artist: String,
    label: String,
    release_date: String,
    genre: String,
    tracks: Vec<MockTrack>,
    price: Option<String>,
    territory: String,
    bitrate: String,
    codec: String,
}

#[derive(Clone)]
struct MockTrack {
    title: String,
    isrc: String,
    duration: String,
}

fn create_original_release() -> MockRelease {
    MockRelease {
        title: "Digital Dreams".to_string(),
        artist: "Synth Collective".to_string(),
        label: "Future Records".to_string(),
        release_date: "2024-01-15".to_string(),
        genre: "Electronic".to_string(),
        tracks: vec![
            MockTrack {
                title: "Neon Pulse".to_string(),
                isrc: "USFC12400001".to_string(),
                duration: "PT3M45S".to_string(),
            },
            MockTrack {
                title: "Cyber Dreams".to_string(),
                isrc: "USFC12400002".to_string(),
                duration: "PT4M12S".to_string(),
            },
        ],
        price: Some("9.99".to_string()),
        territory: "US".to_string(),
        bitrate: "1411".to_string(),
        codec: "FLAC".to_string(),
    }
}

fn create_updated_release() -> MockRelease {
    let mut updated = create_original_release();
    
    // Make updates that will be detected by diff engine
    updated.title = "Digital Dreams (Deluxe Edition)".to_string(); // Title change
    updated.price = Some("12.99".to_string()); // Price change
    updated.territory = "Worldwide".to_string(); // Territory expansion
    updated.bitrate = "2822".to_string(); // Higher quality audio
    
    // Add a new track
    updated.tracks.push(MockTrack {
        title: "Bonus Track".to_string(),
        isrc: "USFC12400003".to_string(),
        duration: "PT3M30S".to_string(),
    });
    
    updated
}

fn generate_mock_xml(release: &MockRelease) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
    <MessageHeader>
        <MessageId>TEST_001</MessageId>
    </MessageHeader>
    <ReleaseList>
        <Release>
            <ReleaseId>REL001</ReleaseId>
            <Title>{}</Title>
            <DisplayArtist>{}</DisplayArtist>
            <LabelName>{}</LabelName>
            <ReleaseDate>{}</ReleaseDate>
            <Genre>{}</Genre>
        </Release>
    </ReleaseList>
    <ResourceList>
        {}
    </ResourceList>
    <DealList>
        <ReleaseDeal>
            <DealId>DEAL001</DealId>
            <TerritoryCode>{}</TerritoryCode>
            <Price>{}</Price>
        </ReleaseDeal>
    </DealList>
</NewReleaseMessage>"#,
        release.title,
        release.artist,
        release.label,
        release.release_date,
        release.genre,
        release.tracks.iter()
            .enumerate()
            .map(|(i, track)| format!(r#"
        <SoundRecording>
            <ResourceId>SR{:03}</ResourceId>
            <Title>{}</Title>
            <ISRC>{}</ISRC>
            <Duration>{}</Duration>
            <TechnicalDetails>
                <BitRate>{}</BitRate>
                <Codec>{}</Codec>
            </TechnicalDetails>
        </SoundRecording>"#, i + 1, track.title, track.isrc, track.duration, release.bitrate, release.codec))
            .collect::<String>(),
        release.territory,
        release.price.as_deref().unwrap_or("0.00")
    )
}

fn demonstrate_version_compatibility_analysis(original_xml: &str, updated_xml: &str) -> Result<(), Box<dyn Error>> {
    println!("🔄 Version Compatibility Analysis:");
    
    // Strict compatibility analysis
    let strict_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let strict_engine = DiffEngine::new(strict_config);
    let strict_changeset = strict_engine.compare_releases(original_xml, updated_xml)?;
    
    println!("  📊 Strict Analysis: {} changes detected", strict_changeset.changes.len());
    
    // Lenient compatibility analysis
    let lenient_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: true, // More lenient
        semantic_analysis: true,
        include_technical_changes: false, // Ignore technical changes
        version_compatibility: VersionCompatibility::Lenient,
    };
    
    let lenient_engine = DiffEngine::new(lenient_config);
    let lenient_changeset = lenient_engine.compare_releases(original_xml, updated_xml)?;
    
    println!("  📊 Lenient Analysis: {} changes detected", lenient_changeset.changes.len());
    
    let compatibility_difference = strict_changeset.changes.len() - lenient_changeset.changes.len();
    if compatibility_difference > 0 {
        println!("  ⚠️  {} additional changes detected in strict mode", compatibility_difference);
    } else {
        println!("  ✅ Both analyses found similar results");
    }
    
    Ok(())
}

async fn demonstrate_incremental_diff_tracking() -> Result<(), Box<dyn Error>> {
    println!("📈 Incremental Diff Tracking:");
    
    let mut current_release = create_original_release();
    let mut previous_xml = generate_mock_xml(&current_release);
    
    let diff_config = DiffConfig {
        ignore_timestamps: true,
        ignore_message_ids: false,
        semantic_analysis: true,
        include_technical_changes: true,
        version_compatibility: VersionCompatibility::Strict,
    };
    
    let diff_engine = DiffEngine::new(diff_config);
    let mut total_changes = 0;
    
    // Simulate incremental updates
    for version in 1..=3 {
        // Make incremental changes
        match version {
            1 => {
                current_release.price = Some(format!("{}.99", 9 + version));
                println!("  📝 Version {}: Price updated", version);
            }
            2 => {
                current_release.territory = "Worldwide".to_string();
                println!("  📝 Version {}: Territory expanded", version);
            }
            3 => {
                current_release.title += " (Remastered)";
                println!("  📝 Version {}: Title updated with remaster info", version);
            }
            _ => {}
        }
        
        let current_xml = generate_mock_xml(&current_release);
        let changeset = diff_engine.compare_releases(&previous_xml, &current_xml)?;
        
        println!("    🔍 Changes detected: {}", changeset.changes.len());
        total_changes += changeset.changes.len();
        
        previous_xml = current_xml;
    }
    
    println!("  📊 Total incremental changes tracked: {}", total_changes);
    
    Ok(())
}

fn demonstrate_impact_assessment(changeset: &ChangeSet) {
    println!("⚖️  Impact Assessment:");
    
    let critical = changeset.changes.iter().filter(|c| c.impact == ImpactLevel::Critical).count();
    let high = changeset.changes.iter().filter(|c| c.impact == ImpactLevel::High).count();
    let medium = changeset.changes.iter().filter(|c| c.impact == ImpactLevel::Medium).count();
    let low = changeset.changes.iter().filter(|c| c.impact == ImpactLevel::Low).count();
    
    println!("  🚨 Critical: {} changes", critical);
    println!("  ⚠️  High: {} changes", high);
    println!("  📝 Medium: {} changes", medium);
    println!("  ℹ️  Low: {} changes", low);
    
    println!("  📊 Overall Impact: {:?}", changeset.impact_assessment.overall_impact);
    println!("  💔 Breaking Changes: {}", changeset.impact_assessment.breaking_changes);
    println!("  🎯 Compatibility Score: {:.1}%", changeset.impact_assessment.compatibility_score);
    
    if !changeset.impact_assessment.recommendations.is_empty() {
        println!("  💡 Recommendations:");
        for rec in &changeset.impact_assessment.recommendations {
            println!("    • {}", rec);
        }
    }
    
    // Risk assessment
    let risk_level = if critical > 0 {
        "🔴 HIGH RISK"
    } else if high > 0 {
        "🟡 MEDIUM RISK"
    } else if medium > 0 {
        "🟢 LOW RISK"
    } else {
        "✅ MINIMAL RISK"
    };
    
    println!("  🎯 Risk Level: {}", risk_level);
    
    // Deployment recommendations
    match changeset.impact_assessment.overall_impact {
        ImpactLevel::Critical => {
            println!("  📋 Deployment: Staged rollout with extensive testing recommended");
        }
        ImpactLevel::High => {
            println!("  📋 Deployment: Gradual rollout with monitoring recommended");
        }
        ImpactLevel::Medium => {
            println!("  📋 Deployment: Standard deployment with monitoring");
        }
        ImpactLevel::Low => {
            println!("  📋 Deployment: Safe for immediate deployment");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_release_creation() {
        let original = create_original_release();
        let updated = create_updated_release();
        
        assert_ne!(original.title, updated.title);
        assert_ne!(original.price, updated.price);
        assert_ne!(original.tracks.len(), updated.tracks.len());
    }
    
    #[test] 
    fn test_xml_generation() {
        let release = create_original_release();
        let xml = generate_mock_xml(&release);
        
        assert!(xml.contains(&release.title));
        assert!(xml.contains(&release.artist));
        assert!(xml.contains("ISRC"));
    }
    
    #[tokio::test]
    async fn test_diff_engine() {
        let original = create_original_release();
        let updated = create_updated_release();
        
        let original_xml = generate_mock_xml(&original);
        let updated_xml = generate_mock_xml(&updated);
        
        let config = DiffConfig {
            ignore_timestamps: true,
            ignore_message_ids: false,
            semantic_analysis: true,
            include_technical_changes: true,
            version_compatibility: VersionCompatibility::Strict,
        };
        
        let engine = DiffEngine::new(config);
        let changeset = engine.compare_releases(&original_xml, &updated_xml).unwrap();
        
        assert!(!changeset.changes.is_empty());
        assert!(changeset.changes.len() >= 3); // Title, price, territory changes minimum
    }
}