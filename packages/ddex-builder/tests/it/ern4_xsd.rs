//! ERN 4.2 / 4.3 builder output validates against the official DDEX XSDs (xmllint), in
//! both the default (DB-C14N) and pretty output modes.

use crate::ern4_support::*;
use ddex_builder::determinism::{CanonMode, DeterminismConfig};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};

type Case = (&'static str, fn(&str) -> BuildRequest);

const CASES: [Case; 4] = [
    ("minimal single", minimal_single),
    ("multi-track album", multi_track_album),
    ("multi-disc album", multi_disc_album),
    ("release with video", release_with_video),
];

#[test]
fn builder_output_is_schema_valid() {
    for version in VERSIONS {
        for (name, make) in CASES {
            let xml = build(make(version));
            assert_xsd_valid(&xml, version, name);
        }
    }
}

#[test]
fn pretty_output_is_schema_valid() {
    let options = || BuildOptions {
        determinism: Some(DeterminismConfig {
            canon_mode: CanonMode::Pretty,
            ..DeterminismConfig::default()
        }),
        ..BuildOptions::default()
    };
    for version in VERSIONS {
        for (name, make) in CASES {
            let xml = DDEXBuilder::new()
                .build(make(version), options())
                .expect("build")
                .xml;
            assert_xsd_valid(&xml, version, name);
        }
    }
}

#[test]
fn version_specific_resource_id_placement() {
    let v42 = build(minimal_single("4.2"));
    let v43 = build(minimal_single("4.3"));
    assert!(v42.contains("xmlns:ern=\"http://ddex.net/xml/ern/42\""));
    assert!(!v42.contains("SoundRecordingEdition"));
    assert!(!v42.contains("AvsVersionId"));
    assert!(v43.contains("xmlns:ern=\"http://ddex.net/xml/ern/43\""));
    assert!(v43.contains("<SoundRecordingEdition>"));
    assert!(v43.contains("AvsVersionId=\"12\""));
}

#[test]
fn multi_disc_tracks_are_grouped_per_disc() {
    let xml = build(multi_disc_album("4.3"));
    assert_eq!(
        xml.matches("ResourceGroupType=\"Component\"").count(),
        2,
        "{xml}"
    );
}

#[test]
fn party_references_follow_input_order() {
    let xml = build(multi_track_album("4.3"));
    // Release artist first, then the label; track artists and TrackRelease labels reuse them.
    let p1 = xml.find("<PartyReference>P1</PartyReference>").unwrap();
    let artist = xml[p1..].find("Example Artist").unwrap();
    let label = xml[p1..].find("Example Records").unwrap();
    assert!(artist < label, "{xml}");
    assert!(
        !xml.contains("<PartyReference>P3</PartyReference>"),
        "{xml}"
    );
    // Same request, same bytes
    assert_eq!(xml, build(multi_track_album("4.3")));
}
