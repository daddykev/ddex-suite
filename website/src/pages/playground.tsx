import React, { useState, useCallback, useEffect } from 'react';
import Layout from '@theme/Layout';
import BrowserOnly from '@docusaurus/BrowserOnly';
import { Allotment } from 'allotment';
import Editor from '@monaco-editor/react';
import 'allotment/dist/style.css';

// WASM will be loaded dynamically
type WasmModule = {
  WasmDdexBuilder: new () => any;
  Release: new (releaseId: string, releaseType: string, title: string, artist: string) => any;
  Resource: new (resourceId: string, resourceType: string, title: string, artist: string) => any;
  default: (wasmPath?: string) => Promise<any>;
};


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
  wasmLoaded: boolean;
  parserLoaded: boolean;
}

let wasmModule: WasmModule | null = null;

function PlaygroundComponent() {
  const [state, setState] = useState<PlaygroundState>({
    mode: 'parser',
    input: SAMPLE_FILES['ERN 4.3 Simple'],
    output: '',
    loading: false,
    error: '',
    wasmLoaded: false,
    parserLoaded: false
  });

  // Load modules on component mount
  useEffect(() => {
    const loadModules = async () => {
      // Load Builder WASM
      if (!wasmModule) {
        try {
          wasmModule = await import('/wasm/ddex_builder_wasm.js') as WasmModule;
          await wasmModule.default('/wasm/ddex_builder_wasm_bg.wasm');
          setState(prev => ({ ...prev, wasmLoaded: true }));
        } catch (error) {
          console.error('Failed to load Builder WASM module:', error);
          setState(prev => ({ 
            ...prev, 
            error: 'Failed to load DDEX Builder. Please refresh the page.' 
          }));
        }
      } else {
        setState(prev => ({ ...prev, wasmLoaded: true }));
      }

      // Parser is now built-in, no need to load external module
      setState(prev => ({ ...prev, parserLoaded: true }));
    };
    
    loadModules();
  }, []);

  const parseXML = useCallback(async (xml: string) => {
    // Simple XML parser that extracts key information from DDEX XML
    try {
      const parser = new DOMParser();
      const doc = parser.parseFromString(xml, 'text/xml');
      
      // Check for parsing errors
      const parseError = doc.querySelector('parsererror');
      if (parseError) {
        throw new Error(`XML parsing error: ${parseError.textContent}`);
      }
      
      // Extract version from namespace or schema version
      const root = doc.documentElement;
      const namespaceURI = root.namespaceURI || '';
      let version = '4.3'; // default
      if (namespaceURI.includes('/42')) version = '4.2';
      else if (namespaceURI.includes('/382')) version = '3.8.2';
      
      // Extract basic message info
      const messageId = doc.querySelector('MessageId')?.textContent || 'Unknown';
      const messageCreatedDateTime = doc.querySelector('MessageCreatedDateTime')?.textContent || '';
      
      // Extract parties
      const parties = Array.from(doc.querySelectorAll('Party')).map(party => ({
        partyReference: party.querySelector('PartyReference')?.textContent || '',
        partyName: party.querySelector('PartyName FullName')?.textContent || '',
        partyId: party.querySelector('PartyId')?.textContent || ''
      }));
      
      // Extract resources
      const resources = Array.from(doc.querySelectorAll('SoundRecording, Image, Video')).map(resource => ({
        resourceReference: resource.querySelector('ResourceReference')?.textContent || '',
        title: resource.querySelector('Title TitleText')?.textContent || '',
        displayArtist: resource.querySelector('DisplayArtist PartyName FullName')?.textContent || '',
        isrc: resource.querySelector('SoundRecordingId ISRC')?.textContent || '',
        duration: resource.querySelector('Duration')?.textContent || '',
        genre: resource.querySelector('Genre GenreText')?.textContent || ''
      }));
      
      // Extract releases
      const releases = Array.from(doc.querySelectorAll('Release')).map(release => ({
        releaseReference: release.querySelector('ReleaseReference')?.textContent || '',
        title: release.querySelector('Title TitleText')?.textContent || '',
        displayArtist: release.querySelector('DisplayArtist PartyName FullName')?.textContent || '',
        releaseType: release.querySelector('ReleaseType')?.textContent || '',
        upc: release.querySelector('ReleaseId ICPN, ReleaseId UPC')?.textContent || '',
        pLine: release.querySelector('PLine')?.textContent || '',
        cLine: release.querySelector('CLine')?.textContent || '',
        genre: release.querySelector('Genre GenreText')?.textContent || '',
        trackReferences: Array.from(release.querySelectorAll('ReleaseResourceReference')).map(ref => ref.textContent || '')
      }));
      
      // Extract deals
      const deals = Array.from(doc.querySelectorAll('ReleaseDeal, ResourceDeal')).map(deal => ({
        dealReference: deal.querySelector('DealReference')?.textContent || '',
        commercialModelType: deal.querySelector('CommercialModelType')?.textContent || '',
        useTypes: Array.from(deal.querySelectorAll('UseType')).map(use => use.textContent || ''),
        territories: Array.from(deal.querySelectorAll('TerritoryCode')).map(territory => territory.textContent || ''),
        startDate: deal.querySelector('StartDate')?.textContent || '',
        endDate: deal.querySelector('EndDate')?.textContent || ''
      }));
      
      // Create flattened representation
      const flatReleases = releases.map(release => ({
        title: release.title,
        artist: release.displayArtist,
        releaseType: release.releaseType,
        upc: release.upc,
        label: parties.find(p => p.partyReference !== release.displayArtist)?.partyName || '',
        genre: release.genre,
        territories: deals.flatMap(d => d.territories),
        tracks: release.trackReferences.map(ref => {
          const resource = resources.find(r => r.resourceReference === ref);
          return {
            title: resource?.title || '',
            artist: resource?.displayArtist || '',
            isrc: resource?.isrc || '',
            duration: resource?.duration || '',
            genre: resource?.genre || ''
          };
        }).filter(track => track.title) // Only include tracks with titles
      }));
      
      return {
        messageId,
        version,
        graph: {
          messageHeader: {
            messageId,
            messageCreatedDateTime
          },
          parties,
          resources,
          releases,
          deals
        },
        flat: {
          releases: flatReleases
        }
      };
    } catch (error) {
      console.error('Error parsing XML:', error);
      throw new Error(`Failed to parse DDEX XML: ${error.message || error}`);
    }
  }, []);

  const buildXML = useCallback(async (json: string) => {
    if (!wasmModule) {
      throw new Error('DDEX Builder WASM module not loaded');
    }
    
    // Check if input looks like XML instead of JSON
    const trimmedInput = json.trim();
    if (trimmedInput.startsWith('<?xml') || trimmedInput.startsWith('<')) {
      throw new Error('Builder mode requires JSON input, but XML was provided. Please switch to Parser mode or load the "Builder Template" sample.');
    }
    
    try {
      const data = JSON.parse(json);
      const builder = new wasmModule.WasmDdexBuilder();
      
      // Add resources first
      if (data.resources && Array.isArray(data.resources)) {
        for (const resource of data.resources) {
          const wasmResource = new wasmModule.Resource(
            resource.resourceId || 'RES_GENERATED',
            resource.resourceType || 'SoundRecording',
            resource.title || 'Untitled Track',
            resource.artist || 'Unknown Artist'
          );
          
          if (resource.isrc) wasmResource.isrc = resource.isrc;
          if (resource.duration) wasmResource.duration = resource.duration;
          if (resource.trackNumber) wasmResource.track_number = resource.trackNumber;
          
          builder.addResource(wasmResource);
        }
      }
      
      // Add releases
      if (data.releases && Array.isArray(data.releases)) {
        for (const release of data.releases) {
          const wasmRelease = new wasmModule.Release(
            release.releaseId || 'REL_GENERATED',
            release.releaseType || 'Album',
            release.title || 'Untitled',
            release.artist || 'Unknown Artist'
          );
          
          if (release.label) wasmRelease.label = release.label;
          if (release.upc) wasmRelease.upc = release.upc;
          if (release.releaseDate) wasmRelease.release_date = release.releaseDate;
          if (release.genres) {
            wasmRelease.genre = Array.isArray(release.genres) ? release.genres.join(', ') : release.genres;
          }
          if (release.trackIds && Array.isArray(release.trackIds)) {
            wasmRelease.track_ids = release.trackIds;
          }
          
          builder.addRelease(wasmRelease);
        }
      }
      
      // Build the XML using the actual WASM ddex-builder
      const xml = await builder.build();
      return xml;
      
    } catch (error) {
      console.error('Error building XML:', error);
      throw new Error(`Failed to build DDEX XML: ${error.message || error}`);
    }
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
    // Always reset input to appropriate default when switching modes
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
    const sampleContent = SAMPLE_FILES[sampleName];
    const isXmlSample = sampleName.startsWith('ERN');
    const isJsonSample = sampleName === 'Builder Template';
    
    // Warn if loading incompatible sample type
    let warning = '';
    if (state.mode === 'builder' && isXmlSample) {
      warning = 'XML samples are for Parser mode. Switch to Parser mode or choose "Builder Template".';
    } else if (state.mode === 'parser' && isJsonSample) {
      warning = 'JSON template is for Builder mode. Switch to Builder mode or choose an ERN sample.';
    }
    
    setState(prev => ({
      ...prev,
      input: sampleContent,
      output: '',
      error: warning
    }));
  }, [state.mode]);

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
              {Object.keys(SAMPLE_FILES).map(name => {
                const isXmlSample = name.startsWith('ERN');
                const isJsonSample = name === 'Builder Template';
                const isCompatible = (state.mode === 'parser' && isXmlSample) || 
                                   (state.mode === 'builder' && isJsonSample) ||
                                   (!isXmlSample && !isJsonSample);
                return (
                  <option key={name} value={name} disabled={!isCompatible}>
                    {name} {!isCompatible ? `(for ${isXmlSample ? 'Parser' : 'Builder'} mode)` : ''}
                  </option>
                );
              })}
            </select>
          </div>
          
          <button
            className="button button--primary"
            onClick={handleProcess}
            disabled={state.loading || 
                     (state.mode === 'builder' && !state.wasmLoaded) || 
                     (state.mode === 'parser' && !state.parserLoaded)}
          >
            {state.loading ? 'Processing...' : 
             (state.mode === 'builder' && !state.wasmLoaded) ? 'Loading Builder...' : 
             (state.mode === 'parser' && !state.parserLoaded) ? 'Loading Parser...' :
             (state.mode === 'parser' ? 'Parse XML' : 'Build XML')}
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
        <strong>Note:</strong> This playground uses the actual DDEX Suite libraries for real parsing and building.
        Parser mode uses the ddex-parser package and Builder mode uses the ddex-builder WASM package.
        Try switching between modes, loading samples, editing content, and exporting results.
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