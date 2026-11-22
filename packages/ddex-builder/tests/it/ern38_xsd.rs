//! ERN 3.8.2 builder output validates against the official DDEX 3.8.2 XSD (xmllint), in
//! both the default (DB-C14N) and pretty output modes; 3.8.1 output is the same document in
//! the 3.8.1 namespace; missing required values are named; escaping and illegal-character
//! rejection behave as for 4.x.

use crate::ern4_support::*;
use ddex_builder::builder::{
    AvsValueRequest, DealRequest, ResourceGroupContentItemRequest, ResourceGroupRequest,
};
use ddex_builder::determinism::{CanonMode, DeterminismConfig};
use ddex_builder::{BuildError, BuildOptions, BuildRequest, DDEXBuilder};

type Case = (&'static str, fn(&str) -> BuildRequest);

/// The 4.x request builders, unchanged: the same requests build for 3.8.2.
const CASES: [Case; 3] = [
    ("minimal single", minimal_single),
    (
        "multi-track album (track releases, deals)",
        multi_track_album,
    ),
    ("multi-disc album", multi_disc_album),
];

fn pretty() -> BuildOptions {
    BuildOptions {
        determinism: Some(DeterminismConfig {
            canon_mode: CanonMode::Pretty,
            ..DeterminismConfig::default()
        }),
        ..BuildOptions::default()
    }
}

fn build_err(request: BuildRequest) -> Vec<String> {
    match DDEXBuilder::new().build(request, BuildOptions::default()) {
        Err(BuildError::ValidationFailed { errors }) => errors,
        Err(e) => panic!("expected ValidationFailed, got {e}"),
        Ok(r) => panic!("expected the build to fail:\n{}", r.xml),
    }
}

/// The document without indentation, for substring checks across elements.
fn compact(xml: &str) -> String {
    xml.lines().map(str::trim).collect()
}

fn assert_named(errors: &[String], expected: &str) {
    assert!(
        errors.iter().any(|e| e == expected),
        "missing error {expected:?} in {errors:#?}"
    );
}

#[test]
fn builder_output_is_schema_valid() {
    for (name, make) in CASES {
        assert_xsd_valid(&build(make("3.8.2")), "3.8.2", name);
        let xml = DDEXBuilder::new()
            .build(make("3.8.2"), pretty())
            .expect("build")
            .xml;
        assert_xsd_valid(&xml, "3.8.2", &format!("{name} (pretty)"));
    }
}

/// Values 3.8.2 accepts and 4.x does not: party IDs that are not DPIDs, several releases
/// per message, a recording shared by two releases, no release artist/label/genre.
#[test]
fn ern382_specific_input_is_schema_valid() {
    let mut r = minimal_single("3.8.2");
    r.header.message_sender.party_id = Some("LABEL382".to_string());
    r.language_and_script_code = None;
    let mut second = release("Second", "Single", vec![track(1, "Example Track")]);
    second.release_reference = Some("R9".to_string());
    second.release_id = "A1-ABCDE-0000000009-M".to_string();
    second.artist = String::new();
    second.label = None;
    second.genre = None;
    second.parental_warning_type = None;
    r.releases.push(second);
    r.deals
        .push(deal(&["R0", "R9"], "PayAsYouGoModel", "PermanentDownload"));
    // A deal without DealTerms: a reference to a deal agreed elsewhere
    r.deals.push(DealRequest {
        deal_reference: Some("YT_MATCH_POLICY:My Saved Policy".to_string()),
        deal_terms: Default::default(),
        release_references: vec!["R0".to_string()],
    });
    let xml = build(r);
    assert_xsd_valid(&xml, "3.8.2", "two releases sharing a recording");
    assert!(
        compact(&xml).contains(
            "<Deal><DealReference>YT_MATCH_POLICY:My Saved Policy</DealReference></Deal>"
        ),
        "{xml}"
    );
    assert_eq!(xml.matches("<SoundRecording>").count(), 1, "{xml}");
    assert_eq!(xml.matches("<Release>").count(), 2, "{xml}");
}

#[test]
fn ern381_is_the_382_document_in_the_381_namespace() {
    for (name, make) in CASES {
        let v381 = build(make("3.8.1"));
        let v382 = build(make("3.8.2"));
        assert!(
            v381.contains("xmlns:ern=\"http://ddex.net/xml/ern/381\""),
            "{name}"
        );
        assert!(
            v381.contains("MessageSchemaVersionId=\"ern/381\""),
            "{name}"
        );
        let as_382 = v381
            .replace("http://ddex.net/xml/ern/381", "http://ddex.net/xml/ern/382")
            .replace(
                "MessageSchemaVersionId=\"ern/381\"",
                "MessageSchemaVersionId=\"ern/382\"",
            );
        assert_eq!(as_382, v382, "{name}");
    }
}

#[test]
fn deals_track_releases_and_details() {
    let xml = compact(&build(multi_track_album("3.8.2")));
    // Album + 4 track releases
    assert_eq!(xml.matches("<Release>").count(), 5, "{xml}");
    assert_eq!(
        xml.matches("<ReleaseType>TrackRelease</ReleaseType>")
            .count(),
        4,
        "{xml}"
    );
    // Consecutive deals on the same releases share a ReleaseDeal
    assert_eq!(xml.matches("<ReleaseDeal>").count(), 2, "{xml}");
    assert_eq!(xml.matches("<Deal>").count(), 3, "{xml}");
    assert_eq!(xml.matches("<DealReleaseReference>").count(), 5, "{xml}");
    assert!(
        xml.contains("<Usage><UseType>PermanentDownload</UseType></Usage>"),
        "{xml}"
    );
    // Non-territorial request details are written for Worldwide
    assert!(
        xml.contains("<SoundRecordingDetailsByTerritory><TerritoryCode>Worldwide</TerritoryCode>"),
        "{xml}"
    );
    assert!(
        xml.contains("<ReleaseDetailsByTerritory><TerritoryCode>Worldwide</TerritoryCode>"),
        "{xml}"
    );
    // Same request, same bytes
    assert_eq!(xml, compact(&build(multi_track_album("3.8.2"))));
}

#[test]
fn multi_disc_groups_have_no_resource_group_type() {
    let xml = compact(&build(multi_disc_album("3.8.2")));
    assert!(!xml.contains("ResourceGroupType"), "{xml}");
    assert!(
        xml.contains("<ResourceGroup><SequenceNumber>1</SequenceNumber>"),
        "{xml}"
    );
    assert!(
        xml.contains("<ResourceGroup><SequenceNumber>2</SequenceNumber>"),
        "{xml}"
    );
}

#[test]
fn missing_required_values_are_named() {
    let mut r = multi_track_album("3.8.2");
    r.header.message_recipient.party_id = None;
    r.releases[0].tracks[0].duration = String::new();
    r.releases[0].tracks[1].isrc = String::new();
    r.releases[0].tracks[3] = video(4, "Video");
    r.deals[0].deal_terms.use_types.clear();
    r.deals[1].deal_terms.territory_code.clear();
    r.deals[2].deal_terms.start_date = None;
    let errors = build_err(r);
    for expected in [
        "ERN 3.8.2 requires MessageHeader/MessageRecipient/PartyId (header.message_recipient.party_id)",
        "ERN 3.8.2 requires SoundRecording/Duration (releases[0].tracks[0].duration)",
        "ERN 3.8.2 requires an identifier in SoundRecording/SoundRecordingId (releases[0].tracks[1].isrc or releases[0].tracks[1].proprietary_ids)",
        "ERN 3.8.2: releases[0].tracks[3].resource_kind is Video; the builder writes Video resources for ERN 4.2/4.3 only, not 3.8.2",
        "ERN 3.8.2 requires DealTerms/Usage/UseType (deals[0].deal_terms.use_types)",
        "ERN 3.8.2 requires DealTerms/TerritoryCode (deals[1].deal_terms.territory_code)",
        // tracks[3] (now a video) no longer has its track release R4
        "ERN 3.8.2: deals[2].release_references lists \"R4\", which is not a release reference in this message",
        "ERN 3.8.2 requires a StartDate or EndDate in DealTerms/ValidityPeriod (deals[2].deal_terms.start_date or deals[2].deal_terms.end_date)",
    ] {
        assert_named(&errors, expected);
    }
    assert_eq!(errors.len(), 8, "{errors:#?}");

    let mut r = minimal_single("3.8.1");
    r.releases[0].title.clear();
    r.releases[0].release_id = String::new();
    r.releases[0].upc = None;
    r.releases[0].tracks.clear();
    r.deals[0].deal_reference = None;
    r.deals[0].deal_terms = Default::default();
    let errors = build_err(r);
    for expected in [
        "ERN 3.8.1 requires Release/ReferenceTitle (releases[0].title)",
        "ERN 3.8.1 requires an identifier in Release/ReleaseId (releases[0].release_id, releases[0].upc or releases[0].release_ids)",
        "ERN 3.8.1 requires a ReleaseResourceReference in Release/ReleaseResourceReferenceList (releases[0].tracks)",
        "ERN 3.8.1 requires Deal/DealReference when a deal has no DealTerms (deals[0].deal_reference or deals[0].deal_terms)",
    ] {
        assert_named(&errors, expected);
    }
}

/// 4.x values 3.8.2 has no place for: group types are not written (the group tree is), a
/// track release without a title takes its track's title.
#[test]
fn ern4_request_values_without_a_382_counterpart() {
    let mut r = multi_track_album("3.8.2");
    let refs: Vec<String> = r.releases[0]
        .tracks
        .iter()
        .map(|t| t.resource_reference.clone().unwrap())
        .collect();
    for t in &mut r.releases[0].tracks {
        t.sequence_number = None;
    }
    r.releases[0].resource_group = Some(ResourceGroupRequest {
        resource_groups: vec![ResourceGroupRequest {
            resource_group_type: Some("Component".to_string()),
            sequence_number: Some(1),
            content_items: refs
                .iter()
                .enumerate()
                .map(|(i, r)| ResourceGroupContentItemRequest {
                    sequence_number: Some(i as u32 + 1),
                    resource_reference: r.clone(),
                })
                .collect(),
            ..Default::default()
        }],
        ..Default::default()
    });
    r.releases[0].tracks[2]
        .track_release
        .as_mut()
        .unwrap()
        .title = None;
    let xml = build(r);
    assert_xsd_valid(
        &xml,
        "3.8.2",
        "nested group with a type, untitled track release",
    );
    let xml = compact(&xml);
    assert!(!xml.contains("ResourceGroupType"), "{xml}");
    assert!(
        xml.contains("<ResourceGroup><ResourceGroup><SequenceNumber>1</SequenceNumber>"),
        "{xml}"
    );
    assert!(
        xml.contains(
            "<ReleaseReference>R3</ReleaseReference><ReferenceTitle><TitleText>Track 3</TitleText>"
        ),
        "{xml}"
    );
}

#[test]
fn values_the_schema_cannot_express_are_rejected() {
    let mut r = minimal_single("3.8.2");
    r.releases[0].release_date = Some("01/02/2024".to_string());
    r.releases[0].tracks[0].resource_reference = Some("T1".to_string());
    r.releases[0].release_type = Some(AvsValueRequest::from(""));
    let errors = build_err(r);
    for expected in [
        "ERN 3.8.2: releases[0].release_date is \"01/02/2024\"; the schema requires an ISO date YYYY[-MM[-DD]]",
        "ERN 3.8.2: releases[0].tracks[0].resource_reference is \"T1\"; the schema requires a reference matching A[\\d\\-_a-zA-Z]+",
        "ERN 3.8.2: releases[0].release_type.value is empty",
    ] {
        assert_named(&errors, expected);
    }
}

#[test]
fn escaping_and_illegal_characters() {
    let mut r = minimal_single("3.8.2");
    r.releases[0].title = text("<script>alert('x')</script> & \"q\" 🎵 مرحبا");
    r.releases[0].artist = "A & B <C>".to_string();
    r.releases[0].tracks[0].title = "]]> it's".to_string();
    let xml = build(r);
    assert_xsd_valid(&xml, "3.8.2", "escaped text");
    assert!(
        xml.contains("&lt;script&gt;alert('x')&lt;/script&gt; &amp; \"q\" 🎵 مرحبا"),
        "{xml}"
    );
    assert!(
        xml.contains("<DisplayArtistName>A &amp; B &lt;C&gt;</DisplayArtistName>"),
        "{xml}"
    );
    assert!(xml.contains("]]&gt; it's"), "{xml}");
    let parsed = parse(&xml);
    assert_eq!(
        parsed.flat.releases[0].default_title.as_deref(),
        Some("<script>alert('x')</script> & \"q\" 🎵 مرحبا")
    );
    assert_eq!(
        parsed.flat.releases[0].tracks[0].title.as_deref(),
        Some("]]> it's")
    );

    let mut r = minimal_single("3.8.2");
    r.releases[0].tracks[0].title = "bad\u{1}title".to_string();
    let err = DDEXBuilder::new()
        .build(r, BuildOptions::default())
        .expect_err("illegal XML character");
    assert!(
        err.to_string().contains("releases[0].tracks[0].title"),
        "{err}"
    );
}
