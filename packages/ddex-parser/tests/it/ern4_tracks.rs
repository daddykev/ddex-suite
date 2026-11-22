//! ERN 4.2 / 4.3 track extraction: ResourceGroup trees, TrackRelease linking, party-resolved
//! artists, IsMainRelease promotion, and fixes to header names and release identifiers.
//!
//! Expected values for the official samples were produced by `scripts/ern4_oracle.py`
//! (lxml, independent of this parser). Absent data must stay `None`, never invented.

use ddex_core::models::flat::{ParsedERNMessage, ParsedRelease, TrackGroup};
use ddex_parser::DDEXParser;
use std::collections::BTreeSet;
use std::io::Cursor;
use std::time::Duration;

fn parse_path(path: &str) -> ParsedERNMessage {
    let xml = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {path}: {e}"));
    DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap_or_else(|e| panic!("parsing {path}: {e}"))
}

fn fixture(name: &str) -> ParsedERNMessage {
    parse_path(&format!(
        "{}/tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    ))
}

fn sample(dir: &str, name: &str) -> ParsedERNMessage {
    parse_path(&format!(
        "{}/../../examples/{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        dir,
        name
    ))
}

fn only_release(message: &ParsedERNMessage) -> &ParsedRelease {
    assert_eq!(message.flat.releases.len(), 1, "exactly one main release");
    &message.flat.releases[0]
}

fn group(group_type: &str, sequence_number: i32) -> TrackGroup {
    TrackGroup {
        group_type: Some(group_type.to_string()),
        sequence_number: Some(sequence_number),
    }
}

// ---------------------------------------------------------------------------
// Synthetic fixture: ern43_multi_disc.xml
// ---------------------------------------------------------------------------

#[test]
fn nested_component_groups_set_disc_and_restart_position() {
    let message = fixture("ern43_multi_disc.xml");
    let release = only_release(&message);

    // A4 is an Image in the top-level group: a resource, not a track.
    let refs: Vec<&str> = release.tracks.iter().map(|t| t.track_id.as_str()).collect();
    assert_eq!(refs, ["A1", "A2", "A3"]);
    assert!(message.flat.resources.contains_key("A4"));

    let layout: Vec<_> = release
        .tracks
        .iter()
        .map(|t| (t.disc_number, t.position, t.track_number))
        .collect();
    assert_eq!(
        layout,
        [
            (Some(1), 1, Some(1)),
            (Some(1), 2, Some(2)),
            // Item without SequenceNumber: track_number stays None, position is order.
            (Some(2), 1, None),
        ]
    );
    assert_eq!(release.tracks[0].group_path, [group("Component", 1)]);
    assert_eq!(release.tracks[2].group_path, [group("Component", 2)]);
    assert_eq!(release.track_count, 3);
    assert_eq!(release.disc_count, Some(2));
    assert_eq!(message.flat.stats.track_count, 3);
}

#[test]
fn track_values_come_from_the_resource() {
    let message = fixture("ern43_multi_disc.xml");
    let tracks = &only_release(&message).tracks;

    assert_eq!(tracks[0].isrc.as_deref(), Some("QZTEST000001"));
    assert_eq!(tracks[0].title.as_deref(), Some("Opening"));
    assert_eq!(tracks[0].duration, Some(Duration::from_secs(185)));
    assert_eq!(tracks[0].duration_formatted.as_deref(), Some("3:05"));
    // No DisplayTitleText: IsDefault DisplayTitle wins over the first one.
    assert_eq!(tracks[1].title.as_deref(), Some("Second"));
    // IsDefault DisplayTitleText / DisplayArtistName win over earlier entries.
    assert_eq!(tracks[2].title.as_deref(), Some("Third"));
    assert_eq!(tracks[2].display_artist.as_deref(), Some("Artist A"));
}

#[test]
fn track_without_duration_has_none() {
    let message = fixture("ern43_multi_disc.xml");
    let a2 = &only_release(&message).tracks[1];
    assert_eq!(a2.track_id, "A2");
    assert_eq!(a2.duration, None);
    assert_eq!(a2.duration_formatted, None);
}

#[test]
fn artist_is_resolved_through_party_reference() {
    let message = fixture("ern43_multi_disc.xml");
    let a2 = &only_release(&message).tracks[1];

    // No DisplayArtistName on A2: the name comes from PartyList via ArtistPartyReference.
    assert_eq!(a2.display_artist.as_deref(), Some("Artist B"));
    assert_eq!(a2.artists.len(), 1);
    assert_eq!(a2.artists[0].name, "Artist B");
    assert_eq!(a2.artists[0].role, "MainArtist");
    assert_eq!(a2.artists[0].party_id.as_deref(), Some("PArtistB"));

    // flat.parties is keyed by PartyReference.
    let keys: Vec<&str> = message.flat.parties.keys().map(String::as_str).collect();
    assert_eq!(keys, ["PArtistA", "PArtistB", "PLabel"]);
}

#[test]
fn track_release_is_linked_and_deals_resolve_to_tracks() {
    let message = fixture("ern43_multi_disc.xml");
    let release = only_release(&message);

    // TrackReleases are not top-level releases.
    assert_eq!(release.release_id, "R0");

    let a1 = &release.tracks[0];
    assert_eq!(a1.track_release_reference.as_deref(), Some("R1"));
    assert_eq!(a1.track_release_id.as_deref(), Some("TR-A1"));
    assert_eq!(release.tracks[1].track_release_reference, None);
    assert_eq!(release.tracks[1].track_release_id, None);

    // Deals keep references exactly as written...
    let deal_refs: Vec<&Vec<String>> = message.flat.deals.iter().map(|d| &d.releases).collect();
    assert_eq!(
        deal_refs,
        [&vec!["R1".to_string()], &vec!["R0".to_string()]]
    );
    // ...and a TrackRelease reference resolves to its track.
    let resolved: Vec<&str> = message.flat.deals[0]
        .releases
        .iter()
        .filter_map(|r| {
            release
                .tracks
                .iter()
                .find(|t| t.track_release_reference.as_deref() == Some(r.as_str()))
        })
        .map(|t| t.track_id.as_str())
        .collect();
    assert_eq!(resolved, ["A1"]);
}

#[test]
fn release_values_and_direct_release_id_only() {
    let message = fixture("ern43_multi_disc.xml");
    let release = only_release(&message);
    assert_eq!(release.default_title.as_deref(), Some("Test Album"));
    assert_eq!(release.display_artist.as_deref(), Some("Artist A"));
    // RelatedRelease ICPN 9999999999994 must not leak into the release's identifiers.
    assert_eq!(release.identifiers.upc.as_deref(), Some("0000000000017"));
}

#[test]
fn graph_keeps_the_full_resource_group_tree() {
    let message = fixture("ern43_multi_disc.xml");
    let group = message.graph.releases[0]
        .resource_group
        .as_ref()
        .expect("ResourceGroup tree");

    assert_eq!(group.resource_group_type, None);
    assert_eq!(group.resource_groups.len(), 2);
    let disc1 = &group.resource_groups[0];
    assert_eq!(disc1.resource_group_type.as_deref(), Some("Component"));
    assert_eq!(disc1.sequence_number, Some(1));
    assert_eq!(disc1.content_items.len(), 2);
    assert_eq!(disc1.content_items[1].release_resource_reference, "A2");
    assert_eq!(disc1.content_items[1].sequence_number, Some(2));
    assert_eq!(group.content_items[0].release_resource_reference, "A4");
    let linked = &group.linked_release_resource_references;
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0].resource_reference, "A4");
    assert_eq!(linked[0].link_description.as_deref(), Some("CoverArt"));

    let tr = &message.graph.track_releases[0];
    assert_eq!(tr.release_reference, "R1");
    assert_eq!(tr.release_resource_reference, "A1");
    assert_eq!(tr.release_label_reference, ["PLabel"]);
    assert_eq!(tr.is_main_release, None);
}

// ---------------------------------------------------------------------------
// IsMainRelease (4.3)
// ---------------------------------------------------------------------------

#[test]
fn main_track_release_becomes_the_release_when_there_is_no_release() {
    let message = fixture("ern43_main_track_release.xml");
    let release = only_release(&message);

    assert_eq!(release.release_id, "R1");
    assert_eq!(release.release_type, "TrackRelease");
    assert_eq!(
        release.default_title.as_deref(),
        Some("Lonely Track (Single)")
    );
    // A TrackRelease has no display artist in the XSD; nothing is borrowed or invented.
    assert_eq!(release.display_artist, None);
    assert_eq!(release.identifiers.proprietary.len(), 1);
    assert_eq!(release.identifiers.proprietary[0].value, "TR-MAIN");

    assert_eq!(release.tracks.len(), 1);
    let track = &release.tracks[0];
    assert_eq!(track.track_id, "A1");
    assert_eq!(track.isrc.as_deref(), Some("QZTEST000009"));
    assert_eq!(track.title.as_deref(), Some("Lonely Track"));
    assert_eq!(track.display_artist.as_deref(), Some("Artist A"));
    assert_eq!(track.position, 1);
    assert_eq!(track.disc_number, None);
    assert_eq!(track.track_number, None);
    assert_eq!(track.track_release_reference.as_deref(), Some("R1"));
    assert_eq!(track.track_release_id.as_deref(), Some("TR-MAIN"));
    assert_eq!(message.flat.stats.release_count, 1);
}

#[test]
fn main_track_release_is_not_promoted_when_a_release_exists() {
    let path = format!(
        "{}/tests/fixtures/ern43_multi_disc.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    let xml = std::fs::read_to_string(path)
        .unwrap()
        .replace("<TrackRelease>", r#"<TrackRelease IsMainRelease="true">"#);
    let message = DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap();
    assert_eq!(message.graph.track_releases[0].is_main_release, Some(true));
    let release = only_release(&message);
    assert_eq!(release.release_id, "R0");
}

// ---------------------------------------------------------------------------
// Parties and header
// ---------------------------------------------------------------------------

#[test]
fn party_without_reference_or_id_is_an_error() {
    let path = format!(
        "{}/tests/fixtures/ern43_multi_disc.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    let xml = std::fs::read_to_string(path)
        .unwrap()
        .replace("<PartyReference>PLabel</PartyReference>", "");
    let err = DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .expect_err("a party with neither PartyReference nor PartyId must not get a key");
    assert!(
        err.to_string().contains("Party/PartyReference"),
        "unexpected error: {err}"
    );
}

#[test]
fn dj_mix_without_sender_party_name_parses() {
    for dir in ["Samples42", "Samples43"] {
        let message = sample(dir, "8 DjMix.xml");
        assert_eq!(
            message.flat.sender.name, None,
            "{dir}: PartyName is optional"
        );
        assert_eq!(message.flat.sender.id, "PADPIDA3897722461G");
        let release = only_release(&message);
        let refs: Vec<&str> = release.tracks.iter().map(|t| t.track_id.as_str()).collect();
        assert_eq!(refs, ["A1"], "{dir}: the mix is the release's one track");
    }
}

// ---------------------------------------------------------------------------
// Official samples
// ---------------------------------------------------------------------------

#[test]
fn official_43_audio_sample_full_track_list() {
    // (resource ref, ISRC, title, seconds, TrackRelease ref, TrackRelease id) from the oracle.
    let expected = [
        (
            "A1",
            "JPTO09404900",
            "Yume no Lullaby",
            148,
            "R1",
            "00094631432057_JPTO09404900_R1",
        ),
        (
            "A2",
            "JPTO09404910",
            "Yume no Hajimari - Kaze no Okuri Mono (Introduction)",
            102,
            "R2",
            "00094631432057_JPTO09404910_R2",
        ),
        (
            "A3",
            "JPTO09404920",
            "-Haru- Soushunfu",
            72,
            "R3",
            "00094631432057_JPTO09404920_R3",
        ),
        (
            "A4",
            "JPTO09404930",
            "Dokoka de Haru ga",
            82,
            "R4",
            "00094631432057_JPTO09404930_R4",
        ),
        (
            "A5",
            "JPTO09404940",
            "Haru ga Kita",
            92,
            "R5",
            "00094631432057_JPTO09404940_R5",
        ),
        (
            "A6",
            "JPTO09404950",
            "Haru no Ogawa (Monbushou Shouka)",
            74,
            "R6",
            "00094631432057_JPTO09404950_R6",
        ),
        (
            "A7",
            "JPTO09404960",
            "-Natsu- Umi (Monbushou Shouka)",
            69,
            "R7",
            "00094631432057_JPTO09404960_R7",
        ),
        (
            "A8",
            "JPTO09404970",
            "Ware wa Umi no Ko",
            72,
            "R8",
            "00094631432057_JPTO09404970_R8",
        ),
        (
            "A9",
            "JPTO09404980",
            "Natsu no Omoide (Instrumental)",
            74,
            "R9",
            "00094631432057_JPTO09404980_R9",
        ),
        (
            "A10",
            "JPTO09404990",
            "Natsu wa Konu",
            90,
            "R10",
            "00094631432057_JPTO09404990_R10",
        ),
        (
            "A11",
            "JPTO09405000",
            "-Aki- Chiisai Aki Mitsuketa",
            65,
            "R11",
            "00094631432057_JPTO09405000_R11",
        ),
        (
            "A12",
            "JPTO09405010",
            "Sato no Aki",
            58,
            "R12",
            "00094631432057_JPTO09405010_R12",
        ),
        (
            "A13",
            "JPTO09405020",
            "Ryoshuu",
            81,
            "R13",
            "00094631432057_JPTO09405020_R13",
        ),
        (
            "A14",
            "JPTO09405030",
            "Aka Tonbo",
            105,
            "R14",
            "00094631432057_JPTO09405030_R14",
        ),
        (
            "A15",
            "JPTO09405040",
            "-Fuyu- Fuyu no Yoru (Monbushou Shouka)",
            86,
            "R15",
            "00094631432057_JPTO09405040_R15",
        ),
        (
            "A16",
            "JPTO09405050",
            "Pechka",
            65,
            "R16",
            "00094631432057_JPTO09405050_R16",
        ),
        (
            "A17",
            "JPTO09405060",
            "Yuki no Furu Machi wo",
            71,
            "R17",
            "00094631432057_JPTO09405060_R17",
        ),
        (
            "A18",
            "JPTO09405070",
            "Komoriuta",
            132,
            "R18",
            "00094631432057_JPTO09405070_R18",
        ),
        (
            "A19",
            "JPTO09332840",
            "Aerobics Mama",
            204,
            "R19",
            "00094631432057_JPTO09332840_R19",
        ),
        (
            "A20",
            "JPTO09405080",
            "Mother's Song -Kodomotachi e-",
            222,
            "R20",
            "00094631432057_JPTO09405080_R20",
        ),
        (
            "A21",
            "JPTO09332830",
            "Kitto Shiawase (NHK Minna no Uta)",
            226,
            "R21",
            "00094631432057_JPTO09332830_R21",
        ),
    ];

    let message = sample("Samples43", "1 Audio.xml");
    let release = only_release(&message);
    assert_eq!(release.release_id, "R0");
    assert_eq!(release.default_title.as_deref(), Some("Yume no Hajmari"));
    assert_eq!(release.identifiers.upc.as_deref(), Some("00094631432057"));
    assert_eq!(release.display_artist.as_deref(), Some("Saeko Shu"));
    assert_eq!(release.tracks.len(), expected.len());

    for (i, (track, (rref, isrc, title, secs, tr_ref, tr_id))) in
        release.tracks.iter().zip(expected).enumerate()
    {
        assert_eq!(track.track_id, rref);
        assert_eq!(track.isrc.as_deref(), Some(isrc), "{rref}");
        assert_eq!(track.title.as_deref(), Some(title), "{rref}");
        assert_eq!(track.display_artist.as_deref(), Some("Saeko Shu"), "{rref}");
        assert_eq!(track.duration, Some(Duration::from_secs(secs)), "{rref}");
        assert_eq!(
            track.track_release_reference.as_deref(),
            Some(tr_ref),
            "{rref}"
        );
        assert_eq!(track.track_release_id.as_deref(), Some(tr_id), "{rref}");
        // Single top-level group: no disc, position = document order = SequenceNumber.
        assert_eq!(track.disc_number, None, "{rref}");
        assert_eq!(track.position, i + 1, "{rref}");
        assert_eq!(track.track_number, Some(i as i32 + 1), "{rref}");
        assert!(track.group_path.is_empty(), "{rref}");
    }
    assert_eq!(message.flat.stats.track_count, 21);
}

#[test]
fn mixed_media_icpn_is_the_releases_own() {
    for dir in ["Samples42", "Samples43"] {
        let message = sample(dir, "3 MixedMedia.xml");
        let release = only_release(&message);
        // Not 04988006110809-style RelatedRelease IDs or ResourceGroup ReleaseIds.
        assert_eq!(
            release.identifiers.upc.as_deref(),
            Some("05099907138655"),
            "{dir}"
        );
        assert_eq!(release.tracks.len(), 17, "{dir}");
        let video = &release.tracks[14];
        assert_eq!(video.track_id, "A15");
        assert_eq!((video.disc_number, video.position), (Some(2), 1));
    }
}

#[test]
fn classical_multi_work_parts_do_not_set_disc() {
    let message = sample("Samples43", "Variant Classical.xml");
    let release = only_release(&message);
    assert_eq!(release.tracks.len(), 12);
    assert_eq!(release.disc_count, None);
    for (i, track) in release.tracks.iter().enumerate() {
        assert_eq!(track.disc_number, None, "{}", track.track_id);
        assert_eq!(track.position, i + 1, "{}", track.track_id);
        assert_eq!(
            track.track_number,
            Some((i % 3) as i32 + 1),
            "{}",
            track.track_id
        );
        assert_eq!(
            track.group_path,
            [group("MultiWorkPart", (i / 3) as i32 + 1)],
            "{}",
            track.track_id
        );
    }
}

#[test]
fn boxed_set_tracks_are_unambiguous() {
    let message = sample("Samples42", "Variant BoxedSet.xml");
    let release = only_release(&message);
    assert_eq!(release.tracks.len(), 121);
    // ReleaseComponent is the disc fallback when there is no Component group.
    assert_eq!(release.disc_count, Some(10));
    assert!(release
        .tracks
        .iter()
        .all(|t| t.disc_number.is_some() && t.group_path.len() == 1));

    let mut seen = BTreeSet::new();
    for track in &release.tracks {
        let path: Vec<_> = track
            .group_path
            .iter()
            .map(|g| (g.group_type.clone(), g.sequence_number))
            .collect();
        assert!(
            seen.insert((path, track.position)),
            "duplicate (group_path, position) for {}",
            track.track_id
        );
    }
    // Every TrackRelease (R1..R121) is linked to exactly one track.
    let linked: BTreeSet<_> = release
        .tracks
        .iter()
        .filter_map(|t| t.track_release_reference.clone())
        .collect();
    assert_eq!(linked.len(), 121);
}

#[test]
fn ern42_isrc_and_43_edition_isrc_agree() {
    let a42 = sample("Samples42", "1 Audio.xml");
    let a43 = sample("Samples43", "1 Audio.xml");
    let isrcs = |m: &ParsedERNMessage| -> Vec<Option<String>> {
        only_release(m)
            .tracks
            .iter()
            .map(|t| t.isrc.clone())
            .collect()
    };
    assert_eq!(isrcs(&a42), isrcs(&a43));
    assert!(isrcs(&a42).iter().all(Option::is_some));
}

/// Track counts for every official 4.2/4.3 sample, as produced by the independent lxml
/// oracle (`scripts/ern4_oracle.py`; see tests/fixtures/README.md). Guards the core
/// ResourceGroup-walk regression without needing Python.
#[test]
fn official_samples_track_counts_match_oracle() {
    // (dir, file, main releases, tracks)
    let expected = [
        ("Samples42", "1 Audio.xml", 1, 21),
        ("Samples42", "2 Video.xml", 1, 2),
        ("Samples42", "3 MixedMedia.xml", 1, 17),
        ("Samples42", "4 SimpleAudioSingle.xml", 1, 1),
        ("Samples42", "5 SimpleVideoSingle.xml", 1, 1),
        ("Samples42", "6 Ringtone.xml", 1, 1),
        ("Samples42", "7 LongformMusicalWorkVideo.xml", 1, 2),
        ("Samples42", "8 DjMix.xml", 1, 1),
        ("Samples42", "Variant BoxedSet.xml", 1, 121),
        ("Samples42", "Variant Classical.xml", 1, 12),
        ("Samples43", "1 Audio.xml", 1, 21),
        ("Samples43", "2 Video.xml", 1, 2),
        ("Samples43", "3 MixedMedia.xml", 1, 17),
        ("Samples43", "4 SimpleAudioSingle.xml", 1, 1),
        ("Samples43", "5 SimpleVideoSingle.xml", 1, 1),
        ("Samples43", "6 Ringtone.xml", 1, 1),
        ("Samples43", "7 LongformMusicalWorkVideo.xml", 1, 2),
        ("Samples43", "8 DjMix.xml", 1, 1),
        ("Samples43", "Variant Classical.xml", 1, 12),
    ];
    for (dir, file, releases, tracks) in expected {
        let message = sample(dir, file);
        assert_eq!(
            message.flat.releases.len(),
            releases,
            "{dir}/{file}: releases"
        );
        let got: usize = message.flat.releases.iter().map(|r| r.tracks.len()).sum();
        assert_eq!(got, tracks, "{dir}/{file}: tracks");
    }
}
