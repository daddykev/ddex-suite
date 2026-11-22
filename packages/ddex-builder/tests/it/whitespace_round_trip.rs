//! Whitespace through build -> parse: titles with leading, trailing and repeated
//! whitespace, tabs and newlines, and a whitespace-only title are written by the builder and
//! read back by ddex-parser exactly (ERN 4.2, 4.3 and 3.8.2), and the output is
//! schema-valid.
//!
//! An empty title cannot make the trip: the builder's request model uses `""` for "not
//! given" (`title: String`), so it reports the required title as missing. The parser side of
//! that case (`<TitleText></TitleText>` -> `Some("")`) is covered in ddex-parser's
//! `whitespace` tests.

use crate::ern4_support::{assert_xsd_valid, build, deal, parse, release, request, track};

const TITLES: &[&str] = &[
    "  Leading and trailing  ",
    "Repeated   inner   spaces",
    "Line one\n  line two\n",
    "\tTabbed\t",
    "   ",
];

#[test]
fn titles_survive_build_then_parse_exactly() {
    let release_title = "\n  The   Album \t";
    for version in ["4.2", "4.3", "3.8.2"] {
        let tracks = TITLES
            .iter()
            .enumerate()
            .map(|(i, t)| track(i as u32 + 1, t))
            .collect();
        let req = request(
            version,
            release(release_title, "Album", tracks),
            vec![deal(&["R0"], "SubscriptionModel", "OnDemandStream")],
        );
        let xml = build(req);
        assert_xsd_valid(&xml, version, "whitespace titles");
        let flat = parse(&xml).flat;
        let r = &flat.releases[0];
        assert_eq!(
            r.default_title.as_deref(),
            Some(release_title),
            "{version} release title\n{xml}"
        );
        let titles: Vec<Option<&str>> = r.tracks.iter().map(|t| t.title.as_deref()).collect();
        let expected: Vec<Option<&str>> = TITLES.iter().map(|t| Some(*t)).collect();
        assert_eq!(titles, expected, "{version} track titles\n{xml}");
    }
}

#[test]
fn empty_title_is_reported_missing_by_the_builder() {
    for (version, element) in [
        ("4.2", "SoundRecording/DisplayTitleText"),
        ("4.3", "SoundRecording/DisplayTitleText"),
        ("3.8.2", "SoundRecording/ReferenceTitle"),
    ] {
        let req = request(
            version,
            release("Album", "Album", vec![track(1, "")]),
            vec![deal(&["R0"], "SubscriptionModel", "OnDemandStream")],
        );
        let err = ddex_builder::DDEXBuilder::new()
            .build(req, ddex_builder::BuildOptions::default())
            .expect_err("empty title is 'not given' in the request model");
        let message = err.to_string();
        assert!(
            message.contains(element) && message.contains("releases[0].tracks[0].title"),
            "{version}: {message}"
        );
    }
}
