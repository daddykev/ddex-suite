//! Test fixtures for integration testing

use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Creates a minimal DDEX ERN 4.3 XML document for testing
pub fn create_minimal_ern43_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" 
                       xsi:schemaLocation="http://ddex.net/xml/ern/43 http://ddex.net/xml/ern/43/release-notification.xsd"
                       MessageSchemaVersionId="ern/43" BusinessTransactionId="1234567890" 
                       ReleaseProfileVersionId="SimpleAudioSingle">
    <MessageHeader>
        <MessageId>CLI_TEST_001</MessageId>
        <MessageSender>
            <PartyId Namespace="PADPIDA2006120701T">PADPIDA2006120701T::TEST_SENDER</PartyId>
            <PartyName>
                <FullName>Test Sender</FullName>
            </PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyId Namespace="PADPIDA2006120701T">PADPIDA2006120701T::TEST_RECIPIENT</PartyId>
            <PartyName>
                <FullName>Test Recipient</FullName>
            </PartyName>
        </MessageRecipient>
        <MessageCreatedDateTime>2024-01-15T10:00:00Z</MessageCreatedDateTime>
    </MessageHeader>
    <ReleaseList>
        <Release>
            <ReleaseId>
                <ICPN IsEan="true">1234567890123</ICPN>
            </ReleaseId>
            <ReleaseReference>REL123456</ReleaseReference>
            <ReferenceTitle>
                <TitleText>Test Single</TitleText>
            </ReferenceTitle>
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>SoundRecording_1</ReleaseResourceReference>
            </ReleaseResourceReferenceList>
            <ReleaseDeal>
                <Deal>
                    <DealTerms>
                        <CommercialModelType>FreeOfChargeModel</CommercialModelType>
                        <TerritoryCode>Worldwide</TerritoryCode>
                        <ValidityPeriod>
                            <StartDate>2024-01-15</StartDate>
                        </ValidityPeriod>
                    </DealTerms>
                </Deal>
            </ReleaseDeal>
        </Release>
    </ReleaseList>
    <ResourceList>
        <SoundRecording>
            <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
            <ResourceReference>SoundRecording_1</ResourceReference>
            <ReferenceTitle>
                <TitleText>Test Track</TitleText>
            </ReferenceTitle>
            <Duration>PT3M30S</Duration>
            <DisplayArtist>
                <PartyName>
                    <FullName>Test Artist</FullName>
                </PartyName>
            </DisplayArtist>
            <SoundRecordingDetailsByTerritory>
                <TerritoryCode>Worldwide</TerritoryCode>
                <Title>
                    <TitleText>Test Track</TitleText>
                </Title>
                <DisplayArtist>
                    <PartyName>
                        <FullName>Test Artist</FullName>
                    </PartyName>
                </DisplayArtist>
            </SoundRecordingDetailsByTerritory>
        </SoundRecording>
    </ResourceList>
</ern:NewReleaseMessage>"#.to_string()
}

/// Creates a more complex DDEX ERN 4.3 XML document for comprehensive testing
pub fn create_complex_ern43_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" 
                       xsi:schemaLocation="http://ddex.net/xml/ern/43 http://ddex.net/xml/ern/43/release-notification.xsd"
                       MessageSchemaVersionId="ern/43" BusinessTransactionId="1234567890" 
                       ReleaseProfileVersionId="AudioAlbumMusicOnly">
    <MessageHeader>
        <MessageId>INTEGRATION_TEST_002</MessageId>
        <MessageSender>
            <PartyId Namespace="PADPIDA2006120701T">PADPIDA2006120701T::INTEGRATION_SENDER</PartyId>
            <PartyName>
                <FullName>Integration Test Sender</FullName>
            </PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyId Namespace="PADPIDA2006120701T">PADPIDA2006120701T::INTEGRATION_RECIPIENT</PartyId>
            <PartyName>
                <FullName>Integration Test Recipient</FullName>
            </PartyName>
        </MessageRecipient>
        <MessageCreatedDateTime>2024-09-09T15:30:00Z</MessageCreatedDateTime>
    </MessageHeader>
    <ReleaseList>
        <Release>
            <ReleaseId>
                <ICPN IsEan="true">1234567890123</ICPN>
                <CatalogNumber Namespace="urn:company:test">TEST001</CatalogNumber>
            </ReleaseId>
            <ReleaseReference>REL_ALBUM_001</ReleaseReference>
            <ReferenceTitle>
                <TitleText>Test Album</TitleText>
            </ReferenceTitle>
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>SoundRecording_1</ReleaseResourceReference>
                <ReleaseResourceReference>SoundRecording_2</ReleaseResourceReference>
            </ReleaseResourceReferenceList>
            <ReleaseType>Album</ReleaseType>
            <ReleaseDetailsByTerritory>
                <TerritoryCode>Worldwide</TerritoryCode>
                <Title>
                    <TitleText>Test Album</TitleText>
                </Title>
                <DisplayArtist>
                    <PartyName>
                        <FullName>Test Artist Collective</FullName>
                    </PartyName>
                </DisplayArtist>
                <LabelName>Test Records</LabelName>
                <ReleaseDate>2024-01-15</ReleaseDate>
                <OriginalReleaseDate>2024-01-15</OriginalReleaseDate>
                <Genre>
                    <GenreText>Electronic</GenreText>
                </Genre>
                <PLine>
                    <Year>2024</Year>
                    <PLineText>Test Records</PLineText>
                </PLine>
                <CLine>
                    <Year>2024</Year>
                    <CLineText>Test Records</CLineText>
                </CLine>
            </ReleaseDetailsByTerritory>
            <ReleaseDeal>
                <Deal>
                    <DealTerms>
                        <CommercialModelType>SubscriptionModel</CommercialModelType>
                        <TerritoryCode>Worldwide</TerritoryCode>
                        <ValidityPeriod>
                            <StartDate>2024-01-15</StartDate>
                        </ValidityPeriod>
                        <PriceInformation>
                            <WholesalePricePerUnit CurrencyCode="USD">9.99</WholesalePricePerUnit>
                        </PriceInformation>
                    </DealTerms>
                </Deal>
            </ReleaseDeal>
        </Release>
    </ReleaseList>
    <ResourceList>
        <SoundRecording>
            <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
            <ResourceReference>SoundRecording_1</ResourceReference>
            <ReferenceTitle>
                <TitleText>Test Track One</TitleText>
            </ReferenceTitle>
            <Duration>PT3M30S</Duration>
            <CreationDate>2024-01-01</CreationDate>
            <DisplayArtist>
                <PartyName>
                    <FullName>Test Artist</FullName>
                </PartyName>
            </DisplayArtist>
            <SoundRecordingDetailsByTerritory>
                <TerritoryCode>Worldwide</TerritoryCode>
                <Title>
                    <TitleText>Test Track One</TitleText>
                </Title>
                <DisplayArtist>
                    <PartyName>
                        <FullName>Test Artist</FullName>
                    </PartyName>
                </DisplayArtist>
                <Genre>
                    <GenreText>Electronic Dance</GenreText>
                </Genre>
                <PLine>
                    <Year>2024</Year>
                    <PLineText>Test Records</PLineText>
                </PLine>
            </SoundRecordingDetailsByTerritory>
        </SoundRecording>
        <SoundRecording>
            <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
            <ResourceReference>SoundRecording_2</ResourceReference>
            <ReferenceTitle>
                <TitleText>Test Track Two</TitleText>
            </ReferenceTitle>
            <Duration>PT4M15S</Duration>
            <CreationDate>2024-01-02</CreationDate>
            <DisplayArtist>
                <PartyName>
                    <FullName>Test Artist</FullName>
                </PartyName>
            </DisplayArtist>
            <SoundRecordingDetailsByTerritory>
                <TerritoryCode>Worldwide</TerritoryCode>
                <Title>
                    <TitleText>Test Track Two</TitleText>
                </Title>
                <DisplayArtist>
                    <PartyName>
                        <FullName>Test Artist</FullName>
                    </PartyName>
                </DisplayArtist>
                <Genre>
                    <GenreText>Electronic Ambient</GenreText>
                </Genre>
                <PLine>
                    <Year>2024</Year>
                    <PLineText>Test Records</PLineText>
                </PLine>
            </SoundRecordingDetailsByTerritory>
        </SoundRecording>
    </ResourceList>
</ern:NewReleaseMessage>"#.to_string()
}

/// Fixture manager for creating test data files
pub struct FixtureManager {
    temp_dir: TempDir,
}

impl FixtureManager {
    /// Create a new fixture manager with temporary directory
    pub fn new() -> Result<Self, std::io::Error> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    /// Get the path to the temporary directory
    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a fixture file with the given content
    pub fn create_fixture_file(&self, filename: &str, content: &str) -> Result<std::path::PathBuf, std::io::Error> {
        let file_path = self.temp_path().join(filename);
        fs::write(&file_path, content)?;
        Ok(file_path)
    }

    /// Create all standard test fixtures
    pub fn create_standard_fixtures(&self) -> Result<StandardFixtures, std::io::Error> {
        let minimal_path = self.create_fixture_file("minimal_ern43.xml", &create_minimal_ern43_xml())?;
        let complex_path = self.create_fixture_file("complex_ern43.xml", &create_complex_ern43_xml())?;
        
        Ok(StandardFixtures {
            minimal_ern43: minimal_path,
            complex_ern43: complex_path,
        })
    }
}

/// Standard fixture paths
pub struct StandardFixtures {
    pub minimal_ern43: std::path::PathBuf,
    pub complex_ern43: std::path::PathBuf,
}

impl StandardFixtures {
    /// Get all fixture paths as a vector
    pub fn all_paths(&self) -> Vec<&std::path::PathBuf> {
        vec![&self.minimal_ern43, &self.complex_ern43]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creation() {
        let manager = FixtureManager::new().unwrap();
        let fixtures = manager.create_standard_fixtures().unwrap();
        
        // Verify files were created
        assert!(fixtures.minimal_ern43.exists());
        assert!(fixtures.complex_ern43.exists());
        
        // Verify content
        let minimal_content = fs::read_to_string(&fixtures.minimal_ern43).unwrap();
        assert!(minimal_content.contains("CLI_TEST_001"));
        assert!(minimal_content.contains("ern:NewReleaseMessage"));
        
        let complex_content = fs::read_to_string(&fixtures.complex_ern43).unwrap();
        assert!(complex_content.contains("INTEGRATION_TEST_002"));
        assert!(complex_content.contains("Test Album"));
    }

    #[test]
    fn test_xml_structure() {
        let minimal_xml = create_minimal_ern43_xml();
        assert!(minimal_xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(minimal_xml.contains("xmlns:ern=\"http://ddex.net/xml/ern/43\""));
        assert!(minimal_xml.contains("MessageSchemaVersionId=\"ern/43\""));
        assert!(minimal_xml.contains("<MessageHeader>"));
        assert!(minimal_xml.contains("<ReleaseList>"));
        assert!(minimal_xml.contains("<ResourceList>"));
    }
}