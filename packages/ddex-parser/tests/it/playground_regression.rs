//! Regression tests for the playground v0.4.3 sample messages.
use ddex_parser::error::ParseError;
use ddex_parser::DDEXParser;
use std::io::Cursor;

#[cfg(test)]
mod playground_regression_tests {
    use super::*;

    // Test the exact XML format used in playground v0.4.3 samples
    #[test]
    fn test_playground_ern43_sample() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">
  <MessageHeader>
    <MessageThreadId>PLAYGROUND_MSG_001</MessageThreadId>
    <MessageId>MSG_PLAYGROUND_2024</MessageId>
    <MessageSender>
      <PartyId>PLAYGROUND_LABEL</PartyId>
      <PartyName><FullName>Playground Record Label</FullName></PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId>PLAYGROUND_DSP</PartyId>
      <PartyName><FullName>Playground Streaming Platform</FullName></PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-15T10:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseType>Single</ReleaseType>
      <ReleaseId>
        <GRid>A1-PLAYGROUND-GRID-001</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Sample Track Release</TitleText>
      </ReferenceTitle>
    </Release>
  </ReleaseList>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>A1</ResourceReference>
      <SoundRecordingId>
        <ISRC>USPLAYG240001</ISRC>
      </SoundRecordingId>
      <ReferenceTitle>
        <TitleText>Sample Track</TitleText>
      </ReferenceTitle>
    </SoundRecording>
  </ResourceList>
  <DealList>
    <ReleaseDeal>
      <DealReleaseReference>R1</DealReleaseReference>
      <Deal>
        <DealReference>D1</DealReference>
        <TerritoryCode>Worldwide</TerritoryCode>
        <StartDate>2024-01-15</StartDate>
      </Deal>
    </ReleaseDeal>
  </DealList>
</ern:NewReleaseMessage>"#;

        let mut parser = DDEXParser::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = parser.parse(cursor);

        // This should NOT fail with "Missing required DDEX field" errors
        assert!(
            result.is_ok(),
            "Playground ERN 4.3 sample should parse successfully: {:?}",
            result.err()
        );

        let parsed = result.unwrap();
        assert_eq!(parsed.flat.message_id, "MSG_PLAYGROUND_2024");
        assert!(!parsed.flat.releases.is_empty());

        // Verify sender/recipient data is parsed correctly

        let sender = &parsed.flat.sender;
        let recipient = &parsed.flat.recipient;

        // These fields should be populated from the XML
        assert_eq!(sender.name.as_deref(), Some("Playground Record Label"));
        assert_eq!(sender.id, "PLAYGROUND_LABEL");
        assert_eq!(
            recipient.name.as_deref(),
            Some("Playground Streaming Platform")
        );
        assert_eq!(recipient.id, "PLAYGROUND_DSP");
    }

    #[test]
    fn test_playground_ern42_sample() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42" MessageSchemaVersionId="ern/42" LanguageAndScriptCode="en">
  <MessageHeader>
    <MessageThreadId>PLAYGROUND_MSG_42</MessageThreadId>
    <MessageId>MSG_PLAYGROUND_42_2024</MessageId>
    <MessageSender>
      <PartyId>INDIE_LABEL_42</PartyId>
      <PartyName><FullName>Indie Music Label 4.2</FullName></PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId>STREAMING_SERVICE_42</PartyId>
      <PartyName><FullName>Streaming Platform 4.2</FullName></PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-15T11:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseType>Album</ReleaseType>
      <ReleaseId>
        <GRid>A1-INDIE-GRID-001</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Indie Rock Album</TitleText>
      </ReferenceTitle>
    </Release>
  </ReleaseList>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>A1</ResourceReference>
      <SoundRecordingId>
        <ISRC>USIND240001</ISRC>
      </SoundRecordingId>
      <ReferenceTitle>
        <TitleText>Indie Rock Anthem</TitleText>
      </ReferenceTitle>
    </SoundRecording>
  </ResourceList>
  <DealList>
    <ReleaseDeal>
      <DealReleaseReference>R1</DealReleaseReference>
      <Deal>
        <DealReference>D1</DealReference>
        <TerritoryCode>US</TerritoryCode>
        <StartDate>2024-02-01</StartDate>
      </Deal>
    </ReleaseDeal>
  </DealList>
</ern:NewReleaseMessage>"#;

        let mut parser = DDEXParser::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = parser.parse(cursor);

        assert!(
            result.is_ok(),
            "Playground ERN 4.2 sample should parse successfully: {:?}",
            result.err()
        );

        let parsed = result.unwrap();
        assert_eq!(parsed.flat.message_id, "MSG_PLAYGROUND_42_2024");
        assert!(!parsed.flat.releases.is_empty());
    }

    // Test simplified PartyName format (the format that was failing)
    #[test]
    fn test_simplified_party_name_format() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">
  <MessageHeader>
    <MessageId>TEST_SIMPLIFIED</MessageId>
    <MessageSender>
      <PartyId>SIMPLE_LABEL</PartyId>
      <PartyName>Simple Label Name</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId>SIMPLE_DSP</PartyId>
      <PartyName>Simple DSP Name</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-15T10:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseType>Single</ReleaseType>
      <ReferenceTitle>
        <TitleText>Simple Title</TitleText>
      </ReferenceTitle>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#;

        let mut parser = DDEXParser::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = parser.parse(cursor);

        // This should work with our fallback parsing logic
        assert!(
            result.is_ok(),
            "Simplified PartyName format should parse successfully: {:?}",
            result.err()
        );

        let parsed = result.unwrap();
        assert_eq!(parsed.flat.message_id, "TEST_SIMPLIFIED");

        // Verify both PartyId and simplified PartyName are captured
        let sender = &parsed.flat.sender;
        let recipient = &parsed.flat.recipient;

        assert_eq!(sender.name.as_deref(), Some("Simple Label Name"));
        assert_eq!(sender.id, "SIMPLE_LABEL");
        assert_eq!(recipient.name.as_deref(), Some("Simple DSP Name"));
        assert_eq!(recipient.id, "SIMPLE_DSP");
    }

    // Test missing optional fields to ensure graceful handling
    #[test]
    fn test_missing_optional_fields() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">
  <MessageHeader>
    <MessageId>TEST_MINIMAL</MessageId>
    <MessageSender>
      <PartyId>MIN_LABEL</PartyId>
      <PartyName><FullName>Minimal Label</FullName></PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId>MIN_DSP</PartyId>
      <PartyName><FullName>Minimal DSP</FullName></PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-15T10:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseType>Single</ReleaseType>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#;

        let mut parser = DDEXParser::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = parser.parse(cursor);

        // Should handle missing optional fields gracefully
        assert!(
            result.is_ok(),
            "Minimal DDEX should parse successfully: {:?}",
            result.err()
        );
    }

    // Test error cases that should still fail appropriately
    #[test]
    fn test_truly_invalid_ddex() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">
  <MessageHeader>
    <!-- Missing MessageId, MessageSender, MessageRecipient completely -->
    <MessageCreatedDateTime>2024-01-15T10:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
</ern:NewReleaseMessage>"#;

        let mut parser = DDEXParser::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = parser.parse(cursor);

        // This should still fail appropriately for truly missing required fields
        assert!(
            result.is_err(),
            "Truly invalid DDEX should fail appropriately"
        );

        if let Err(ParseError::MissingField(field)) = result {
            // Should report missing required field with helpful context
            assert!(field.contains("MessageId") || field.contains("MessageSender"));
        }
    }
}
