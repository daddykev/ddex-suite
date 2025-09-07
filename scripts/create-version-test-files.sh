#!/bin/bash
# scripts/create-version-test-files.sh

# Create directories for version-specific test files
mkdir -p test-suite/valid/ern-382
mkdir -p test-suite/valid/ern-42
mkdir -p test-suite/valid/ern-43
mkdir -p test-suite/vendor-quirks/missing-thread-id
mkdir -p test-suite/vendor-quirks/empty-audit-trail
mkdir -p test-suite/vendor-quirks/mixed-namespaces

echo "Creating ERN version test files..."

# ERN 3.8.2 test file
cat > test-suite/valid/ern-382/basic_release.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/382" 
                       xmlns:xs="http://www.w3.org/2001/XMLSchema-instance"
                       MessageSchemaVersionId="ern/382">
  <MessageHeader>
    <MessageThreadId>Thread_382_001</MessageThreadId>
    <MessageId>MSG_382_001</MessageId>
    <MessageSender>
      <PartyName>Test Label 382</PartyName>
      <PartyId>LABEL382</PartyId>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP</PartyName>
      <PartyId>DSP001</PartyId>
    </MessageRecipient>
    <MessageCreatedDateTime>2025-01-01T12:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>
        <GRid>A1-B2C3D-4E5F6G-H</GRid>
        <ProprietaryId Namespace="LABEL382">REL001</ProprietaryId>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Test Album 3.8.2</TitleText>
      </ReferenceTitle>
      <ReleaseType>Album</ReleaseType>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>
EOF

# ERN 4.2 test file
cat > test-suite/valid/ern-42/basic_release.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42"
                       MessageSchemaVersionId="ern/42">
  <MessageHeader>
    <MessageThreadId>Thread_42_001</MessageThreadId>
    <MessageId>MSG_42_001</MessageId>
    <MessageSender>
      <PartyName>
        <FullName>Test Label 4.2</FullName>
      </PartyName>
      <PartyId>
        <ProprietaryId Namespace="DPID">LABEL42</ProprietaryId>
      </PartyId>
    </MessageSender>
    <MessageRecipient>
      <PartyName>
        <FullName>Test DSP</FullName>
      </PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2025-01-01T12:00:00Z</MessageCreatedDateTime>
    <MessageAuditTrail>
      <MessageAuditTrailEvent>
        <MessageAuditTrailEventType>MessageCreated</MessageAuditTrailEventType>
        <DateTime>2025-01-01T12:00:00Z</DateTime>
      </MessageAuditTrailEvent>
    </MessageAuditTrail>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseId>
        <GRid>A1-B2C3D-4E5F6G-H</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Test Album 4.2</TitleText>
      </ReferenceTitle>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>
EOF

# ERN 4.3 test file with new features
cat > test-suite/valid/ern-43/advanced_release.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43"
                       MessageSchemaVersionId="ern/43"
                       ReleaseProfileVersionId="AudioAlbumMusicOnly">
  <MessageHeader>
    <MessageThreadId>Thread_43_001</MessageThreadId>
    <MessageId>MSG_43_001</MessageId>
    <MessageSender>
      <PartyName>
        <FullName>Test Label 4.3</FullName>
      </PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>
        <FullName>Test DSP</FullName>
      </PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2025-01-01T12:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseId>
        <GRid>A1-B2C3D-4E5F6G-H</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Test Album 4.3</TitleText>
      </ReferenceTitle>
      <ReleaseResourceReferenceList>
        <ReleaseResourceReference>
          <ResourceReference>A1</ResourceReference>
          <ResourceGroup>RG1</ResourceGroup>
        </ReleaseResourceReference>
      </ReleaseResourceReferenceList>
    </Release>
  </ReleaseList>
  <ResourceGroupList>
    <ResourceGroup>
      <ResourceGroupReference>RG1</ResourceGroupReference>
      <ResourceGroupType>MainRelease</ResourceGroupType>
    </ResourceGroup>
  </ResourceGroupList>
  <DealList>
    <ReleaseDeal>
      <DealReference>D1</DealReference>
      <DealReleaseReference>R1</DealReleaseReference>
      <Deal>
        <DealTerms>
          <CommercialModelType>SubscriptionModel</CommercialModelType>
          <PreOrderDate>2025-01-01</PreOrderDate>
          <PreOrderPreviewDate>2024-12-25</PreOrderPreviewDate>
        </DealTerms>
      </Deal>
    </ReleaseDeal>
  </DealList>
</ern:NewReleaseMessage>
EOF

echo "✓ Created version-specific test files"