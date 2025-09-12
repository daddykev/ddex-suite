#!/usr/bin/env node

const { DDEXParser } = require('ddex-parser');
const { DdexBuilder } = require('ddex-builder');
const { performance } = require('perf_hooks');

console.log('🔌 Testing Extension Preservation for Partner Extensions');
console.log('=' + '='.repeat(60));

async function testExtensionPreservation() {
    const parser = new DDEXParser();
    const builder = new DdexBuilder();
    
    // Create XML with various partner extensions
    const xmlWithExtensions = `<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage 
        xmlns:ern="http://ddex.net/xml/ern/43"
        xmlns:spotify="http://www.spotify.com/metadata"
        xmlns:apple="http://www.apple.com/itunes"
        xmlns:youtube="http://www.youtube.com/music"
        xmlns:custom="http://example.com/custom">
        <MessageHeader>
            <MessageId>EXT-TEST-001</MessageId>
            <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
            <MessageSender>
                <PartyName>Extension Test</PartyName>
                <spotify:PartyIdentifier>spotify:artist:123456</spotify:PartyIdentifier>
            </MessageSender>
            <MessageRecipient>
                <PartyName>Test Recipient</PartyName>
                <apple:StoreId>12345</apple:StoreId>
            </MessageRecipient>
        </MessageHeader>
        <ResourceList>
            <SoundRecording>
                <ResourceReference>SR001</ResourceReference>
                <Type>MusicalWorkSoundRecording</Type>
                <Title>
                    <TitleText>Test Track with Extensions</TitleText>
                </Title>
                <Duration>PT3M30S</Duration>
                <DisplayArtist>Test Artist</DisplayArtist>
                
                <!-- Spotify Extensions -->
                <spotify:Loudness>-14.0</spotify:Loudness>
                <spotify:ContentRating>0</spotify:ContentRating>
                <spotify:Popularity>75</spotify:Popularity>
                
                <!-- Apple Extensions -->
                <apple:Explicit>false</apple:Explicit>
                <apple:PreviewStart>30</apple:PreviewStart>
                <apple:PreviewLength>30</apple:PreviewLength>
                
                <!-- YouTube Extensions -->
                <youtube:ContentTier>premium</youtube:ContentTier>
                <youtube:VideoId>dQw4w9WgXcQ</youtube:VideoId>
                <youtube:Category>Music</youtube:Category>
                
                <!-- Custom Extensions -->
                <custom:QualityRating>HD</custom:QualityRating>
                <custom:MarketingTags>
                    <custom:Tag>trending</custom:Tag>
                    <custom:Tag>featured</custom:Tag>
                </custom:MarketingTags>
            </SoundRecording>
        </ResourceList>
    </ern:NewReleaseMessage>`;
    
    const results = {
        parsing: { success: false, extensions: [] },
        preservation: { success: false, details: {} },
        roundTrip: { success: false, fidelityScore: 0 }
    };
    
    try {
        console.log('1. Parsing XML with partner extensions...');
        const startTime = performance.now();
        const parsed = await parser.parse(xmlWithExtensions);
        const parseTime = performance.now() - startTime;
        
        console.log(`   ✅ Parsed successfully in ${parseTime.toFixed(2)}ms`);
        console.log(`   📊 Result structure: ${Object.keys(parsed).join(', ')}`);
        
        if (parsed.graph && parsed.flat) {
            console.log('   ✅ Both graph and flattened models available');
            results.parsing.success = true;
        }
        
        // Check if extensions are preserved in the result
        const xmlStr = JSON.stringify(parsed);
        const extensionChecks = {
            spotify: xmlStr.includes('spotify') || xmlStr.includes('Loudness') || xmlStr.includes('ContentRating'),
            apple: xmlStr.includes('apple') || xmlStr.includes('Explicit') || xmlStr.includes('PreviewStart'),
            youtube: xmlStr.includes('youtube') || xmlStr.includes('ContentTier') || xmlStr.includes('VideoId'),
            custom: xmlStr.includes('custom') || xmlStr.includes('QualityRating') || xmlStr.includes('MarketingTags')
        };
        
        console.log('   🔍 Extension preservation check:');
        for (const [platform, found] of Object.entries(extensionChecks)) {
            console.log(`      ${platform}: ${found ? '✅' : '❌'} ${found ? 'preserved' : 'not found'}`);
            if (found) results.parsing.extensions.push(platform);
        }
        
        results.preservation.details = extensionChecks;
        results.preservation.success = Object.values(extensionChecks).some(Boolean);
        
    } catch (error) {
        console.log(`   ❌ Parsing failed: ${error.message}`);
        results.parsing.success = false;
    }
    
    // Test with builder if available
    try {
        console.log('\n2. Testing builder with extension-aware data...');
        
        // Add some test data to builder
        builder.addResource({
            resourceId: 'SR001',
            resourceType: 'SoundRecording',
            title: 'Test Track with Extensions',
            artist: 'Test Artist',
            isrc: 'TEST12345678',
            duration: 'PT3M30S',
            metadata: {
                'spotify:Loudness': '-14.0',
                'apple:Explicit': 'false',
                'youtube:ContentTier': 'premium'
            }
        });
        
        builder.addRelease({
            releaseId: 'REL001',
            releaseType: 'Album',
            title: 'Test Album with Extensions',
            artist: 'Test Artist',
            upc: '123456789012',
            trackIds: ['SR001'],
            metadata: {
                'spotify:Popularity': '75',
                'apple:PreviewLength': '30',
                'youtube:Category': 'Music'
            }
        });
        
        const startTime = performance.now();
        const builtXml = await builder.build();
        const buildTime = performance.now() - startTime;
        
        console.log(`   ✅ Built XML successfully in ${buildTime.toFixed(2)}ms`);
        console.log(`   📏 Output size: ${Math.round(builtXml.length / 1024)}KB`);
        
        // Check if extensions are preserved in built XML
        const extensionChecks = {
            spotify: builtXml.includes('spotify') || builtXml.includes('Loudness'),
            apple: builtXml.includes('apple') || builtXml.includes('Explicit'),
            youtube: builtXml.includes('youtube') || builtXml.includes('ContentTier')
        };
        
        console.log('   🔍 Extension preservation in built XML:');
        for (const [platform, found] of Object.entries(extensionChecks)) {
            console.log(`      ${platform}: ${found ? '✅' : '❌'} ${found ? 'preserved' : 'not found'}`);
        }
        
        results.roundTrip.success = Object.values(extensionChecks).some(Boolean);
        
    } catch (error) {
        console.log(`   ❌ Builder test failed: ${error.message}`);
    }
    
    // Summary
    console.log('\n' + '='.repeat(70));
    console.log('📊 Extension Preservation Test Results:');
    console.log(`   Parsing Success: ${results.parsing.success ? '✅' : '❌'}`);
    console.log(`   Extensions Found: ${results.parsing.extensions.join(', ') || 'None'}`);
    console.log(`   Builder Support: ${results.roundTrip.success ? '✅' : '❌'}`);
    
    const overallSuccess = results.parsing.success && results.preservation.success;
    console.log(`   Overall: ${overallSuccess ? '✅ PASSED' : '❌ FAILED'}`);
    console.log('='.repeat(70));
    
    return overallSuccess;
}

// Test streaming API
async function testStreamingAPI() {
    console.log('\n🌊 Testing Streaming API...');
    
    const parser = new DDEXParser();
    
    try {
        // Generate a larger XML for streaming test
        const largeXml = generateLargeXML();
        console.log(`   📏 Generated large XML: ~${Math.round(largeXml.length / 1024 / 1024)}MB`);
        
        // Test streaming parsing
        const startTime = performance.now();
        
        // Check if stream method exists
        if (typeof parser.stream === 'function') {
            console.log('   🌊 Using streaming parser...');
            const stream = parser.stream(largeXml);
            
            if (stream && typeof stream.next === 'function') {
                let releaseCount = 0;
                let release;
                
                do {
                    release = await stream.next();
                    if (release) {
                        releaseCount++;
                        if (releaseCount <= 3) { // Log first few releases
                            console.log(`      Release ${releaseCount}: ${release.title || 'Unknown'}`);
                        }
                    }
                } while (release);
                
                console.log(`   ✅ Streamed ${releaseCount} releases`);
            } else {
                console.log('   ⚠️  Stream object created but no next() method');
            }
        } else {
            // Fallback to regular parsing
            console.log('   📝 Fallback to regular parsing (no streaming method)...');
            const result = await parser.parse(largeXml);
            console.log(`   ✅ Parsed successfully with ${result.flat?.releases?.length || 0} releases`);
        }
        
        const streamTime = performance.now() - startTime;
        console.log(`   ⏱️  Total time: ${streamTime.toFixed(2)}ms`);
        
        return true;
        
    } catch (error) {
        console.log(`   ❌ Streaming test failed: ${error.message}`);
        return false;
    }
}

function generateLargeXML() {
    // Generate a large XML (~5MB) with many releases
    const releases = [];
    for (let i = 1; i <= 100; i++) {
        releases.push(`
        <Release>
            <ReleaseReference>REL${i.toString().padStart(3, '0')}</ReleaseReference>
            <ReleaseType>Album</ReleaseType>
            <DisplayTitleText>Large Test Album ${i}</DisplayTitleText>
            <DisplayArtist>Test Artist ${Math.floor(i / 10)}</DisplayArtist>
            <ReleaseId><ICPN>${(1234567890000 + i).toString()}</ICPN></ReleaseId>
        </Release>`);
    }
    
    const resources = [];
    for (let i = 1; i <= 500; i++) {
        resources.push(`
        <SoundRecording>
            <ResourceReference>SR${i.toString().padStart(4, '0')}</ResourceReference>
            <Type>MusicalWorkSoundRecording</Type>
            <Title><TitleText>Large Test Track ${i}</TitleText></Title>
            <Duration>PT${3 + (i % 5)}M${(i % 60).toString().padStart(2, '0')}S</Duration>
            <DisplayArtist>Test Artist ${Math.floor(i / 50)}</DisplayArtist>
        </SoundRecording>`);
    }
    
    return `<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <MessageHeader>
            <MessageId>LARGE-STREAM-TEST-001</MessageId>
            <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
            <MessageSender><PartyName>Streaming Test</PartyName></MessageSender>
            <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
        </MessageHeader>
        <ResourceList>
            ${resources.join('\n')}
        </ResourceList>
        <ReleaseList>
            ${releases.join('\n')}
        </ReleaseList>
    </ern:NewReleaseMessage>`;
}

// Run tests if called directly
if (require.main === module) {
    Promise.all([
        testExtensionPreservation(),
        testStreamingAPI()
    ]).then(([extensionResult, streamingResult]) => {
        console.log('\n🎯 Final Results:');
        console.log(`   Extension Preservation: ${extensionResult ? '✅ PASSED' : '❌ FAILED'}`);
        console.log(`   Streaming API: ${streamingResult ? '✅ PASSED' : '❌ FAILED'}`);
        
        const allPassed = extensionResult && streamingResult;
        console.log(`   Overall: ${allPassed ? '🎉 ALL TESTS PASSED' : '⚠️  SOME TESTS FAILED'}`);
    }).catch(error => {
        console.error('Fatal error:', error);
        process.exit(1);
    });
}

module.exports = { testExtensionPreservation, testStreamingAPI };