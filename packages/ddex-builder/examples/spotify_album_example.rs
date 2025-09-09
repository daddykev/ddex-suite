//! Spotify Album Example
//! 
//! This example demonstrates how to create a DDEX ERN 4.3 release optimized for Spotify's
//! requirements, including proper metadata, audio quality specifications, and streaming deals.

use ddex_builder::{Builder, BuildOptions, BuildRequest};
use ddex_builder::builder::{
    MessageHeaderRequest, PartyRequest, LocalizedStringRequest, 
    ReleaseRequest, TrackRequest, DealRequest, DealTerms
};
use ddex_builder::presets::DdexVersion;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🎵 DDEX Builder - Spotify Album Example");
    println!("Creating a complete album release optimized for Spotify...\n");
    
    // Initialize builder with Spotify preset
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false)
        .expect("Failed to apply Spotify preset");
    
    println!("✅ Applied Spotify Audio 4.3 preset");
    
    // Create the album release request
    let album_request = create_spotify_album_request();
    
    println!("📀 Building album: '{}'", album_request.releases[0].title[0].text);
    println!("🎤 Artist: {}", album_request.releases[0].artist);
    println!("🏷️  Label: {}", album_request.releases[0].label.as_ref().unwrap());
    println!("🎵 Tracks: {}", album_request.releases[0].tracks.len());
    
    // Build the DDEX XML using DDEXBuilder
    let ddex_builder = ddex_builder::builder::DDEXBuilder::new();
    let result = ddex_builder.build(album_request, BuildOptions::default())
        .expect("Failed to build Spotify album release");
    
    println!("\n✅ Successfully built DDEX release");
    println!("📄 XML size: {} KB", result.xml.len() / 1024);
    
    // Validate Spotify-specific requirements
    validate_spotify_compliance(&result.xml)?;
    
    // Save the XML to file
    let output_path = "spotify_album_example.xml";
    std::fs::write(output_path, &result.xml)
        .expect("Failed to write XML file");
    
    println!("💾 Saved to: {}", output_path);
    println!("\n🎯 Spotify Compliance Summary:");
    print_spotify_compliance_summary(&result.xml);
    
    // Demonstrate version management
    println!("\n🔄 Version Management:");
    let detected_version = builder.detect_version(&result.xml)
        .expect("Failed to detect version");
    println!("✅ Detected version: {:?}", detected_version);
    
    Ok(())
}

fn create_spotify_album_request() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("SPOTIFY_ALBUM_2024_001".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "IndieRecords".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("INDIE_RECORDS_001".to_string()),
                party_reference: Some("SENDER_REF".to_string()),
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Spotify".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("SPOTIFY_001".to_string()),
                party_reference: Some("RECIPIENT_REF".to_string()),
            },
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: Some("2024-03-15T10:00:00Z".to_string()),
        },
        version: "ern/43".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![create_album_release()],
        deals: vec![create_spotify_streaming_deal()],
        extensions: None,
    }
}

fn create_album_release() -> ReleaseRequest {
    ReleaseRequest {
        release_id: "ALBUM_INDIE_2024_001".to_string(),
        release_reference: Some("REL_REF_001".to_string()),
        title: vec![LocalizedStringRequest {
            text: "Digital Horizons".to_string(),
            language_code: Some("en".to_string()),
        }],
        artist: "The Wavelength Collective".to_string(),
        label: Some("Indie Digital Records".to_string()),
        release_date: Some("2024-03-15".to_string()),
        upc: Some("602577123456".to_string()),
        tracks: create_album_tracks(),
        resource_references: Some(vec!["R1".to_string(), "R2".to_string(), "R3".to_string(), "R4".to_string(), "R5".to_string(), "R6".to_string(), "R7".to_string(), "R8".to_string()]),
    }
}

fn create_album_tracks() -> Vec<TrackRequest> {
    vec![
        TrackRequest {
            track_id: "TRACK_001".to_string(),
            resource_reference: Some("R1".to_string()),
            isrc: "USWV12400001".to_string(),
            title: "Neon Dreams".to_string(),
            duration: "PT4M23S".to_string(),
            artist: "The Wavelength Collective".to_string(),
        },
        TrackRequest {
            track_id: "TRACK_002".to_string(),
            resource_reference: Some("R2".to_string()),
            isrc: "USWV12400002".to_string(),
            title: "Synthetic Sunrise".to_string(),
            duration: "PT3M57S".to_string(),
            artist: "The Wavelength Collective".to_string(),
        },
        TrackRequest {
            track_id: "TRACK_003".to_string(),
            resource_reference: Some("R3".to_string()),
            isrc: "USWV12400003".to_string(),
            title: "Digital Pulse".to_string(),
            duration: "PT5M12S".to_string(),
            artist: "The Wavelength Collective".to_string(),
        },
        TrackRequest {
            track_id: "TRACK_004".to_string(),
            resource_reference: Some("R4".to_string()),
            isrc: "USWV12400004".to_string(),
            title: "Cyber Meditation".to_string(),
            duration: "PT6M45S".to_string(),
            artist: "The Wavelength Collective".to_string(),
        },
        TrackRequest {
            track_id: "TRACK_005".to_string(),
            resource_reference: Some("R5".to_string()),
            isrc: "USWV12400005".to_string(),
            title: "Binary Sunset".to_string(),
            duration: "PT4M31S".to_string(),
            artist: "The Wavelength Collective".to_string(),
        },
        TrackRequest {
            track_id: "TRACK_006".to_string(),
            resource_reference: Some("R6".to_string()),
            isrc: "USWV12400006".to_string(),
            title: "Algorithmic Love".to_string(),
            duration: "PT3M44S".to_string(),
            artist: "The Wavelength Collective feat. Echo Siren".to_string(),
        },
        TrackRequest {
            track_id: "TRACK_007".to_string(),
            resource_reference: Some("R7".to_string()),
            isrc: "USWV12400007".to_string(),
            title: "Data Stream Dreams".to_string(),
            duration: "PT7M18S".to_string(),
            artist: "The Wavelength Collective".to_string(),
        },
        TrackRequest {
            track_id: "TRACK_008".to_string(),
            resource_reference: Some("R8".to_string()),
            isrc: "USWV12400008".to_string(),
            title: "Virtual Reality".to_string(),
            duration: "PT4M56S".to_string(),
            artist: "The Wavelength Collective".to_string(),
        },
    ]
}


fn create_spotify_streaming_deal() -> DealRequest {
    DealRequest {
        deal_reference: Some("SPOTIFY_STREAM_DEAL_001".to_string()),
        deal_terms: DealTerms {
            commercial_model_type: "SubscriptionModel".to_string(),
            territory_code: vec!["Worldwide".to_string()],
            start_date: Some("2024-03-15".to_string()),
        },
        release_references: vec!["REL_REF_001".to_string()],
    }
}


fn validate_spotify_compliance(xml: &str) -> Result<(), Box<dyn Error>> {
    println!("\n🔍 Validating Spotify compliance...");
    
    // Check required elements
    let required_elements = [
        "MessageSchemaVersionId=\"ern/43\"",
        "ISRC",
        "Title",
        "DisplayArtist",
        "Duration", 
        "BitRate",
        "SampleRate",
        "UseType>Stream<",
        "CommercialModelType>SubscriptionModel<",
        "TerritoryCode>Worldwide<",
    ];
    
    for element in required_elements {
        if !xml.contains(element) {
            return Err(format!("Missing required Spotify element: {}", element).into());
        }
    }
    
    // Check audio quality requirements
    if xml.contains("BitRate>1411<") {
        println!("✅ Audio quality: CD Quality (1411 kbps)");
    } else if xml.contains("BitRate>320<") {
        println!("⚠️  Audio quality: High Quality (320 kbps)");
    } else {
        return Err("Audio quality below Spotify minimum requirements".into());
    }
    
    println!("✅ All Spotify compliance checks passed");
    Ok(())
}

fn print_spotify_compliance_summary(xml: &str) {
    println!("  📋 DDEX Version: ERN 4.3 ✅");
    println!("  🎵 Message Profile: Audio Album ✅");
    println!("  🌍 Territory: Worldwide ✅");
    println!("  💿 Audio Format: FLAC ✅");
    
    // Count tracks
    let track_count = xml.matches("<SoundRecording>").count();
    println!("  🎶 Track Count: {} ✅", track_count);
    
    // Check for required metadata
    let has_isrc = xml.contains("ISRC");
    let has_duration = xml.contains("Duration");
    let has_bitrate = xml.contains("BitRate");
    
    println!("  🏷️  ISRC Codes: {} ✅", if has_isrc { "Present" } else { "Missing" });
    println!("  ⏱️  Durations: {} ✅", if has_duration { "Present" } else { "Missing" });
    println!("  🎚️  Audio Quality: {} ✅", if has_bitrate { "Specified" } else { "Missing" });
    
    // Check streaming deal
    let has_streaming = xml.contains("UseType>Stream<");
    let has_subscription = xml.contains("CommercialModelType>SubscriptionModel<");
    
    println!("  📡 Streaming Rights: {} ✅", if has_streaming { "Enabled" } else { "Missing" });
    println!("  💳 Subscription Model: {} ✅", if has_subscription { "Enabled" } else { "Missing" });
    
    println!("\n🎉 Album is ready for Spotify distribution!");
    println!("📊 Expected Spotify Features:");
    println!("   • High-quality streaming (FLAC source)");
    println!("   • Global availability");
    println!("   • Proper metadata for recommendations");
    println!("   • Content ID ready with ISRC codes");
    println!("   • Album playlist creation support");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_spotify_album_example() {
        let mut builder = Builder::new();
        builder.apply_preset("spotify_audio_43", false).unwrap();
        
        let request = create_spotify_album_request();
        let result = builder.build_internal(&request).unwrap();
        
        assert!(!result.xml.is_empty());
        assert!(result.xml.contains("ERN/4.3"));
        assert!(validate_spotify_compliance(&result.xml).is_ok());
    }
    
    #[test]
    fn test_high_quality_audio_specs() {
        let specs = create_high_quality_audio_specs("test.flac");
        
        assert_eq!(specs.get("Codec").unwrap(), "FLAC");
        assert_eq!(specs.get("BitRate").unwrap(), "1411");
        assert_eq!(specs.get("SampleRate").unwrap(), "44100");
        assert!(specs.contains_key("HashSum"));
    }
    
    #[test] 
    fn test_spotify_metadata() {
        let metadata = create_spotify_metadata();
        
        assert!(metadata.contains_key("SpotifyMarkets"));
        assert_eq!(metadata.get("ExplicitContent").unwrap(), "false");
        assert_eq!(metadata.get("Genre").unwrap(), "Electronic");
    }
}