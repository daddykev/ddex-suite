#!/usr/bin/env node

console.log('Debugging Node.js bindings loading...');

try {
    console.log('Platform:', process.platform, 'Arch:', process.arch);
    
    // Test parser
    console.log('\n--- Testing Parser ---');
    const path = require('path');
    const fs = require('fs');
    
    const parserPath = '../packages/ddex-parser/bindings/node';
    const parserIndexPath = path.resolve(__dirname, parserPath, 'index.js');
    console.log('Parser path:', parserIndexPath);
    console.log('Parser exists:', fs.existsSync(parserIndexPath));
    
    const parserNodeFile = path.resolve(__dirname, parserPath, 'index.darwin-arm64.node');
    console.log('Parser native file:', parserNodeFile);
    console.log('Parser native exists:', fs.existsSync(parserNodeFile));
    
    if (fs.existsSync(parserNodeFile)) {
        try {
            const nativeBinding = require(parserNodeFile);
            console.log('Native parser binding loaded:', Object.keys(nativeBinding));
        } catch (e) {
            console.error('Failed to load native parser binding:', e.message);
        }
    }
    
    // Now try to load through the module
    const parserModule = require('ddex-parser');
    console.log('Parser module:', Object.keys(parserModule));
    console.log('DdexParser type:', typeof parserModule.DdexParser);
    
    // Test builder
    console.log('\n--- Testing Builder ---');
    const builderPath = '../packages/ddex-builder/bindings/node';
    const builderIndexPath = path.resolve(__dirname, builderPath, 'index.js');
    console.log('Builder path:', builderIndexPath);
    console.log('Builder exists:', fs.existsSync(builderIndexPath));
    
    const builderNodeFile = path.resolve(__dirname, builderPath, 'ddex-builder-node.darwin-arm64.node');
    console.log('Builder native file:', builderNodeFile);
    console.log('Builder native exists:', fs.existsSync(builderNodeFile));
    
    if (fs.existsSync(builderNodeFile)) {
        try {
            const nativeBinding = require(builderNodeFile);
            console.log('Native builder binding loaded:', Object.keys(nativeBinding));
        } catch (e) {
            console.error('Failed to load native builder binding:', e.message);
        }
    }
    
    const builderModule = require('ddex-builder');
    console.log('Builder module:', Object.keys(builderModule));
    console.log('DdexBuilder type:', typeof builderModule.DdexBuilder);
    
} catch (error) {
    console.error('Debug error:', error.message);
    console.error(error.stack);
}