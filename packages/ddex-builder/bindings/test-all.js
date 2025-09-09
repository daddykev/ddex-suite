const { DDEXBuilder } = require('./node');
const assert = require('assert');

async function testNodeBindings() {
    console.log('Testing Node.js bindings...');
    
    const builder = new DDEXBuilder();
    
    // Test basic build
    const request = {
        releases: [{
            releaseId: 'R1',
            title: 'Test Album',
            displayArtist: 'Test Artist',
            tracks: []
        }],
        profile: 'AudioAlbum',
        version: '4.3'
    };
    
    const result = await builder.build(request);
    assert(result.xml.includes('<?xml'));
    assert(result.xml.includes('Test Album'));
    
    console.log('✓ Node.js bindings working');
}

testNodeBindings().catch(console.error);