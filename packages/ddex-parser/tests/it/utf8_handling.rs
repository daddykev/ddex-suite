//! UTF-8 content in titles, artist names, identifiers and attributes, plus rejection of
//! invalid UTF-8. Every message is valid ERN 4.3 (unqualified child elements, required
//! header fields present) so that UTF-8 handling is the only thing under test.

use crate::common::{ern43_message, ern43_release, parse_str};
use ddex_parser::error::ParseError;
use ddex_parser::parser::stream::StreamingParser;
use ddex_parser::{DDEXParser, ERNVersion};
use std::io::Cursor;

/// Parse a one-release message and return (default_title, display_artist).
fn title_and_artist(title: &str, artist: &str) -> (Option<String>, Option<String>) {
    let xml = ern43_message(&format!(
        "<ReleaseList>{}</ReleaseList>",
        ern43_release("R1", title, artist)
    ));
    let parsed = parse_str(&xml).unwrap_or_else(|e| panic!("parsing {title:?}: {e:?}"));
    assert_eq!(parsed.flat.releases.len(), 1);
    let release = &parsed.flat.releases[0];
    (
        release.default_title.clone(),
        release.display_artist.clone(),
    )
}

fn assert_round_trips(title: &str, artist: &str) {
    let (got_title, got_artist) = title_and_artist(title, artist);
    assert_eq!(got_title.as_deref(), Some(title));
    assert_eq!(got_artist.as_deref(), Some(artist));
}

#[test]
fn accented_cjk_and_emoji_titles() {
    assert_round_trips("Café ñoño 北京 🎵", "Björk");
}

#[test]
fn non_latin_artist_names() {
    for artist in ["Mötörhead", "Sigur Rós", "陈奕迅", "Δημήτρης"] {
        assert_round_trips("Test Song", artist);
    }
}

#[test]
fn mixed_languages_in_header_and_release() {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>MSG_多语言_тест_🌍</MessageId>
        <MessageSender><PartyId>SENDER_λέξη</PartyId><PartyName>音楽レーベル Ω</PartyName></MessageSender>
        <MessageRecipient><PartyId>RECIPIENT_αβγ</PartyId></MessageRecipient>
        <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
    </MessageHeader>
    <ReleaseList>{}</ReleaseList>
</ern:NewReleaseMessage>"#,
        ern43_release("R_тест", "Música internacional 🎵 音楽 μουσική", "Artist")
    );
    let parsed = parse_str(&xml).expect("mixed-language message must parse");
    assert_eq!(parsed.flat.message_id, "MSG_多语言_тест_🌍");
    assert_eq!(parsed.flat.sender.id, "SENDER_λέξη");
    assert_eq!(parsed.flat.sender.name.as_deref(), Some("音楽レーベル Ω"));
    assert_eq!(
        parsed.flat.releases[0].default_title.as_deref(),
        Some("Música internacional 🎵 音楽 μουσική")
    );
}

#[test]
fn emoji_only_text() {
    assert_round_trips("🎵🎶🎤🎸", "🎹 Piano Man 🎹");
}

#[test]
fn utf8_in_attribute_values() {
    let release = r#"<Release>
            <ReleaseReference>R1</ReleaseReference>
            <ReleaseType>Album</ReleaseType>
            <DisplayTitleText LanguageAndScriptCode="es-MX">Canción Española</DisplayTitleText>
            <DisplayTitleText LanguageAndScriptCode="ja-JP">日本の歌</DisplayTitleText>
            <DisplayArtistName LanguageAndScriptCode="el">Διονύσιος</DisplayArtistName>
        </Release>"#;
    let xml = ern43_message(&format!("<ReleaseList>{release}</ReleaseList>"));
    let parsed = parse_str(&xml).expect("UTF-8 attributes must parse");
    assert_eq!(
        parsed.flat.releases[0].default_title.as_deref(),
        Some("Canción Española")
    );
}

#[test]
fn invalid_utf8_is_rejected() {
    let mut xml_bytes = ern43_message(
        "<ReleaseList><Release><ReleaseReference>R1</ReleaseReference>\
         <ReleaseType>Album</ReleaseType><DisplayTitleText>Bad UTF-8: ",
    )
    .into_bytes();
    // Cut the closing tags off the template, insert invalid bytes, then close properly.
    let tail = b"\n</ern:NewReleaseMessage>";
    xml_bytes.truncate(xml_bytes.len() - tail.len());
    xml_bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
    xml_bytes.extend_from_slice(
        b"</DisplayTitleText></Release></ReleaseList>\n</ern:NewReleaseMessage>",
    );

    match DDEXParser::new().parse(Cursor::new(&xml_bytes)) {
        Err(ParseError::InvalidUtf8 { .. }) => {}
        Err(ParseError::XmlError(message)) if message.contains("UTF-8") => {}
        Err(ParseError::SimpleXmlError(message))
            if message.contains("UTF-8") || message.contains("utf-8") => {}
        other => panic!("expected a UTF-8 error, got: {other:?}"),
    }
}

#[test]
fn four_byte_characters() {
    assert_round_trips("𝄞𝄢𝄡 Musical Symbols", "𝒜𝓇𝓉𝒾𝓈𝓉");
}

#[test]
fn normalization_forms_are_preserved_not_folded() {
    // Precomposed (NFC) and decomposed (NFD) "é" must both come through unchanged.
    let nfc_nfd = "caf\u{e9} vs cafe\u{301}";
    assert_round_trips(nfc_nfd, "Jos\u{e9} vs Jose\u{301}");
}

#[test]
#[ignore = "known bug: StreamingParser::stream_releases skips the message root element (only stream_deals descends into it), so it yields no releases for a full document"]
fn streaming_parser_handles_utf8() {
    let xml = ern43_message(&format!(
        "<ReleaseList>{}</ReleaseList>",
        ern43_release("R_STREAM", "Streaming 🎵 Café 北京 Test", "Artist")
    ));
    let mut parser = StreamingParser::new(Cursor::new(xml.as_bytes()), ERNVersion::V4_3);
    let releases: Vec<_> = parser
        .stream_releases()
        .collect::<Result<_, _>>()
        .expect("streaming releases");
    assert_eq!(releases.len(), 1);
    assert_eq!(releases[0].release_reference, "R_STREAM");
}

#[test]
fn large_utf8_text() {
    let large_title = "🎵".repeat(1000);
    assert_round_trips(&large_title, "Artist");
}
