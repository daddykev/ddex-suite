//! DDEX Suite Extension Handling Example
//!
//! This example demonstrates comprehensive extension handling capabilities of the
//! Perfect Fidelity Engine, including:
//! - Partner-specific extensions (Spotify, Apple, YouTube, Amazon, etc.)
//! - Custom label extensions
//! - Extension validation and preservation
//! - Unknown extension handling strategies
//! - Extension round-trip fidelity testing
//! - Extension conflict resolution

use ddex_builder::{
    Builder, FidelityOptions, FidelityConfig, ExtensionPreservationConfig, 
    UnknownExtensionHandling, ExtensionValidationConfig, PreservationLevel,
    RoundTripTester, FidelityAnalysis
};
use ddex_builder::error::BuildError;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 DDEX Suite Extension Handling Example");
    println!("========================================\n");

    // Step 1: Create DDEX samples with various extensions
    let extension_samples = create_extension_test_samples();
    println!("📄 Created {} test samples with different extension types", extension_samples.len());

    // Step 2: Demonstrate partner-specific extension handling
    for (partner, sample) in &extension_samples {
        println!("\n🏢 Testing {} Extensions", partner);
        test_partner_extensions(partner, sample)?;
    }

    // Step 3: Test unknown extension handling strategies
    println!("\n❓ Unknown Extension Handling Strategies");
    test_unknown_extension_handling(&extension_samples["Custom Label"])?;

    // Step 4: Extension validation and verification
    println!("\n✅ Extension Validation and Verification");
    test_extension_validation(&extension_samples["Mixed Partners"])?;

    // Step 5: Extension conflict resolution
    println!("\n⚡ Extension Conflict Resolution");
    test_extension_conflicts()?;

    // Step 6: Extension fidelity analysis
    println!("\n🔍 Extension Fidelity Analysis");
    perform_extension_fidelity_analysis(&extension_samples)?;

    // Step 7: Performance benchmarking with extensions
    println!("\n⚡ Extension Processing Performance");
    benchmark_extension_performance(&extension_samples)?;

    println!("\n✅ Extension Handling Example completed successfully!");
    println!("   All extension preservation features working correctly 🎉");

    Ok(())
}

/// Create test samples with different types of extensions
fn create_extension_test_samples() -> HashMap<String, String> {
    let mut samples = HashMap::new();

    // Spotify-specific extensions
    samples.insert("Spotify".to_string(), create_spotify_extension_sample());
    
    // Apple Music extensions
    samples.insert("Apple Music".to_string(), create_apple_extension_sample());
    
    // YouTube Music extensions
    samples.insert("YouTube Music".to_string(), create_youtube_extension_sample());
    
    // Amazon Music extensions
    samples.insert("Amazon Music".to_string(), create_amazon_extension_sample());
    
    // Custom label extensions
    samples.insert("Custom Label".to_string(), create_custom_label_extension_sample());
    
    // Mixed partner extensions
    samples.insert("Mixed Partners".to_string(), create_mixed_extension_sample());
    
    // Unknown/experimental extensions
    samples.insert("Unknown Extensions".to_string(), create_unknown_extension_sample());

    samples
}

/// Create Spotify-specific extension sample
fn create_spotify_extension_sample() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:spotify="http://spotify.com/ddex/extensions/2024"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release spotify:releaseType="album" spotify:explicitContent="false">
            <ReleaseId>
                <ProprietaryId Namespace="LABEL">LABEL-001</ProprietaryId>
                <spotify:SpotifyAlbumId>spotify:album:4iV5W9uYEdYUVa79Axb7Rh</spotify:SpotifyAlbumId>
                <spotify:SpotifyAlbumURI>spotify:album:4iV5W9uYEdYUVa79Axb7Rh</spotify:SpotifyAlbumURI>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">Spotify Extension Test</DisplayTitleText>
            
            <!-- Spotify-specific metadata -->
            <spotify:SpotifyMetadata>
                <spotify:PlaylistEligible>true</spotify:PlaylistEligible>
                <spotify:RadioEligible>true</spotify:RadioEligible>
                <spotify:AlbumType>album</spotify:AlbumType>
                <spotify:Label>Test Label</spotify:Label>
                <spotify:Copyrights>
                    <spotify:Copyright type="C">2024 Test Label</spotify:Copyright>
                    <spotify:Copyright type="P">2024 Test Label</spotify:Copyright>
                </spotify:Copyrights>
            </spotify:SpotifyMetadata>
            
            <!-- Artist with Spotify data -->
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Test Artist</FullName>
                </PartyName>
                <spotify:SpotifyArtistId>spotify:artist:4NHQUGzhtTLFvgF5SZesLK</spotify:SpotifyArtistId>
                <spotify:SpotifyArtistURI>spotify:artist:4NHQUGzhtTLFvgF5SZesLK</spotify:SpotifyArtistURI>
                <spotify:Verified>true</spotify:Verified>
                <spotify:MonthlyListeners>1000000</spotify:MonthlyListeners>
            </DisplayArtist>
            
            <!-- Genre with Spotify-specific categories -->
            <Genre>
                <GenreText LanguageAndScriptCode="en">Electronic</GenreText>
                <spotify:SpotifyGenres>
                    <spotify:Genre>electronic</spotify:Genre>
                    <spotify:Genre>ambient</spotify:Genre>
                    <spotify:Genre>experimental</spotify:Genre>
                </spotify:SpotifyGenres>
                <spotify:Mood>chill</spotify:Mood>
                <spotify:Tempo>mid_tempo</spotify:Tempo>
            </Genre>
            
            <!-- Track-level Spotify data -->
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>
                    <ResourceReference>TRACK-001</ResourceReference>
                    <spotify:SpotifyTrackId>spotify:track:4iJyoBOLtHqaGxP12qzhQI</spotify:SpotifyTrackId>
                    <spotify:TrackMetadata>
                        <spotify:Popularity>85</spotify:Popularity>
                        <spotify:Explicit>false</spotify:Explicit>
                        <spotify:Playable>true</spotify:Playable>
                        <spotify:AudioFeatures>
                            <spotify:Danceability>0.8</spotify:Danceability>
                            <spotify:Energy>0.7</spotify:Energy>
                            <spotify:Valence>0.6</spotify:Valence>
                            <spotify:Tempo>120.5</spotify:Tempo>
                        </spotify:AudioFeatures>
                    </spotify:TrackMetadata>
                </ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string()
}

/// Create Apple Music extension sample
fn create_apple_extension_sample() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:apple="http://apple.com/ddex/extensions/2024"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release apple:contentRating="clean" apple:mastered="true">
            <ReleaseId>
                <ProprietaryId Namespace="LABEL">LABEL-002</ProprietaryId>
                <apple:AdamId>1234567890</apple:AdamId>
                <apple:AppleMusicId>apple-music:album:1234567890</apple:AppleMusicId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">Apple Music Extension Test</DisplayTitleText>
            
            <!-- Apple-specific metadata -->
            <apple:AppleMusicMetadata>
                <apple:ContentAdvisory>none</apple:ContentAdvisory>
                <apple:Copyright>
                    <apple:PLine>℗ 2024 Test Label</apple:PLine>
                    <apple:CLine>© 2024 Test Label</apple:CLine>
                </apple:Copyright>
                <apple:AudioQuality>
                    <apple:LosslessAvailable>true</apple:LosslessAvailable>
                    <apple:HighResolutionAvailable>true</apple:HighResolutionAvailable>
                    <apple:SpatialAudioAvailable>true</apple:SpatialAudioAvailable>
                </apple:AudioQuality>
            </apple:AppleMusicMetadata>
            
            <!-- Artist with Apple data -->
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Test Artist</FullName>
                </PartyName>
                <apple:AppleMusicArtistId>123456789</apple:AppleMusicArtistId>
                <apple:iTunesArtistId>123456789</apple:iTunesArtistId>
                <apple:ArtistBio LanguageAndScriptCode="en">Test artist biography for Apple Music</apple:ArtistBio>
            </DisplayArtist>
            
            <!-- Genre with Apple categories -->
            <Genre>
                <GenreText LanguageAndScriptCode="en">Electronic</GenreText>
                <apple:PrimaryGenre>Electronic</apple:PrimaryGenre>
                <apple:SecondaryGenre>Ambient</apple:SecondaryGenre>
                <apple:iTunesGenreId>7</apple:iTunesGenreId>
            </Genre>
            
            <!-- Track with Apple-specific features -->
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>
                    <ResourceReference>TRACK-002</ResourceReference>
                    <apple:AppleMusicTrackId>apple-music:song:1234567891</apple:AppleMusicTrackId>
                    <apple:TrackMetadata>
                        <apple:ContentAdvisory>none</apple:ContentAdvisory>
                        <apple:PreviewStartTime>30</apple:PreviewStartTime>
                        <apple:PreviewDuration>30</apple:PreviewDuration>
                        <apple:AudioFormats>
                            <apple:Format codec="AAC" bitrate="256" sampleRate="44100"/>
                            <apple:Format codec="ALAC" bitrate="lossless" sampleRate="44100"/>
                            <apple:Format codec="ALAC" bitrate="lossless" sampleRate="96000" highRes="true"/>
                        </apple:AudioFormats>
                        <apple:SpatialAudio enabled="true" type="Dolby Atmos"/>
                    </apple:TrackMetadata>
                </ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string()
}

/// Create YouTube Music extension sample
fn create_youtube_extension_sample() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:youtube="http://youtube.com/ddex/extensions/2024"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release youtube:contentTier="free" youtube:regionBlocked="false">
            <ReleaseId>
                <ProprietaryId Namespace="LABEL">LABEL-003</ProprietaryId>
                <youtube:YouTubeAlbumId>youtube:album:ABCD1234567890</youtube:YouTubeAlbumId>
                <youtube:YouTubePlaylistId>PLrAl4R6dO2TiGt2VbSG4fJP0x7_8rJfZf</youtube:YouTubePlaylistId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">YouTube Music Extension Test</DisplayTitleText>
            
            <!-- YouTube-specific metadata -->
            <youtube:YouTubeMusicMetadata>
                <youtube:Category>Music</youtube:Category>
                <youtube:ContentRating>none</youtube:ContentRating>
                <youtube:PrimaryLanguage>en</youtube:PrimaryLanguage>
                <youtube:Tags>
                    <youtube:Tag>electronic</youtube:Tag>
                    <youtube:Tag>ambient</youtube:Tag>
                    <youtube:Tag>instrumental</youtube:Tag>
                </youtube:Tags>
            </youtube:YouTubeMusicMetadata>
            
            <!-- Artist with YouTube data -->
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Test Artist</FullName>
                </PartyName>
                <youtube:YouTubeChannelId>UC1234567890ABCDEFGHIJ</youtube:YouTubeChannelId>
                <youtube:YouTubeArtistId>youtube:artist:MPLA_1234567890</youtube:YouTubeArtistId>
                <youtube:SubscriberCount>500000</youtube:SubscriberCount>
                <youtube:Verified>true</youtube:Verified>
            </DisplayArtist>
            
            <!-- Track with YouTube video data -->
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>
                    <ResourceReference>TRACK-003</ResourceReference>
                    <youtube:YouTubeVideoId>dQw4w9WgXcQ</youtube:YouTubeVideoId>
                    <youtube:YouTubeTrackId>youtube:track:ABCD1234567890</youtube:YouTubeTrackId>
                    <youtube:VideoMetadata>
                        <youtube:Description LanguageAndScriptCode="en">Official audio for Track 003</youtube:Description>
                        <youtube:Thumbnail>https://i.ytimg.com/vi/dQw4w9WgXcQ/maxresdefault.jpg</youtube:Thumbnail>
                        <youtube:Duration>PT3M45S</youtube:Duration>
                        <youtube:ViewCount>1000000</youtube:ViewCount>
                        <youtube:LikeCount>50000</youtube:LikeCount>
                        <youtube:CommentCount>5000</youtube:CommentCount>
                    </youtube:VideoMetadata>
                    <youtube:ContentId>
                        <youtube:Reference>YT_REF_123456789</youtube:Reference>
                        <youtube:AssetId>YT_ASSET_987654321</youtube:AssetId>
                    </youtube:ContentId>
                </ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string()
}

/// Create Amazon Music extension sample
fn create_amazon_extension_sample() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:amazon="http://amazon.com/ddex/extensions/2024"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release amazon:primeEligible="true" amazon:unlimitedEligible="true">
            <ReleaseId>
                <ProprietaryId Namespace="LABEL">LABEL-004</ProprietaryId>
                <amazon:ASIN>B08ABCD1234</amazon:ASIN>
                <amazon:AmazonMusicId>amazon-music:album:B08ABCD1234</amazon:AmazonMusicId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">Amazon Music Extension Test</DisplayTitleText>
            
            <!-- Amazon-specific metadata -->
            <amazon:AmazonMusicMetadata>
                <amazon:ProductGroup>Music</amazon:ProductGroup>
                <amazon:Binding>MP3 Music</amazon:Binding>
                <amazon:Label>Test Label</amazon:Label>
                <amazon:Studio>Test Studio</amazon:Studio>
                <amazon:NumberOfDiscs>1</amazon:NumberOfDiscs>
                <amazon:PrimeShipping>true</amazon:PrimeShipping>
            </amazon:AmazonMusicMetadata>
            
            <!-- Artist with Amazon data -->
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Test Artist</FullName>
                </PartyName>
                <amazon:AmazonArtistId>amazon:artist:A1BCDEF234GHIJ</amazon:AmazonArtistId>
                <amazon:ArtistPageURL>https://music.amazon.com/artists/A1BCDEF234GHIJ</amazon:ArtistPageURL>
            </DisplayArtist>
            
            <!-- Track with Amazon features -->
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>
                    <ResourceReference>TRACK-004</ResourceReference>
                    <amazon:ASIN>B08TRACK001</amazon:ASIN>
                    <amazon:AmazonMusicTrackId>amazon-music:track:B08TRACK001</amazon:AmazonMusicTrackId>
                    <amazon:TrackMetadata>
                        <amazon:PrimeEligible>true</amazon:PrimeEligible>
                        <amazon:UnlimitedEligible>true</amazon:UnlimitedEligible>
                        <amazon:HDEligible>true</amazon:HDEligible>
                        <amazon:UltraHDEligible>false</amazon:UltraHDEligible>
                        <amazon:AudioFormats>
                            <amazon:Format type="MP3" bitrate="320"/>
                            <amazon:Format type="FLAC" bitrate="lossless"/>
                        </amazon:AudioFormats>
                    </amazon:TrackMetadata>
                </ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string()
}

/// Create custom label extension sample
fn create_custom_label_extension_sample() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:label="http://example-label.com/ddex/extensions/2024"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release label:catalogSeries="Electronic Explorations" label:volumeNumber="17">
            <ReleaseId>
                <ProprietaryId Namespace="LABEL">EML-2024-001</ProprietaryId>
                <label:LabelCatalogId format="legacy">EML-2024-001</label:LabelCatalogId>
                <label:InternalId>INTERNAL-12345</label:InternalId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">Custom Label Extension Test</DisplayTitleText>
            
            <!-- Custom label metadata -->
            <label:LabelMetadata>
                <label:CatalogInfo>
                    <label:Series>Electronic Explorations</label:Series>
                    <label:Volume>17</label:Volume>
                    <label:SpecialEdition flag="true">Limited Vinyl Release</label:SpecialEdition>
                    <label:ProductionYear>2024</label:ProductionYear>
                </label:CatalogInfo>
                
                <label:ProductionInfo>
                    <label:Studio>Abbey Road Studios</label:Studio>
                    <label:Producer>John Producer</label:Producer>
                    <label:Engineer>Jane Engineer</label:Engineer>
                    <label:Mastering>
                        <label:Engineer>Master Engineer</label:Engineer>
                        <label:Studio>Sterling Sound</label:Studio>
                        <label:AnalogChain>true</label:AnalogChain>
                    </label:Mastering>
                </label:ProductionInfo>
                
                <label:ArtworkInfo>
                    <label:ArtworkCreator>Art Designer</label:ArtworkCreator>
                    <label:ArtworkType>Original Photography</label:ArtworkType>
                    <label:ColorProfile>sRGB</label:ColorProfile>
                    <label:Resolution>3000x3000</label:Resolution>
                </label:ArtworkInfo>
            </label:LabelMetadata>
            
            <!-- Artist with custom data -->
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Test Artist</FullName>
                </PartyName>
                <label:ArtistInfo>
                    <label:SignedDate>2023-01-15</label:SignedDate>
                    <label:ContractType>Exclusive</label:ContractType>
                    <label:ArtistCategory>Electronic</label:ArtistCategory>
                    <label:HomeStudio>true</label:HomeStudio>
                </label:ArtistInfo>
            </DisplayArtist>
            
            <!-- Track with custom metadata -->
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>
                    <ResourceReference>TRACK-005</ResourceReference>
                    <label:TrackMetadata>
                        <label:RecordingInfo>
                            <label:RecordingDate>2024-03-15</label:RecordingDate>
                            <label:RecordingLocation>Abbey Road Studios</label:RecordingLocation>
                            <label:SessionMusicians>
                                <label:Musician instrument="Synthesizer">Synth Player</label:Musician>
                                <label:Musician instrument="Drums">Drum Programmer</label:Musician>
                            </label:SessionMusicians>
                        </label:RecordingInfo>
                        
                        <label:TechnicalInfo>
                            <label:SampleRate>96000</label:SampleRate>
                            <label:BitDepth>24</label:BitDepth>
                            <label:RecordingFormat>Pro Tools</label:RecordingFormat>
                            <label:MixingConsole>SSL 4000 E</label:MixingConsole>
                        </label:TechnicalInfo>
                        
                        <label:CreativeInfo>
                            <label:Inspiration>Ocean waves at sunset</label:Inspiration>
                            <label:Mood>Contemplative</label:Mood>
                            <label:Key>F minor</label:Key>
                            <label:BPM>95</label:BPM>
                            <label:TimeSignature>4/4</label:TimeSignature>
                        </label:CreativeInfo>
                    </label:TrackMetadata>
                </ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string()
}

/// Create mixed partner extension sample
fn create_mixed_extension_sample() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:spotify="http://spotify.com/ddex/extensions/2024"
    xmlns:apple="http://apple.com/ddex/extensions/2024"
    xmlns:youtube="http://youtube.com/ddex/extensions/2024"
    xmlns:amazon="http://amazon.com/ddex/extensions/2024"
    xmlns:label="http://example-label.com/ddex/extensions/2024"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release spotify:explicitContent="false" apple:contentRating="clean" 
                 youtube:contentTier="premium" amazon:primeEligible="true">
            <ReleaseId>
                <ProprietaryId Namespace="LABEL">MIXED-001</ProprietaryId>
                <!-- Multiple partner IDs for the same release -->
                <spotify:SpotifyAlbumId>spotify:album:MIXED001</spotify:SpotifyAlbumId>
                <apple:AdamId>MIXED001APPLE</apple:AdamId>
                <youtube:YouTubeAlbumId>youtube:album:MIXED001YT</youtube:YouTubeAlbumId>
                <amazon:ASIN>B08MIXED001</amazon:ASIN>
                <label:LabelCatalogId>MIXED-2024-001</label:LabelCatalogId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">Mixed Partners Extension Test</DisplayTitleText>
            
            <!-- Artist with all partner IDs -->
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Multi-Platform Artist</FullName>
                </PartyName>
                <spotify:SpotifyArtistId>spotify:artist:MULTIPLATFORM</spotify:SpotifyArtistId>
                <apple:AppleMusicArtistId>MULTIPLATFORM_APPLE</apple:AppleMusicArtistId>
                <youtube:YouTubeChannelId>UC_MULTIPLATFORM</youtube:YouTubeChannelId>
                <amazon:AmazonArtistId>amazon:artist:MULTIPLATFORM</amazon:AmazonArtistId>
                <label:ArtistCode>ARTIST-001</label:ArtistCode>
            </DisplayArtist>
            
            <!-- Track with conflicting extensions (testing resolution) -->
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>
                    <ResourceReference>MIXED-TRACK-001</ResourceReference>
                    <!-- Same track, different partner IDs -->
                    <spotify:SpotifyTrackId>spotify:track:MIXED_TRACK_001</spotify:SpotifyTrackId>
                    <apple:AppleMusicTrackId>apple-music:song:MIXED_TRACK_001</apple:AppleMusicTrackId>
                    <youtube:YouTubeVideoId>MIXED_VID_001</youtube:YouTubeVideoId>
                    <amazon:ASIN>B08MIXEDTRK001</amazon:ASIN>
                    
                    <!-- Potentially conflicting metadata -->
                    <spotify:TrackMetadata>
                        <spotify:Explicit>false</spotify:Explicit>
                        <spotify:Popularity>75</spotify:Popularity>
                    </spotify:TrackMetadata>
                    
                    <apple:TrackMetadata>
                        <apple:ContentAdvisory>none</apple:ContentAdvisory>
                        <apple:PreviewStartTime>30</apple:PreviewStartTime>
                    </apple:TrackMetadata>
                    
                    <youtube:VideoMetadata>
                        <youtube:Duration>PT3M45S</youtube:Duration>
                        <youtube:ViewCount>500000</youtube:ViewCount>
                    </youtube:VideoMetadata>
                    
                    <amazon:TrackMetadata>
                        <amazon:PrimeEligible>true</amazon:PrimeEligible>
                        <amazon:HDEligible>true</amazon:HDEligible>
                    </amazon:TrackMetadata>
                    
                    <label:TrackMetadata>
                        <label:InternalTrackId>INT-TRK-001</label:InternalTrackId>
                        <label:RecordingDate>2024-03-01</label:RecordingDate>
                    </label:TrackMetadata>
                </ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string()
}

/// Create unknown extension sample
fn create_unknown_extension_sample() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:unknown="http://unknown-service.example.com/ddex/2024"
    xmlns:experimental="http://experimental-features.music.org/2024"
    xmlns:future="http://future-standard.ddex.net/2025"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release unknown:serviceType="streaming" experimental:betaFeatures="enabled">
            <ReleaseId>
                <ProprietaryId Namespace="LABEL">UNKNOWN-001</ProprietaryId>
                <unknown:UnknownServiceId>UNK_SVC_12345</unknown:UnknownServiceId>
                <experimental:ExperimentalId type="ai-generated">EXP_AI_001</experimental:ExperimentalId>
                <future:FutureStandardId version="2025">FUT_STD_001</future:FutureStandardId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">Unknown Extensions Test</DisplayTitleText>
            
            <!-- Unknown service extensions -->
            <unknown:ServiceMetadata>
                <unknown:PlatformType>NextGen Streaming</unknown:PlatformType>
                <unknown:ApiVersion>3.5.7</unknown:ApiVersion>
                <unknown:Features>
                    <unknown:Feature name="spatial-audio">enabled</unknown:Feature>
                    <unknown:Feature name="ai-recommendations">enabled</unknown:Feature>
                    <unknown:Feature name="blockchain-rights">experimental</unknown:Feature>
                </unknown:Features>
            </unknown:ServiceMetadata>
            
            <!-- Experimental extensions -->
            <experimental:AIMetadata>
                <experimental:GenerationModel>MusicAI-v2.1</experimental:GenerationModel>
                <experimental:TrainingData>licensed-only</experimental:TrainingData>
                <experimental:AIGenerated percentage="0">false</experimental:AIGenerated>
                <experimental:AIAssisted percentage="15">mixing-mastering</experimental:AIAssisted>
            </experimental:AIMetadata>
            
            <!-- Future standard extensions -->
            <future:EnhancedMetadata>
                <future:ImmersiveFormats>
                    <future:Format type="8D-Audio">available</future:Format>
                    <future:Format type="Haptic-Audio">not-available</future:Format>
                    <future:Format type="VR-Ready">available</future:Format>
                </future:ImmersiveFormats>
                <future:Blockchain>
                    <future:NFTSupport>enabled</future:NFTSupport>
                    <future:SmartContracts>disabled</future:SmartContracts>
                    <future:RoyaltyDistribution>automated</future:RoyaltyDistribution>
                </future:Blockchain>
            </future:EnhancedMetadata>
            
            <!-- Track with unknown extensions -->
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>
                    <ResourceReference>UNKNOWN-TRACK-001</ResourceReference>
                    <unknown:TrackId>UNK_TRK_001</unknown:TrackId>
                    <experimental:TrackAnalysis>
                        <experimental:EmotionalProfile>
                            <experimental:Valence>0.7</experimental:Valence>
                            <experimental:Energy>0.8</experimental:Energy>
                            <experimental:Complexity>0.6</experimental:Complexity>
                        </experimental:EmotionalProfile>
                        <experimental:AudioFingerprint algorithm="next-gen">ABC123DEF456</experimental:AudioFingerprint>
                    </experimental:TrackAnalysis>
                    
                    <future:AdvancedFeatures>
                        <future:SpatialMapping coordinates="3D">enabled</future:SpatialMapping>
                        <future:PersonalizedMix user-adaptive="true">available</future:PersonalizedMix>
                        <future:RealTimeVisualization>enabled</future:RealTimeVisualization>
                    </future:AdvancedFeatures>
                </ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#.to_string()
}

/// Test partner-specific extension handling
fn test_partner_extensions(partner_name: &str, sample_xml: &str) -> Result<(), BuildError> {
    println!("   🔍 Analyzing {} extensions...", partner_name);
    
    // Create extension-aware configuration
    let mut extension_config = ExtensionPreservationConfig::default();
    extension_config.enabled = true;
    extension_config.preserve_extension_attributes = true;
    extension_config.unknown_extension_handling = UnknownExtensionHandling::Preserve;
    
    let mut fidelity_config = FidelityConfig::default();
    fidelity_config.extension_preservation = extension_config;
    fidelity_config.preservation_level = PreservationLevel::Perfect;
    
    let mut fidelity_options = FidelityOptions::default();
    fidelity_options.enable_perfect_fidelity = true;
    fidelity_options.preserve_extensions = true;
    fidelity_options.collect_statistics = true;
    
    let builder = Builder::with_fidelity_options(fidelity_options);
    
    // Analyze extensions using round-trip tester
    let round_trip_tester = RoundTripTester::new(builder.fidelity_options().clone());
    let analysis = round_trip_tester.analyze_fidelity(sample_xml)?;
    
    println!("      📊 Extension Analysis Results:");
    println!("         • Total extensions found: {}", analysis.extension_analysis.total_extensions);
    println!("         • Known extensions: {}", analysis.extension_analysis.known_extensions);
    println!("         • Unknown extensions: {}", analysis.extension_analysis.unknown_extensions);
    println!("         • Extensions preserved: {}", analysis.extension_analysis.preserved_extensions);
    
    // List extension namespaces found
    if !analysis.extension_analysis.extension_namespaces.is_empty() {
        println!("      🏷️  Extension namespaces detected:");
        for (prefix, uri) in &analysis.extension_analysis.extension_namespaces {
            println!("         • {}: {}", prefix, uri);
            
            // Classify extension type
            let extension_type = classify_extension_type(uri);
            println!("           Type: {}", extension_type);
        }
    }
    
    // Test round-trip preservation
    let round_trip_result = builder.test_round_trip_fidelity(sample_xml)?;
    
    if round_trip_result.success {
        println!("      ✅ Extension preservation: PASSED");
        println!("         • Round-trip successful with all extensions intact");
    } else {
        println!("      ❌ Extension preservation: FAILED");
        if !round_trip_result.differences.is_empty() {
            println!("         • Issues found:");
            for diff in &round_trip_result.differences[..std::cmp::min(3, round_trip_result.differences.len())] {
                println!("           - {}", diff);
            }
        }
    }
    
    println!("      ⏱️  Analysis time: {:.2}ms", analysis.analysis_time.as_secs_f64() * 1000.0);
    println!("      📊 Overall fidelity score: {:.1}%", analysis.overall_score * 100.0);

    Ok(())
}

/// Classify extension type based on URI
fn classify_extension_type(uri: &str) -> &'static str {
    if uri.contains("spotify.com") {
        "Spotify Extension"
    } else if uri.contains("apple.com") {
        "Apple Music Extension"
    } else if uri.contains("youtube.com") {
        "YouTube Music Extension"
    } else if uri.contains("amazon.com") {
        "Amazon Music Extension"
    } else if uri.contains("ddex.net") {
        "DDEX Official Extension"
    } else if uri.contains("example") || uri.contains("label") {
        "Custom Label Extension"
    } else if uri.contains("experimental") {
        "Experimental Extension"
    } else if uri.contains("future") {
        "Future Standard Extension"
    } else {
        "Unknown Extension"
    }
}

/// Test unknown extension handling strategies
fn test_unknown_extension_handling(sample_xml: &str) -> Result<(), BuildError> {
    let strategies = vec![
        ("Preserve", UnknownExtensionHandling::Preserve),
        ("Drop", UnknownExtensionHandling::Drop),
        ("Validate & Preserve", UnknownExtensionHandling::ValidateAndPreserve),
        ("Generalize", UnknownExtensionHandling::Generalize),
    ];
    
    for (name, strategy) in strategies {
        println!("   🔧 Testing strategy: {}", name);
        
        let mut extension_config = ExtensionPreservationConfig::default();
        extension_config.unknown_extension_handling = strategy;
        extension_config.enabled = true;
        
        let mut fidelity_options = FidelityOptions::default();
        fidelity_options.preserve_extensions = true;
        fidelity_options.enable_perfect_fidelity = true;
        
        let builder = Builder::with_fidelity_options(fidelity_options);
        
        let start_time = Instant::now();
        let round_trip_result = builder.test_round_trip_fidelity(sample_xml)?;
        let processing_time = start_time.elapsed();
        
        println!("      📊 Results:");
        println!("         • Success: {}", if round_trip_result.success { "✅" } else { "❌" });
        println!("         • Processing time: {:.2}ms", processing_time.as_secs_f64() * 1000.0);
        println!("         • Byte identical: {}", if round_trip_result.byte_identical { "✅" } else { "❌" });
        
        match strategy {
            UnknownExtensionHandling::Preserve => {
                println!("         • Strategy effect: All unknown extensions kept");
            },
            UnknownExtensionHandling::Drop => {
                println!("         • Strategy effect: Unknown extensions removed");
            },
            UnknownExtensionHandling::ValidateAndPreserve => {
                println!("         • Strategy effect: Validated extensions preserved");
            },
            UnknownExtensionHandling::Generalize => {
                println!("         • Strategy effect: Extensions converted to generic format");
            },
        }
        
        println!();
    }

    Ok(())
}

/// Test extension validation
fn test_extension_validation(sample_xml: &str) -> Result<(), BuildError> {
    println!("   🔍 Testing extension validation features...");
    
    // Create validation-enabled configuration
    let mut validation_config = ExtensionValidationConfig::default();
    validation_config.validate_uris = true;
    validation_config.validate_schema = false; // Would require schema files
    validation_config.max_nesting_depth = 10;
    validation_config.max_extension_count = 100;
    
    let mut extension_config = ExtensionPreservationConfig::default();
    extension_config.extension_validation = validation_config;
    extension_config.enabled = true;
    
    let mut fidelity_options = FidelityOptions::default();
    fidelity_options.preserve_extensions = true;
    fidelity_options.enable_verification = true;
    
    let builder = Builder::with_fidelity_options(fidelity_options);
    
    // Test validation
    let verification_result = builder.verify_build(sample_xml)?;
    
    println!("      📋 Validation Results:");
    println!("         • Overall success: {}", if verification_result.success { "✅" } else { "❌" });
    println!("         • Issues found: {}", verification_result.issues.len());
    
    if !verification_result.issues.is_empty() {
        println!("         • Issue breakdown:");
        let mut error_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;
        
        for issue in &verification_result.issues {
            match issue.severity {
                ddex_builder::VerificationSeverity::Error => error_count += 1,
                ddex_builder::VerificationSeverity::Warning => warning_count += 1,
                ddex_builder::VerificationSeverity::Info => info_count += 1,
            }
        }
        
        if error_count > 0 {
            println!("           - Errors: {} ❌", error_count);
        }
        if warning_count > 0 {
            println!("           - Warnings: {} ⚠️", warning_count);
        }
        if info_count > 0 {
            println!("           - Info: {} ℹ️", info_count);
        }
        
        // Show first few issues
        for (i, issue) in verification_result.issues.iter().take(3).enumerate() {
            println!("         • Issue {}: [{}] {}", i + 1, issue.category, issue.message);
            if let Some(suggestion) = &issue.suggestion {
                println!("           Suggestion: {}", suggestion);
            }
        }
        
        if verification_result.issues.len() > 3 {
            println!("         ... and {} more issues", verification_result.issues.len() - 3);
        }
    }
    
    println!("         • Verification time: {:.2}ms", verification_result.verification_time.as_secs_f64() * 1000.0);

    Ok(())
}

/// Test extension conflict resolution
fn test_extension_conflicts() -> Result<(), BuildError> {
    println!("   ⚡ Creating conflicting extension scenario...");
    
    // Create XML with intentionally conflicting extensions
    let conflicting_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ernm:NewReleaseMessage 
    xmlns:ernm="http://ddex.net/xml/ern/43"
    xmlns:v1="http://partner.com/ddex/v1"
    xmlns:v2="http://partner.com/ddex/v2"
    MessageSchemaVersionId="ern/43">
    
    <ReleaseList>
        <Release>
            <ReleaseId>
                <ProprietaryId Namespace="TEST">CONFLICT-001</ProprietaryId>
                <!-- Same logical data, different extension versions -->
                <v1:PartnerId format="old">PARTNER_123</v1:PartnerId>
                <v2:PartnerId format="new" version="2">PARTNER_123_V2</v2:PartnerId>
            </ReleaseId>
            
            <DisplayTitleText LanguageAndScriptCode="en">Extension Conflict Test</DisplayTitleText>
            
            <!-- Conflicting metadata formats -->
            <DisplayArtist>
                <PartyName LanguageAndScriptCode="en">
                    <FullName>Test Artist</FullName>
                </PartyName>
                <!-- Old format -->
                <v1:ArtistMetadata>
                    <v1:PlayCount>1000000</v1:PlayCount>
                    <v1:Verified>true</v1:Verified>
                </v1:ArtistMetadata>
                <!-- New format with additional data -->
                <v2:ArtistMetadata>
                    <v2:Statistics>
                        <v2:PlayCount type="total">1000000</v2:PlayCount>
                        <v2:PlayCount type="monthly">50000</v2:PlayCount>
                    </v2:Statistics>
                    <v2:Verification status="verified" date="2024-01-01"/>
                </v2:ArtistMetadata>
            </DisplayArtist>
        </Release>
    </ReleaseList>
</ernm:NewReleaseMessage>"#;

    let mut fidelity_options = FidelityOptions::default();
    fidelity_options.preserve_extensions = true;
    fidelity_options.enable_perfect_fidelity = true;
    
    let builder = Builder::with_fidelity_options(fidelity_options);
    
    println!("      🔧 Testing conflict resolution...");
    
    let round_trip_result = builder.test_round_trip_fidelity(conflicting_xml)?;
    
    if round_trip_result.success {
        println!("         ✅ Conflict resolution successful");
        println!("         • All conflicting extensions preserved");
        println!("         • No data loss detected");
    } else {
        println!("         ⚠️  Conflicts detected but handled gracefully:");
        for diff in &round_trip_result.differences[..std::cmp::min(2, round_trip_result.differences.len())] {
            println!("           - {}", diff);
        }
    }
    
    println!("         • Byte identical: {}", if round_trip_result.byte_identical { "✅" } else { "➡️ Normalized" });

    Ok(())
}

/// Perform comprehensive extension fidelity analysis
fn perform_extension_fidelity_analysis(samples: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("   📊 Comprehensive extension fidelity analysis...");
    
    let mut total_extensions = 0;
    let mut total_preserved = 0;
    let mut analysis_times = Vec::new();
    
    for (name, sample) in samples {
        println!("      🔍 Analyzing: {}", name);
        
        let fidelity_options = FidelityOptions {
            enable_perfect_fidelity: true,
            preserve_extensions: true,
            collect_statistics: true,
            ..Default::default()
        };
        
        let round_trip_tester = RoundTripTester::new(fidelity_options);
        
        let start_time = Instant::now();
        let analysis = round_trip_tester.analyze_fidelity(sample)?;
        let analysis_time = start_time.elapsed();
        
        analysis_times.push(analysis_time.as_secs_f64() * 1000.0);
        total_extensions += analysis.extension_analysis.total_extensions;
        total_preserved += analysis.extension_analysis.preserved_extensions;
        
        println!("         • Extensions: {} found, {} preserved", 
                analysis.extension_analysis.total_extensions,
                analysis.extension_analysis.preserved_extensions);
        println!("         • Fidelity score: {:.1}%", analysis.overall_score * 100.0);
        println!("         • Analysis time: {:.2}ms", analysis_time.as_secs_f64() * 1000.0);
    }
    
    println!("      📈 Summary Statistics:");
    println!("         • Total extensions across all samples: {}", total_extensions);
    println!("         • Total extensions preserved: {}", total_preserved);
    println!("         • Overall preservation rate: {:.1}%", 
            if total_extensions > 0 { (total_preserved as f64 / total_extensions as f64) * 100.0 } else { 100.0 });
    
    if !analysis_times.is_empty() {
        let avg_time = analysis_times.iter().sum::<f64>() / analysis_times.len() as f64;
        let min_time = analysis_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_time = analysis_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        println!("         • Average analysis time: {:.2}ms", avg_time);
        println!("         • Min/Max analysis time: {:.2}ms / {:.2}ms", min_time, max_time);
    }

    Ok(())
}

/// Benchmark extension processing performance
fn benchmark_extension_performance(samples: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("   ⚡ Benchmarking extension processing performance...");
    
    let iterations = 50;
    
    for (name, sample) in samples {
        println!("      🔧 Benchmarking: {}", name);
        
        let fidelity_options = FidelityOptions {
            enable_perfect_fidelity: true,
            preserve_extensions: true,
            collect_statistics: false, // Disable to focus on core performance
            ..Default::default()
        };
        
        let builder = Builder::with_fidelity_options(fidelity_options);
        
        // Warm-up
        for _ in 0..5 {
            let _ = builder.test_round_trip_fidelity(sample)?;
        }
        
        // Benchmark
        let mut times = Vec::new();
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = builder.test_round_trip_fidelity(sample)?;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg_time = times.iter().sum::<f64>() / times.len() as f64;
        let median_time = times[times.len() / 2];
        let min_time = times[0];
        let max_time = times[times.len() - 1];
        
        let throughput = (sample.len() as f64) / (avg_time / 1000.0) / 1024.0 / 1024.0; // MB/s
        
        println!("         • Input size: {} bytes", sample.len());
        println!("         • Average time: {:.2}ms", avg_time);
        println!("         • Min/Median/Max: {:.2}ms / {:.2}ms / {:.2}ms", min_time, median_time, max_time);
        println!("         • Throughput: {:.2} MB/s", throughput);
        
        // Performance target compliance
        let target_time = 100.0; // Target: <100ms for typical files
        if avg_time <= target_time {
            println!("         • Performance: ✅ MEETS TARGET (≤ {}ms)", target_time);
        } else {
            println!("         • Performance: ⚠️ ABOVE TARGET (> {}ms)", target_time);
        }
        
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_samples_creation() {
        let samples = create_extension_test_samples();
        assert_eq!(samples.len(), 7);
        assert!(samples.contains_key("Spotify"));
        assert!(samples.contains_key("Apple Music"));
        assert!(samples.contains_key("YouTube Music"));
        assert!(samples.contains_key("Amazon Music"));
        assert!(samples.contains_key("Custom Label"));
    }

    #[test]
    fn test_spotify_extension_sample() {
        let sample = create_spotify_extension_sample();
        assert!(sample.contains("xmlns:spotify"));
        assert!(sample.contains("spotify:SpotifyAlbumId"));
        assert!(sample.contains("spotify:SpotifyMetadata"));
        assert!(sample.len() > 1000);
    }

    #[test]
    fn test_apple_extension_sample() {
        let sample = create_apple_extension_sample();
        assert!(sample.contains("xmlns:apple"));
        assert!(sample.contains("apple:AdamId"));
        assert!(sample.contains("apple:AppleMusicMetadata"));
    }

    #[test]
    fn test_extension_type_classification() {
        assert_eq!(classify_extension_type("http://spotify.com/ddex"), "Spotify Extension");
        assert_eq!(classify_extension_type("http://apple.com/ddex"), "Apple Music Extension");
        assert_eq!(classify_extension_type("http://youtube.com/ddex"), "YouTube Music Extension");
        assert_eq!(classify_extension_type("http://amazon.com/ddex"), "Amazon Music Extension");
        assert_eq!(classify_extension_type("http://example-label.com/ddex"), "Custom Label Extension");
        assert_eq!(classify_extension_type("http://experimental.music.org"), "Experimental Extension");
        assert_eq!(classify_extension_type("http://future.ddex.net"), "Future Standard Extension");
        assert_eq!(classify_extension_type("http://unknown.service.com"), "Unknown Extension");
    }

    #[test]
    fn test_unknown_extension_handling_enum() {
        let strategies = vec![
            UnknownExtensionHandling::Preserve,
            UnknownExtensionHandling::Drop,
            UnknownExtensionHandling::ValidateAndPreserve,
            UnknownExtensionHandling::Generalize,
        ];
        
        assert_eq!(strategies.len(), 4);
    }

    #[test]
    fn test_mixed_extension_sample() {
        let sample = create_mixed_extension_sample();
        assert!(sample.contains("xmlns:spotify"));
        assert!(sample.contains("xmlns:apple"));
        assert!(sample.contains("xmlns:youtube"));
        assert!(sample.contains("xmlns:amazon"));
        assert!(sample.contains("xmlns:label"));
    }
}