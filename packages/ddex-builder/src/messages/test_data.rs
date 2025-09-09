//! Test data for UpdateReleaseMessage testing

/// Sample DDEX message version 1 for update testing
pub const SAMPLE_DDEX_V1: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
  <MessageHeader>
    <MessageId>MSG-ORIGINAL-001</MessageId>
    <MessageSender>
      <PartyName>Test Label Records</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP Platform</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-01T10:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R001</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-001</ResourceId>
      <ReferenceTitle>Amazing Song</ReferenceTitle>
      <DisplayArtist>The Test Band</DisplayArtist>
      <ISRC>TEST0012024001</ISRC>
      <Duration>PT3M30S</Duration>
      <TechnicalResourceDetails>
        <FileName>track001.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
        <BitRate>320</BitRate>
      </TechnicalResourceDetails>
    </SoundRecording>
    <SoundRecording>
      <ResourceReference>R002</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-002</ResourceId>
      <ReferenceTitle>Another Great Song</ReferenceTitle>
      <DisplayArtist>The Test Band</DisplayArtist>
      <ISRC>TEST0012024002</ISRC>
      <Duration>PT4M15S</Duration>
      <TechnicalResourceDetails>
        <FileName>track002.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
        <BitRate>320</BitRate>
      </TechnicalResourceDetails>
    </SoundRecording>
  </ResourceList>
  
  <ReleaseList>
    <Release>
      <ReleaseReference>REL001</ReleaseReference>
      <ReleaseId>album-001</ReleaseId>
      <ReleaseType>Album</ReleaseType>
      <Title>Test Album</Title>
      <DisplayArtist>The Test Band</DisplayArtist>
      <LabelName>Test Label Records</LabelName>
      <UPC>123456789012</UPC>
      <ReleaseDate>2024-03-15</ReleaseDate>
      <Genre>Rock</Genre>
      <ResourceGroup>
        <ResourceReference>R001</ResourceReference>
        <ResourceReference>R002</ResourceReference>
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
        </ValidityPeriod>
        <Price>
          <PriceAmount>9.99</PriceAmount>
          <PriceCurrencyCode>USD</PriceCurrencyCode>
        </Price>
      </DealTerms>
      <ReleaseReference>REL001</ReleaseReference>
    </ReleaseDeal>
  </DealList>
</NewReleaseMessage>"#;

/// Sample DDEX message version 2 with updates for testing
pub const SAMPLE_DDEX_V2_UPDATES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
  <MessageHeader>
    <MessageId>MSG-UPDATED-002</MessageId>
    <MessageSender>
      <PartyName>Test Label Records</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP Platform</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-15T14:30:00Z</MessageCreatedDateTime>
  </MessageHeader>
  
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R001</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-001</ResourceId>
      <ReferenceTitle>Amazing Song (Remastered)</ReferenceTitle>
      <DisplayArtist>The Test Band</DisplayArtist>
      <ISRC>TEST0012024001</ISRC>
      <Duration>PT3M35S</Duration>
      <TechnicalResourceDetails>
        <FileName>track001_remastered.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
        <BitRate>320</BitRate>
      </TechnicalResourceDetails>
    </SoundRecording>
    <SoundRecording>
      <ResourceReference>R002</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-002</ResourceId>
      <ReferenceTitle>Another Great Song</ReferenceTitle>
      <DisplayArtist>The Test Band</DisplayArtist>
      <ISRC>TEST0012024002</ISRC>
      <Duration>PT4M15S</Duration>
      <TechnicalResourceDetails>
        <FileName>track002.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
        <BitRate>320</BitRate>
      </TechnicalResourceDetails>
    </SoundRecording>
    <SoundRecording>
      <ResourceReference>R003</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-003</ResourceId>
      <ReferenceTitle>Brand New Track</ReferenceTitle>
      <DisplayArtist>The Test Band</DisplayArtist>
      <ISRC>TEST0012024003</ISRC>
      <Duration>PT3M45S</Duration>
      <TechnicalResourceDetails>
        <FileName>track003.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
        <BitRate>320</BitRate>
      </TechnicalResourceDetails>
    </SoundRecording>
  </ResourceList>
  
  <ReleaseList>
    <Release>
      <ReleaseReference>REL001</ReleaseReference>
      <ReleaseId>album-001</ReleaseId>
      <ReleaseType>Album</ReleaseType>
      <Title>Test Album (Deluxe Edition)</Title>
      <DisplayArtist>The Test Band</DisplayArtist>
      <LabelName>Test Label Records</LabelName>
      <UPC>123456789012</UPC>
      <ReleaseDate>2024-03-15</ReleaseDate>
      <Genre>Rock</Genre>
      <ResourceGroup>
        <ResourceReference>R001</ResourceReference>
        <ResourceReference>R002</ResourceReference>
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
        <TerritoryCode>CA</TerritoryCode>
        <ValidityPeriod>
          <StartDate>2024-03-15</StartDate>
          <EndDate>2025-03-15</EndDate>
        </ValidityPeriod>
        <Price>
          <PriceAmount>12.99</PriceAmount>
          <PriceCurrencyCode>USD</PriceCurrencyCode>
        </Price>
      </DealTerms>
      <ReleaseReference>REL001</ReleaseReference>
    </ReleaseDeal>
  </DealList>
</NewReleaseMessage>"#;

/// Sample DDEX with critical business changes
pub const SAMPLE_DDEX_V3_CRITICAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
  <MessageHeader>
    <MessageId>MSG-CRITICAL-003</MessageId>
    <MessageSender>
      <PartyName>Test Label Records</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP Platform</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-02-01T09:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R001</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-001</ResourceId>
      <ReferenceTitle>Amazing Song</ReferenceTitle>
      <DisplayArtist>The Test Band</DisplayArtist>
      <ISRC>CHANGED567890123</ISRC>
      <Duration>PT3M30S</Duration>
      <TechnicalResourceDetails>
        <FileName>track001.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
        <BitRate>320</BitRate>
      </TechnicalResourceDetails>
    </SoundRecording>
    <SoundRecording>
      <ResourceReference>R002</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>track-002</ResourceId>
      <ReferenceTitle>Another Great Song</ReferenceTitle>
      <DisplayArtist>The Test Band</DisplayArtist>
      <ISRC>TEST0012024002</ISRC>
      <Duration>PT4M15S</Duration>
      <TechnicalResourceDetails>
        <FileName>track002.mp3</FileName>
        <AudioCodecType>MP3</AudioCodecType>
        <BitRate>320</BitRate>
      </TechnicalResourceDetails>
    </SoundRecording>
  </ResourceList>
  
  <ReleaseList>
    <Release>
      <ReleaseReference>REL001</ReleaseReference>
      <ReleaseId>album-001</ReleaseId>
      <ReleaseType>Album</ReleaseType>
      <Title>Test Album</Title>
      <DisplayArtist>The Test Band</DisplayArtist>
      <LabelName>Test Label Records</LabelName>
      <UPC>987654321098</UPC>
      <ReleaseDate>2024-06-01</ReleaseDate>
      <Genre>Rock</Genre>
      <ResourceGroup>
        <ResourceReference>R001</ResourceReference>
        <ResourceReference>R002</ResourceReference>
      </ResourceGroup>
    </Release>
  </ReleaseList>
  
  <DealList>
    <ReleaseDeal>
      <DealReference>DEAL001</DealReference>
      <DealTerms>
        <CommercialModelType>PurchaseModel</CommercialModelType>
        <TerritoryCode>Worldwide</TerritoryCode>
        <ValidityPeriod>
          <StartDate>2024-06-01</StartDate>
        </ValidityPeriod>
        <Price>
          <PriceAmount>14.99</PriceAmount>
          <PriceCurrencyCode>USD</PriceCurrencyCode>
        </Price>
      </DealTerms>
      <ReleaseReference>REL001</ReleaseReference>
    </ReleaseDeal>
  </DealList>
</NewReleaseMessage>"#;