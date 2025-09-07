// bindings/node/js/index.ts
import { DDEXParser as NativeDDEXParser } from '../index.node';

export interface ParseOptions {
  mode?: 'auto' | 'dom' | 'stream';
  autoThreshold?: number;
  resolveReferences?: boolean;
  includeRaw?: boolean;
  maxMemory?: number;
  timeoutMs?: number;
  allowBlocking?: boolean;
  chunkSize?: number;
}

export interface StreamOptions {
  chunkSize?: number;
  maxMemory?: number;
  onProgress?: (progress: ProgressInfo) => void;
}

export interface ProgressInfo {
  bytes: number;
  releases: number;
  elapsedMs: number;
}

export interface ParsedERNMessage {
  messageId: string;
  messageType: string;
  version: string;
  releaseCount: number;
  trackCount: number;
  dealCount: number;
  // Full types to be generated from Rust
}

/**
 * High-performance DDEX XML parser
 */
export class DDEXParser {
  private native: NativeDDEXParser;

  constructor() {
    this.native = new NativeDDEXParser();
  }

  /**
   * Parse DDEX XML asynchronously (recommended)
   */
  async parse(xml: string | Buffer, options?: ParseOptions): Promise<ParsedERNMessage> {
    return this.native.parse(xml, options);
  }

  /**
   * Parse DDEX XML synchronously
   * Warning: Not recommended for files >5MB unless allowBlocking is true
   */
  parseSync(xml: string | Buffer, options?: ParseOptions): ParsedERNMessage {
    const fileSize = Buffer.isBuffer(xml) ? xml.length : Buffer.byteLength(xml, 'utf8');
    
    if (fileSize > 5 * 1024 * 1024 && !options?.allowBlocking) {
      throw new Error(
        'Files larger than 5MB should use parse() or set allowBlocking: true. ' +
        `File size: ${(fileSize / 1024 / 1024).toFixed(2)}MB`
      );
    }
    
    return this.native.parseSync(xml, options);
  }

  /**
   * Stream parse large DDEX files
   */
  async *stream(xml: string | Buffer, options?: StreamOptions): AsyncIterableIterator<ParsedRelease> {
    const stream = this.native.stream(xml, options);
    
    // Implement backpressure-aware iteration
    while (true) {
      const release = await stream.next();
      if (!release) break;
      
      if (options?.onProgress) {
        // Track progress
        options.onProgress({
          bytes: 0, // TODO: Track actual bytes
          releases: 1,
          elapsedMs: Date.now(),
        });
      }
      
      yield release;
    }
  }

  /**
   * Perform structural sanity check
   */
  async sanityCheck(xml: string | Buffer): Promise<SanityCheckResult> {
    return this.native.sanityCheck(xml);
  }

  /**
   * Detect DDEX version from XML
   */
  detectVersion(xml: string | Buffer): string {
    return this.native.detectVersion(xml);
  }
}

export interface SanityCheckResult {
  isValid: boolean;
  version: string;
  errors: string[];
  warnings: string[];
}

export interface ParsedRelease {
  releaseId: string;
  title: string;
  artist: string;
  trackCount: number;
}

// Re-export types generated from Rust
export * from './types';