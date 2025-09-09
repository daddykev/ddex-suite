use ddex_builder::presets::DdexVersion;
use ddex_builder::{Builder, BuildOptions, BuildRequest};
use ddex_builder::builder::{ReleaseRequest, SoundRecordingRequest, VideoResourceRequest, DealRequest};
use ddex_builder::presets::MessageProfile;
use std::collections::HashMap;

#[tokio::test]
async fn test_youtube_video_43_preset() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let request = create_youtube_compliant_request();
    let result = builder.build_internal(&request).expect("Failed to build with YouTube preset");
    
    // Verify YouTube-specific requirements
    assert!(result.xml.contains("ERN/4.3")); // YouTube requires latest version
    assert!(result.xml.contains("MessageSchemaVersionId=\"ern/43\""));
    
    // Verify video resource support
    assert!(result.xml.contains("VideoResource"));
    assert!(result.xml.contains("VideoCodec"));
    assert!(result.xml.contains("VideoResolution"));
}

#[tokio::test]
async fn test_youtube_video_quality_requirements() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let mut request = create_youtube_compliant_request();
    
    // Test YouTube-supported video qualities
    let youtube_qualities = vec![
        ("1920x1080", "1080p", "30"), // HD
        ("1280x720", "720p", "30"),   // HD Ready
        ("3840x2160", "2160p", "60"), // 4K
    ];
    
    for (resolution, quality, fps) in youtube_qualities {
        if let Some(video) = request.resources.video_resources.get_mut(0) {
            video.technical_details.insert("VideoResolution".to_string(), resolution.to_string());
            video.technical_details.insert("VideoQuality".to_string(), quality.to_string());
            video.technical_details.insert("FrameRate".to_string(), fps.to_string());
        }
        
        let result = builder.build_internal(&request).expect(&format!("Failed to build for resolution {}", resolution));
        assert!(result.xml.contains(&format!("VideoResolution>{}<", resolution)));
        assert!(result.xml.contains(&format!("FrameRate>{}<", fps)));
    }
}

#[tokio::test]
async fn test_youtube_content_id_requirements() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let request = create_youtube_compliant_request();
    let result = builder.build_internal(&request).expect("Failed to build YouTube Content ID compliant release");
    
    // Verify Content ID requirements
    assert!(result.xml.contains("ISRC")); // Required for audio fingerprinting
    assert!(result.xml.contains("HashSum")); // Required for video fingerprinting
    assert!(result.xml.contains("Duration")); // Required for matching
    assert!(result.xml.contains("RightsClaim")); // Required for monetization
}

#[tokio::test]
async fn test_youtube_video_codecs() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let mut request = create_youtube_compliant_request();
    
    // Test YouTube-supported video codecs
    let youtube_codecs = vec!["H.264", "VP9", "AV1"];
    
    for codec in youtube_codecs {
        if let Some(video) = request.resources.video_resources.get_mut(0) {
            video.technical_details.insert("VideoCodec".to_string(), codec.to_string());
            video.technical_details.insert("VideoBitRate".to_string(), "8000".to_string()); // 8 Mbps
        }
        
        let result = builder.build_internal(&request).expect(&format!("Failed to build with video codec {}", codec));
        assert!(result.xml.contains(&format!("VideoCodec>{}<", codec)));
    }
}

#[tokio::test]
async fn test_youtube_audio_requirements() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let mut request = create_youtube_compliant_request();
    
    // YouTube audio requirements for video content
    if let Some(sound_recording) = request.resources.sound_recordings.get_mut(0) {
        // High-quality audio for YouTube
        sound_recording.technical_details.insert("Codec".to_string(), "AAC".to_string());
        sound_recording.technical_details.insert("BitRate".to_string(), "256".to_string());
        sound_recording.technical_details.insert("SampleRate".to_string(), "48000".to_string());
        sound_recording.technical_details.insert("NumberOfChannels".to_string(), "2".to_string());
    }
    
    let result = builder.build_internal(&request).expect("Failed to build with YouTube audio requirements");
    
    assert!(result.xml.contains("Codec>AAC<"));
    assert!(result.xml.contains("SampleRate>48000<"));
    assert!(result.xml.contains("BitRate>256<"));
}

#[tokio::test]
async fn test_youtube_territorial_licensing() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let mut request = create_youtube_compliant_request();
    
    // Test YouTube's global territories
    let youtube_territories = vec!["US", "GB", "CA", "AU", "DE", "FR", "JP", "BR", "IN", "Worldwide"];
    
    for territory in youtube_territories {
        request.deals[0].territory_code = territory.to_string();
        let result = builder.build_internal(&request).expect(&format!("Failed to build for territory {}", territory));
        assert!(result.xml.contains(&format!("TerritoryCode>{}<", territory)));
    }
}

#[tokio::test]
async fn test_youtube_monetization_models() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let mut request = create_youtube_compliant_request();
    
    // Test YouTube monetization models
    let youtube_models = vec![
        "AdvertisementSupportedModel",  // Pre-roll, mid-roll ads
        "SubscriptionModel",            // YouTube Premium
        "PayPerViewModel",              // Pay-per-view/rental
    ];
    
    for model in youtube_models {
        request.deals[0].commercial_model_type = Some(model.to_string());
        let result = builder.build_internal(&request).expect(&format!("Failed to build for model {}", model));
        assert!(result.xml.contains(&format!("CommercialModelType>{}<", model)));
    }
}

#[tokio::test]
async fn test_youtube_content_policy_compliance() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let request = create_youtube_compliant_request();
    let result = builder.build_internal(&request).expect("Failed to build YouTube policy compliant release");
    
    // Verify policy compliance metadata
    assert!(result.xml.contains("Genre")); // Required for content classification
    assert!(result.xml.contains("ParentalWarningType")); // Age rating
    assert!(result.xml.contains("RightsController")); // Rights management
    
    // Verify no restricted content markers
    assert!(!result.xml.contains("ExplicitContent>true<")); // Should be properly tagged
}

#[tokio::test]
async fn test_youtube_metadata_completeness() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let request = create_youtube_compliant_request();
    let result = builder.build_internal(&request).expect("Failed to build YouTube metadata complete release");
    
    // Verify comprehensive metadata for YouTube
    assert!(result.xml.contains("Title"));
    assert!(result.xml.contains("DisplayArtist"));
    assert!(result.xml.contains("Duration"));
    assert!(result.xml.contains("ReleaseDate"));
    assert!(result.xml.contains("Genre"));
    assert!(result.xml.contains("Keywords")); // For discoverability
    assert!(result.xml.contains("Description")); // Video description
    assert!(result.xml.contains("Language")); // Content language
}

fn create_youtube_compliant_request() -> BuildRequest {
    BuildRequest {
        message_id: "YOUTUBE_TEST_001".to_string(),
        version: Some(DdexVersion::Ern43),
        profile: Some(MessageProfile::VideoSingle),
        sender: "TestSender".to_string(),
        recipient: "YouTube".to_string(),
        release: ReleaseRequest {
            release_id: "VID123456".to_string(),
            title: "Test Music Video for YouTube".to_string(),
            display_artist: "Test Artist".to_string(),
            label_name: Some("Test Music Videos".to_string()),
            release_date: "2024-01-20".to_string(),
            original_release_date: Some("2024-01-20".to_string()),
            genre: Some("Pop".to_string()),
            pline: Some("℗ 2024 Test Music Videos".to_string()),
            cline: Some("© 2024 Test Music Videos".to_string()),
            upc: None,
            grid: Some("GRD123456789".to_string()), // Video-specific ID
            icpn: None,
            catalog_number: Some("TMV001".to_string()),
            release_type: Some("Single".to_string()),
        },
        resources: ddex_builder::builder::ResourcesRequest {
            sound_recordings: vec![
                SoundRecordingRequest {
                    resource_reference: "R1".to_string(),
                    resource_id: "SR001".to_string(),
                    title: "Test Song".to_string(),
                    display_artist: "Test Artist".to_string(),
                    isrc: Some("USRC17607840".to_string()),
                    duration: Some("PT3M30S".to_string()),
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), "audio.aac".to_string());
                        details.insert("Codec".to_string(), "AAC".to_string());
                        details.insert("BitRate".to_string(), "256".to_string());
                        details.insert("SampleRate".to_string(), "48000".to_string());
                        details.insert("NumberOfChannels".to_string(), "2".to_string());
                        details.insert("HashSum".to_string(), "sha256:audio1234hash5678abcd".to_string());
                        details
                    },
                },
            ],
            image_resources: vec![],
            video_resources: vec![
                VideoResourceRequest {
                    resource_reference: "V1".to_string(),
                    resource_id: "VR001".to_string(),
                    title: "Test Music Video".to_string(),
                    duration: Some("PT3M30S".to_string()),
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), "video.mp4".to_string());
                        details.insert("VideoCodec".to_string(), "H.264".to_string());
                        details.insert("AudioCodec".to_string(), "AAC".to_string());
                        details.insert("VideoResolution".to_string(), "1920x1080".to_string());
                        details.insert("VideoQuality".to_string(), "1080p".to_string());
                        details.insert("FrameRate".to_string(), "30".to_string());
                        details.insert("VideoBitRate".to_string(), "8000".to_string());
                        details.insert("AudioBitRate".to_string(), "256".to_string());
                        details.insert("AspectRatio".to_string(), "16:9".to_string());
                        details.insert("HashSum".to_string(), "sha256:video5678hash9012efgh".to_string());
                        details.insert("FileSize".to_string(), "157286400".to_string()); // ~150MB
                        details
                    },
                },
            ],
        },
        deals: vec![
            DealRequest {
                deal_id: "YTDEAL001".to_string(),
                commercial_model_type: Some("AdvertisementSupportedModel".to_string()),
                usage_type: "Stream".to_string(),
                territory_code: "Worldwide".to_string(),
                start_date: Some("2024-01-20".to_string()),
                end_date: None,
                price: None,
                currency: None,
                resources: vec!["R1".to_string(), "V1".to_string()],
            },
        ],
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("Keywords".to_string(), "pop,music video,test artist".to_string());
            metadata.insert("Description".to_string(), "Official music video for Test Song by Test Artist".to_string());
            metadata.insert("Language".to_string(), "en".to_string());
            metadata.insert("ParentalWarningType".to_string(), "NotExplicit".to_string());
            metadata.insert("RightsController".to_string(), "Test Music Videos".to_string());
            metadata.insert("RightsClaim".to_string(), "Monetize".to_string());
            metadata
        },
        options: BuildOptions::default(),
    }
}

#[tokio::test]
async fn test_youtube_multi_language_support() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let mut request = create_youtube_compliant_request();
    
    // Test multiple language support for global YouTube distribution
    let languages = vec!["en", "es", "fr", "de", "ja", "ko"];
    
    for lang in languages {
        request.metadata.insert("Language".to_string(), lang.to_string());
        let result = builder.build_internal(&request).expect(&format!("Failed to build for language {}", lang));
        assert!(result.xml.contains(&format!("Language>{}<", lang)));
    }
}

#[tokio::test]
async fn test_youtube_content_id_fingerprinting() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let request = create_youtube_compliant_request();
    let result = builder.build_internal(&request).expect("Failed to build Content ID ready release");
    
    // Verify Content ID fingerprinting requirements
    assert!(result.xml.contains("HashSum")); // File hashes for matching
    assert!(result.xml.contains("ISRC")); // Audio fingerprint reference
    assert!(result.xml.contains("Duration")); // Timing reference
    assert!(result.xml.contains("FrameRate")); // Video timing reference
    
    // Verify hash format is suitable for Content ID
    let hash_regex = regex::Regex::new(r"sha256:[a-f0-9]{64}").unwrap();
    assert!(hash_regex.is_match(&result.xml));
}

#[tokio::test]
async fn test_youtube_rights_management() {
    let mut builder = Builder::new();
    builder.apply_preset("youtube_video_43", false).expect("Failed to apply YouTube preset");
    
    let mut request = create_youtube_compliant_request();
    
    // Test YouTube rights management scenarios
    let rights_scenarios = vec![
        ("Monetize", "Full monetization rights"),
        ("Track", "Track views without monetization"),  
        ("Block", "Block in specified territories"),
    ];
    
    for (rights_claim, _description) in rights_scenarios {
        request.metadata.insert("RightsClaim".to_string(), rights_claim.to_string());
        let result = builder.build_internal(&request).expect(&format!("Failed to build with rights claim {}", rights_claim));
        
        // Rights should be embedded in the deal terms
        assert!(result.xml.contains("Deal"));
        assert!(result.xml.contains("CommercialModelType"));
    }
}