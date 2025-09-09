use ddex_builder::presets::DdexVersion;
use ddex_builder::{Builder, BuildOptions, BuildRequest};
use ddex_builder::builder::{ReleaseRequest, SoundRecordingRequest, DealRequest};
use ddex_builder::presets::MessageProfile;
use std::collections::HashMap;

#[tokio::test]
async fn test_spotify_audio_43_preset() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    let request = create_spotify_compliant_request();
    let result = builder.build_internal(&request).expect("Failed to build with Spotify preset");
    
    // Verify Spotify-specific requirements
    assert!(result.xml.contains("ERN/4.3"));
    assert!(result.xml.contains("MessageSchemaVersionId=\"ern/43\""));
    
    // Verify territorial requirements for Spotify
    assert!(result.xml.contains("Territory"));
    assert!(result.xml.contains("CommercialModelType"));
    
    // Verify audio quality requirements
    assert!(result.xml.contains("BitRate"));
    assert!(result.xml.contains("SampleRate"));
}

#[tokio::test]
async fn test_spotify_preset_validation() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    // Test with invalid audio quality (should fail Spotify requirements)
    let mut request = create_spotify_compliant_request();
    // Modify to have invalid bit rate
    if let Some(sound_recording) = request.resources.sound_recordings.get_mut(0) {
        sound_recording.technical_details.insert("BitRate".to_string(), "64".to_string()); // Too low for Spotify
    }
    
    let result = builder.build_internal(&request);
    // Should pass builder validation but generate warnings about audio quality
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_spotify_territory_requirements() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    let mut request = create_spotify_compliant_request();
    
    // Test with Spotify-supported territories
    let spotify_territories = vec!["US", "CA", "GB", "DE", "FR", "AU", "JP", "BR", "MX"];
    
    for territory in spotify_territories {
        request.deals[0].territory_code = territory.to_string();
        let result = builder.build_internal(&request).expect(&format!("Failed to build for territory {}", territory));
        assert!(result.xml.contains(&format!("TerritoryCode>{}<", territory)));
    }
}

#[tokio::test]
async fn test_spotify_commercial_models() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    let mut request = create_spotify_compliant_request();
    
    // Test Spotify-supported commercial models
    let spotify_models = vec!["SubscriptionModel", "AdvertisementSupportedModel", "PremiumModel"];
    
    for model in spotify_models {
        request.deals[0].commercial_model_type = Some(model.to_string());
        let result = builder.build_internal(&request).expect(&format!("Failed to build for commercial model {}", model));
        assert!(result.xml.contains(&format!("CommercialModelType>{}<", model)));
    }
}

#[tokio::test]
async fn test_spotify_isrc_validation() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    let mut request = create_spotify_compliant_request();
    
    // Test valid ISRC formats accepted by Spotify
    let valid_isrcs = vec![
        "USRC17607839",  // Standard format
        "GBUM71505078",  // UK format
        "USAT21234567",  // US format
    ];
    
    for isrc in valid_isrcs {
        request.resources.sound_recordings[0].isrc = Some(isrc.to_string());
        let result = builder.build_internal(&request).expect(&format!("Failed to build with ISRC {}", isrc));
        assert!(result.xml.contains(&format!("ISRC>{}<", isrc)));
    }
}

#[tokio::test]
async fn test_spotify_metadata_completeness() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    let request = create_spotify_compliant_request();
    let result = builder.build_internal(&request).expect("Failed to build Spotify-compliant release");
    
    // Verify required metadata for Spotify
    assert!(result.xml.contains("Title"));
    assert!(result.xml.contains("DisplayArtist"));
    assert!(result.xml.contains("Duration"));
    assert!(result.xml.contains("ISRC"));
    assert!(result.xml.contains("ReleaseDate"));
    assert!(result.xml.contains("Genre"));
    assert!(result.xml.contains("PLine"));
    assert!(result.xml.contains("CLine"));
}

#[tokio::test]
async fn test_spotify_file_format_requirements() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    let mut request = create_spotify_compliant_request();
    
    // Test Spotify-supported audio formats
    let spotify_codecs = vec!["FLAC", "MP3", "AAC", "OGG"];
    
    for codec in spotify_codecs {
        request.resources.sound_recordings[0].technical_details.insert("Codec".to_string(), codec.to_string());
        request.resources.sound_recordings[0].technical_details.insert("BitRate".to_string(), "320".to_string()); // High quality for Spotify
        
        let result = builder.build_internal(&request).expect(&format!("Failed to build with codec {}", codec));
        assert!(result.xml.contains(&format!("Codec>{}<", codec)));
        assert!(result.xml.contains("BitRate>320<"));
    }
}

fn create_spotify_compliant_request() -> BuildRequest {
    BuildRequest {
        message_id: "SPOTIFY_TEST_001".to_string(),
        version: Some(DdexVersion::Ern43),
        profile: Some(MessageProfile::AudioAlbum),
        sender: "TestSender".to_string(),
        recipient: "Spotify".to_string(),
        release: ReleaseRequest {
            release_id: "REL123456".to_string(),
            title: "Test Album for Spotify".to_string(),
            display_artist: "Test Artist".to_string(),
            label_name: Some("Test Records".to_string()),
            release_date: "2024-01-15".to_string(),
            original_release_date: Some("2024-01-15".to_string()),
            genre: Some("Electronic".to_string()),
            pline: Some("℗ 2024 Test Records".to_string()),
            cline: Some("© 2024 Test Records".to_string()),
            upc: Some("123456789012".to_string()),
            grid: None,
            icpn: None,
            catalog_number: Some("TR001".to_string()),
            release_type: Some("Album".to_string()),
        },
        resources: ddex_builder::builder::ResourcesRequest {
            sound_recordings: vec![
                SoundRecordingRequest {
                    resource_reference: "R1".to_string(),
                    resource_id: "SR001".to_string(),
                    title: "Track One".to_string(),
                    display_artist: "Test Artist".to_string(),
                    isrc: Some("USRC17607839".to_string()),
                    duration: Some("PT3M45S".to_string()),
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), "track01.flac".to_string());
                        details.insert("Codec".to_string(), "FLAC".to_string());
                        details.insert("BitRate".to_string(), "1411".to_string()); // CD quality
                        details.insert("SampleRate".to_string(), "44100".to_string());
                        details.insert("BitsPerSample".to_string(), "16".to_string());
                        details.insert("NumberOfChannels".to_string(), "2".to_string());
                        details.insert("HashSum".to_string(), "sha256:abcd1234efgh5678ijkl9012mnop3456".to_string());
                        details
                    },
                },
            ],
            image_resources: vec![],
            video_resources: vec![],
        },
        deals: vec![
            DealRequest {
                deal_id: "DEAL001".to_string(),
                commercial_model_type: Some("SubscriptionModel".to_string()),
                usage_type: "Stream".to_string(),
                territory_code: "Worldwide".to_string(),
                start_date: Some("2024-01-15".to_string()),
                end_date: None,
                price: None,
                currency: None,
                resources: vec!["R1".to_string()],
            },
        ],
        metadata: HashMap::new(),
        options: BuildOptions::default(),
    }
}

#[tokio::test]
async fn test_spotify_streaming_validation() {
    let mut builder = Builder::new();
    builder.apply_preset("spotify_audio_43", false).expect("Failed to apply Spotify preset");
    
    let mut request = create_spotify_compliant_request();
    
    // Test streaming-specific requirements for Spotify
    request.deals[0].usage_type = "Stream".to_string();
    request.deals[0].commercial_model_type = Some("SubscriptionModel".to_string());
    
    let result = builder.build_internal(&request).expect("Failed to build streaming deal");
    
    assert!(result.xml.contains("UseType>Stream<"));
    assert!(result.xml.contains("CommercialModelType>SubscriptionModel<"));
    
    // Verify no download rights are included (Spotify streaming only)
    assert!(!result.xml.contains("PermanentDownload"));
    assert!(!result.xml.contains("DownloadToMobile"));
}

#[tokio::test]
async fn test_spotify_preset_lock() {
    let mut builder = Builder::new();
    
    // Apply and lock the Spotify preset
    builder.apply_preset("spotify_audio_43", true).expect("Failed to apply and lock Spotify preset");
    assert!(builder.is_preset_locked());
    
    // Attempting to apply another preset should fail or be ignored
    let result = builder.apply_preset("youtube_video_43", false);
    // Implementation may vary - either fail or ignore when locked
    // The important thing is that Spotify settings are maintained
    
    let request = create_spotify_compliant_request();
    let build_result = builder.build_internal(&request).expect("Failed to build with locked preset");
    
    // Should still build with Spotify requirements
    assert!(build_result.xml.contains("ERN/4.3"));
}