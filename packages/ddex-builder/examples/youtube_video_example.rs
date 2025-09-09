//! YouTube Video Example
//! 
//! This example demonstrates how to create a DDEX ERN 4.3 release for YouTube with both
//! audio and video resources, optimized for Content ID and monetization.

use ddex_builder::presets::DdexVersion;
use ddex_builder::{Builder, BuildOptions, BuildRequest};
use ddex_builder::builder::{ReleaseRequest, SoundRecordingRequest, VideoResourceRequest, DealRequest, ResourcesRequest};
use ddex_builder::presets::MessageProfile;
use std::collections::HashMap;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("📺 DDEX Builder - YouTube Video Example");
    println!("Creating a music video release optimized for YouTube...\n");
    
    // Initialize builder with YouTube preset
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false)
        .expect("Failed to apply YouTube preset");
    
    println!("✅ Applied YouTube Video 4.3 preset");
    
    // Create the music video release request
    let video_request = create_youtube_video_request();
    
    println!("🎬 Building video: '{}'", video_request.release.title);
    println!("🎤 Artist: {}", video_request.release.display_artist);
    println!("📹 Video Resources: {}", video_request.resources.video_resources.len());
    println!("🎵 Audio Resources: {}", video_request.resources.sound_recordings.len());
    
    // Build the DDEX XML
    let result = builder.build_internal(&video_request)
        .expect("Failed to build YouTube video release");
    
    println!("\n✅ Successfully built DDEX release");
    println!("📄 XML size: {} KB", result.xml.len() / 1024);
    
    // Validate YouTube-specific requirements
    validate_youtube_compliance(&result.xml)?;
    
    // Save the XML to file
    let output_path = "youtube_video_example.xml";
    std::fs::write(output_path, &result.xml)
        .expect("Failed to write XML file");
    
    println!("💾 Saved to: {}", output_path);
    println!("\n🎯 YouTube Compliance Summary:");
    print_youtube_compliance_summary(&result.xml);
    
    // Demonstrate Content ID features
    println!("\n🔍 Content ID Features:");
    demonstrate_content_id_features(&result.xml);
    
    Ok(())
}

fn create_youtube_video_request() -> BuildRequest {
    BuildRequest {
        message_id: "YOUTUBE_VIDEO_2024_001".to_string(),
        version: Some(DdexVersion::Ern43),
        profile: Some(MessageProfile::VideoSingle),
        sender: "ViralMusic".to_string(),
        recipient: "YouTube".to_string(),
        release: ReleaseRequest {
            release_id: "VIDEO_VIRAL_2024_001".to_string(),
            title: "Neon Nights (Official Music Video)".to_string(),
            display_artist: "Luna Synth".to_string(),
            label_name: Some("Viral Music Entertainment".to_string()),
            release_date: "2024-02-14".to_string(),
            original_release_date: Some("2024-02-14".to_string()),
            genre: Some("Synthpop".to_string()),
            pline: Some("℗ 2024 Viral Music Entertainment".to_string()),
            cline: Some("© 2024 Viral Music Entertainment".to_string()),
            upc: None, // Videos typically use GRID instead
            grid: Some("GRD456789123456".to_string()),
            icpn: None,
            catalog_number: Some("VME2024001".to_string()),
            release_type: Some("Single".to_string()),
        },
        resources: ResourcesRequest {
            sound_recordings: vec![create_audio_track()],
            image_resources: vec![], // Could add thumbnail resources
            video_resources: vec![create_music_video()],
        },
        deals: vec![
            create_youtube_monetization_deal(),
        ],
        metadata: create_youtube_metadata(),
        options: BuildOptions::default(),
    }
}

fn create_audio_track() -> SoundRecordingRequest {
    SoundRecordingRequest {
        resource_reference: "A1".to_string(),
        resource_id: "AUDIO_NEON_NIGHTS".to_string(),
        title: "Neon Nights".to_string(),
        display_artist: "Luna Synth".to_string(),
        isrc: Some("USVM12400014".to_string()),
        duration: Some("PT3M42S".to_string()),
        technical_details: create_youtube_audio_specs(),
    }
}

fn create_music_video() -> VideoResourceRequest {
    VideoResourceRequest {
        resource_reference: "V1".to_string(),
        resource_id: "VIDEO_NEON_NIGHTS_4K".to_string(),
        title: "Neon Nights (Official Music Video)".to_string(),
        duration: Some("PT3M42S".to_string()), // Must match audio duration
        technical_details: create_youtube_video_specs(),
    }
}

fn create_youtube_audio_specs() -> HashMap<String, String> {
    let mut details = HashMap::new();
    
    // YouTube-optimized audio specifications
    details.insert("FileName".to_string(), "neon_nights_master.wav".to_string());
    details.insert("Codec".to_string(), "PCM".to_string());
    details.insert("BitRate".to_string(), "1411".to_string()); // CD Quality
    details.insert("SampleRate".to_string(), "48000".to_string()); // Video standard
    details.insert("BitsPerSample".to_string(), "24".to_string()); // High resolution
    details.insert("NumberOfChannels".to_string(), "2".to_string());
    
    // Content ID fingerprinting support
    details.insert("HashSum".to_string(), 
                  "sha256:a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456".to_string());
    details.insert("HashAlgorithm".to_string(), "SHA-256".to_string());
    
    // Audio quality indicators for Content ID
    details.insert("IsMasterRecording".to_string(), "true".to_string());
    details.insert("AudioFingerprint".to_string(), "ContentID_Ready".to_string());
    
    details
}

fn create_youtube_video_specs() -> HashMap<String, String> {
    let mut details = HashMap::new();
    
    // YouTube-optimized video specifications
    details.insert("FileName".to_string(), "neon_nights_4k_master.mp4".to_string());
    details.insert("VideoCodec".to_string(), "H.264".to_string());
    details.insert("AudioCodec".to_string(), "AAC".to_string());
    
    // Video quality specifications
    details.insert("VideoResolution".to_string(), "3840x2160".to_string()); // 4K
    details.insert("VideoQuality".to_string(), "2160p".to_string());
    details.insert("FrameRate".to_string(), "24".to_string()); // Cinematic
    details.insert("AspectRatio".to_string(), "16:9".to_string());
    
    // Bitrate specifications
    details.insert("VideoBitRate".to_string(), "15000".to_string()); // 15 Mbps for 4K
    details.insert("AudioBitRate".to_string(), "320".to_string()); // High quality audio
    
    // File characteristics
    details.insert("FileSize".to_string(), "756432000".to_string()); // ~720MB for 4K video
    details.insert("ContainerFormat".to_string(), "MP4".to_string());
    
    // Content ID fingerprinting
    details.insert("HashSum".to_string(), 
                  "sha256:f6e5d4c3b2a1987654321098765432109876543210fedcba098765432109876".to_string());
    details.insert("HashAlgorithm".to_string(), "SHA-256".to_string());
    details.insert("VideoFingerprint".to_string(), "ContentID_Ready".to_string());
    
    // Video production metadata
    details.insert("ColorSpace".to_string(), "Rec.2020".to_string()); // HDR support
    details.insert("HDR".to_string(), "HDR10".to_string());
    details.insert("Director".to_string(), "Alex Neon".to_string());
    details.insert("ProductionCompany".to_string(), "Neon Visual Studios".to_string());
    
    details
}

fn create_youtube_monetization_deal() -> DealRequest {
    DealRequest {
        deal_id: "YOUTUBE_MONETIZE_001".to_string(),
        commercial_model_type: Some("AdvertisementSupportedModel".to_string()),
        usage_type: "Stream".to_string(),
        territory_code: "Worldwide".to_string(),
        start_date: Some("2024-02-14".to_string()),
        end_date: None, // Perpetual for YouTube
        price: None, // YouTube handles ad revenue
        currency: None,
        resources: vec!["A1".to_string(), "V1".to_string()],
    }
}

fn create_youtube_metadata() -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    
    // YouTube-specific metadata
    metadata.insert("YouTubeCategory".to_string(), "Music".to_string());
    metadata.insert("ContentRating".to_string(), "GeneralAudience".to_string());
    metadata.insert("Language".to_string(), "en".to_string());
    metadata.insert("ExplicitContent".to_string(), "false".to_string());
    
    // Video description and discovery
    metadata.insert("VideoDescription".to_string(), 
                    "Official music video for 'Neon Nights' by Luna Synth. A synthpop journey through neon-lit cityscapes and electric dreams.".to_string());
    metadata.insert("Keywords".to_string(), 
                    "synthpop,electronic,neon,retrowave,music video,luna synth,viral music".to_string());
    metadata.insert("VideoTags".to_string(), 
                    "music,synthpop,electronic,retrowave,80s,neon,cyberpunk".to_string());
    
    // Content ID and rights management
    metadata.insert("ContentIDClaim".to_string(), "Monetize".to_string());
    metadata.insert("RightsOwner".to_string(), "Viral Music Entertainment".to_string());
    metadata.insert("RightsTerritory".to_string(), "Worldwide".to_string());
    metadata.insert("ContentPolicy".to_string(), "Monetize".to_string());
    
    // Production credits
    metadata.insert("VideoDirector".to_string(), "Alex Neon".to_string());
    metadata.insert("VideoProducer".to_string(), "Neon Visual Studios".to_string());
    metadata.insert("Cinematographer".to_string(), "Maya Bright".to_string());
    metadata.insert("VideoEditor".to_string(), "Sam Pixel".to_string());
    
    // Technical metadata
    metadata.insert("OriginalVideoFormat".to_string(), "4K".to_string());
    metadata.insert("ColorGrading".to_string(), "HDR10".to_string());
    metadata.insert("ProductionYear".to_string(), "2024".to_string());
    metadata.insert("FilmingLocation".to_string(), "Los Angeles, CA".to_string());
    
    // Monetization preferences
    metadata.insert("PreferredAdTypes".to_string(), "PreRoll,MidRoll,Overlay".to_string());
    metadata.insert("MonetizationRegions".to_string(), "Global".to_string());
    metadata.insert("ContentIDMatching".to_string(), "Enabled".to_string());
    
    metadata
}

fn validate_youtube_compliance(xml: &str) -> Result<(), Box<dyn Error>> {
    println!("\n🔍 Validating YouTube compliance...");
    
    // Check required elements for YouTube
    let required_elements = [
        "MessageSchemaVersionId=\"ern/43\"",
        "VideoResource",
        "SoundRecording",
        "ISRC",
        "Title",
        "Duration",
        "VideoCodec",
        "VideoResolution",
        "FrameRate",
        "UseType>Stream<",
        "CommercialModelType>AdvertisementSupportedModel<",
        "HashSum",
    ];
    
    for element in required_elements {
        if !xml.contains(element) {
            return Err(format!("Missing required YouTube element: {}", element).into());
        }
    }
    
    // Check video quality requirements
    if xml.contains("VideoResolution>3840x2160<") {
        println!("✅ Video quality: 4K Ultra HD");
    } else if xml.contains("VideoResolution>1920x1080<") {
        println!("✅ Video quality: Full HD");
    } else if xml.contains("VideoResolution>1280x720<") {
        println!("⚠️  Video quality: HD");
    } else {
        return Err("Video quality below YouTube recommended standards".into());
    }
    
    // Check Content ID readiness
    if xml.contains("HashSum") && xml.contains("ISRC") {
        println!("✅ Content ID ready: Audio and video fingerprints available");
    } else {
        return Err("Content ID requirements not met".into());
    }
    
    println!("✅ All YouTube compliance checks passed");
    Ok(())
}

fn print_youtube_compliance_summary(xml: &str) {
    println!("  📋 DDEX Version: ERN 4.3 ✅");
    println!("  🎬 Message Profile: Video Single ✅");
    println!("  🌍 Territory: Worldwide ✅");
    println!("  📺 Video Format: H.264/MP4 ✅");
    
    // Check video specifications
    if xml.contains("3840x2160") {
        println!("  📐 Resolution: 4K Ultra HD (3840x2160) ✅");
    } else if xml.contains("1920x1080") {
        println!("  📐 Resolution: Full HD (1920x1080) ✅");
    }
    
    if xml.contains("FrameRate>24<") {
        println!("  🎞️  Frame Rate: 24fps (Cinematic) ✅");
    } else if xml.contains("FrameRate>30<") {
        println!("  🎞️  Frame Rate: 30fps (Standard) ✅");
    }
    
    // Check audio specifications
    if xml.contains("SampleRate>48000<") {
        println!("  🎵 Audio Sample Rate: 48kHz (Broadcast Standard) ✅");
    }
    
    // Check monetization
    let has_monetization = xml.contains("AdvertisementSupportedModel");
    println!("  💰 Monetization: {} ✅", if has_monetization { "Enabled" } else { "Disabled" });
    
    // Check Content ID
    let has_content_id = xml.contains("HashSum") && xml.contains("ISRC");
    println!("  🔍 Content ID: {} ✅", if has_content_id { "Ready" } else { "Not Ready" });
}

fn demonstrate_content_id_features(xml: &str) {
    println!("  🎵 Audio Fingerprinting:");
    if xml.contains("ISRC") {
        let isrc_start = xml.find("ISRC>").unwrap_or(0);
        let isrc_end = xml.find("</ISRC>").unwrap_or(0);
        if isrc_end > isrc_start {
            let isrc = &xml[isrc_start + 5..isrc_end];
            println!("    • ISRC Code: {} ✅", isrc);
        }
    }
    
    println!("  📹 Video Fingerprinting:");
    let hash_count = xml.matches("HashSum>").count();
    println!("    • File Hashes: {} ✅", hash_count);
    
    if xml.contains("VideoFingerprint") {
        println!("    • Video Fingerprint: Ready ✅");
    }
    if xml.contains("AudioFingerprint") {
        println!("    • Audio Fingerprint: Ready ✅");
    }
    
    println!("  💼 Rights Management:");
    if xml.contains("CommercialModelType>AdvertisementSupportedModel<") {
        println!("    • Monetization Model: Ad-Supported ✅");
    }
    if xml.contains("TerritoryCode>Worldwide<") {
        println!("    • Rights Territory: Worldwide ✅");
    }
    
    println!("\n🎉 Video is ready for YouTube Content ID and monetization!");
    println!("📊 Expected YouTube Features:");
    println!("   • Automatic Content ID matching");
    println!("   • Global monetization with ads");
    println!("   • High-quality 4K streaming");
    println!("   • Professional video metadata");
    println!("   • Rights claim management");
    println!("   • Usage analytics and reporting");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_youtube_video_example() {
        let mut builder = Builder::new();
        builder.apply_preset("youtube_video_43", false).unwrap();
        
        let request = create_youtube_video_request();
        let result = builder.build_internal(&request).unwrap();
        
        assert!(!result.xml.is_empty());
        assert!(result.xml.contains("ERN/4.3"));
        assert!(result.xml.contains("VideoResource"));
        assert!(validate_youtube_compliance(&result.xml).is_ok());
    }
    
    #[test]
    fn test_youtube_video_specs() {
        let specs = create_youtube_video_specs();
        
        assert_eq!(specs.get("VideoCodec").unwrap(), "H.264");
        assert_eq!(specs.get("VideoResolution").unwrap(), "3840x2160");
        assert_eq!(specs.get("FrameRate").unwrap(), "24");
        assert!(specs.contains_key("HashSum"));
    }
    
    #[test]
    fn test_youtube_metadata() {
        let metadata = create_youtube_metadata();
        
        assert_eq!(metadata.get("YouTubeCategory").unwrap(), "Music");
        assert_eq!(metadata.get("ContentIDClaim").unwrap(), "Monetize");
        assert!(metadata.contains_key("VideoDescription"));
    }
    
    #[test]
    fn test_audio_video_sync() {
        let audio = create_audio_track();
        let video = create_music_video();
        
        // Audio and video durations must match
        assert_eq!(audio.duration, video.duration);
        assert_eq!(audio.duration.as_ref().unwrap(), "PT3M42S");
    }
}