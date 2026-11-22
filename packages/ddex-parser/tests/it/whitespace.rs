//! Text whitespace follows the XSD `whiteSpace` facet of each value's type:
//! - `xs:string`-derived values (titles, names, identifiers, AVS codes, `ddex_IsoDate`) are
//!   kept exactly as written: leading, trailing and repeated whitespace and newlines;
//! - references (`xs:ID` / `xs:IDREF`, which derive from `xs:token`) are collapsed;
//! - typed values (`xs:duration`, `xs:integer`, `xs:gYear`, `xs:dateTime`, ...) are collapsed
//!   before they are parsed.
//!
//! A present element is `Some(text)` even when the text is empty or whitespace only; an
//! absent element is `None`. Line endings are normalized to LF (XML 1.0 §2.11); a `&#13;`
//! character reference stays a carriage return.

use ddex_core::models::flat::ParsedERNMessage;
use ddex_parser::DDEXParser;
use std::io::Cursor;

fn parse(xml: &str) -> ParsedERNMessage {
    DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap_or_else(|e| panic!("parse failed: {e}\n{xml}"))
}

/// Titles exercised by both versions, in track order.
const TITLES: &[&str] = &[
    "  Leading and trailing  ",
    "Repeated   inner   spaces",
    "Line one\n  line two\n",
    "\tTabbed\t",
    "",
    "   ",
];

fn ern43(release_title: &str) -> String {
    let resources: String = TITLES
        .iter()
        .enumerate()
        .map(|(i, title)| {
            format!(
                r#"<SoundRecording><ResourceReference>
   A{n}
 </ResourceReference><Type>MusicalWorkSoundRecording</Type>
<SoundRecordingEdition><ResourceId><ISRC>ZZ000000000{n}</ISRC></ResourceId>
<PLine><Year> 2017 </Year><PLineText>  (P) 2017 Label  </PLineText></PLine></SoundRecordingEdition>
<DisplayTitleText>{title}</DisplayTitleText><DisplayTitle><TitleText>x</TitleText></DisplayTitle>
<DisplayArtistName>  Artist  </DisplayArtistName>
<Duration> PT3M{n}S </Duration><ParentalWarningType>NotExplicit</ParentalWarningType></SoundRecording>"#,
                n = i + 1
            )
        })
        .collect();
    let items: String = (1..=TITLES.len())
        .map(|n| {
            format!(
                "<ResourceGroupContentItem><SequenceNumber> {n} </SequenceNumber>\
                 <ReleaseResourceReference>\n A{n}\n</ReleaseResourceReference></ResourceGroupContentItem>"
            )
        })
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" LanguageAndScriptCode="en" AvsVersionId="12">
  <MessageHeader>
    <MessageId>  MSG 1  </MessageId>
    <MessageSender><PartyId>PADPIDA0000000001</PartyId><PartyName><FullName> Sender </FullName></PartyName></MessageSender>
    <MessageRecipient><PartyId>PADPIDA0000000002</PartyId><PartyName><FullName>Recipient</FullName></PartyName></MessageRecipient>
    <MessageCreatedDateTime>
      2024-01-01T00:00:00Z
    </MessageCreatedDateTime>
    <MessageControlType>TestMessage</MessageControlType>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference> R0 </ReleaseReference>
      <ReleaseType>Album</ReleaseType>
      <ReleaseId><ICPN>123456789012</ICPN></ReleaseId>
      <DisplayTitleText>{release_title}</DisplayTitleText>
      <DisplayTitle><TitleText>x</TitleText></DisplayTitle>
      <DisplayArtistName>Artist</DisplayArtistName>
      <Genre><GenreText>  Hip   Hop  </GenreText><SubGenre> </SubGenre></Genre>
      <ReleaseDate>2017-03-01</ReleaseDate>
      <ParentalWarningType>NotExplicit</ParentalWarningType>
      <ResourceGroup>{items}</ResourceGroup>
    </Release>
  </ReleaseList>
  <ResourceList>{resources}</ResourceList>
  <DealList>
    <ReleaseDeal>
      <DealReleaseReference>
        R0
      </DealReleaseReference>
      <Deal><DealTerms><TerritoryCode>Worldwide</TerritoryCode>
        <ValidityPeriod><StartDate> 2024-01-01 </StartDate><EndDate>2024-12-31</EndDate></ValidityPeriod>
      </DealTerms></Deal>
    </ReleaseDeal>
  </DealList>
</ern:NewReleaseMessage>"#
    )
}

fn ern382(release_title: &str) -> String {
    let resources: String = TITLES
        .iter()
        .enumerate()
        .map(|(i, title)| {
            format!(
                r#"<SoundRecording><SoundRecordingId><ISRC>ZZ000000000{n}</ISRC></SoundRecordingId>
<ResourceReference> A{n} </ResourceReference><ReferenceTitle><TitleText>{title}</TitleText></ReferenceTitle>
<Duration>PT3M{n}S</Duration>
<SoundRecordingDetailsByTerritory><TerritoryCode>Worldwide</TerritoryCode>
<DisplayArtist><PartyName><FullName>  Artist  </FullName></PartyName><ArtistRole>MainArtist</ArtistRole></DisplayArtist>
<LabelName>  Label  </LabelName></SoundRecordingDetailsByTerritory></SoundRecording>"#,
                n = i + 1
            )
        })
        .collect();
    let refs: String = (1..=TITLES.len())
        .map(|n| format!("<ReleaseResourceReference>\n  A{n}  \n</ReleaseResourceReference>"))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/382" MessageSchemaVersionId="ern/382" LanguageAndScriptCode="en">
  <MessageHeader>
    <MessageId>MSG2</MessageId>
    <MessageSender><PartyId>PADPIDA0000000001</PartyId><PartyName><FullName>Sender</FullName></PartyName></MessageSender>
    <MessageRecipient><PartyId>PADPIDA0000000002</PartyId><PartyName><FullName>Recipient</FullName></PartyName></MessageRecipient>
    <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ResourceList>{resources}</ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseId><ICPN>123456789012</ICPN></ReleaseId>
      <ReleaseReference>R0</ReleaseReference>
      <ReferenceTitle><TitleText>{release_title}</TitleText></ReferenceTitle>
      <ReleaseResourceReferenceList>{refs}</ReleaseResourceReferenceList>
      <ReleaseType>Album</ReleaseType>
      <ReleaseDetailsByTerritory><TerritoryCode>Worldwide</TerritoryCode>
        <LabelName> Label </LabelName>
        <Genre><GenreText> Pop </GenreText></Genre>
      </ReleaseDetailsByTerritory>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#
    )
}

fn assert_titles(msg: &ParsedERNMessage, release_title: &str) {
    let release = &msg.flat.releases[0];
    assert_eq!(release.default_title.as_deref(), Some(release_title));
    let titles: Vec<Option<&str>> = release.tracks.iter().map(|t| t.title.as_deref()).collect();
    let expected: Vec<Option<&str>> = TITLES.iter().map(|t| Some(*t)).collect();
    assert_eq!(titles, expected, "track titles");
    // References are collapsed, so every track still resolves to its resource.
    let ids: Vec<&str> = release.tracks.iter().map(|t| t.track_id.as_str()).collect();
    assert_eq!(ids, ["A1", "A2", "A3", "A4", "A5", "A6"]);
}

#[test]
fn ern43_text_is_exact_and_references_collapse() {
    let title = "\n   The   Album  \n";
    let msg = parse(&ern43(title));
    assert_titles(&msg, title);
    let flat = &msg.flat;
    assert_eq!(flat.message_id, "  MSG 1  ");
    assert_eq!(flat.sender.name.as_deref(), Some(" Sender "));
    assert_eq!(flat.message_control_type.as_deref(), Some("TestMessage"));
    // MessageCreatedDateTime is an xs:dateTime: parsed after collapsing.
    assert_eq!(flat.message_date.to_rfc3339(), "2024-01-01T00:00:00+00:00");
    let release = &flat.releases[0];
    assert_eq!(release.release_id, "R0");
    assert_eq!(release.genre.as_deref(), Some("  Hip   Hop  "));
    assert_eq!(
        release.sub_genre.as_deref(),
        Some(" "),
        "present whitespace-only SubGenre"
    );
    let track = &release.tracks[0];
    assert_eq!(track.display_artist.as_deref(), Some("  Artist  "));
    assert_eq!(
        track.track_number,
        Some(1),
        "xs:integer SequenceNumber collapsed"
    );
    assert_eq!(
        track.duration.map(|d| d.as_secs()),
        Some(181),
        "xs:duration collapsed"
    );
    let p_line = track.p_line.as_ref().expect("P line");
    assert_eq!(p_line.text, "  (P) 2017 Label  ");
    assert_eq!(p_line.year, Some(2017), "xs:gYear collapsed");
    let deal = &flat.deals[0];
    assert_eq!(deal.releases, ["R0"], "DealReleaseReference collapsed");
    assert!(deal.validity.start.is_some() && deal.validity.end.is_some());
}

#[test]
fn ern382_text_is_exact_and_references_collapse() {
    let title = "  Album\twith\ttabs ";
    let msg = parse(&ern382(title));
    assert_titles(&msg, title);
    let release = &msg.flat.releases[0];
    assert_eq!(release.label_name.as_deref(), Some(" Label "));
    assert_eq!(release.genre.as_deref(), Some(" Pop "));
    let track = &release.tracks[0];
    assert_eq!(track.display_artist.as_deref(), Some("  Artist  "));
    assert_eq!(track.label_name.as_deref(), Some("  Label  "));
}

#[test]
fn line_endings_are_normalized_but_char_references_kept() {
    let xml = ern43("Line one\r\nline two\rthree &#13;four")
        .replace("  <MessageHeader>", "\r\n  <MessageHeader>");
    let msg = parse(&xml);
    assert_eq!(
        msg.flat.releases[0].default_title.as_deref(),
        Some("Line one\nline two\nthree \rfour")
    );
}

#[test]
fn absent_title_is_none_and_empty_title_is_some() {
    // Image without any title: None. An empty TitleText element: Some("").
    let xml = ern382("").replace(
        "<ResourceList>",
        "<ResourceList><Image><ImageType>FrontCoverImage</ImageType><ResourceReference>IMG</ResourceReference></Image>",
    );
    let msg = parse(&xml);
    assert_eq!(msg.flat.resources["IMG"].title, None);
    assert_eq!(msg.flat.releases[0].default_title.as_deref(), Some(""));
    let empty = msg.flat.resources.get("A5").expect("A5");
    assert_eq!(empty.title.as_deref(), Some(""));
}
