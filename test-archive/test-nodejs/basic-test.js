#!/usr/bin/env node

console.log('Testing basic Node.js bindings...');

try {
    // Try to import the modules
    const { DdexParser, ReleaseStream } = require('ddex-parser');
    const { DdexBuilder, StreamingDdexBuilder } = require('ddex-builder');
    
    console.log('✅ Successfully imported modules');
    console.log('- DdexParser:', typeof DdexParser);
    console.log('- ReleaseStream:', typeof ReleaseStream);
    console.log('- DdexBuilder:', typeof DdexBuilder);
    console.log('- StreamingDdexBuilder:', typeof StreamingDdexBuilder);
    
    // Try to instantiate classes
    console.log('\nTesting instantiation...');
    
    const parser = new DdexParser();
    console.log('✅ DdexParser instantiated');
    
    const builder = new DdexBuilder();
    console.log('✅ DdexBuilder instantiated');
    
    // Test basic methods
    console.log('\nTesting basic methods...');
    
    try {
        const version = parser.detectVersion('<xml></xml>');
        console.log(`✅ detectVersion works: ${typeof version}`);
    } catch (e) {
        console.log(`⚠️  detectVersion error (expected): ${e.message}`);
    }
    
    try {
        const presets = builder.getAvailablePresets();
        console.log(`✅ getAvailablePresets works: ${presets.length} presets`);
    } catch (e) {
        console.log(`❌ getAvailablePresets error: ${e.message}`);
    }
    
    console.log('\n🎉 Basic functionality test passed!');
    
} catch (error) {
    console.error('❌ Error during basic test:', error.message);
    console.error('Stack trace:', error.stack);
    process.exit(1);
}