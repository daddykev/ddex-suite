# Error Handling

Implement robust error handling patterns for production DDEX parsing applications.

## Error Types

DDEX Suite provides specific error types for different failure scenarios:

```typescript
try {
  const result = await parser.parse(xmlContent);
} catch (error) {
  switch (error.code) {
    case 'INVALID_XML':
      // Malformed XML
      handleXmlError(error);
      break;
    
    case 'DDEX_VALIDATION_ERROR':  
      // DDEX schema validation failed
      handleValidationError(error);
      break;
    
    case 'REFERENCE_ERROR':
      // Internal reference validation failed
      handleReferenceError(error);
      break;
    
    case 'MEMORY_ERROR':
      // Out of memory during processing
      handleMemoryError(error);
      break;
    
    case 'TIMEOUT_ERROR':
      // Processing timeout
      handleTimeoutError(error);
      break;
    
    default:
      // Unknown error
      handleUnknownError(error);
  }
}
```

## Detailed Error Information

Access comprehensive error details for debugging:

```typescript
try {
  const result = await parser.parse(xmlContent);
} catch (error) {
  console.log('Error Code:', error.code);
  console.log('Message:', error.message);
  console.log('Line:', error.line);
  console.log('Column:', error.column);
  console.log('Context:', error.context);
  console.log('Suggestions:', error.suggestions);
  
  if (error.details) {
    console.log('Validation Details:', error.details);
  }
}
```

## Validation Error Handling

Handle DDEX validation errors with specific recovery strategies:

```typescript
function handleValidationError(error: DDEXValidationError) {
  console.log(`Validation failed: ${error.message}`);
  
  error.details.forEach(detail => {
    console.log(`Field: ${detail.field}`);
    console.log(`Issue: ${detail.issue}`);
    console.log(`Expected: ${detail.expected}`);
    console.log(`Actual: ${detail.actual}`);
    
    // Attempt automatic fixes
    if (detail.canAutoFix) {
      console.log(`Auto-fix available: ${detail.autoFixDescription}`);
    }
  });
  
  // Return partially parsed data if available
  if (error.partialResult) {
    console.log('Using partial result for processing...');
    return error.partialResult;
  }
  
  throw error; // Re-throw if no recovery possible
}
```

## Graceful Degradation

Continue processing when encountering non-critical errors:

```typescript
const parser = new DDEXParser();

const result = await parser.parse(xmlContent, {
  errorStrategy: 'continue',    // Continue on non-critical errors
  maxErrors: 10,               // Stop after 10 errors
  collectErrors: true          // Collect all errors for review
});

// Check if errors occurred during parsing
if (result.hasErrors) {
  console.log(`Parsing completed with ${result.errors.length} errors:`);
  
  result.errors.forEach(error => {
    if (error.severity === 'warning') {
      console.warn(`Warning: ${error.message} at line ${error.line}`);
    } else {
      console.error(`Error: ${error.message} at line ${error.line}`);
    }
  });
  
  // Data is still usable despite errors
  console.log(`Successfully parsed ${result.flat.releases.length} releases`);
}
```

## Retry Logic

Implement sophisticated retry mechanisms for transient failures:

```typescript
class DDEXParserWithRetry {
  private parser = new DDEXParser();
  private readonly maxRetries = 3;
  private readonly baseDelay = 1000; // 1 second

  async parseWithRetry(xmlContent: string, options: any = {}): Promise<any> {
    for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
      try {
        return await this.parser.parse(xmlContent, options);
      } catch (error) {
        console.log(`Attempt ${attempt} failed: ${error.message}`);
        
        if (!this.isRetryableError(error) || attempt === this.maxRetries) {
          throw error;
        }
        
        // Exponential backoff
        const delay = this.baseDelay * Math.pow(2, attempt - 1);
        console.log(`Retrying in ${delay}ms...`);
        await this.sleep(delay);
      }
    }
  }

  private isRetryableError(error: any): boolean {
    const retryableCodes = ['TIMEOUT_ERROR', 'NETWORK_ERROR', 'MEMORY_ERROR'];
    return retryableCodes.includes(error.code);
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Usage
const robustParser = new DDEXParserWithRetry();
const result = await robustParser.parseWithRetry(xmlContent);
```

## Error Recovery Patterns

### Pattern 1: Fallback to Simplified Parsing

```typescript
async function parseWithFallback(xmlContent: string): Promise<any> {
  try {
    // Try full parsing with all validations
    return await parser.parse(xmlContent, {
      validateReferences: true,
      includeRawExtensions: true,
      strict: true
    });
  } catch (error) {
    console.warn('Full parsing failed, trying simplified parsing...');
    
    try {
      // Fallback to basic parsing
      return await parser.parse(xmlContent, {
        validateReferences: false,
        includeRawExtensions: false,
        strict: false,
        errorStrategy: 'continue'
      });
    } catch (fallbackError) {
      console.error('All parsing attempts failed');
      throw fallbackError;
    }
  }
}
```

### Pattern 2: Progressive Enhancement

```typescript
async function parseProgressively(xmlContent: string): Promise<any> {
  const options = {
    validateReferences: false,
    includeRawExtensions: false,
    strict: false
  };

  // Start with minimal parsing
  let result = await parser.parse(xmlContent, options);
  
  // Progressively enhance if basic parsing succeeds
  if (result) {
    try {
      // Add reference validation
      options.validateReferences = true;
      result = await parser.parse(xmlContent, options);
      
      // Add extension parsing
      options.includeRawExtensions = true;
      result = await parser.parse(xmlContent, options);
      
      // Enable strict mode
      options.strict = true;
      result = await parser.parse(xmlContent, options);
      
    } catch (enhancementError) {
      console.warn('Could not apply all enhancements:', enhancementError.message);
      // Use the last successful result
    }
  }
  
  return result;
}
```

## Production Error Handling

Complete production-ready error handling system:

```typescript
import { createLogger } from 'winston';

class ProductionDDEXParser {
  private parser = new DDEXParser();
  private logger = createLogger({
    level: 'info',
    format: winston.format.json(),
    transports: [
      new winston.transports.File({ filename: 'ddex-errors.log', level: 'error' }),
      new winston.transports.File({ filename: 'ddex-combined.log' })
    ]
  });

  async parse(xmlContent: string, metadata: any = {}): Promise<any> {
    const startTime = Date.now();
    const parseId = this.generateParseId();
    
    try {
      this.logger.info('Starting DDEX parse', {
        parseId,
        contentLength: xmlContent.length,
        metadata
      });

      const result = await this.parser.parse(xmlContent, {
        errorStrategy: 'continue',
        maxErrors: 50,
        collectErrors: true,
        timeout: 30000 // 30 second timeout
      });

      // Log warnings for partial failures
      if (result.hasErrors) {
        this.logger.warn('Parse completed with errors', {
          parseId,
          errorCount: result.errors.length,
          warningCount: result.warnings?.length || 0
        });
      }

      this.logger.info('Parse completed successfully', {
        parseId,
        duration: Date.now() - startTime,
        releaseCount: result.flat.releases?.length || 0
      });

      return result;

    } catch (error) {
      this.logger.error('Parse failed completely', {
        parseId,
        error: error.message,
        stack: error.stack,
        duration: Date.now() - startTime,
        metadata
      });

      // Send error to monitoring system
      await this.reportError(error, { parseId, metadata });
      
      throw error;
    }
  }

  private generateParseId(): string {
    return `parse_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private async reportError(error: any, context: any): Promise<void> {
    // Report to error tracking service (e.g., Sentry, Rollbar)
    // await errorReporter.captureException(error, context);
  }
}
```

## Python Error Handling

```python
import logging
from typing import Optional
from ddex_parser import DDEXParser, DDEXError

class RobustDDEXParser:
    def __init__(self):
        self.parser = DDEXParser()
        self.logger = logging.getLogger(__name__)
        
    def parse_with_recovery(self, xml_content: str) -> Optional[dict]:
        try:
            # Try full parsing first
            return self.parser.parse(xml_content, strict=True)
        except DDEXError as e:
            self.logger.warning(f"Strict parsing failed: {e}")
            
            try:
                # Fallback to lenient parsing
                return self.parser.parse(xml_content, 
                                       strict=False,
                                       continue_on_error=True)
            except Exception as fallback_error:
                self.logger.error(f"All parsing attempts failed: {fallback_error}")
                return None

    def parse_batch_with_errors(self, xml_files: list) -> tuple:
        successful = []
        failed = []
        
        for file_path in xml_files:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                result = self.parse_with_recovery(content)
                if result:
                    successful.append((file_path, result))
                else:
                    failed.append((file_path, "Parse failed"))
            except Exception as e:
                failed.append((file_path, str(e)))
                
        return successful, failed
```

## Testing Error Scenarios

Create comprehensive tests for error conditions:

```typescript
describe('DDEX Parser Error Handling', () => {
  let parser: DDEXParser;

  beforeEach(() => {
    parser = new DDEXParser();
  });

  test('should handle malformed XML', async () => {
    const invalidXml = '<invalid><unclosed>';
    
    await expect(parser.parse(invalidXml))
      .rejects
      .toThrow(expect.objectContaining({
        code: 'INVALID_XML'
      }));
  });

  test('should handle missing required fields', async () => {
    const incompleteXml = `
      <NewReleaseMessage>
        <!-- Missing MessageHeader -->
        <ReleaseList>
          <Release></Release>
        </ReleaseList>
      </NewReleaseMessage>
    `;
    
    await expect(parser.parse(incompleteXml))
      .rejects
      .toThrow(expect.objectContaining({
        code: 'DDEX_VALIDATION_ERROR'
      }));
  });

  test('should continue on non-critical errors', async () => {
    const xmlWithWarnings = getXmlWithMinorIssues();
    
    const result = await parser.parse(xmlWithWarnings, {
      errorStrategy: 'continue'
    });
    
    expect(result).toBeDefined();
    expect(result.hasErrors).toBe(true);
    expect(result.errors.length).toBeGreaterThan(0);
  });
});
```

## Monitoring and Alerting

Set up monitoring for production error patterns:

```typescript
class DDEXErrorMonitor {
  private errorCounts = new Map<string, number>();
  private readonly alertThreshold = 10;

  recordError(error: any, context: any): void {
    const errorKey = `${error.code}_${context.source}`;
    const count = this.errorCounts.get(errorKey) || 0;
    this.errorCounts.set(errorKey, count + 1);

    // Alert if threshold exceeded
    if (count >= this.alertThreshold) {
      this.sendAlert({
        errorType: error.code,
        count: count + 1,
        message: error.message,
        context
      });
    }
  }

  private async sendAlert(alertData: any): Promise<void> {
    // Send alert to monitoring system
    console.error('ALERT: High error count detected', alertData);
  }
}
```

## Next Steps

- [Performance Optimization](./performance) - Optimize error-prone operations
- [Large File Processing](./large-files) - Error handling in streaming scenarios
- [Advanced Memory Management](../advanced/memory) - Handle memory-related errors