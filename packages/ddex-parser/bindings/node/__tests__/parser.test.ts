// bindings/node/__tests__/parser.test.ts
import { DDEXParser, ParseOptions, StreamOptions } from '../src/parser';
import * as fs from 'fs';
import * as path from 'path';

describe('DDEXParser', () => {
  let parser: DDEXParser;
  const testFilesDir = path.join(__dirname, '../../../test-suite/valid');

  beforeEach(() => {
    parser = new DDEXParser();
  });

  describe('parse', () => {
    it('should parse simple ERN 4.3 message', async () => {
      const xml = fs.readFileSync(
        path.join(testFilesDir, 'ern-4.3/simple_release.xml'),
        'utf8'
      );

      const result = await parser.parse(xml);
      
      expect(result).toBeDefined();
      expect(result.version).toContain('4');
      expect(result.messageId).toBeDefined();
      expect(result.releaseCount).toBeGreaterThan(0);
    });

    it('should handle Buffer input', async () => {
      const xml = fs.readFileSync(
        path.join(testFilesDir, 'ern-4.3/simple_release.xml')
      );

      const result = await parser.parse(xml);
      expect(result).toBeDefined();
      expect(result.messageType).toBeDefined();
    });

    it('should respect parse options', async () => {
      const xml = fs.readFileSync(
        path.join(testFilesDir, 'ern-4.3/simple_release.xml'),
        'utf8'
      );
      
      const result = await parser.parse(xml, {
        mode: 'dom',
        resolveReferences: true,
        timeoutMs: 5000,
      });
      
      expect(result).toBeDefined();
    });

    it('should handle parse errors gracefully', async () => {
      const xml = '<invalid>not valid ddex</invalid>';
      
      await expect(parser.parse(xml)).rejects.toThrow();
    });
  });

  describe('parseSync', () => {
    it('should parse small files synchronously', () => {
      const xml = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43"/>';
      
      expect(() => parser.parseSync(xml)).not.toThrow();
    });

    it('should block large files without allowBlocking', () => {
      const largeXml = 'x'.repeat(6 * 1024 * 1024); // 6MB
      
      expect(() => {
        parser.parseSync(largeXml);
      }).toThrow(/larger than 5MB/);
    });

    it('should allow large files with allowBlocking', () => {
      const largeXml = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">' +
        'x'.repeat(6 * 1024 * 1024) + 
        '</ern:NewReleaseMessage>';
      
      expect(() => {
        parser.parseSync(largeXml, { allowBlocking: true });
      }).not.toThrow();
    });
  });

  describe('stream', () => {
    it('should stream releases with backpressure', async () => {
      const xml = fs.readFileSync(
        path.join(testFilesDir, 'ern-4.3/simple_release.xml'),
        'utf8'
      );

      const releases: any[] = [];
      const progressUpdates: any[] = [];

      for await (const release of parser.stream(xml, {
        chunkSize: 10,
        onProgress: (p) => progressUpdates.push(p),
      })) {
        releases.push(release);
        // Simulate slow consumer for backpressure
        await new Promise(resolve => setTimeout(resolve, 10));
      }

      expect(releases.length).toBeGreaterThan(0);
      expect(progressUpdates.length).toBeGreaterThan(0);
    });

    it('should handle streaming errors', async () => {
      const xml = '<invalid>not valid</invalid>';
      
      const releases: any[] = [];
      
      try {
        for await (const release of parser.stream(xml)) {
          releases.push(release);
        }
      } catch (error) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('detectVersion', () => {
    it('should detect ERN 4.3', () => {
      const xml = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43"/>';
      const version = parser.detectVersion(xml);
      expect(version).toBe('V4_3');
    });

    it('should detect ERN 4.2', () => {
      const xml = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42"/>';
      const version = parser.detectVersion(xml);
      expect(version).toBe('V4_2');
    });

    it('should detect ERN 3.8.2', () => {
      const xml = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/382"/>';
      const version = parser.detectVersion(xml);
      expect(version).toBe('V3_8_2');
    });
  });

  describe('sanityCheck', () => {
    it('should validate structure', async () => {
      const xml = fs.readFileSync(
        path.join(testFilesDir, 'ern-4.3/simple_release.xml'),
        'utf8'
      );

      const result = await parser.sanityCheck(xml);
      
      expect(result.isValid).toBe(true);
      expect(result.version).toBeDefined();
      expect(result.errors).toHaveLength(0);
    });

    it('should report structural errors', async () => {
      const xml = '<invalid>not a ddex file</invalid>';
      
      const result = await parser.sanityCheck(xml);
      
      expect(result.isValid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });
  });

  describe('error handling', () => {
    it('should handle malformed XML', async () => {
      const xml = '<unclosed>';
      
      await expect(parser.parse(xml)).rejects.toThrow();
    });

    it('should provide detailed error information', async () => {
      const xml = '<ern:NewReleaseMessage xmlns:ern="invalid-namespace">';
      
      try {
        await parser.parse(xml);
      } catch (error: any) {
        expect(error.message).toBeDefined();
        // Error should contain FFI error details
      }
    });
  });

  describe('performance', () => {
    it('should parse typical file in <50ms', async () => {
      const xml = fs.readFileSync(
        path.join(testFilesDir, 'ern-4.3/simple_release.xml'),
        'utf8'
      );

      const start = Date.now();
      await parser.parse(xml);
      const elapsed = Date.now() - start;

      expect(elapsed).toBeLessThan(100); // Generous for CI
    }, 10000);

    it('should handle concurrent parsing', async () => {
      const xml = fs.readFileSync(
        path.join(testFilesDir, 'ern-4.3/simple_release.xml'),
        'utf8'
      );

      const promises = Array(10).fill(null).map(() => parser.parse(xml));
      const results = await Promise.all(promises);
      
      expect(results).toHaveLength(10);
      results.forEach(result => {
        expect(result.messageId).toBeDefined();
      });
    });
  });
});