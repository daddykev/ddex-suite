//! Test data for DDEX semantic diff testing

/// Sample DDEX XML for testing - Version 1
pub const SAMPLE_DDEX_V1: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
  <MessageHeader>
    <MessageId>MSG-001</MessageId>
    <MessageSender>
      <PartyName>Test Label</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R001</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-001</ResourceId>
      <ReferenceTitle>Test Track</ReferenceTitle>
      <DisplayArtist>Test Artist</DisplayArtist>
      <ISRC>TEST0012024001</ISRC>
      <Duration>PT3M30S</Duration>
      <TechnicalResourceDetails>
        <FileName>track001.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
      </TechnicalResourceDetails>
    </SoundRecording>
  </ResourceList>
  
  <ReleaseList>
    <Release>
      <ReleaseReference>REL001</ReleaseReference>
      <ReleaseId>album-001</ReleaseId>
      <ReleaseType>Album</ReleaseType>
      <Title>Test Album</Title>
      <DisplayArtist>Test Artist</DisplayArtist>
      <LabelName>Test Label</LabelName>
      <UPC>123456789012</UPC>
      <ReleaseDate>2024-03-15</ReleaseDate>
      <Genre>Pop</Genre>
      <ResourceGroup>
        <ResourceReference>R001</ResourceReference>
      </ResourceGroup>
    </Release>
  </ReleaseList>
</NewReleaseMessage>"#;

/// Sample DDEX XML for testing - Version 2 (with changes)
pub const SAMPLE_DDEX_V2: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
  <MessageHeader>
    <MessageId>MSG-002</MessageId>
    <MessageSender>
      <PartyName>Test Label</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-02T12:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R002</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-001</ResourceId>
      <ReferenceTitle>Test Track (Remastered)</ReferenceTitle>
      <DisplayArtist>Test Artist</DisplayArtist>
      <ISRC>TEST0012024001</ISRC>
      <Duration>PT3M45S</Duration>
      <TechnicalResourceDetails>
        <FileName>track001_remastered.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
      </TechnicalResourceDetails>
    </SoundRecording>
  </ResourceList>
  
  <ReleaseList>
    <Release>
      <ReleaseReference>REL002</ReleaseReference>
      <ReleaseId>album-001</ReleaseId>
      <ReleaseType>Album</ReleaseType>
      <Title>Test Album (Deluxe Edition)</Title>
      <DisplayArtist>Test Artist</DisplayArtist>
      <LabelName>New Test Label</LabelName>
      <UPC>987654321098</UPC>
      <ReleaseDate>2024-03-15</ReleaseDate>
      <Genre>Rock</Genre>
      <ResourceGroup>
        <ResourceReference>R002</ResourceReference>
      </ResourceGroup>
    </Release>
  </ReleaseList>
</NewReleaseMessage>"#;

/// Sample DDEX XML with critical business changes
pub const SAMPLE_DDEX_CRITICAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
  <MessageHeader>
    <MessageId>MSG-003</MessageId>
    <MessageSender>
      <PartyName>Test Label</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-03T18:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R003</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-001</ResourceId>
      <ReferenceTitle>Test Track</ReferenceTitle>
      <DisplayArtist>Test Artist</DisplayArtist>
      <ISRC>DIFFERENT123456</ISRC>
      <Duration>PT3M30S</Duration>
      <TechnicalResourceDetails>
        <FileName>track001.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
      </TechnicalResourceDetails>
    </SoundRecording>
  </ResourceList>
  
  <ReleaseList>
    <Release>
      <ReleaseReference>REL003</ReleaseReference>
      <ReleaseId>album-001</ReleaseId>
      <ReleaseType>Album</ReleaseType>
      <Title>Test Album</Title>
      <DisplayArtist>Test Artist</DisplayArtist>
      <LabelName>Test Label</LabelName>
      <UPC>555666777888</UPC>
      <ReleaseDate>2024-12-01</ReleaseDate>
      <Genre>Pop</Genre>
      <ResourceGroup>
        <ResourceReference>R003</ResourceReference>
      </ResourceGroup>
    </Release>
  </ReleaseList>
  
  <DealList>
    <ReleaseDeal>
      <DealReference>DEAL001</DealReference>
      <DealTerms>
        <CommercialModelType>SubscriptionModel</CommercialModelType>
        <TerritoryCode>US</TerritoryCode>
        <ValidityPeriod>
          <StartDate>2024-03-15</StartDate>
          <EndDate>2025-03-15</EndDate>
        </ValidityPeriod>
        <Price>
          <PriceAmount>9.99</PriceAmount>
          <PriceCurrencyCode>USD</PriceCurrencyCode>
        </Price>
      </DealTerms>
      <ReleaseReference>REL003</ReleaseReference>
    </ReleaseDeal>
  </DealList>
</NewReleaseMessage>"#;

/// Sample DDEX with only formatting differences
pub const SAMPLE_DDEX_FORMATTING: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage   xmlns="http://ddex.net/xml/ern/43"    MessageSchemaVersionId="ern/43"  >
    <MessageHeader>
        <MessageId>MSG-001</MessageId>
        <MessageSender>
            <PartyName>Test Label</PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyName>Test DSP</PartyName>
        </MessageRecipient>
        <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
    </MessageHeader>
    
    <ResourceList>
        <SoundRecording>
            <ResourceReference>R001</ResourceReference>
            <Type>SoundRecording</Type>
            <ResourceId>track-001</ResourceId>
            <ReferenceTitle>  Test Track  </ReferenceTitle>
            <DisplayArtist>Test Artist</DisplayArtist>
            <ISRC>TEST0012024001</ISRC>
            <Duration>PT3M30S</Duration>
            <TechnicalResourceDetails>
                <FileName>track001.mp3</FileName>
                <AudioCodecType>MP3</AudioCodecType>
            </TechnicalResourceDetails>
        </SoundRecording>
    </ResourceList>
    
    <ReleaseList>
        <Release>
            <ReleaseReference>REL001</ReleaseReference>
            <ReleaseId>album-001</ReleaseId>
            <ReleaseType>Album</ReleaseType>
            <Title>Test Album</Title>
            <DisplayArtist>Test Artist</DisplayArtist>
            <LabelName>Test Label</LabelName>
            <UPC>123456789012</UPC>
            <ReleaseDate>2024-03-15</ReleaseDate>
            <Genre>Pop</Genre>
            <ResourceGroup>
                <ResourceReference>R001</ResourceReference>
            </ResourceGroup>
        </Release>
    </ReleaseList>
</NewReleaseMessage>"#;