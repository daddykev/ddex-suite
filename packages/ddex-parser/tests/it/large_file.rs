//! Large inputs go through the complete (DOM) parser: `parse()` no longer switches to the
//! experimental streaming path above 10 MB, so every field is still read. The streaming path
//! is used only when `ParseMode::Stream` is requested explicitly.

use ddex_parser::parser::mode::ParseMode;
use ddex_parser::parser::ParseOptions;
use ddex_parser::DDEXParser;
use std::io::Cursor;

const TRACKS: usize = 2_000;

/// An ERN 4.3 message of more than 10 MB: `TRACKS` sound recordings in one release, padded
/// with a comment.
fn large_ern43() -> String {
    let mut resources = String::new();
    let mut items = String::new();
    for n in 1..=TRACKS {
        resources.push_str(&format!(
            "<SoundRecording><ResourceReference>A{n}</ResourceReference>\
             <Type>MusicalWorkSoundRecording</Type>\
             <SoundRecordingEdition><ResourceId><ISRC>ZZ{n:010}</ISRC></ResourceId>\
             <PLine><Year>2020</Year><PLineText>(P) 2020 Label</PLineText></PLine></SoundRecordingEdition>\
             <DisplayTitleText>Track {n}</DisplayTitleText><DisplayTitle><TitleText>Track {n}</TitleText></DisplayTitle>\
             <DisplayArtistName>Artist</DisplayArtistName><Duration>PT3M</Duration>\
             <ParentalWarningType>NotExplicit</ParentalWarningType></SoundRecording>\n"
        ));
        items.push_str(&format!(
            "<ResourceGroupContentItem><SequenceNumber>{n}</SequenceNumber>\
             <ReleaseResourceReference>A{n}</ReleaseResourceReference></ResourceGroupContentItem>"
        ));
    }
    let padding = "x".repeat(11 * 1024 * 1024);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" LanguageAndScriptCode="en" AvsVersionId="12">
  <!-- {padding} -->
  <MessageHeader>
    <MessageId>LARGE-1</MessageId>
    <MessageSender><PartyId>PADPIDA0000000001</PartyId><PartyName><FullName>Sender</FullName></PartyName></MessageSender>
    <MessageRecipient><PartyId>PADPIDA0000000002</PartyId><PartyName><FullName>Recipient</FullName></PartyName></MessageRecipient>
    <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
    <MessageControlType>LiveMessage</MessageControlType>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference>R0</ReleaseReference>
      <ReleaseType>Album</ReleaseType>
      <ReleaseId><ICPN>123456789012</ICPN></ReleaseId>
      <DisplayTitleText>Large Album</DisplayTitleText>
      <DisplayTitle><TitleText>Large Album</TitleText></DisplayTitle>
      <DisplayArtistName>Artist</DisplayArtistName>
      <CLine><Year>2020</Year><CLineText>(C) 2020 Label</CLineText></CLine>
      <Genre><GenreText>Pop</GenreText><SubGenre>Synth</SubGenre></Genre>
      <ReleaseDate>2020-05-01</ReleaseDate>
      <ParentalWarningType>NotExplicit</ParentalWarningType>
      <ResourceGroup>{items}</ResourceGroup>
    </Release>
  </ReleaseList>
  <ResourceList>{resources}</ResourceList>
  <DealList>
    <ReleaseDeal>
      <DealReleaseReference>R0</DealReleaseReference>
      <Deal><DealTerms><TerritoryCode>Worldwide</TerritoryCode>
        <ValidityPeriod><StartDate>2020-05-01</StartDate><EndDate>2030-05-01</EndDate></ValidityPeriod>
      </DealTerms></Deal>
    </ReleaseDeal>
  </DealList>
</ern:NewReleaseMessage>"#
    )
}

#[test]
fn input_over_10_mb_uses_the_complete_parser() {
    let xml = large_ern43();
    assert!(xml.len() > 10 * 1024 * 1024, "input is {} bytes", xml.len());

    let msg = DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .expect("parse");
    let flat = &msg.flat;
    assert_eq!(flat.message_control_type.as_deref(), Some("LiveMessage"));
    let release = &flat.releases[0];
    assert_eq!(release.tracks.len(), TRACKS);
    assert_eq!(release.genre.as_deref(), Some("Pop"));
    assert_eq!(release.sub_genre.as_deref(), Some("Synth"));
    assert_eq!(release.parental_warning.as_deref(), Some("NotExplicit"));
    assert_eq!(release.release_dates[0].date, "2020-05-01");
    assert!(release.release_date.is_some());
    assert_eq!(release.c_line.as_ref().map(|c| c.year), Some(Some(2020)));
    let last = release.tracks.last().unwrap();
    assert_eq!(last.track_number, Some(TRACKS as i32));
    assert_eq!(
        last.p_line.as_ref().map(|p| p.text.as_str()),
        Some("(P) 2020 Label")
    );
    assert_eq!(last.parental_warning.as_deref(), Some("NotExplicit"));
    assert!(flat.deals[0].validity.end.is_some());
}

#[test]
fn dom_is_the_default_mode() {
    assert_eq!(ParseMode::default(), ParseMode::Dom);
    assert_eq!(ParseOptions::default().mode, ParseMode::Dom);
}
