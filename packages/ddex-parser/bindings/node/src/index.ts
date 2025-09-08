// packages/ddex-parser/bindings/node/src/index.ts
export interface DDEXParserOptions {
  includeRawExtensions?: boolean;
  includeComments?: boolean;
  validateReferences?: boolean;
  streaming?: boolean;
}

export class DDEXParser {
  private implementation: any;
  
  constructor() {
    if (typeof window !== 'undefined') {
      // Browser environment - use WASM
      this.implementation = this.loadWASM();
    } else {
      // Node.js environment - use native addon
      try {
        this.implementation = require('../index.node');
      } catch {
        // Fallback to WASM if native fails
        this.implementation = this.loadWASM();
      }
    }
  }
  
  private async loadWASM() {
    const wasm = await import('../wasm/ddex_parser');
    await wasm.default();
    return new wasm.DDEXParser();
  }
  
  async parse(xml: string, options?: DDEXParserOptions) {
    if (!this.implementation) {
      this.implementation = await this.loadWASM();
    }
    return this.implementation.parse(xml, options || {});
  }
  
  async stream(source: ReadableStream | NodeJS.ReadableStream, options?: DDEXParserOptions) {
    if (!this.implementation) {
      this.implementation = await this.loadWASM();
    }
    
    if (typeof window !== 'undefined') {
      // Browser streaming
      return this.implementation.parse_stream(source, options || {});
    } else {
      // Node.js streaming
      return this.implementation.streamParse(source, options || {});
    }
  }
}