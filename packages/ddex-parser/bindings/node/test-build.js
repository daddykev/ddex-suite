const { DDEXParser } = require('./dist/index.js');

console.log('Testing DDEX Parser build...');
try {
  const parser = new DDEXParser();
  console.log('✅ DDEXParser instantiated successfully');
  console.log('Parser methods:', Object.getOwnPropertyNames(Object.getPrototypeOf(parser)));
} catch (error) {
  console.error('❌ Error:', error.message);
  process.exit(1);
}
