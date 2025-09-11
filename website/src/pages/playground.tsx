import React, { useState, useCallback } from 'react';
import Layout from '@theme/Layout';
import BrowserOnly from '@docusaurus/BrowserOnly';
import { Allotment } from 'allotment';
import Editor from '@monaco-editor/react';
import 'allotment/dist/style.css';

// Sample DDEX XML files for testing
const SAMPLE_FILES = {
  'ERN 4.3 Simple': `<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" BusinessProfileVersionId="CommonReleaseProfile/14" ReleaseProfileVersionId="CommonReleaseProfile/14">
  <MessageHeader>
    <MessageThreadId>MSG001</MessageThreadId>
    <MessageId>MSG001_001</MessageId>
    <MessageCreatedDateTime>2024-01-15T10:00:00Z</MessageCreatedDateTime>
    <MessageSender>
      <PartyId Namespace="UserDefined">LABEL001</PartyId>
      <PartyName>
        <FullName>Sample Record Label</FullName>
      </PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId Namespace="UserDefined">DSP001</PartyId>
      <PartyName>
        <FullName>Sample DSP</FullName>
      </PartyName>
    </MessageRecipient>
  </MessageHeader>
  <PartyList>
    <Party>
      <PartyReference>P1</PartyReference>
      <PartyId Namespace="UserDefined">LABEL001</PartyId>
      <PartyName>
        <FullName>Sample Record Label</FullName>
      </PartyName>
    </Party>
    <Party>
      <PartyReference>P2</PartyReference>
      <PartyId Namespace="UserDefined">ARTIST001</PartyId>
      <PartyName>
        <FullName>Sample Artist</FullName>
      </PartyName>
    </Party>
  </PartyList>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>A1</ResourceReference>
      <Type>MusicalWorkSoundRecording</Type>
      <Title>
        <TitleText>Sample Track</TitleText>
      </Title>
      <DisplayArtist>
        <PartyName>
          <FullName>Sample Artist</FullName>
        </PartyName>
        <PartyReference>P2</PartyReference>
      </DisplayArtist>
      <SoundRecordingId>
        <ISRC>US-S1Z-99-00001</ISRC>
      </SoundRecordingId>
      <Duration>PT3M45S</Duration>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseType>Album</ReleaseType>
      <Title>
        <TitleText>Sample Album</TitleText>
      </Title>
      <DisplayArtist>
        <PartyName>
          <FullName>Sample Artist</FullName>
        </PartyName>
        <PartyReference>P2</PartyReference>
      </DisplayArtist>
      <ReleaseId>
        <ICPN>1234567890123</ICPN>
      </ReleaseId>
      <ReleaseResourceReferenceList>
        <ReleaseResourceReference>A1</ReleaseResourceReference>
      </ReleaseResourceReferenceList>
    </Release>
  </ReleaseList>
  <DealList>
    <ReleaseDeal>
      <DealReference>D1</DealReference>
      <DealTerms>
        <CommercialModelType>SubscriptionAndPurchase</CommercialModelType>
        <UseType>Stream</UseType>
        <UseType>PermanentDownload</UseType>
        <TerritoryCode>Worldwide</TerritoryCode>
        <ValidityPeriod>
          <StartDate>2024-01-15</StartDate>
        </ValidityPeriod>
      </DealTerms>
      <DealReleaseReference>R1</DealReleaseReference>
    </ReleaseDeal>
  </DealList>
</NewReleaseMessage>`,
  
  'ERN 4.2 Example': `<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/42" MessageSchemaVersionId="ern/42" BusinessProfileVersionId="CommonReleaseProfile/13" ReleaseProfileVersionId="CommonReleaseProfile/13">
  <MessageHeader>
    <MessageThreadId>MSG002</MessageThreadId>
    <MessageId>MSG002_001</MessageId>
    <MessageCreatedDateTime>2024-01-15T11:00:00Z</MessageCreatedDateTime>
    <MessageSender>
      <PartyId Namespace="UserDefined">INDIE_LABEL</PartyId>
      <PartyName>
        <FullName>Indie Music Label</FullName>
      </PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId Namespace="UserDefined">STREAMING_SERVICE</PartyId>
      <PartyName>
        <FullName>Streaming Platform</FullName>
      </PartyName>
    </MessageRecipient>
  </MessageHeader>
  <PartyList>
    <Party>
      <PartyReference>P1</PartyReference>
      <PartyId Namespace="UserDefined">INDIE_LABEL</PartyId>
      <PartyName>
        <FullName>Indie Music Label</FullName>
      </PartyName>
    </Party>
    <Party>
      <PartyReference>P2</PartyReference>
      <PartyId Namespace="UserDefined">INDIE_ARTIST</PartyId>
      <PartyName>
        <FullName>The Indie Band</FullName>
      </PartyName>
    </Party>
  </PartyList>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>A1</ResourceReference>
      <Type>MusicalWorkSoundRecording</Type>
      <Title>
        <TitleText>Indie Rock Anthem</TitleText>
      </Title>
      <DisplayArtist>
        <PartyName>
          <FullName>The Indie Band</FullName>
        </PartyName>
        <PartyReference>P2</PartyReference>
      </DisplayArtist>
      <SoundRecordingId>
        <ISRC>US-IND-24-00001</ISRC>
      </SoundRecordingId>
      <Duration>PT4M12S</Duration>
      <Genre>
        <GenreText>Indie Rock</GenreText>
      </Genre>
    </SoundRecording>
    <SoundRecording>
      <ResourceReference>A2</ResourceReference>
      <Type>MusicalWorkSoundRecording</Type>
      <Title>
        <TitleText>Alternative Dreams</TitleText>
      </Title>
      <DisplayArtist>
        <PartyName>
          <FullName>The Indie Band</FullName>
        </PartyName>
        <PartyReference>P2</PartyReference>
      </DisplayArtist>
      <SoundRecordingId>
        <ISRC>US-IND-24-00002</ISRC>
      </SoundRecordingId>
      <Duration>PT3M58S</Duration>
      <Genre>
        <GenreText>Alternative</GenreText>
      </Genre>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseReference>R1</ReleaseReference>
      <ReleaseType>Single</ReleaseType>
      <Title>
        <TitleText>Indie Rock Single</TitleText>
      </Title>
      <DisplayArtist>
        <PartyName>
          <FullName>The Indie Band</FullName>
        </PartyName>
        <PartyReference>P2</PartyReference>
      </DisplayArtist>
      <ReleaseId>
        <ICPN>1234567890124</ICPN>
      </ReleaseId>
      <ReleaseResourceReferenceList>
        <ReleaseResourceReference>A1</ReleaseResourceReference>
        <ReleaseResourceReference>A2</ReleaseResourceReference>
      </ReleaseResourceReferenceList>
      <Genre>
        <GenreText>Indie Rock</GenreText>
      </Genre>
    </Release>
  </ReleaseList>
  <DealList>
    <ReleaseDeal>
      <DealReference>D1</DealReference>
      <DealTerms>
        <CommercialModelType>Subscription</CommercialModelType>
        <UseType>Stream</UseType>
        <TerritoryCode>US</TerritoryCode>
        <TerritoryCode>CA</TerritoryCode>
        <TerritoryCode>GB</TerritoryCode>
        <ValidityPeriod>
          <StartDate>2024-02-01</StartDate>
        </ValidityPeriod>
      </DealTerms>
      <DealReleaseReference>R1</DealReleaseReference>
    </ReleaseDeal>
  </DealList>
</NewReleaseMessage>`,

  'Builder Template': JSON.stringify({
    messageHeader: {
      messageId: "MSG_BUILD_001",
      messageSenderName: "My Record Label",
      messageRecipientName: "Streaming Platform",
      messageCreatedDateTime: new Date().toISOString()
    },
    releases: [{
      releaseId: "REL_001",
      title: "My New Album",
      artist: "Amazing Artist",
      releaseType: "Album",
      label: "My Record Label",
      upc: "123456789012",
      releaseDate: "2024-03-01",
      territories: ["US", "CA", "GB"],
      genres: ["Pop", "Electronic"],
      trackIds: ["TR_001", "TR_002"]
    }],
    resources: [{
      resourceId: "TR_001",
      resourceType: "SoundRecording",
      title: "Hit Single",
      artist: "Amazing Artist",
      isrc: "US-AWE-24-00001",
      duration: "PT3M30S",
      trackNumber: 1
    }, {
      resourceId: "TR_002", 
      resourceType: "SoundRecording",
      title: "Another Track",
      artist: "Amazing Artist",
      isrc: "US-AWE-24-00002",
      duration: "PT4M15S",
      trackNumber: 2
    }],
    deals: [{
      dealId: "DEAL_001",
      releaseId: "REL_001",
      territories: ["US", "CA", "GB"],
      useTypes: ["Stream", "PermanentDownload"],
      commercialModelType: "Subscription",
      dealStartDate: "2024-03-01"
    }]
  }, null, 2)
};

interface PlaygroundState {
  mode: 'parser' | 'builder';
  input: string;
  output: string;
  loading: boolean;
  error: string;
}

function PlaygroundComponent() {
  const [state, setState] = useState<PlaygroundState>({
    mode: 'parser',
    input: SAMPLE_FILES['ERN 4.3 Simple'],
    output: '',
    loading: false,
    error: ''
  });

  // Mock DDEX processing functions (would use actual WASM builds in production)
  const parseXML = useCallback(async (_xml: string) => {
    // Simulate parsing delay
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Mock parser output
    return {
      messageId: "MSG001_001",
      version: "4.3",
      flat: {
        releases: [{
          title: "Sample Album",
          artist: "Sample Artist",
          upc: "1234567890123",
          releaseDate: "2024-01-15",
          territories: ["Worldwide"],
          tracks: [{
            title: "Sample Track",
            isrc: "US-S1Z-99-00001",
            duration: "PT3M45S"
          }]
        }]
      },
      graph: {
        messageHeader: {
          messageId: "MSG001_001",
          messageCreatedDateTime: "2024-01-15T10:00:00Z"
        },
        parties: [
          { partyReference: "P1", partyName: "Sample Record Label" },
          { partyReference: "P2", partyName: "Sample Artist" }
        ],
        resources: [{
          resourceReference: "A1",
          title: "Sample Track",
          displayArtist: "Sample Artist",
          isrc: "US-S1Z-99-00001",
          duration: "PT3M45S"
        }],
        releases: [{
          releaseReference: "R1",
          title: "Sample Album",
          displayArtist: "Sample Artist",
          upc: "1234567890123"
        }]
      }
    };
  }, []);

  const buildXML = useCallback(async (json: string) => {
    // Simulate building delay
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Mock builder output (would use actual builder in production)
    const data = JSON.parse(json);
    const messageId = data.messageHeader?.messageId || 'MSG_GENERATED';
    const releaseTitle = data.releases?.[0]?.title || 'Generated Release';
    
    return `<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" BusinessProfileVersionId="CommonReleaseProfile/14" ReleaseProfileVersionId="CommonReleaseProfile/14">
  <MessageHeader>
    <MessageThreadId>${messageId}</MessageThreadId>
    <MessageId>${messageId}</MessageId>
    <MessageCreatedDateTime>${new Date().toISOString()}</MessageCreatedDateTime>
    <MessageSender>
      <PartyId Namespace="UserDefined">GENERATED</PartyId>
      <PartyName>
        <FullName>${data.messageHeader?.messageSenderName || 'Generated Sender'}</FullName>
      </PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId Namespace="UserDefined">RECIPIENT</PartyId>
      <PartyName>
        <FullName>${data.messageHeader?.messageRecipientName || 'Generated Recipient'}</FullName>
      </PartyName>
    </MessageRecipient>
  </MessageHeader>
  <!-- Generated DDEX XML from playground -->
  <!-- Release: ${releaseTitle} -->
  <!-- This is a mock output. In production, this would be generated by the actual DDEX Builder. -->
</NewReleaseMessage>`;
  }, []);

  const handleProcess = useCallback(async () => {
    if (!state.input.trim()) {
      setState(prev => ({ ...prev, error: 'Please provide input data', output: '' }));
      return;
    }

    setState(prev => ({ ...prev, loading: true, error: '', output: '' }));

    try {
      let result: string;
      
      if (state.mode === 'parser') {
        const parsed = await parseXML(state.input);
        result = JSON.stringify(parsed, null, 2);
      } else {
        result = await buildXML(state.input);
      }
      
      setState(prev => ({ ...prev, output: result, loading: false }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      setState(prev => ({ 
        ...prev, 
        loading: false, 
        error: errorMessage,
        output: `Error: ${errorMessage}`
      }));
    }
  }, [state.input, state.mode, parseXML, buildXML]);

  const handleModeChange = useCallback((newMode: 'parser' | 'builder') => {
    const defaultInput = newMode === 'parser' 
      ? SAMPLE_FILES['ERN 4.3 Simple']
      : SAMPLE_FILES['Builder Template'];
    
    setState(prev => ({
      ...prev,
      mode: newMode,
      input: defaultInput,
      output: '',
      error: ''
    }));
  }, []);

  const loadSample = useCallback((sampleName: keyof typeof SAMPLE_FILES) => {
    setState(prev => ({
      ...prev,
      input: SAMPLE_FILES[sampleName],
      output: '',
      error: ''
    }));
  }, []);

  const exportOutput = useCallback(() => {
    if (!state.output) return;
    
    const blob = new Blob([state.output], { 
      type: state.mode === 'parser' ? 'application/json' : 'application/xml' 
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = state.mode === 'parser' ? 'parsed-data.json' : 'generated.xml';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, [state.output, state.mode]);

  return (
    <div style={{ height: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <div style={{ 
        padding: '1rem', 
        borderBottom: '1px solid var(--ifm-color-emphasis-200)',
        backgroundColor: 'var(--ifm-background-color)'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1rem' }}>
          <h1 style={{ margin: 0, fontSize: '1.5rem' }}>DDEX Suite Playground</h1>
          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button
              className={`button ${state.mode === 'parser' ? 'button--primary' : 'button--secondary'}`}
              onClick={() => handleModeChange('parser')}
            >
              Parser Mode
            </button>
            <button
              className={`button ${state.mode === 'builder' ? 'button--primary' : 'button--secondary'}`}
              onClick={() => handleModeChange('builder')}
            >
              Builder Mode
            </button>
          </div>
        </div>
        
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', flexWrap: 'wrap' }}>
          <div>
            <label style={{ marginRight: '0.5rem', fontSize: '0.9rem' }}>Load Sample:</label>
            <select onChange={(e) => loadSample(e.target.value as keyof typeof SAMPLE_FILES)}>
              <option value="">Choose a sample...</option>
              {Object.keys(SAMPLE_FILES).map(name => (
                <option key={name} value={name}>{name}</option>
              ))}
            </select>
          </div>
          
          <button
            className="button button--primary"
            onClick={handleProcess}
            disabled={state.loading}
          >
            {state.loading ? 'Processing...' : (state.mode === 'parser' ? 'Parse XML' : 'Build XML')}
          </button>
          
          {state.output && (
            <button
              className="button button--secondary"
              onClick={exportOutput}
            >
              Export {state.mode === 'parser' ? 'JSON' : 'XML'}
            </button>
          )}
        </div>

        {state.error && (
          <div style={{ 
            marginTop: '1rem', 
            padding: '0.5rem', 
            backgroundColor: 'var(--ifm-color-danger-contrast-background)',
            color: 'var(--ifm-color-danger)',
            borderRadius: '4px',
            fontSize: '0.9rem'
          }}>
            {state.error}
          </div>
        )}
      </div>

      {/* Main content area with split panes */}
      <div style={{ flex: 1, overflow: 'hidden' }}>
        <Allotment defaultSizes={[50, 50]}>
          <Allotment.Pane>
            <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
              <div style={{ 
                padding: '0.5rem 1rem', 
                borderBottom: '1px solid var(--ifm-color-emphasis-200)',
                backgroundColor: 'var(--ifm-color-emphasis-100)',
                fontSize: '0.9rem',
                fontWeight: 'bold'
              }}>
                Input ({state.mode === 'parser' ? 'XML' : 'JSON'})
              </div>
              <div style={{ flex: 1 }}>
                <Editor
                  language={state.mode === 'parser' ? 'xml' : 'json'}
                  value={state.input}
                  onChange={(value) => setState(prev => ({ ...prev, input: value || '' }))}
                  options={{
                    minimap: { enabled: false },
                    lineNumbers: 'on',
                    wordWrap: 'on',
                    automaticLayout: true,
                    scrollBeyondLastLine: false,
                    fontSize: 14
                  }}
                  theme="vs-dark"
                />
              </div>
            </div>
          </Allotment.Pane>
          
          <Allotment.Pane>
            <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
              <div style={{ 
                padding: '0.5rem 1rem', 
                borderBottom: '1px solid var(--ifm-color-emphasis-200)',
                backgroundColor: 'var(--ifm-color-emphasis-100)',
                fontSize: '0.9rem',
                fontWeight: 'bold'
              }}>
                Output ({state.mode === 'parser' ? 'JSON' : 'XML'})
              </div>
              <div style={{ flex: 1 }}>
                <Editor
                  language={state.mode === 'parser' ? 'json' : 'xml'}
                  value={state.output}
                  options={{
                    readOnly: true,
                    minimap: { enabled: false },
                    lineNumbers: 'on',
                    wordWrap: 'on',
                    automaticLayout: true,
                    scrollBeyondLastLine: false,
                    fontSize: 14
                  }}
                  theme="vs-dark"
                />
              </div>
            </div>
          </Allotment.Pane>
        </Allotment>
      </div>

      {/* Footer with info */}
      <div style={{ 
        padding: '0.5rem 1rem', 
        borderTop: '1px solid var(--ifm-color-emphasis-200)',
        backgroundColor: 'var(--ifm-background-color)',
        fontSize: '0.8rem',
        color: 'var(--ifm-color-emphasis-600)'
      }}>
        <strong>Note:</strong> This playground uses mock implementations for demonstration. 
        In production, it would use the actual DDEX Parser and Builder WASM packages. 
        Try switching between Parser and Builder modes, loading samples, and exporting results.
      </div>
    </div>
  );
}

export default function Playground() {
  return (
    <Layout title="DDEX Playground" description="Interactive DDEX Suite playground">
      <BrowserOnly fallback={<div>Loading playground...</div>}>
        {() => <PlaygroundComponent />}
      </BrowserOnly>
    </Layout>
  );
}