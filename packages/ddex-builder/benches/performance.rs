use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ddex_builder::{DDEXBuilder, BuildRequest, BuildOptions};
use ddex_builder::builder::{
    MessageHeaderRequest, PartyRequest, LocalizedStringRequest, 
    ReleaseRequest, SoundRecordingRequest, ResourceRequest, 
    DisplayArtistRequest, ContributorRequest
};
use std::time::Duration;

// Target metrics:
// - Single track: <5ms
// - 12-track album: <10ms  
// - 100-track compilation: <50ms

fn benchmark_performance_scales(c: &mut Criterion) {
    let mut group = c.benchmark_group("ddex_builder_performance");
    
    // Set reasonable measurement time
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    // Small: Single track (~5KB output)
    group.bench_with_input(
        BenchmarkId::new("single_track", "1_track"),
        &1,
        |b, &track_count| {
            let builder = DDEXBuilder::new();
            let request = create_test_request_with_tracks(track_count);
            
            b.iter(|| {
                let result = builder.build(
                    black_box(request.clone()),
                    black_box(BuildOptions::default())
                );
                black_box(result)
            });
        },
    );
    
    // Medium: 12-track album (~60KB output) - TARGET: <10ms
    group.bench_with_input(
        BenchmarkId::new("typical_album", "12_tracks"),
        &12,
        |b, &track_count| {
            let builder = DDEXBuilder::new();
            let request = create_test_request_with_tracks(track_count);
            
            b.iter(|| {
                let result = builder.build(
                    black_box(request.clone()),
                    black_box(BuildOptions::default())
                );
                black_box(result)
            });
        },
    );
    
    // Large: 100-track compilation (~500KB output) - TARGET: <50ms
    group.bench_with_input(
        BenchmarkId::new("large_compilation", "100_tracks"),
        &100,
        |b, &track_count| {
            let builder = DDEXBuilder::new();
            let request = create_test_request_with_tracks(track_count);
            
            b.iter(|| {
                let result = builder.build(
                    black_box(request.clone()),
                    black_box(BuildOptions::default())
                );
                black_box(result)
            });
        },
    );
    
    group.finish();
}

fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    // Benchmark builder creation overhead
    group.bench_function("builder_creation", |b| {
        b.iter(|| {
            black_box(DDEXBuilder::new())
        });
    });
    
    // Benchmark request cloning (measures serialization overhead)
    group.bench_function("request_clone_12_tracks", |b| {
        let request = create_test_request_with_tracks(12);
        b.iter(|| {
            black_box(request.clone())
        });
    });
    
    group.finish();
}

fn benchmark_xml_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("xml_generation");
    
    // Test different components of XML generation
    for track_count in [1, 12, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("xml_write", format!("{}_tracks", track_count)),
            track_count,
            |b, &track_count| {
                let builder = DDEXBuilder::new();
                let request = create_test_request_with_tracks(track_count);
                
                b.iter(|| {
                    // Only measure the XML serialization part
                    let result = builder.build(
                        black_box(request.clone()),
                        black_box(BuildOptions::default())
                    );
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn create_test_request_with_tracks(track_count: usize) -> BuildRequest {
    let mut releases = vec![];
    let mut sound_recordings = vec![];
    
    // Create sound recordings
    for i in 0..track_count {
        sound_recordings.push(SoundRecordingRequest {
            sound_recording_id: format!("TRACK_{:03}", i + 1),
            reference_title: LocalizedStringRequest {
                text: format!("Track {} Title", i + 1),
                language_code: None,
            },
            display_artist: vec![DisplayArtistRequest {
                artist_name: format!("Artist {}", (i % 5) + 1), // Simulate repeated artists
                party_id: None,
                artist_role: None,
            }],
            contributors: vec![ContributorRequest {
                contributor_name: format!("Contributor {}", (i % 3) + 1),
                contributor_role: "MainArtist".to_string(),
                party_id: None,
            }],
            resources: vec![ResourceRequest {
                resource_id: format!("RES_{:03}", i + 1),
                resource_reference: format!("track_{:03}.mp3", i + 1),
                resource_type: "SoundRecording".to_string(),
            }],
            duration: Some(format!("PT{}M{}S", 3 + (i % 4), 30 + (i % 30))), // Vary durations
            genre: vec!["Rock".to_string()], // Common genre for interning
            p_line: vec![LocalizedStringRequest {
                text: format!("℗ 2024 Label {}", (i % 2) + 1), // Simulate label repetition
                language_code: None,
            }],
        });
    }
    
    // Create release
    releases.push(ReleaseRequest {
        release_id: "REL_BENCH_001".to_string(),
        reference_title: LocalizedStringRequest {
            text: if track_count == 1 {
                "Single Release".to_string()
            } else if track_count <= 20 {
                "Album Release".to_string()
            } else {
                "Compilation Release".to_string()
            },
            language_code: None,
        },
        display_artist: vec![DisplayArtistRequest {
            artist_name: "Main Artist".to_string(), // Common artist for interning
            party_id: None,
            artist_role: None,
        }],
        sound_recordings,
        release_type: if track_count == 1 { "Single" } else { "Album" }.to_string(),
        p_line: vec![LocalizedStringRequest {
            text: "℗ 2024 Test Label".to_string(), // Common P-line for interning
            language_code: None,
        }],
        c_line: vec![LocalizedStringRequest {
            text: "© 2024 Test Label".to_string(), // Common C-line for interning
            language_code: None,
        }],
        genre: vec!["Rock".to_string()], // Common genre for interning
        release_date: Some("2024-01-01".to_string()),
    });

    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some(format!("BENCH_{:03}_TRACKS", track_count)),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Benchmark Sender".to_string(), // Static for interning
                    language_code: None,
                }],
                party_id: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Benchmark Recipient".to_string(), // Static for interning
                    language_code: None,
                }],
                party_id: None,
            },
            message_control_type: None,
        },
        version: "4.3".to_string(), // Static for interning
        profile: None,
        releases,
        deals: vec![],
        extensions: None,
    }
}

criterion_group!(
    benches,
    benchmark_performance_scales,
    benchmark_memory_allocation,
    benchmark_xml_generation
);
criterion_main!(benches);