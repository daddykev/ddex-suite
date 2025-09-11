# TypeScript API Reference

Complete API documentation for the DDEX Parser TypeScript/JavaScript bindings.

## Installation

```bash
npm install ddex-parser
```

## Imports

```typescript
import { DDEXParser, DDEXParserOptions, ParseResult } from 'ddex-parser';

// Or using require (CommonJS)
const { DDEXParser } = require('ddex-parser');
```

## Classes

### DDEXParser

High-level parser class with automatic WASM/Native detection.

```typescript
class DDEXParser {
  constructor();
  parse(xml: string, options?: DDEXParserOptions): Promise<ParseResult>;
  stream(source: ReadableStream | NodeJS.ReadableStream, options?: DDEXParserOptions): Promise<ParseResult>;
  get version(): string;
}
```

#### Constructor

```typescript
const parser = new DDEXParser();
```

Creates a new DDEX parser instance. The parser automatically detects the best available implementation (native or WASM).

#### parse()

```typescript
parse(xml: string, options?: DDEXParserOptions): Promise<ParseResult>
```

Parses DDEX XML content and returns structured data.

**Parameters:**
- `xml: string` - The DDEX XML content to parse
- `options?: DDEXParserOptions` - Optional parsing configuration

**Returns:** `Promise<ParseResult>` - Parsed DDEX message data

**Example:**
```typescript
const parser = new DDEXParser();
const result = await parser.parse(xmlContent, {
  includeRawExtensions: true,
  validateReferences: true
});

console.log(`Parsed ${result.flat.releases.length} releases`);
```

#### stream() {#streaming}

```typescript
stream(source: ReadableStream | NodeJS.ReadableStream, options?: DDEXParserOptions): Promise<ParseResult>
```

Streams DDEX XML from a readable source for memory-efficient processing.

**Parameters:**
- `source: ReadableStream | NodeJS.ReadableStream` - Input stream
- `options?: DDEXParserOptions` - Optional parsing configuration

**Returns:** `Promise<ParseResult>` - Parsed DDEX message data

**Example:**
```typescript
import { createReadStream } from 'fs';

const parser = new DDEXParser();
const stream = createReadStream('large-catalog.xml');
const result = await parser.stream(stream, {
  streaming: true
});
```

#### version

```typescript
get version(): string
```

Returns the parser version string.

**Example:**
```typescript
const parser = new DDEXParser();
console.log(`Parser version: ${parser.version}`);
```

---

### DdexParser (Native)

Low-level native parser class providing direct access to Rust implementation.

```typescript
class DdexParser {
  constructor();
  detectVersion(xml: string): string;
  parseSync(xml: string, options?: ParseOptions): ParsedMessage;
  parse(xml: string, options?: ParseOptions): Promise<ParsedMessage>;
  sanityCheck(xml: string): Promise<SanityCheckResult>;
  stream(xml: string, options?: StreamOptions): ReleaseStream;
}
```

#### Constructor

```typescript
const parser = new DdexParser();
```

Creates a new native parser instance.

#### detectVersion()

```typescript
detectVersion(xml: string): string
```

Detects the DDEX version from XML content.

**Parameters:**
- `xml: string` - DDEX XML content

**Returns:** `string` - Detected version (e.g., "4.3", "4.2", "3.8.2")

**Example:**
```typescript
const parser = new DdexParser();
const version = parser.detectVersion(xmlContent);
console.log(`Detected DDEX version: ${version}`);
```

#### parseSync()

```typescript
parseSync(xml: string, options?: ParseOptions): ParsedMessage
```

Synchronously parses DDEX XML content.

**Parameters:**
- `xml: string` - DDEX XML content
- `options?: ParseOptions` - Optional parsing configuration

**Returns:** `ParsedMessage` - Parsed message data

**Example:**
```typescript
const parser = new DdexParser();
const result = parser.parseSync(xmlContent, {
  includeRaw: true,
  resolveReferences: true
});
```

#### parse()

```typescript
parse(xml: string, options?: ParseOptions): Promise<ParsedMessage>
```

Asynchronously parses DDEX XML content.

**Parameters:**
- `xml: string` - DDEX XML content
- `options?: ParseOptions` - Optional parsing configuration

**Returns:** `Promise<ParsedMessage>` - Parsed message data

#### sanityCheck()

```typescript
sanityCheck(xml: string): Promise<SanityCheckResult>
```

Performs a quick validation check on DDEX XML.

**Parameters:**
- `xml: string` - DDEX XML content

**Returns:** `Promise<SanityCheckResult>` - Validation results

**Example:**
```typescript
const parser = new DdexParser();
const check = await parser.sanityCheck(xmlContent);
if (!check.isValid) {
  console.error('Validation errors:', check.errors);
}
```

#### stream()

```typescript
stream(xml: string, options?: StreamOptions): ReleaseStream
```

Creates a streaming parser for large DDEX files.

**Parameters:**
- `xml: string` - DDEX XML content
- `options?: StreamOptions` - Optional streaming configuration

**Returns:** `ReleaseStream` - Stream iterator for releases

**Example:**
```typescript
const parser = new DdexParser();
const stream = parser.stream(xmlContent, { chunkSize: 1024 });

let release;
while ((release = await stream.next()) !== null) {
  console.log(`Processing: ${release.title}`);
  
  // Check progress
  const progress = await stream.progress();
  console.log(`Progress: ${progress.elapsedMs}ms`);
}
```

---

### ReleaseStream

Iterator for streaming through releases in large DDEX files.

```typescript
class ReleaseStream {
  next(): Promise<StreamedRelease | null>;
  progress(): Promise<ProgressInfo>;
}
```

#### next()

```typescript
next(): Promise<StreamedRelease | null>
```

Gets the next release from the stream.

**Returns:** `Promise<StreamedRelease | null>` - Next release or null if finished

#### progress()

```typescript
progress(): Promise<ProgressInfo>
```

Gets current streaming progress information.

**Returns:** `Promise<ProgressInfo>` - Progress statistics

---

## Interfaces

### DDEXParserOptions

High-level parser configuration options.

```typescript
interface DDEXParserOptions {
  includeRawExtensions?: boolean;
  includeComments?: boolean;
  validateReferences?: boolean;
  streaming?: boolean;
}
```

#### Properties

- **`includeRawExtensions?: boolean`** (default: `false`)  
  Include raw XML for extension elements to preserve round-trip fidelity
  
- **`includeComments?: boolean`** (default: `false`)  
  Include XML comments in the parsed output
  
- **`validateReferences?: boolean`** (default: `true`)  
  Validate that all resource references are resolvable
  
- **`streaming?: boolean`** (default: `false`)  
  Enable streaming mode for large files

**Example:**
```typescript
const options: DDEXParserOptions = {
  includeRawExtensions: true,
  validateReferences: false,
  streaming: true
};
```

---

### ParseOptions

Native parser configuration options.

```typescript
interface ParseOptions {
  mode?: string;
  autoThreshold?: number;
  resolveReferences?: boolean;
  includeRaw?: boolean;
  maxMemory?: number;
  timeoutMs?: number;
  allowBlocking?: boolean;
  chunkSize?: number;
}
```

#### Properties

- **`mode?: string`** (default: `"auto"`)  
  Parsing mode: "auto", "streaming", or "blocking"
  
- **`autoThreshold?: number`** (default: `1048576`)  
  File size threshold for automatic streaming mode (bytes)
  
- **`resolveReferences?: boolean`** (default: `true`)  
  Resolve and validate resource references
  
- **`includeRaw?: boolean`** (default: `false`)  
  Include raw XML content for extensions
  
- **`maxMemory?: number`** (default: `134217728`)  
  Maximum memory usage for parsing (bytes)
  
- **`timeoutMs?: number`** (default: `30000`)  
  Parsing timeout in milliseconds
  
- **`allowBlocking?: boolean`** (default: `false`)  
  Allow blocking operations during parsing
  
- **`chunkSize?: number`** (default: `8192`)  
  Chunk size for streaming operations

---

### StreamOptions

Streaming parser configuration.

```typescript
interface StreamOptions {
  chunkSize?: number;
  maxMemory?: number;
}
```

#### Properties

- **`chunkSize?: number`** (default: `8192`)  
  Size of chunks for streaming operations
  
- **`maxMemory?: number`** (default: `67108864`)  
  Maximum memory usage for streaming (bytes)

---

### ParseResult

High-level parse result structure.

```typescript
interface ParseResult {
  graph: any;
  flat: any;
}
```

#### Properties

- **`graph: any`**  
  Faithful representation of the DDEX XML structure
  
- **`flat: any`**  
  Flattened, developer-friendly representation

**Example:**
```typescript
const result = await parser.parse(xmlContent);

// Access graph structure (faithful to XML)
console.log(result.graph.MessageHeader.MessageId);

// Access flat structure (developer-friendly)
console.log(result.flat.releases[0].title);
console.log(result.flat.soundRecordings[0].isrc);
```

---

### ParsedMessage

Native parser result structure.

```typescript
interface ParsedMessage {
  messageId: string;
  messageType: string;
  messageDate: string;
  senderName: string;
  senderId: string;
  recipientName: string;
  recipientId: string;
  version: string;
  profile?: string;
  releaseCount: number;
  trackCount: number;
  dealCount: number;
  resourceCount: number;
  totalDurationSeconds: number;
}
```

#### Properties

- **`messageId: string`** - Unique message identifier
- **`messageType: string`** - Type of DDEX message (e.g., "NewReleaseMessage")
- **`messageDate: string`** - Message creation date (ISO 8601)
- **`senderName: string`** - Name of the message sender
- **`senderId: string`** - Identifier of the message sender
- **`recipientName: string`** - Name of the message recipient
- **`recipientId: string`** - Identifier of the message recipient
- **`version: string`** - DDEX version (e.g., "4.3")
- **`profile?: string`** - Optional message profile
- **`releaseCount: number`** - Number of releases in the message
- **`trackCount: number`** - Number of tracks across all releases
- **`dealCount: number`** - Number of commercial deals
- **`resourceCount: number`** - Number of sound recordings and other resources
- **`totalDurationSeconds: number`** - Total duration of all audio content

---

### SanityCheckResult

Result of XML validation check.

```typescript
interface SanityCheckResult {
  isValid: boolean;
  version: string;
  errors: Array<string>;
  warnings: Array<string>;
}
```

#### Properties

- **`isValid: boolean`** - Whether the XML passes basic validation
- **`version: string`** - Detected DDEX version
- **`errors: Array<string>`** - List of validation errors
- **`warnings: Array<string>`** - List of validation warnings

---

### StreamedRelease

Individual release from streaming parser.

```typescript
interface StreamedRelease {
  releaseReference: string;
  title: string;
  releaseType?: string;
  resourceCount: number;
}
```

#### Properties

- **`releaseReference: string`** - Unique release identifier
- **`title: string`** - Release title
- **`releaseType?: string`** - Type of release (e.g., "Album", "Single")
- **`resourceCount: number`** - Number of resources in this release

---

### ProgressInfo

Streaming progress information.

```typescript
interface ProgressInfo {
  bytesProcessed: number;
  releasesParsed: number;
  elapsedMs: number;
}
```

#### Properties

- **`bytesProcessed: number`** - Number of bytes processed so far
- **`releasesParsed: number`** - Number of releases parsed so far
- **`elapsedMs: number`** - Elapsed time in milliseconds

---

## Error Handling

The parser throws standard JavaScript errors for various failure conditions:

```typescript
try {
  const result = await parser.parse(xmlContent);
} catch (error) {
  if (error.message.includes('Invalid XML')) {
    console.error('XML parsing failed:', error.message);
  } else if (error.message.includes('Unsupported version')) {
    console.error('Unsupported DDEX version:', error.message);
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

### Common Error Types

- **XML Parsing Errors**: Invalid or malformed XML structure
- **Schema Validation Errors**: DDEX schema violations
- **Reference Resolution Errors**: Unresolvable resource references
- **Memory Limit Errors**: File too large for available memory
- **Timeout Errors**: Parsing took longer than specified timeout

---

## Performance Tips

### Memory Management

```typescript
// For large files, use streaming
const parser = new DDEXParser();
const result = await parser.parse(largeXml, { streaming: true });

// Set memory limits for native parser
const nativeParser = new DdexParser();
const result = nativeParser.parseSync(xml, { maxMemory: 50 * 1024 * 1024 });
```

### Batch Processing

```typescript
// Process multiple files efficiently
const parser = new DDEXParser();
const results = await Promise.all(
  xmlFiles.map(xml => parser.parse(xml, { validateReferences: false }))
);
```

### Reference Resolution

```typescript
// Skip reference validation for faster parsing
const result = await parser.parse(xml, {
  validateReferences: false,
  includeRawExtensions: false
});
```