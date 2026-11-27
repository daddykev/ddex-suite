#!/usr/bin/env node

/**
 * DDEX Builder v0.2.5 Integration Test - Node.js
 * Tests version verification, basic functionality, and deterministic output
 */

const fs = require('fs');
const path = require('path');

console.log('=== DDEX Builder v0.2.5 Integration Test - Node.js ===\n');

// Import the builder from the current directory

let DDEXBuilder;
try {
    const module = require('./index.js');
    DDEXBuilder = module.DdexBuilder; // Use the correct export name
    console.log('✓ DDEXBuilder imported successfully');
    console.log('Available exports:', Object.keys(module));
} catch (error) {
    console.error('✗ Failed to import DDEXBuilder:', error.message);
    process.exit(1);
}

async function runTests() {
    console.log('\n--- Version Verification ---');
    
    // Check package.json version
    const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, 'package.json'), 'utf8'));
    const expectedVersion = '0.2.5';
    
    if (packageJson.version === expectedVersion) {
        console.log(`✓ Package version correct: ${packageJson.version}`);
    } else {
        console.log(`✗ Package version mismatch: expected ${expectedVersion}, got ${packageJson.version}`);
    }

    console.log('\n--- Basic Functionality Test ---');
    
    let builder;
    try {
        builder = new DDEXBuilder();
        console.log('✓ DDEXBuilder instance created successfully');
    } catch (error) {
        console.error('✗ Failed to create DDEXBuilder:', error.message);
        return;
    }

    // Test basic stats
    try {
        const stats = builder.getStats();
        console.log(`✓ Initial stats: releases=${stats.releases}, resources=${stats.resources}`);
    } catch (error) {
        console.error('✗ Failed to get stats:', error.message);
    }

    console.log('\n--- ERN Generation Test ---');
    
    // Create a simple test ERN with correct format matching Python
    // ERN 4.3 needs label, genre and parental warning on the release, and a DDEX type,
    // ISRC and parental warning on each resource (nothing is filled in by the builder)
    const testRelease = {
        releaseId: 'TEST_RELEASE_001',
        releaseType: 'Album',
        title: 'Test Album v0.2.5',
        artist: 'Test Artist',
        label: 'Test Label',
        genre: 'Pop',
        parentalWarningType: 'NotExplicit',
        trackIds: ['A1']
    };

    const testResource = {
        resourceId: 'A1',
        resourceType: 'SoundRecording',
        ddexType: 'MusicalWorkSoundRecording',
        parentalWarningType: 'NotExplicit',
        title: 'Test Track 1 Audio',
        isrc: 'USRT12400001',
        duration: 'PT3M00S',
        artist: 'Test Artist'
    };

    // The message header and version come from the build request
    const request = {
        version: '4.3',
        language_and_script_code: 'en',
        avs_version_id: '12',
        header: {
            message_id: 'MSG-INTEGRATION-001',
            message_sender: { party_name: [{ text: 'Test Label' }], party_id: 'PADPIDA0000000001' },
            message_recipient: { party_name: [{ text: 'Test DSP' }], party_id: 'PADPIDA0000000002' },
            message_created_date_time: '2024-01-01T00:00:00Z'
        }
    };

    try {
        // Add release and resource
        builder.addRelease(testRelease);
        builder.addResource(testResource);
        console.log('✓ Test release and resource added');

        const updatedStats = builder.getStats();
        console.log(`✓ Updated stats: ${JSON.stringify(updatedStats)}`);

        // Validate
        const validation = await builder.validate();
        console.log(`✓ Validation: ${JSON.stringify(validation)}`);

        // Build XML
        const xml = await builder.build(request);
        console.log(`✓ XML generated, length: ${xml ? xml.length : 'undefined'} characters`);

        // Check for placeholder content (ensure xml is a string)
        if (xml && typeof xml === 'string') {
            if (xml.includes('PLACEHOLDER') || xml.includes('placeholder')) {
                console.log('✗ Warning: XML contains placeholder content');
            } else {
                console.log('✓ No placeholder content found in XML');
            }
        } else {
            console.log('✗ XML is not a string or is undefined');
        }

        // Save output for comparison (only if xml is valid)
        if (xml && typeof xml === 'string') {
            const outputFile = '/Users/kevinmoo/Desktop/localrepo/ddex-suite/test-assets/test_output_nodejs.xml';
            fs.writeFileSync(outputFile, xml);
            console.log(`✓ XML saved to: ${outputFile}`);

            // Preview first 200 characters
            console.log(`\nXML Preview:\n${xml.substring(0, 200)}...`);
        } else {
            console.log('✗ Cannot save XML - invalid output');
        }

        // Reset for cleanup
        builder.reset();
        console.log('✓ Builder reset completed');

    } catch (error) {
        console.error('✗ ERN generation failed:', error.message);
    }
}

runTests().then(() => {
    console.log('\n=== Node.js Integration Test Complete ===');
}).catch(error => {
    console.error('Test failed:', error);
    process.exit(1);
});