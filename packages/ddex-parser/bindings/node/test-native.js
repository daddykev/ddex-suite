// bindings/node/test-native.js
try {
  const binding = require('./ddex-parser.darwin-arm64.node');
  console.log('✅ Native module loaded successfully');
  console.log('Exported items:', Object.keys(binding));
  console.log('Full binding object:', binding);
  
  // Try to see what's actually there
  for (const key in binding) {
    console.log(`  ${key}:`, typeof binding[key]);
  }
} catch (error) {
  console.error('❌ Error:', error.message);
  console.error(error.stack);
}