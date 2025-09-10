const { DdexBuilder } = require('./index.js');

async function testCompleteFix() {
    console.log('🎯 Testing Complete DDEX Builder Fix\n');
    
    // Test 1: User's expected API with object data
    console.log('Test 1: Object-based API (user\'s expected format)');
    const builder1 = new DdexBuilder();
    
    const data = {
        version: '4.3',
        releases: [{
            release_id: 'TEST001',
            title: 'Test Album',
            display_artist: 'Test Artist',
            label: 'Test Label',
            upc: '123456789012',
            release_date: '2024-01-01'
        }]
    };
    
    try {
        const xml1 = await builder1.build(data);
        console.log('✅ Build with object succeeded');
        console.log(`  XML length: ${xml1.length} bytes`);
        
        // Verify content
        const checks = [
            { field: 'Release ID', value: 'TEST001', present: xml1.includes('TEST001') },
            { field: 'Title', value: 'Test Album', present: xml1.includes('Test Album') },
            { field: 'Artist', value: 'Test Artist', present: xml1.includes('Test Artist') },
            { field: 'Label', value: 'Test Label', present: xml1.includes('Test Label') },
            { field: 'UPC', value: '123456789012', present: xml1.includes('123456789012') },
            { field: 'Release Date', value: '2024-01-01', present: xml1.includes('2024-01-01') }
        ];
        
        let allPresent = true;
        for (const check of checks) {
            if (check.present) {
                console.log(`  ✅ ${check.field}: ${check.value}`);
            } else {
                console.log(`  ❌ ${check.field}: ${check.value} (MISSING)`);
                allPresent = false;
            }
        }
        
        if (allPresent) {
            console.log('🎉 SUCCESS: All release data is present in XML!\n');
        } else {
            console.log('❌ FAILURE: Some release data is missing\n');
        }
        
    } catch (error) {
        console.log('❌ Object-based build failed:', error.message);
    }
    
    // Test 2: Traditional API with addRelease/addResource
    console.log('Test 2: Traditional API (existing format)');
    const builder2 = new DdexBuilder();
    
    const release = {
        releaseId: 'R002',
        releaseType: 'Album',
        title: 'Traditional Album',
        artist: 'Traditional Artist',
        label: 'Traditional Label',
        upc: '999888777666',
        releaseDate: '2024-02-01',
        genre: 'Electronic',
        parentalWarning: false,
        trackIds: ['T001'],
        metadata: { description: 'Test album' }
    };
    
    const resource = {
        resourceId: 'T001',
        resourceType: 'SoundRecording',
        title: 'Traditional Track',
        artist: 'Traditional Artist',
        isrc: 'USRC17607839',
        duration: 'PT3M30S',
        trackNumber: 1,
        volumeNumber: 1
    };
    
    try {
        builder2.addRelease(release);
        builder2.addResource(resource);
        
        const xml2 = await builder2.build(); // No data parameter
        console.log('✅ Traditional build succeeded');
        console.log(`  XML length: ${xml2.length} bytes`);
        
        const traditionalChecks = [
            { field: 'Release ID', value: 'R002', present: xml2.includes('R002') },
            { field: 'Title', value: 'Traditional Album', present: xml2.includes('Traditional Album') },
            { field: 'Artist', value: 'Traditional Artist', present: xml2.includes('Traditional Artist') },
            { field: 'Resource', value: 'T001', present: xml2.includes('T001') }
        ];
        
        let allTraditionalPresent = true;
        for (const check of traditionalChecks) {
            if (check.present) {
                console.log(`  ✅ ${check.field}: ${check.value}`);
            } else {
                console.log(`  ❌ ${check.field}: ${check.value} (MISSING)`);
                allTraditionalPresent = false;
            }
        }
        
        if (allTraditionalPresent) {
            console.log('🎉 SUCCESS: Traditional API working correctly!\n');
        } else {
            console.log('❌ FAILURE: Traditional API has issues\n');
        }
        
    } catch (error) {
        console.log('❌ Traditional build failed:', error.message);
    }
    
    // Summary
    console.log('🔧 Fix Summary:');
    console.log('• Fixed Node.js binding to use actual DDEX builder instead of placeholder');
    console.log('• Added support for object-based API: builder.build(data)');
    console.log('• Maintained compatibility with existing API: addRelease/addResource + build()'); 
    console.log('• Enhanced Rust generator to include title, artist, label, UPC, and date');
    console.log('• XML now contains complete release data instead of minimal headers');
    console.log('• Output size increased from ~500 bytes to 1000+ bytes with actual content');
}

testCompleteFix().catch(console.error);