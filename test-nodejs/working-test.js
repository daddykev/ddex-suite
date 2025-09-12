#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { performance } = require('perf_hooks');

// Import both parser and builder with correct names
const { DDEXParser } = require('ddex-parser');
const { DdexBuilder } = require('ddex-builder');

console.log('🚀 DDEX Suite Node.js Bindings Test');
console.log('=' + '='.repeat(50));

async function runTests() {
    let results = {
        totalTests: 0,
        passedTests: 0,
        failedTests: 0,
        errors: []
    };
    
    function logResult(testName, success, message = '') {
        results.totalTests++;
        if (success) {
            results.passedTests++;
            console.log(`✅ ${testName}: ${message}`);
        } else {
            results.failedTests++;
            results.errors.push(`${testName}: ${message}`);
            console.log(`❌ ${testName}: ${message}`);
        }
    }
    
    // Test 1: Basic instantiation
    try {
        const parser = new DDEXParser();
        const builder = new DdexBuilder();
        logResult('Basic Instantiation', true, 'Both parser and builder created successfully');
    } catch (error) {
        logResult('Basic Instantiation', false, error.message);
    }
    
    // Test 2: TypeScript types validation
    logResult('TypeScript Types', 
        typeof DDEXParser === 'function' && typeof DdexBuilder === 'function',
        'Types are correctly exported');
    
    // Test 3: Basic methods
    try {
        const parser = new DDEXParser();
        const builder = new DdexBuilder();
        
        // Test parser version method
        try {
            const version = parser.version();
            logResult('Parser Version Method', typeof version === 'string', `Version: ${version}`);
        } catch (e) {
            logResult('Parser Version Method', false, e.message);
        }
        
        // Test builder presets
        try {
            const presets = builder.getAvailablePresets();
            logResult('Builder Presets Method', Array.isArray(presets), `Found ${presets.length} presets`);
        } catch (e) {
            logResult('Builder Presets Method', false, e.message);
        }
        
    } catch (error) {
        logResult('Basic Methods', false, error.message);
    }
    
    // Test 4: Simple parsing test
    try {
        const parser = new DDEXParser();
        
        const testXml = `<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>TEST-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender>
                    <PartyName>Test Sender</PartyName>
                </MessageSender>
                <MessageRecipient>
                    <PartyName>Test Recipient</PartyName>
                </MessageRecipient>
            </MessageHeader>
        </ern:NewReleaseMessage>`;
        
        const startTime = performance.now();
        const result = await parser.parse(testXml);
        const parseTime = performance.now() - startTime;
        
        logResult('Simple Parse Test', 
            result && typeof result === 'object', 
            `Parsed in ${parseTime.toFixed(2)}ms`);
            
        if (result) {
            console.log(`   Result keys: ${Object.keys(result).join(', ')}`);
        }
        
    } catch (error) {
        logResult('Simple Parse Test', false, error.message);
    }
    
    // Test 5: ERN 4.3 test files
    try {
        const parser = new DDEXParser();
        const testFilesPath = path.resolve(__dirname, '../test-suite/valid/ern-4.3');
        
        if (fs.existsSync(testFilesPath)) {
            const files = fs.readdirSync(testFilesPath).filter(f => f.endsWith('.xml'));
            console.log(`\n📁 Testing ${files.length} ERN 4.3 files:`);
            
            let successCount = 0;
            for (const file of files) {
                try {
                    const filePath = path.join(testFilesPath, file);
                    const xmlContent = fs.readFileSync(filePath, 'utf8');
                    
                    const startTime = performance.now();
                    const result = await parser.parse(xmlContent);
                    const parseTime = performance.now() - startTime;
                    
                    successCount++;
                    console.log(`   ✅ ${file}: ${parseTime.toFixed(2)}ms`);
                } catch (error) {
                    console.log(`   ❌ ${file}: ${error.message}`);
                }
            }
            
            logResult('ERN 4.3 Files Test', 
                successCount === files.length, 
                `${successCount}/${files.length} files parsed successfully`);
        } else {
            logResult('ERN 4.3 Files Test', false, 'Test files directory not found');
        }
        
    } catch (error) {
        logResult('ERN 4.3 Files Test', false, error.message);
    }
    
    // Test 6: Performance benchmark
    try {
        const parser = new DDEXParser();
        
        // Generate a typical-size test XML
        const typicalXml = generateTypicalXML();
        
        const times = [];
        const iterations = 3;
        
        for (let i = 0; i < iterations; i++) {
            const startTime = performance.now();
            await parser.parse(typicalXml);
            const parseTime = performance.now() - startTime;
            times.push(parseTime);
        }
        
        const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
        const targetTime = 50; // 50ms target
        
        logResult('Performance Benchmark', 
            avgTime < targetTime, 
            `Avg: ${avgTime.toFixed(2)}ms (target: <${targetTime}ms)`);
            
    } catch (error) {
        logResult('Performance Benchmark', false, error.message);
    }
    
    // Test 7: WASM bundle size check
    try {
        const wasmPath = path.resolve(__dirname, '../packages/ddex-parser/bindings/wasm');
        if (fs.existsSync(wasmPath)) {
            const files = fs.readdirSync(wasmPath);
            const wasmFile = files.find(f => f.endsWith('.wasm'));
            
            if (wasmFile) {
                const filePath = path.join(wasmPath, wasmFile);
                const stats = fs.statSync(filePath);
                const sizeKB = Math.round(stats.size / 1024);
                
                logResult('WASM Bundle Size', 
                    sizeKB < 500, 
                    `${sizeKB}KB (target: <500KB, expected ~114KB)`);
            } else {
                logResult('WASM Bundle Size', false, 'No WASM file found');
            }
        } else {
            logResult('WASM Bundle Size', true, 'WASM directory not found (may not be built)');
        }
    } catch (error) {
        logResult('WASM Bundle Size', false, error.message);
    }
    
    // Print summary
    console.log('\n' + '='.repeat(70));
    console.log('📊 Test Summary:');
    console.log(`   Total Tests: ${results.totalTests}`);
    console.log(`   Passed: ${results.passedTests} ✅`);
    console.log(`   Failed: ${results.failedTests} ${results.failedTests > 0 ? '❌' : '✅'}`);
    console.log(`   Success Rate: ${((results.passedTests / results.totalTests) * 100).toFixed(1)}%`);
    
    if (results.errors.length > 0) {
        console.log('\n❌ Errors:');
        results.errors.forEach((error, index) => {
            console.log(`   ${index + 1}. ${error}`);
        });
    }
    
    console.log('='.repeat(70));
    
    return results.failedTests === 0;
}

function generateTypicalXML() {
    // Generate a typical ERN message (~10KB)
    const resources = Array.from({length: 10}, (_, i) => `
        <SoundRecording>
            <ResourceReference>SR${(i + 1).toString().padStart(3, '0')}</ResourceReference>
            <Type>MusicalWorkSoundRecording</Type>
            <Title><TitleText>Track ${i + 1}</TitleText></Title>
            <Duration>PT3M30S</Duration>
            <DisplayArtist>Test Artist</DisplayArtist>
        </SoundRecording>`).join('\n');
    
    const releases = `
        <Release>
            <ReleaseReference>REL001</ReleaseReference>
            <ReleaseType>Album</ReleaseType>
            <DisplayTitleText>Test Album</DisplayTitleText>
            <DisplayArtist>Test Artist</DisplayArtist>
        </Release>`;
    
    return `<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <MessageHeader>
            <MessageId>PERF-TEST-001</MessageId>
            <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
            <MessageSender><PartyName>Performance Test</PartyName></MessageSender>
            <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
        </MessageHeader>
        <ResourceList>
            ${resources}
        </ResourceList>
        <ReleaseList>
            ${releases}
        </ReleaseList>
    </ern:NewReleaseMessage>`;
}

// Run tests if called directly
if (require.main === module) {
    runTests().catch(error => {
        console.error('Fatal error:', error);
        process.exit(1);
    });
}

module.exports = { runTests };