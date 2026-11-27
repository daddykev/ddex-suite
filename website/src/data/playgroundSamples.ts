// Playground samples. The two ERN messages validate against the ERN 4.3 XSD, and the
// build request builds with ddex-builder 0.5.1 into XML that validates against the same XSD.
// Party IDs are DPID-format placeholders; ISRCs, ICPNs and the GRid are made up.

export const PARSE_SAMPLES: Record<string, string> = {
  'ERN 4.3 single (1 track)': `<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" LanguageAndScriptCode="en" AvsVersionId="12">
  <MessageHeader>
    <MessageThreadId>PLAYGROUND-THREAD-1</MessageThreadId>
    <MessageId>PLAYGROUND-MSG-1</MessageId>
    <MessageSender>
      <PartyId>PADPIDA0000000001</PartyId>
      <PartyName><FullName>Playground Records</FullName></PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId>PADPIDA0000000002</PartyId>
      <PartyName><FullName>Playground Streaming</FullName></PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2026-01-15T10:00:00Z</MessageCreatedDateTime>
    <MessageControlType>TestMessage</MessageControlType>
  </MessageHeader>
  <PartyList>
    <Party>
      <PartyReference>PArtist</PartyReference>
      <PartyName><FullName>The Example Band</FullName></PartyName>
    </Party>
    <Party>
      <PartyReference>PLabel</PartyReference>
      <PartyName><FullName>Playground Records</FullName></PartyName>
    </Party>
  </PartyList>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>A1</ResourceReference>
      <Type>MusicalWorkSoundRecording</Type>
      <SoundRecordingEdition>
        <ResourceId><ISRC>USPG12600001</ISRC></ResourceId>
        <PLine>
          <Year>2026</Year>
          <PLineText>(P) 2026 Playground Records</PLineText>
        </PLine>
      </SoundRecordingEdition>
      <DisplayTitleText>Midnight Signals</DisplayTitleText>
      <DisplayTitle><TitleText>Midnight Signals</TitleText></DisplayTitle>
      <DisplayArtistName>The Example Band</DisplayArtistName>
      <DisplayArtist>
        <ArtistPartyReference>PArtist</ArtistPartyReference>
        <DisplayArtistRole>MainArtist</DisplayArtistRole>
      </DisplayArtist>
      <Duration>PT3M42S</Duration>
      <ParentalWarningType>NotExplicit</ParentalWarningType>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseReference>R0</ReleaseReference>
      <ReleaseType>Single</ReleaseType>
      <ReleaseId><ICPN>123456789012</ICPN></ReleaseId>
      <DisplayTitleText>Midnight Signals</DisplayTitleText>
      <DisplayTitle><TitleText>Midnight Signals</TitleText></DisplayTitle>
      <DisplayArtistName>The Example Band</DisplayArtistName>
      <DisplayArtist>
        <ArtistPartyReference>PArtist</ArtistPartyReference>
        <DisplayArtistRole>MainArtist</DisplayArtistRole>
      </DisplayArtist>
      <ReleaseLabelReference>PLabel</ReleaseLabelReference>
      <Genre><GenreText>Electronic</GenreText></Genre>
      <ReleaseDate>2026-03-01</ReleaseDate>
      <ParentalWarningType>NotExplicit</ParentalWarningType>
      <ResourceGroup>
        <ResourceGroupContentItem>
          <SequenceNumber>1</SequenceNumber>
          <ReleaseResourceReference>A1</ReleaseResourceReference>
        </ResourceGroupContentItem>
      </ResourceGroup>
    </Release>
  </ReleaseList>
  <DealList>
    <ReleaseDeal>
      <DealReleaseReference>R0</DealReleaseReference>
      <Deal>
        <DealTerms>
          <TerritoryCode>Worldwide</TerritoryCode>
          <ValidityPeriod><StartDate>2026-03-01</StartDate></ValidityPeriod>
          <CommercialModelType>SubscriptionModel</CommercialModelType>
          <UseType>OnDemandStream</UseType>
        </DealTerms>
      </Deal>
    </ReleaseDeal>
  </DealList>
</ern:NewReleaseMessage>`,

  'ERN 4.3 album (2 tracks)': `<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" LanguageAndScriptCode="en" AvsVersionId="12">
  <MessageHeader>
    <MessageThreadId>PLAYGROUND-THREAD-2</MessageThreadId>
    <MessageId>PLAYGROUND-MSG-2</MessageId>
    <MessageSender>
      <PartyId>PADPIDA0000000001</PartyId>
      <PartyName><FullName>Playground Records</FullName></PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId>PADPIDA0000000002</PartyId>
      <PartyName><FullName>Playground Streaming</FullName></PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2026-01-15T10:00:00Z</MessageCreatedDateTime>
    <MessageControlType>TestMessage</MessageControlType>
  </MessageHeader>
  <PartyList>
    <Party>
      <PartyReference>PArtist</PartyReference>
      <PartyName><FullName>The Example Band</FullName></PartyName>
    </Party>
    <Party>
      <PartyReference>PLabel</PartyReference>
      <PartyName><FullName>Playground Records</FullName></PartyName>
    </Party>
  </PartyList>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>A1</ResourceReference>
      <Type>MusicalWorkSoundRecording</Type>
      <SoundRecordingEdition>
        <ResourceId><ISRC>USPG12600011</ISRC></ResourceId>
        <PLine>
          <Year>2026</Year>
          <PLineText>(P) 2026 Playground Records</PLineText>
        </PLine>
      </SoundRecordingEdition>
      <DisplayTitleText>Opening Lights</DisplayTitleText>
      <DisplayTitle><TitleText>Opening Lights</TitleText></DisplayTitle>
      <DisplayArtistName>The Example Band</DisplayArtistName>
      <DisplayArtist>
        <ArtistPartyReference>PArtist</ArtistPartyReference>
        <DisplayArtistRole>MainArtist</DisplayArtistRole>
      </DisplayArtist>
      <Duration>PT4M05S</Duration>
      <ParentalWarningType>NotExplicit</ParentalWarningType>
    </SoundRecording>
    <SoundRecording>
      <ResourceReference>A2</ResourceReference>
      <Type>MusicalWorkSoundRecording</Type>
      <SoundRecordingEdition>
        <ResourceId><ISRC>USPG12600012</ISRC></ResourceId>
        <PLine>
          <Year>2026</Year>
          <PLineText>(P) 2026 Playground Records</PLineText>
        </PLine>
      </SoundRecordingEdition>
      <DisplayTitleText>Northbound</DisplayTitleText>
      <DisplayTitle><TitleText>Northbound</TitleText></DisplayTitle>
      <DisplayArtistName>The Example Band</DisplayArtistName>
      <DisplayArtist>
        <ArtistPartyReference>PArtist</ArtistPartyReference>
        <DisplayArtistRole>MainArtist</DisplayArtistRole>
      </DisplayArtist>
      <Duration>PT3M18S</Duration>
      <ParentalWarningType>NotExplicit</ParentalWarningType>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseReference>R0</ReleaseReference>
      <ReleaseType>Album</ReleaseType>
      <ReleaseId><ICPN>123456789029</ICPN></ReleaseId>
      <DisplayTitleText>Northern Lights</DisplayTitleText>
      <DisplayTitle><TitleText>Northern Lights</TitleText></DisplayTitle>
      <DisplayArtistName>The Example Band</DisplayArtistName>
      <DisplayArtist>
        <ArtistPartyReference>PArtist</ArtistPartyReference>
        <DisplayArtistRole>MainArtist</DisplayArtistRole>
      </DisplayArtist>
      <ReleaseLabelReference>PLabel</ReleaseLabelReference>
      <Genre><GenreText>Electronic</GenreText></Genre>
      <ReleaseDate>2026-03-01</ReleaseDate>
      <ParentalWarningType>NotExplicit</ParentalWarningType>
      <ResourceGroup>
        <ResourceGroupContentItem>
          <SequenceNumber>1</SequenceNumber>
          <ReleaseResourceReference>A1</ReleaseResourceReference>
        </ResourceGroupContentItem>
        <ResourceGroupContentItem>
          <SequenceNumber>2</SequenceNumber>
          <ReleaseResourceReference>A2</ReleaseResourceReference>
        </ResourceGroupContentItem>
      </ResourceGroup>
    </Release>
  </ReleaseList>
  <DealList>
    <ReleaseDeal>
      <DealReleaseReference>R0</DealReleaseReference>
      <Deal>
        <DealTerms>
          <TerritoryCode>Worldwide</TerritoryCode>
          <ValidityPeriod><StartDate>2026-03-01</StartDate></ValidityPeriod>
          <CommercialModelType>SubscriptionModel</CommercialModelType>
          <UseType>OnDemandStream</UseType>
        </DealTerms>
      </Deal>
    </ReleaseDeal>
  </DealList>
</ern:NewReleaseMessage>`,
};

/** A complete ddex-builder 0.5.1 BuildRequest for ERN 4.3 (snake_case field names). */
export const BUILD_SAMPLES: Record<string, string> = {
  'ERN 4.3 single build request': `{
  "version": "4.3",
  "language_and_script_code": "en",
  "avs_version_id": "12",
  "header": {
    "message_id": "PLAYGROUND-0001",
    "message_created_date_time": "2026-01-15T10:00:00Z",
    "message_sender": {
      "party_id": "PADPIDA0000000001",
      "party_name": [
        {
          "text": "Playground Records"
        }
      ]
    },
    "message_recipient": {
      "party_id": "PADPIDA0000000002",
      "party_name": [
        {
          "text": "Playground Streaming"
        }
      ]
    },
    "message_control_type": "TestMessage"
  },
  "releases": [
    {
      "release_id": "A1-12345-0000000001-M",
      "release_reference": "R0",
      "title": [
        {
          "text": "Midnight Signals"
        }
      ],
      "artist": "The Example Band",
      "label": "Playground Records",
      "upc": "123456789012",
      "release_type": "Single",
      "genre": "Electronic",
      "parental_warning_type": "NotExplicit",
      "release_date": "2026-03-01",
      "tracks": [
        {
          "track_id": "T1",
          "resource_reference": "A1",
          "isrc": "USPG12600001",
          "title": "Midnight Signals",
          "duration": "PT3M42S",
          "artist": "The Example Band",
          "resource_type": "MusicalWorkSoundRecording",
          "parental_warning_type": "NotExplicit"
        }
      ]
    }
  ],
  "deals": [
    {
      "deal_reference": "D1",
      "deal_terms": {
        "commercial_model_type": "SubscriptionModel",
        "territory_code": [
          "Worldwide"
        ],
        "start_date": "2026-03-01",
        "use_types": [
          "OnDemandStream"
        ]
      },
      "release_references": [
        "R0"
      ]
    }
  ]
}`,
};
