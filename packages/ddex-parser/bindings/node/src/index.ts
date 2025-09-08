// packages/ddex-parser/bindings/node/src/index.ts
/// <reference path="./wasm.d.ts" />

export interface DDEXParserOptions {
  includeRawExtensions?: boolean;
  includeComments?: boolean;
  validateReferences?: boolean;
  streaming?: boolean;
}

export interface ParseResult {
  graph: any;  // TODO: Add proper types
  flat: any;   // TODO: Add proper types
}

declare global {
  interface Window {}
}

export class DDEXParser {
  private implementation: any = null;
  private implementationType: 'native' | 'wasm' | null = null;
  
  constructor() {
    // Check if we're in a browser environment
    const isBrowser = typeof window !== 'undefined' && typeof window.document !== 'undefined';
    
    if (!isBrowser) {
      // Try to load native binding in Node.js
      try {
        // This will fail for now since we don't have native bindings yet
        this.implementation = require('../build/Release/ddex_parser.node');
        this.implementationType = 'native';
      } catch (e) {
        // Expected for now - native binding not built yet
        console.debug('Native binding not found, will use WASM fallback when available');
      }
    }
  }
  
  private async loadWASM(): Promise<void> {
    try {
      // Use require for optional dependencies
      const wasmModule = await new Promise<any>((resolve, reject) => {
        try {
          resolve(require('../wasm/ddex_parser'));
        } catch (e) {
          reject(e);
        }
      });
      
      if (wasmModule.default) {
        await wasmModule.default(); // Initialize WASM
      }
      this.implementation = new wasmModule.DDEXParser();
      this.implementationType = 'wasm';
    } catch (e) {
      // For now, create a mock implementation
      console.debug('WASM not available yet, using mock implementation');
      this.implementation = {
        parse: async (xml: string, options: any) => ({
          graph: { messageHeader: {}, parties: [], resources: [], releases: [] },
          flat: { releases: [] }
        }),
        parse_stream: async (stream: any, options: any) => ({
          graph: { messageHeader: {}, parties: [], resources: [], releases: [] },
          flat: { releases: [] }
        }),
        version: () => '0.1.0-mock'
      };
      this.implementationType = 'wasm';
    }
  }
  
  private async ensureImplementation(): Promise<void> {
    if (!this.implementation) {
      await this.loadWASM();
    }
  }
  
  async parse(xml: string, options?: DDEXParserOptions): Promise<ParseResult> {
    await this.ensureImplementation();
    return this.implementation.parse(xml, options || {});
  }
  
  async stream(
    source: ReadableStream | NodeJS.ReadableStream,
    options?: DDEXParserOptions
  ): Promise<ParseResult> {
    await this.ensureImplementation();
    
    const isBrowser = typeof window !== 'undefined';
    
    if (isBrowser) {
      // Browser streaming with WASM
      return this.implementation.parse_stream(source, options || {});
    } else {
      // Node.js streaming
      if (this.implementationType === 'native' && this.implementation.streamParse) {
        return this.implementation.streamParse(source, options || {});
      } else {
        // WASM in Node.js
        return this.implementation.parse_stream(source, options || {});
      }
    }
  }
  
  get version(): string {
    if (!this.implementation) {
      return '0.1.0';
    }
    return this.implementation.version?.() || '0.1.0';
  }
}

// Export a default parser instance for convenience
export default DDEXParser;