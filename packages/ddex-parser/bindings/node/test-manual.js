// packages/ddex-parser/bindings/node/test-manual.js
/**
 * Quick manual test - run from bindings/node directory
 * node test-manual.js
 */

console.log('🧪 Manual Test for ddex-parser\n');

try {
  // Test the built package
  const { DDEXParser } = require('./dist/index.js');
  console.log('✓ Package loads');
  
  const parser = new DDEXParser();
  console.log('✓ Parser instantiated');
  
  console.log(`✓ Version: ${parser.version || 'not implemented'}`);
  
  console.log('\n✅ Basic checks passed!');
} catch (error) {
  console.error('❌ Error:', error.message);
  process.exit(1);
}