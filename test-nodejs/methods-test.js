#!/usr/bin/env node

console.log('Testing available methods...');

try {
    const { DDEXParser } = require('ddex-parser');
    const { DdexBuilder } = require('ddex-builder');
    
    const parser = new DDEXParser();
    const builder = new DdexBuilder();
    
    console.log('Parser methods:');
    console.log(Object.getOwnPropertyNames(Object.getPrototypeOf(parser)));
    console.log('Parser prototype methods:');
    console.log(Object.getOwnPropertyNames(DDEXParser.prototype));
    
    console.log('\nBuilder methods:');  
    console.log(Object.getOwnPropertyNames(Object.getPrototypeOf(builder)));
    console.log('Builder prototype methods:');
    console.log(Object.getOwnPropertyNames(DdexBuilder.prototype));
    
} catch (error) {
    console.error('Methods test error:', error.message);
}