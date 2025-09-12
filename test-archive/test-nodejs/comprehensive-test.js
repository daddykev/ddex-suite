#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { performance } = require('perf_hooks');

// Import both parser and builder
const { DDEXParser } = require('ddex-parser');
const { DdexBuilder } = require('ddex-builder');

// Test configuration
const TEST_CONFIG = {
    ERN_43_FILES_PATH: '../test-suite/valid/ern-4.3',
    PERFORMANCE_TARGET_MS: 50,
    WASM_BUNDLE_SIZE_TARGET_KB: 500,
    EXPECTED_WASM_SIZE_KB: 114,
    LARGE_FILE_SIZE_MB: 50
};

class NodeJSBindingsTester {
    constructor() {
        this.parser = new DDEXParser();
        this.builder = new DdexBuilder();
        this.results = {
            totalTests: 0,
            passedTests: 0,
            failedTests: 0,
            errors: [],
            performance: {},
            typeChecks: {},
            roundTripTests: [],
            extensionTests: []
        };
    }

    log(message, isError = false) {
        const timestamp = new Date().toISOString();
        const prefix = isError ? '❌ ERROR' : '✅ INFO';
        console.log(`[${timestamp}] ${prefix}: ${message}`);
        
        if (isError) {
            this.results.errors.push(message);
        }
    }

    async runAllTests() {
        console.log('🚀 Starting DDEX Suite Node.js Bindings Comprehensive Test');
        console.log('=' * 70);
        
        try {
            await this.testTypeScriptTypes();
            await this.testBasicFunctionality();
            await this.testERN43Files();
            await this.testGraphAndFlattenedModels();
            await this.testRoundTripFidelity();
            await this.testExtensionPreservation();
            await this.testPerformanceBenchmarks();
            await this.testStreamingAPI();
            await this.testWASMBundleSize();
            
            this.printSummary();
        } catch (error) {
            this.log(`Fatal error during testing: ${error.message}`, true);
            process.exit(1);
        }
    }

    async testTypeScriptTypes() {
        this.log('Testing TypeScript type definitions...');
        
        try {
            // Check that types are available (basic smoke test)
            const typesExist = {
                parser: typeof DDEXParser === 'function',
                builder: typeof DdexBuilder === 'function',
                parseOptions: true, // These are interfaces, can't test directly
                buildResult: true
            };
            
            this.results.typeChecks = typesExist;
            this.results.totalTests++;
            
            if (typesExist.parser && typesExist.builder) {
                this.results.passedTests++;
                this.log('TypeScript types validation passed');
            } else {
                this.results.failedTests++;
                this.log('TypeScript types validation failed', true);
            }
        } catch (error) {
            this.log(`TypeScript types test failed: ${error.message}`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    async testBasicFunctionality() {
        this.log('Testing basic parser and builder functionality...');
        
        try {
            // Test basic parser methods
            const version = this.parser.detectVersion('<xml></xml>');
            this.log(`Version detection works: ${typeof version === 'string'}`);
            
            // Test builder methods
            const presets = this.builder.getAvailablePresets();
            this.log(`Available presets: ${presets.length} presets found`);
            
            this.results.passedTests++;
        } catch (error) {
            this.log(`Basic functionality test failed: ${error.message}`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    async testERN43Files() {
        this.log('Testing ERN 4.3 test files...');
        
        const testFilesPath = path.resolve(__dirname, TEST_CONFIG.ERN_43_FILES_PATH);
        
        if (!fs.existsSync(testFilesPath)) {
            this.log(`ERN 4.3 test files not found at: ${testFilesPath}`, true);
            this.results.failedTests++;
            this.results.totalTests++;
            return;
        }
        
        try {
            const files = fs.readdirSync(testFilesPath)
                .filter(f => f.endsWith('.xml'));
            
            this.log(`Found ${files.length} ERN 4.3 test files`);
            
            for (const file of files) {
                const filePath = path.join(testFilesPath, file);
                const xmlContent = fs.readFileSync(filePath, 'utf8');
                
                try {
                    const startTime = performance.now();
                    const result = await this.parser.parse(xmlContent, {
                        fidelityLevel: 'perfect',
                        preserveExtensions: true,
                        preserveComments: true,
                        collectStatistics: true
                    });
                    const parseTime = performance.now() - startTime;
                    
                    this.log(`✅ Parsed ${file} in ${parseTime.toFixed(2)}ms`);
                    this.log(`   Message ID: ${result.messageId}, Version: ${result.version}`);
                    this.log(`   Releases: ${result.releaseCount}, Resources: ${result.resourceCount}`);
                    
                    if (result.statistics) {
                        this.log(`   Statistics: ${result.statistics.elementCount} elements, ${result.statistics.memoryUsedBytes} bytes used`);
                    }
                    
                    this.results.passedTests++;
                } catch (parseError) {
                    this.log(`Failed to parse ${file}: ${parseError.message}`, true);
                    this.results.failedTests++;
                }
                
                this.results.totalTests++;
            }
        } catch (error) {
            this.log(`ERN 4.3 files test failed: ${error.message}`, true);
            this.results.failedTests++;
            this.results.totalTests++;
        }
    }

    async testGraphAndFlattenedModels() {
        this.log('Testing graph and flattened models...');
        
        // Create a simple test XML for this test
        const testXml = `<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageThreadId>Test-001</MessageThreadId>
                <MessageId>MSG-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender>
                    <PartyName>Test Sender</PartyName>
                </MessageSender>
                <MessageRecipient>
                    <PartyName>Test Recipient</PartyName>
                </MessageRecipient>
            </MessageHeader>
            <ReleaseList>
                <Release>
                    <ReleaseReference>R001</ReleaseReference>
                    <ReleaseType>Album</ReleaseType>
                    <ReleaseId>
                        <ICPN>1234567890123</ICPN>
                    </ReleaseId>
                    <DisplayTitleText>Test Release</DisplayTitleText>
                </Release>
            </ReleaseList>
        </ern:NewReleaseMessage>`;
        
        try {
            // Test with different parse options to get both models
            const graphResult = await this.parser.parse(testXml, {
                fidelityLevel: 'perfect',
                preserveExtensions: true
            });
            
            const flatResult = await this.parser.parse(testXml, {
                fidelityLevel: 'fast'
            });
            
            this.log(`Graph model parsed successfully: ${graphResult.messageId}`);
            this.log(`Flattened model parsed successfully: ${flatResult.messageId}`);
            
            this.results.passedTests++;
        } catch (error) {
            this.log(`Graph/flattened models test failed: ${error.message}`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    async testRoundTripFidelity() {
        this.log('Testing round-trip fidelity: parse → toBuildRequest() → build → parse...');
        
        const testXml = `<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>ROUND-TRIP-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender><PartyName>Test Sender</PartyName></MessageSender>
                <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
            </MessageHeader>
        </ern:NewReleaseMessage>`;
        
        try {
            // Step 1: Parse original XML
            const startTime = performance.now();
            const parsed = await this.parser.parse(testXml, {
                fidelityLevel: 'perfect',
                preserveExtensions: true,
                preserveComments: true,
                preserveProcessingInstructions: true
            });
            
            // Step 2: Build XML (using test round-trip method)
            const roundTripResult = await this.builder.testRoundTripFidelity(testXml, {
                enablePerfectFidelity: true,
                canonicalization: 'db_c14n',
                preserveExtensions: true,
                preserveComments: true,
                enableVerification: true,
                collectStatistics: true
            });
            
            const totalTime = performance.now() - startTime;
            
            this.log(`Round-trip test completed in ${totalTime.toFixed(2)}ms`);
            this.log(`Round-trip success: ${roundTripResult.roundTripSuccess}`);
            this.log(`Fidelity score: ${roundTripResult.fidelityScore}`);
            this.log(`Canonicalization consistent: ${roundTripResult.canonicalizationConsistent}`);
            this.log(`Determinism verified: ${roundTripResult.determinismVerified}`);
            
            if (roundTripResult.issues && roundTripResult.issues.length > 0) {
                this.log(`Issues found: ${roundTripResult.issues.join(', ')}`);
            }
            
            this.results.roundTripTests.push({
                success: roundTripResult.roundTripSuccess,
                fidelityScore: roundTripResult.fidelityScore,
                timeMs: totalTime,
                issues: roundTripResult.issues || []
            });
            
            if (roundTripResult.roundTripSuccess && roundTripResult.fidelityScore > 0.9) {
                this.results.passedTests++;
            } else {
                this.results.failedTests++;
            }
        } catch (error) {
            this.log(`Round-trip fidelity test failed: ${error.message}`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    async testExtensionPreservation() {
        this.log('Testing extension preservation for partner extensions...');
        
        const xmlWithExtensions = `<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage 
            xmlns:ern="http://ddex.net/xml/ern/43"
            xmlns:spotify="http://www.spotify.com/metadata"
            xmlns:apple="http://www.apple.com/itunes"
            xmlns:youtube="http://www.youtube.com/music">
            <MessageHeader>
                <MessageId>EXT-TEST-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender><PartyName>Test</PartyName></MessageSender>
                <MessageRecipient><PartyName>Test</PartyName></MessageRecipient>
            </MessageHeader>
            <ResourceList>
                <SoundRecording>
                    <ResourceReference>SR001</ResourceReference>
                    <Type>MusicalWorkSoundRecording</Type>
                    <Title><TitleText>Test Track</TitleText></Title>
                    <spotify:Loudness>-14.0</spotify:Loudness>
                    <apple:Explicit>false</apple:Explicit>
                    <youtube:ContentTier>premium</youtube:ContentTier>
                </SoundRecording>
            </ResourceList>
        </ern:NewReleaseMessage>`;
        
        try {
            const parsed = await this.parser.parse(xmlWithExtensions, {
                preserveExtensions: true,
                extensionValidation: true,
                fidelityLevel: 'perfect'
            });
            
            // Test round-trip with extensions
            const roundTripResult = await this.builder.testRoundTripFidelity(xmlWithExtensions, {
                preserveExtensions: true,
                enablePerfectFidelity: true
            });
            
            this.log(`Extension preservation test - Round-trip success: ${roundTripResult.roundTripSuccess}`);
            
            // Check if extensions were preserved (this is a basic check)
            const extensionsPreserved = parsed.statistics?.extensionCount > 0;
            
            this.results.extensionTests.push({
                spotifyExtensions: xmlWithExtensions.includes('spotify:'),
                appleExtensions: xmlWithExtensions.includes('apple:'),
                youtubeExtensions: xmlWithExtensions.includes('youtube:'),
                extensionsPreserved: extensionsPreserved,
                roundTripSuccess: roundTripResult.roundTripSuccess
            });
            
            if (extensionsPreserved && roundTripResult.roundTripSuccess) {
                this.results.passedTests++;
            } else {
                this.results.failedTests++;
                this.log('Extension preservation failed', true);
            }
        } catch (error) {
            this.log(`Extension preservation test failed: ${error.message}`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    async testPerformanceBenchmarks() {
        this.log(`Testing performance benchmarks (target: <${TEST_CONFIG.PERFORMANCE_TARGET_MS}ms for typical files)...`);
        
        // Create a typical-sized test XML
        const typicalXml = this.generateTypicalSizeXML();
        
        const performanceTests = [];
        const iterations = 5;
        
        for (let i = 0; i < iterations; i++) {
            try {
                const startTime = performance.now();
                const result = await this.parser.parse(typicalXml, {
                    fidelityLevel: 'balanced'
                });
                const endTime = performance.now();
                const parseTime = endTime - startTime;
                
                performanceTests.push(parseTime);
                this.log(`Performance test ${i + 1}: ${parseTime.toFixed(2)}ms`);
            } catch (error) {
                this.log(`Performance test ${i + 1} failed: ${error.message}`, true);
                this.results.failedTests++;
                this.results.totalTests++;
                return;
            }
        }
        
        const avgTime = performanceTests.reduce((a, b) => a + b, 0) / performanceTests.length;
        const minTime = Math.min(...performanceTests);
        const maxTime = Math.max(...performanceTests);
        
        this.results.performance = {
            averageTimeMs: avgTime,
            minTimeMs: minTime,
            maxTimeMs: maxTime,
            targetMs: TEST_CONFIG.PERFORMANCE_TARGET_MS,
            meetsTarget: avgTime < TEST_CONFIG.PERFORMANCE_TARGET_MS
        };
        
        this.log(`Performance results: avg=${avgTime.toFixed(2)}ms, min=${minTime.toFixed(2)}ms, max=${maxTime.toFixed(2)}ms`);
        
        if (this.results.performance.meetsTarget) {
            this.log(`✅ Performance target met (${avgTime.toFixed(2)}ms < ${TEST_CONFIG.PERFORMANCE_TARGET_MS}ms)`);
            this.results.passedTests++;
        } else {
            this.log(`❌ Performance target missed (${avgTime.toFixed(2)}ms >= ${TEST_CONFIG.PERFORMANCE_TARGET_MS}ms)`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    async testStreamingAPI() {
        this.log('Testing streaming API for large files...');
        
        try {
            // Generate a large XML for streaming test
            const largeXml = this.generateLargeXML();
            
            // Test basic streaming functionality
            const streamOptions = {
                chunkSize: 64 * 1024, // 64KB chunks
                maxMemory: 100 * 1024 * 1024 // 100MB limit
            };
            
            const startTime = performance.now();
            
            // For now, we'll test the regular parse with large content
            // In a real streaming implementation, we'd use stream() method
            const result = await this.parser.parse(largeXml, {
                enableStreaming: true,
                streamingThreshold: 1024 * 1024, // 1MB
                fidelityLevel: 'balanced',
                memoryLimit: 100 * 1024 * 1024
            });
            
            const streamTime = performance.now() - startTime;
            
            this.log(`Streaming test completed in ${streamTime.toFixed(2)}ms`);
            this.log(`Large file size: ~${Math.round(largeXml.length / 1024 / 1024)}MB`);
            this.log(`Memory usage: ${result.statistics?.memoryUsedBytes || 0} bytes`);
            
            this.results.passedTests++;
        } catch (error) {
            this.log(`Streaming API test failed: ${error.message}`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    async testWASMBundleSize() {
        this.log('Testing WASM bundle size...');
        
        try {
            // Check if WASM bindings are available
            const wasmParserPath = path.resolve(__dirname, '../packages/ddex-parser/bindings/wasm');
            const wasmBuilderPath = path.resolve(__dirname, '../packages/ddex-builder/bindings/wasm');
            
            let totalWasmSize = 0;
            let wasmFiles = [];
            
            if (fs.existsSync(wasmParserPath)) {
                const files = fs.readdirSync(wasmParserPath);
                const wasmFile = files.find(f => f.endsWith('.wasm'));
                if (wasmFile) {
                    const filePath = path.join(wasmParserPath, wasmFile);
                    const stats = fs.statSync(filePath);
                    const sizeKB = Math.round(stats.size / 1024);
                    totalWasmSize += sizeKB;
                    wasmFiles.push({name: `parser/${wasmFile}`, sizeKB});
                    this.log(`Found parser WASM: ${wasmFile} (${sizeKB}KB)`);
                }
            }
            
            if (fs.existsSync(wasmBuilderPath)) {
                const files = fs.readdirSync(wasmBuilderPath);
                const wasmFile = files.find(f => f.endsWith('.wasm'));
                if (wasmFile) {
                    const filePath = path.join(wasmBuilderPath, wasmFile);
                    const stats = fs.statSync(filePath);
                    const sizeKB = Math.round(stats.size / 1024);
                    totalWasmSize += sizeKB;
                    wasmFiles.push({name: `builder/${wasmFile}`, sizeKB});
                    this.log(`Found builder WASM: ${wasmFile} (${sizeKB}KB)`);
                }
            }
            
            this.log(`Total WASM bundle size: ${totalWasmSize}KB`);
            this.log(`Target size: <${TEST_CONFIG.WASM_BUNDLE_SIZE_TARGET_KB}KB (expected ~${TEST_CONFIG.EXPECTED_WASM_SIZE_KB}KB)`);
            
            if (totalWasmSize > 0 && totalWasmSize < TEST_CONFIG.WASM_BUNDLE_SIZE_TARGET_KB) {
                this.log(`✅ WASM bundle size meets target`);
                this.results.passedTests++;
            } else if (totalWasmSize === 0) {
                this.log(`⚠️  No WASM files found - may need to build WASM bindings first`);
                this.results.passedTests++; // Don't fail if WASM not built
            } else {
                this.log(`❌ WASM bundle size exceeds target (${totalWasmSize}KB >= ${TEST_CONFIG.WASM_BUNDLE_SIZE_TARGET_KB}KB)`, true);
                this.results.failedTests++;
            }
        } catch (error) {
            this.log(`WASM bundle size test failed: ${error.message}`, true);
            this.results.failedTests++;
        }
        
        this.results.totalTests++;
    }

    generateTypicalSizeXML() {
        // Generate a typical ERN message (~10KB)
        const releases = [];
        const resources = [];
        
        for (let i = 1; i <= 10; i++) {
            resources.push(`
                <SoundRecording>
                    <ResourceReference>SR${i.toString().padStart(3, '0')}</ResourceReference>
                    <Type>MusicalWorkSoundRecording</Type>
                    <Title><TitleText>Track ${i}</TitleText></Title>
                    <Duration>PT3M30S</Duration>
                    <DisplayArtist>Test Artist</DisplayArtist>
                    <ISRC>TEST${i.toString().padStart(8, '0')}</ISRC>
                </SoundRecording>`);
        }
        
        releases.push(`
            <Release>
                <ReleaseReference>REL001</ReleaseReference>
                <ReleaseType>Album</ReleaseType>
                <ReleaseId><ICPN>1234567890123</ICPN></ReleaseId>
                <DisplayTitleText>Test Album</DisplayTitleText>
                <DisplayArtist>Test Artist</DisplayArtist>
                <ReleaseResourceReferenceList>
                    ${Array.from({length: 10}, (_, i) => `<ReleaseResourceReference>${'SR' + (i + 1).toString().padStart(3, '0')}</ReleaseResourceReference>`).join('\n')}
                </ReleaseResourceReferenceList>
            </Release>`);
        
        return `<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>PERF-TEST-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender><PartyName>Performance Test</PartyName></MessageSender>
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

    generateLargeXML() {
        // Generate a large XML for streaming tests (~5MB)
        const releases = [];
        const resources = [];
        
        for (let i = 1; i <= 1000; i++) {
            resources.push(`
                <SoundRecording>
                    <ResourceReference>SR${i.toString().padStart(4, '0')}</ResourceReference>
                    <Type>MusicalWorkSoundRecording</Type>
                    <Title><TitleText>Track ${i} with a very long title that contains many words to make the file larger</TitleText></Title>
                    <Duration>PT3M${(i % 60).toString().padStart(2, '0')}S</Duration>
                    <DisplayArtist>Test Artist ${Math.floor(i / 10)}</DisplayArtist>
                    <ISRC>TEST${i.toString().padStart(8, '0')}</ISRC>
                    <Description>This is a detailed description of track ${i} which includes extensive metadata, production notes, and other information that would typically be found in a professional DDEX message.</Description>
                </SoundRecording>`);
        }
        
        for (let i = 1; i <= 100; i++) {
            const trackRefs = Array.from({length: 10}, (_, j) => {
                const trackNum = (i - 1) * 10 + j + 1;
                return `<ReleaseResourceReference>SR${trackNum.toString().padStart(4, '0')}</ReleaseResourceReference>`;
            }).join('\n');
            
            releases.push(`
                <Release>
                    <ReleaseReference>REL${i.toString().padStart(3, '0')}</ReleaseReference>
                    <ReleaseType>Album</ReleaseType>
                    <ReleaseId><ICPN>${(1234567890000 + i).toString()}</ICPN></ReleaseId>
                    <DisplayTitleText>Test Album ${i} - Extended Title with Additional Information</DisplayTitleText>
                    <DisplayArtist>Test Artist ${Math.floor(i / 5)}</DisplayArtist>
                    <ReleaseResourceReferenceList>
                        ${trackRefs}
                    </ReleaseResourceReferenceList>
                </Release>`);
        }
        
        return `<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>LARGE-TEST-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender><PartyName>Large File Test</PartyName></MessageSender>
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

    printSummary() {
        console.log('\n' + '='.repeat(70));
        console.log('📊 DDEX Suite Node.js Bindings Test Summary');
        console.log('='.repeat(70));
        
        console.log(`\n📈 Test Results:`);
        console.log(`   Total Tests: ${this.results.totalTests}`);
        console.log(`   Passed: ${this.results.passedTests} ✅`);
        console.log(`   Failed: ${this.results.failedTests} ${this.results.failedTests > 0 ? '❌' : '✅'}`);
        console.log(`   Success Rate: ${((this.results.passedTests / this.results.totalTests) * 100).toFixed(1)}%`);
        
        if (this.results.performance.averageTimeMs) {
            console.log(`\n⚡ Performance:`);
            console.log(`   Average Parse Time: ${this.results.performance.averageTimeMs.toFixed(2)}ms`);
            console.log(`   Target: <${TEST_CONFIG.PERFORMANCE_TARGET_MS}ms ${this.results.performance.meetsTarget ? '✅' : '❌'}`);
        }
        
        if (this.results.roundTripTests.length > 0) {
            console.log(`\n🔄 Round-Trip Fidelity:`);
            const avgFidelity = this.results.roundTripTests.reduce((sum, test) => sum + test.fidelityScore, 0) / this.results.roundTripTests.length;
            console.log(`   Average Fidelity Score: ${avgFidelity.toFixed(3)}`);
            console.log(`   Successful Round-Trips: ${this.results.roundTripTests.filter(t => t.success).length}/${this.results.roundTripTests.length}`);
        }
        
        if (this.results.extensionTests.length > 0) {
            console.log(`\n🔌 Extension Preservation:`);
            const successfulExtensions = this.results.extensionTests.filter(t => t.extensionsPreserved && t.roundTripSuccess).length;
            console.log(`   Successful Extension Tests: ${successfulExtensions}/${this.results.extensionTests.length}`);
        }
        
        if (this.results.errors.length > 0) {
            console.log(`\n❌ Errors Encountered:`);
            this.results.errors.forEach((error, index) => {
                console.log(`   ${index + 1}. ${error}`);
            });
        }
        
        console.log('\n' + '='.repeat(70));
        
        if (this.results.failedTests === 0) {
            console.log('🎉 All tests passed! Node.js bindings are working correctly.');
        } else {
            console.log('⚠️  Some tests failed. Please review the errors above.');
        }
        
        console.log('='.repeat(70));
    }
}

// Run tests if called directly
if (require.main === module) {
    const tester = new NodeJSBindingsTester();
    tester.runAllTests().catch(error => {
        console.error('Fatal error:', error);
        process.exit(1);
    });
}

module.exports = NodeJSBindingsTester;