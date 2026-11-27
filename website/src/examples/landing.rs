use ddex_builder::builder::{LocalizedStringRequest, PartyRequest, ReleaseRequest, TrackRequest};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use ddex_parser::DDEXParser;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse any ERN 3.8.x, 4.2 or 4.3 message
    let message = DDEXParser::new().parse(Cursor::new(std::fs::read("release.xml")?))?;
    for release in &message.flat.releases {
        println!("{:?} by {:?}", release.default_title, release.display_artist);
        for track in &release.tracks {
            println!("  {}. {:?} [{:?}]", track.position, track.title, track.isrc);
        }
    }

    // Build ERN 4.3: a required field that is missing fails the build, naming the field
    let text = |s: &str| vec![LocalizedStringRequest { text: s.into(), language_code: None }];
    let party = |name: &str, id: &str| PartyRequest {
        party_name: text(name),
        party_id: Some(id.into()),
        party_reference: None,
    };
    let mut request = BuildRequest {
        version: "4.3".into(),
        avs_version_id: Some("12".into()),
        language_and_script_code: Some("en".into()),
        releases: vec![ReleaseRequest {
            release_id: Some("A1-ABCDE-0000000001-M".into()),
            title: text("Example Single"),
            artist: Some("Example Artist".into()),
            label: Some("Example Label".into()),
            release_type: Some("Single".into()),
            genre: Some("Pop".into()),
            parental_warning_type: Some("NotExplicit".into()),
            tracks: vec![TrackRequest {
                track_id: "T1".into(),
                isrc: Some("ZZ0000000001".into()),
                title: "Example Track".into(),
                duration: "PT3M42S".into(),
                artist: Some("Example Artist".into()),
                resource_type: Some("MusicalWorkSoundRecording".into()),
                parental_warning_type: Some("NotExplicit".into()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    request.header.message_sender = party("Example Label", "PADPIDA0000000001");
    request.header.message_recipient = party("Example DSP", "PADPIDA0000000002");
    request.header.message_id = "MSG-001".into();
    request.header.message_created_date_time = "2024-01-01T00:00:00Z".into();

    let result = DDEXBuilder::new().build(request, BuildOptions::default())?;
    println!("{}", result.xml);
    Ok(())
}
