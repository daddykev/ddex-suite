#!/usr/bin/env node

console.log('Testing parser exports...');

try {
    const parserModule = require('ddex-parser');
    console.log('Full parser module:', parserModule);
    console.log('Keys:', Object.keys(parserModule));
    
    if (parserModule.DDEXParser) {
        console.log('DDEXParser type:', typeof parserModule.DDEXParser);
        try {
            const parser = new parserModule.DDEXParser();
            console.log('✅ DDEXParser instantiated successfully');
            
            // Test a method
            try {
                const version = parser.detectVersion('<?xml version="1.0"?><test/>');
                console.log('Version detection result:', version);
            } catch (e) {
                console.log('Version detection error (expected):', e.message);
            }
        } catch (e) {
            console.log('❌ Failed to instantiate DDEXParser:', e.message);
        }
    }
    
    if (parserModule.default) {
        console.log('Default export type:', typeof parserModule.default);
        if (typeof parserModule.default === 'function') {
            try {
                const parser = new parserModule.default();
                console.log('✅ Default parser instantiated successfully');
            } catch (e) {
                console.log('❌ Failed to instantiate default parser:', e.message);
            }
        }
    }
    
} catch (error) {
    console.error('Parser test error:', error.message);
    console.error(error.stack);
}