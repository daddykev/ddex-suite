//! Shared request fixtures for the ddex-builder benchmarks.

use ddex_builder::builder::{
    LocalizedStringRequest, MessageHeaderRequest, PartyRequest, ReleaseRequest, TrackRequest,
};
use ddex_builder::BuildRequest;

fn party(name: &str) -> PartyRequest {
    PartyRequest {
        party_name: vec![LocalizedStringRequest {
            text: name.to_string(),
            language_code: None,
        }],
        party_id: None,
        party_reference: None,
    }
}

/// An ERN 4.3 request with one release of `track_count` tracks.
/// `track_count == 0` gives a header-only message.
pub fn request_with_tracks(track_count: usize) -> BuildRequest {
    let releases = if track_count == 0 {
        vec![]
    } else {
        vec![ReleaseRequest {
            release_id: "R0001".to_string(),
            release_reference: Some("R1".to_string()),
            title: vec![LocalizedStringRequest {
                text: format!("Benchmark Album ({track_count} tracks)"),
                language_code: Some("en".to_string()),
            }],
            artist: "Benchmark Artist".to_string(),
            label: Some("Benchmark Label".to_string()),
            release_date: Some("2024-01-01".to_string()),
            upc: Some("123456789012".to_string()),
            tracks: (0..track_count)
                .map(|i| TrackRequest {
                    track_id: format!("T{:04}", i + 1),
                    resource_reference: Some(format!("A{}", i + 1)),
                    isrc: format!("USRC1{:07}", i + 1),
                    title: format!("Track {}", i + 1),
                    duration: format!("PT{}M{}S", 2 + i % 4, (i * 7) % 60),
                    artist: "Benchmark Artist".to_string(),
                })
                .collect(),
            resource_references: None,
        }]
    };

    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("BENCH_001".to_string()),
            message_sender: party("Benchmark Sender"),
            message_recipient: party("Benchmark Recipient"),
            message_control_type: None,
            // Fixed timestamp so every build of the same request is comparable.
            message_created_date_time: Some("2024-01-01T00:00:00Z".to_string()),
        },
        version: "4.3".to_string(),
        profile: None,
        releases,
        deals: vec![],
        extensions: None,
    }
}
