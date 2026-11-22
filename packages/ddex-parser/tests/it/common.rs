//! Shared helpers for the ddex-parser integration tests.

use ddex_core::models::flat::ParsedERNMessage;
use ddex_parser::error::ParseError;
use ddex_parser::DDEXParser;
use std::io::Cursor;

/// Absolute path of a file under `tests/fixtures/`.
pub fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

/// Contents of a file under `tests/fixtures/`.
pub fn read_fixture(name: &str) -> String {
    let path = fixture_path(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path}: {e}"))
}

/// Parse a string with a default parser.
pub fn parse_str(xml: &str) -> Result<ParsedERNMessage, ParseError> {
    DDEXParser::new().parse(Cursor::new(xml.as_bytes()))
}

/// Parse a fixture, panicking with the parse error on failure.
pub fn parse_fixture(name: &str) -> ParsedERNMessage {
    parse_str(&read_fixture(name)).unwrap_or_else(|e| panic!("parsing {name}: {e}"))
}

/// A minimal ERN 4.3 `MessageHeader` with every field the parser requires.
pub const ERN43_HEADER: &str = r#"<MessageHeader>
        <MessageId>MSG001</MessageId>
        <MessageSender><PartyId>PADPIDA0000000001</PartyId><PartyName>Sender</PartyName></MessageSender>
        <MessageRecipient><PartyId>PADPIDA0000000002</PartyId><PartyName>Recipient</PartyName></MessageRecipient>
        <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
    </MessageHeader>"#;

/// Wrap `body` (lists such as `<ReleaseList>`) in an ERN 4.3 message with a valid header.
/// Child elements are unqualified, as the ERN schemas require.
pub fn ern43_message(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    {ERN43_HEADER}
    {body}
</ern:NewReleaseMessage>"#
    )
}

/// An ERN 4.3 release with the given reference, title and display artist name.
pub fn ern43_release(reference: &str, title: &str, artist: &str) -> String {
    format!(
        r#"<Release>
            <ReleaseReference>{reference}</ReleaseReference>
            <ReleaseType>Album</ReleaseType>
            <ReleaseId><GRid>A1-TEST-GRID-0001</GRid></ReleaseId>
            <DisplayTitleText>{title}</DisplayTitleText>
            <DisplayArtistName>{artist}</DisplayArtistName>
        </Release>"#
    )
}
