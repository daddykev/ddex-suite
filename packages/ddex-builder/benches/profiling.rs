use criterion::{black_box, criterion_group, criterion_main, Criterion, profiler::PProfProfiler};
use ddex_builder::{DDEXBuilder, BuildRequest, BuildOptions};
use ddex_builder::builder::{
    MessageHeaderRequest, PartyRequest, LocalizedStringRequest, 
    ReleaseRequest, SoundRecordingRequest, ResourceRequest, 
    DisplayArtistRequest, ContributorRequest
};
use std::time::Duration;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn benchmark_with_profiler(c: &mut Criterion) {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let mut group = c.benchmark_group("profiling");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(50);

    // Profile 12-track album build (our main target)
    group.bench_function("profile_12_track_album", |b| {
        let builder = DDEXBuilder::new();
        let request = create_realistic_album_request();
        
        b.iter(|| {
            let result = builder.build(
                black_box(request.clone()),
                black_box(BuildOptions::default())
            );
            black_box(result)
        });
    });

    group.finish();
}

fn create_realistic_album_request() -> BuildRequest {
    let mut sound_recordings = vec![];
    
    // Create 12 tracks with realistic data
    let track_titles = [
        "Opening Theme", "Midnight Drive", "Lost in Translation", "Electric Dreams",
        "Analog Heart", "Digital Soul", "Neon Lights", "City Rain",
        "Memory Lane", "Future Past", "Closing Time", "After Hours"
    ];
    
    let artists = ["Main Artist", "Featured Artist", "Main Artist", "Main Artist"];
    let contributors = ["Producer A", "Producer B", "Engineer"];
    let genres = ["Electronic", "Pop", "Ambient"];
    
    for (i, title) in track_titles.iter().enumerate() {
        sound_recordings.push(SoundRecordingRequest {
            sound_recording_id: format!("ISRC-TEST-{:02}-{:05}", 24, i + 1),
            reference_title: LocalizedStringRequest {
                text: title.to_string(),
                language_code: Some("en".to_string()),
            },
            display_artist: vec![DisplayArtistRequest {
                artist_name: artists[i % artists.len()].to_string(),
                party_id: Some(format!("ARTIST_{}", (i % artists.len()) + 1)),
                artist_role: Some("MainArtist".to_string()),
            }],
            contributors: vec![
                ContributorRequest {
                    contributor_name: contributors[i % contributors.len()].to_string(),
                    contributor_role: "Producer".to_string(),
                    party_id: Some(format!("CONTRIB_{}", (i % contributors.len()) + 1)),
                },
                ContributorRequest {
                    contributor_name: "Sound Engineer".to_string(),
                    contributor_role: "Engineer".to_string(),
                    party_id: Some("ENG_001".to_string()),
                }
            ],
            resources: vec![ResourceRequest {
                resource_id: format!("RES_{:03}", i + 1),
                resource_reference: format!("{:02}_{}.mp3", i + 1, title.replace(" ", "_").to_lowercase()),
                resource_type: "SoundRecording".to_string(),
            }],
            duration: Some(format!("PT{}M{}S", 3 + (i % 3), 15 + (i * 7) % 45)),
            genre: vec![genres[i % genres.len()].to_string()],
            p_line: vec![LocalizedStringRequest {
                text: "℗ 2024 Test Music Label Ltd.".to_string(),
                language_code: Some("en".to_string()),
            }],
        });
    }
    
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("TEST_ALBUM_PROFILE_2024_001".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Music Label Ltd.".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("LABEL_TEST_001".to_string()),
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Digital Service Provider".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("DSP_SPOTIFY_001".to_string()),
            },
            message_control_type: Some("LiveMessage".to_string()),
        },
        version: "4.3".to_string(),
        profile: Some("CommonReleaseTypes/14/AudioAlbumMusicOnly".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "REL_ALBUM_2024_001".to_string(),
            reference_title: LocalizedStringRequest {
                text: "Digital Horizons - Complete Album".to_string(),
                language_code: Some("en".to_string()),
            },
            display_artist: vec![DisplayArtistRequest {
                artist_name: "Main Artist".to_string(),
                party_id: Some("ARTIST_1".to_string()),
                artist_role: Some("MainArtist".to_string()),
            }],
            sound_recordings,
            release_type: "Album".to_string(),
            p_line: vec![LocalizedStringRequest {
                text: "℗ 2024 Test Music Label Ltd. All rights reserved.".to_string(),
                language_code: Some("en".to_string()),
            }],
            c_line: vec![LocalizedStringRequest {
                text: "© 2024 Test Music Label Ltd.".to_string(),
                language_code: Some("en".to_string()),
            }],
            genre: vec!["Electronic".to_string(), "Pop".to_string()],
            release_date: Some("2024-03-15".to_string()),
        }],
        deals: vec![],
        extensions: None,
    }
}

// Create a profiler group with pprof
criterion_group! {
    name = profiling;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, pprof::protos::Message::new()));
    targets = benchmark_with_profiler
}

criterion_main!(profiling);