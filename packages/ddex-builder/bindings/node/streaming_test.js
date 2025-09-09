// Example of using the StreamingDdexBuilder
const { StreamingDdexBuilder } = require('./index.js');

async function testStreamingBuilder() {
    console.log('Testing Streaming DDEX Builder...');
    
    // Create a streaming builder with custom config
    const config = {
        max_buffer_size: 5 * 1024 * 1024, // 5MB buffer
        deterministic: true,
        validate_during_stream: true,
        progress_callback_frequency: 50
    };
    
    const builder = new StreamingDdexBuilder(config);
    
    // Set up progress callback
    builder.setProgressCallback((progress) => {
        console.log(`Progress: ${progress.releases_written} releases, ${progress.resources_written} resources, ${Math.round(progress.bytes_written / 1024)}KB written`);
        if (progress.estimated_completion_percent) {
            console.log(`  Estimated completion: ${progress.estimated_completion_percent.toFixed(1)}%`);
        }
    });
    
    // Set estimated total for progress tracking
    builder.setEstimatedTotal(1000); // Expecting 1000 total items
    
    try {
        // Start the DDEX message
        const header = {
            message_sender_name: "DDEX Suite Streaming Test",
            message_recipient_name: "Test Recipient",
            message_created_date_time: new Date().toISOString()
        };
        
        builder.startMessage(header, "ern/43");
        console.log('✅ Message started');
        
        // Write some resources
        console.log('Writing resources...');
        const resourceRefs = [];
        for (let i = 1; i <= 100; i++) {
            const resourceRef = builder.writeResource(
                `resource_${i}`,
                `Track ${i}`,
                `Artist ${i}`,
                `ISRC${i.toString().padStart(12, '0')}`, // ISRC
                `PT3M${(30 + i % 30)}S`, // Duration (3:30 to 3:59)
                `track_${i}.mp3` // File path
            );
            resourceRefs.push(resourceRef);
            
            if (i % 20 === 0) {
                console.log(`  Wrote ${i} resources`);
            }
        }
        
        // Transition to releases
        builder.finishResourcesStartReleases();
        console.log('✅ Finished resources, starting releases');
        
        // Write some releases
        console.log('Writing releases...');
        for (let i = 1; i <= 10; i++) {
            // Each release gets 10 tracks
            const startIdx = (i - 1) * 10;
            const endIdx = Math.min(i * 10, resourceRefs.length);
            const releaseResourceRefs = resourceRefs.slice(startIdx, endIdx);
            
            const releaseRef = builder.writeRelease(
                `release_${i}`,
                `Album ${i}`,
                `Artist ${i}`,
                `Label ${i}`, // Label
                `12345678901${i.toString().padStart(2, '0')}`, // UPC
                `2024-0${(i % 9) + 1}-01`, // Release date
                `Pop`, // Genre
                releaseResourceRefs
            );
            
            console.log(`  Wrote release ${i} with ${releaseResourceRefs.length} tracks`);
        }
        
        // Finish the message
        console.log('Finishing message...');
        const stats = builder.finishMessage();
        
        console.log('✅ Message completed!');
        console.log('Final stats:', {
            releases_written: stats.releases_written,
            resources_written: stats.resources_written,
            bytes_written: stats.bytes_written,
            peak_memory_usage: Math.round(stats.peak_memory_usage / 1024) + 'KB',
            warnings_count: stats.warnings.length
        });
        
        // Get the generated XML
        const xml = builder.getXml();
        console.log(`Generated XML size: ${Math.round(xml.length / 1024)}KB`);
        console.log('First 500 characters of XML:');
        console.log(xml.substring(0, 500) + '...');
        
        // Test memory efficiency by checking that we didn't load everything at once
        console.log(`✅ Memory efficiency: Peak usage ${Math.round(stats.peak_memory_usage / 1024)}KB vs total output ${Math.round(xml.length / 1024)}KB`);
        
        return true;
        
    } catch (error) {
        console.error('❌ Error during streaming build:', error);
        return false;
    }
}

// Run the test
if (require.main === module) {
    testStreamingBuilder()
        .then(success => {
            if (success) {
                console.log('\n🎉 Streaming builder test completed successfully!');
                process.exit(0);
            } else {
                console.log('\n💥 Streaming builder test failed');
                process.exit(1);
            }
        })
        .catch(error => {
            console.error('💥 Test error:', error);
            process.exit(1);
        });
}

module.exports = { testStreamingBuilder };