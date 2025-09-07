// bindings/node/__tests__/basic.test.ts
describe('DDEXParser Basic', () => {
  let DDEXParser: any;
  
  beforeAll(() => {
    // Load the compiled JavaScript
    DDEXParser = require('../dist/parser').DDEXParser;
  });

  it('should create parser instance', () => {
    const parser = new DDEXParser();
    expect(parser).toBeDefined();
  });

  it('should detect version', () => {
    const parser = new DDEXParser();
    const xml = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43"/>';
    const version = parser.detectVersion(xml);
    expect(version).toBe('V4_3');
  });
});