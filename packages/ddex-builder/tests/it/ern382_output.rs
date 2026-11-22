//! Pins the builder's ERN 3.8.2 output byte for byte (insta snapshots), so changes made
//! for other versions cannot alter it. Every value is fixed (MessageId, created time,
//! references), so the output is a pure function of the request.

use ddex_builder::builder::{
    DealRequest, DealTerms, IdStrategy, LocalizedStringRequest, MessageHeaderRequest,
    PartyRequest, ReleaseRequest, TrackRequest,
};
use ddex_builder::determinism::{CanonMode, DeterminismConfig};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};

fn text(value: &str, lang: Option<&str>) -> LocalizedStringRequest {
    LocalizedStringRequest {
        text: value.to_string(),
        language_code: lang.map(str::to_string),
    }
}

fn party(id: &str, name: &str) -> PartyRequest {
    PartyRequest {
        party_name: vec![text(name, None)],
        party_id: Some(id.to_string()),
        party_reference: None,
    }
}

fn track(n: u32, title: &str, artist: &str, with_reference: bool) -> TrackRequest {
    TrackRequest {
        track_id: format!("TRK{n:03}"),
        resource_reference: with_reference.then(|| format!("A{n}")),
        isrc: format!("USRT124{n:05}"),
        title: title.to_string(),
        duration: "PT3M42S".to_string(),
        artist: artist.to_string(),
        ..Default::default()
    }
}

fn request() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("MSG-382".to_string()),
            message_sender: party("PADPIDA0000000001", "Example Label"),
            message_recipient: party("PADPIDA0000000002", "Example DSP"),
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: Some("2024-09-13T12:00:00Z".to_string()),
        },
        version: "3.8.2".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![
            ReleaseRequest {
                release_id: "A1-ABCDE-0000000001-M".to_string(),
                release_reference: Some("R1".to_string()),
                title: vec![text("Album", Some("en")), text("Álbum", Some("es"))],
                artist: "Artist & Friends".to_string(),
                label: Some("Label <Records>".to_string()),
                release_date: Some("2024-01-01".to_string()),
                upc: Some("123456789012".to_string()),
                tracks: vec![
                    track(1, "Say \"hi\" it's 🎵", "Artist", true),
                    track(2, "أغنية", "Feat", false),
                ],
                resource_references: Some(vec!["A1".to_string(), "A2".to_string()]),
                ..Default::default()
            },
            ReleaseRequest {
                release_id: "REL2".to_string(),
                title: vec![text("<script>alert('x')</script>", None)],
                artist: "Other".to_string(),
                tracks: vec![track(3, "Three", "Other", false)],
                ..Default::default()
            },
        ],
        deals: vec![DealRequest {
            deal_reference: Some("D1".to_string()),
            deal_terms: DealTerms {
                commercial_model_type: "SubscriptionModel".to_string(),
                territory_code: vec!["Worldwide".to_string()],
                start_date: Some("2024-01-01".to_string()),
                ..Default::default()
            },
            release_references: vec!["R1".to_string()],
        }],
        extensions: None,
        ..Default::default()
    }
}

fn build(canon_mode: CanonMode) -> String {
    let determinism = DeterminismConfig {
        canon_mode,
        ..DeterminismConfig::default()
    };
    DDEXBuilder::new()
        .build(
            request(),
            BuildOptions {
                determinism: Some(determinism),
                id_strategy: IdStrategy::Sequential,
                ..BuildOptions::default()
            },
        )
        .expect("3.8.2 build")
        .xml
}

#[test]
fn ern382_canonical_output_is_pinned() {
    insta::assert_snapshot!(build(CanonMode::DbC14n));
}

#[test]
fn ern382_pretty_output_is_pinned() {
    insta::assert_snapshot!(build(CanonMode::Pretty));
}
