const { DdexBuilder } = require('./index.js');

console.log('Testing DDEX Builder build...');
try {
  const builder = new DdexBuilder();
  console.log('✅ DdexBuilder instantiated successfully');
  console.log('Builder methods:', Object.getOwnPropertyNames(Object.getPrototypeOf(builder)));
} catch (error) {
  console.error('❌ Error:', error.message);
  process.exit(1);
}
