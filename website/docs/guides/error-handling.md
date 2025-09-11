# Error Handling Guide

Comprehensive error handling patterns for the DDEX Suite, covering all types of errors from validation failures to system-level issues.

## Problem Statement

DDEX processing involves complex XML parsing, validation against multiple schemas, and integration with external systems. Errors can occur at multiple levels:

- **Parse Errors**: Invalid XML, unsupported DDEX versions, malformed structure
- **Validation Errors**: Schema violations, missing required fields, invalid references
- **System Errors**: Memory exhaustion, file I/O failures, network timeouts
- **Business Logic Errors**: Duplicate releases, invalid metadata combinations
- **Integration Errors**: DSP API failures, database constraint violations

Proper error handling ensures robust production systems and provides clear feedback for debugging and resolution.

## Solution Approach

The DDEX Suite provides a comprehensive error hierarchy with structured error information, context preservation, and recovery strategies. This guide demonstrates patterns for handling each error type effectively.

## Error Hierarchy Overview

### TypeScript Error Types

```typescript
// Base error types
interface DDEXError {
  code: string;
  message: string;
  context?: Record<string, any>;
  cause?: Error;
  severity: 'fatal' | 'error' | 'warning' | 'info';
}

// Specific error types
interface ParseError extends DDEXError {
  line?: number;
  column?: number;
  xmlPath?: string;
}

interface ValidationError extends DDEXError {
  field: string;
  value?: any;
  expectedType?: string;
  validationRule: string;
}

interface SystemError extends DDEXError {
  systemCode: string;
  retryable: boolean;
  retryAfter?: number;
}
```

### Python Error Types

```python
from typing import Optional, Dict, Any, List
from enum import Enum

class ErrorSeverity(Enum):
    FATAL = "fatal"
    ERROR = "error"
    WARNING = "warning"
    INFO = "info"

class DDEXError(Exception):
    def __init__(
        self,
        message: str,
        code: str,
        context: Optional[Dict[str, Any]] = None,
        severity: ErrorSeverity = ErrorSeverity.ERROR,
        cause: Optional[Exception] = None
    ):
        super().__init__(message)
        self.code = code
        self.context = context or {}
        self.severity = severity
        self.cause = cause

class ParseError(DDEXError):
    def __init__(
        self,
        message: str,
        code: str = "PARSE_ERROR",
        line: Optional[int] = None,
        column: Optional[int] = None,
        xml_path: Optional[str] = None,
        **kwargs
    ):
        super().__init__(message, code, **kwargs)
        self.line = line
        self.column = column
        self.xml_path = xml_path
```

## Parse Error Handling

### Basic Parse Error Recovery

```typescript
import { DDEXParser, ParseError } from 'ddex-parser';

async function robustParse(xmlContent: string) {
  const parser = new DDEXParser();
  
  try {
    return await parser.parse(xmlContent);
  } catch (error) {
    if (error instanceof ParseError) {
      // Handle specific parse errors
      switch (error.code) {
        case 'INVALID_XML':
          console.error(`XML is malformed at line ${error.line}, column ${error.column}`);
          // Attempt XML repair or preprocessing
          return await attemptXmlRepair(xmlContent, error);
          
        case 'UNSUPPORTED_VERSION':
          console.error(`DDEX version ${error.context?.version} not supported`);
          // Try version detection and conversion
          return await convertToSupportedVersion(xmlContent);
          
        case 'SCHEMA_VIOLATION':
          console.error(`Schema validation failed: ${error.message}`);
          // Log for manual review
          await logForReview(xmlContent, error);
          throw error;
          
        default:
          console.error(`Unknown parse error: ${error.code}`);
          throw error;
      }
    }
    
    // Re-throw non-parse errors
    throw error;
  }
}

async function attemptXmlRepair(xmlContent: string, error: ParseError): Promise<any> {
  console.log('Attempting XML repair...');
  
  // Common repairs
  let repairedXml = xmlContent
    .replace(/&(?![a-zA-Z]+;)/g, '&amp;')  // Fix unescaped ampersands
    .replace(/<!\[CDATA\[.*?\]\]>/gs, (match) => {
      // Ensure CDATA sections are properly closed
      return match.endsWith(']]>') ? match : match + ']]>';
    });
  
  // Try parsing the repaired XML
  const parser = new DDEXParser();
  try {
    return await parser.parse(repairedXml);
  } catch (repairError) {
    console.error('XML repair failed, original error was more informative');
    throw error; // Throw original error
  }
}
```

### Python Parse Error Handling

```python
from ddex_parser import DDEXParser, ParseError, ValidationError
import logging
from typing import Optional

async def robust_parse(xml_content: str) -> Optional[dict]:
    parser = DDEXParser()
    
    try:
        return await parser.parse(xml_content)
    except ParseError as e:
        logging.error(f"Parse error {e.code}: {e.message}")
        
        if e.code == "INVALID_XML":
            # Attempt repair
            repaired = await attempt_xml_repair(xml_content, e)
            if repaired:
                return repaired
                
        elif e.code == "UNSUPPORTED_VERSION":
            # Version conversion
            converted = await convert_version(xml_content, e.context.get("version"))
            if converted:
                return await parser.parse(converted)
                
        # Log for manual review
        await log_parse_failure(xml_content, e)
        return None
        
    except Exception as e:
        logging.error(f"Unexpected error during parsing: {e}")
        raise

async def attempt_xml_repair(xml_content: str, error: ParseError) -> Optional[dict]:
    """Attempt common XML repairs"""
    import re
    
    # Fix common issues
    repaired = xml_content
    
    # Fix encoding issues
    repaired = re.sub(r'[^\x09\x0A\x0D\x20-\x7E]', '', repaired)
    
    # Fix unescaped characters
    repaired = repaired.replace('&', '&amp;').replace('&amp;amp;', '&amp;')
    repaired = repaired.replace('<', '&lt;').replace('&lt;?', '<?').replace('&lt;/', '</')
    
    # Try parsing repaired XML
    parser = DDEXParser()
    try:
        return await parser.parse(repaired)
    except Exception:
        return None
```

## Validation Error Handling

### Comprehensive Validation with Error Collection

```typescript
import { DDEXBuilder, ValidationError, ValidationResult } from 'ddex-builder';

interface ValidationReport {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationError[];
  summary: {
    totalErrors: number;
    criticalErrors: number;
    fixableErrors: number;
  };
}

async function validateWithReport(buildRequest: any): Promise<ValidationReport> {
  const builder = new DDEXBuilder();
  const errors: ValidationError[] = [];
  const warnings: ValidationError[] = [];
  
  try {
    // Enable comprehensive validation
    const result = await builder.validate(buildRequest, {
      validateReferences: true,
      checkBusinessRules: true,
      validateMetadata: true,
      allowPartialValidation: true
    });
    
    return {
      isValid: true,
      errors: [],
      warnings: [],
      summary: { totalErrors: 0, criticalErrors: 0, fixableErrors: 0 }
    };
    
  } catch (error) {
    if (error instanceof ValidationError) {
      // Collect all validation errors
      const allErrors = error.validationErrors || [error];
      
      for (const validationError of allErrors) {
        if (validationError.severity === 'fatal' || validationError.severity === 'error') {
          errors.push(validationError);
        } else {
          warnings.push(validationError);
        }
      }
      
      return {
        isValid: false,
        errors,
        warnings,
        summary: {
          totalErrors: errors.length + warnings.length,
          criticalErrors: errors.filter(e => e.severity === 'fatal').length,
          fixableErrors: errors.filter(e => isFixableError(e)).length
        }
      };
    }
    
    throw error;
  }
}

function isFixableError(error: ValidationError): boolean {
  const fixableCodes = [
    'MISSING_REQUIRED_FIELD',
    'INVALID_FORMAT',
    'DUPLICATE_REFERENCE',
    'INVALID_DATE_FORMAT'
  ];
  
  return fixableCodes.includes(error.code);
}

async function autoFixValidationErrors(
  buildRequest: any, 
  errors: ValidationError[]
): Promise<{ fixed: any; remainingErrors: ValidationError[] }> {
  let fixed = JSON.parse(JSON.stringify(buildRequest)); // Deep copy
  const remainingErrors: ValidationError[] = [];
  
  for (const error of errors) {
    try {
      switch (error.code) {
        case 'MISSING_REQUIRED_FIELD':
          fixed = await addMissingField(fixed, error);
          break;
          
        case 'INVALID_DATE_FORMAT':
          fixed = await fixDateFormat(fixed, error);
          break;
          
        case 'DUPLICATE_REFERENCE':
          fixed = await removeDuplicateReference(fixed, error);
          break;
          
        default:
          remainingErrors.push(error);
      }
    } catch (fixError) {
      console.warn(`Failed to auto-fix error ${error.code}: ${fixError.message}`);
      remainingErrors.push(error);
    }
  }
  
  return { fixed, remainingErrors };
}
```

### Python Validation Error Handling

```python
from dataclasses import dataclass
from typing import List, Dict, Any, Optional
from ddex_builder import DDEXBuilder, ValidationError

@dataclass
class ValidationReport:
    is_valid: bool
    errors: List[ValidationError]
    warnings: List[ValidationError]
    fixable_errors: List[ValidationError]
    critical_count: int

async def validate_with_comprehensive_report(build_request: Dict[str, Any]) -> ValidationReport:
    builder = DDEXBuilder()
    errors = []
    warnings = []
    
    try:
        await builder.validate(
            build_request,
            validate_references=True,
            check_business_rules=True,
            validate_metadata=True
        )
        
        return ValidationReport(
            is_valid=True,
            errors=[],
            warnings=[],
            fixable_errors=[],
            critical_count=0
        )
        
    except ValidationError as e:
        # Collect all validation errors
        all_errors = getattr(e, 'validation_errors', [e])
        
        for error in all_errors:
            if error.severity in ['fatal', 'error']:
                errors.append(error)
            else:
                warnings.append(error)
        
        fixable_errors = [e for e in errors if is_fixable_error(e)]
        critical_count = len([e for e in errors if e.severity == 'fatal'])
        
        return ValidationReport(
            is_valid=False,
            errors=errors,
            warnings=warnings,
            fixable_errors=fixable_errors,
            critical_count=critical_count
        )

def is_fixable_error(error: ValidationError) -> bool:
    fixable_codes = {
        'MISSING_REQUIRED_FIELD',
        'INVALID_FORMAT',
        'DUPLICATE_REFERENCE',
        'INVALID_DATE_FORMAT'
    }
    return error.code in fixable_codes

async def auto_fix_validation_errors(
    build_request: Dict[str, Any],
    errors: List[ValidationError]
) -> Dict[str, Any]:
    """Auto-fix common validation errors"""
    import copy
    fixed = copy.deepcopy(build_request)
    
    for error in errors:
        try:
            if error.code == 'MISSING_REQUIRED_FIELD':
                fixed = await add_missing_field(fixed, error)
            elif error.code == 'INVALID_DATE_FORMAT':
                fixed = await fix_date_format(fixed, error)
            elif error.code == 'DUPLICATE_REFERENCE':
                fixed = await remove_duplicate_reference(fixed, error)
        except Exception as fix_error:
            logging.warning(f"Failed to auto-fix {error.code}: {fix_error}")
    
    return fixed
```

## System Error Handling

### Retry Strategies and Circuit Breakers

```typescript
import { DDEXParser, SystemError } from 'ddex-parser';

class RetryConfig {
  maxRetries: number = 3;
  baseDelay: number = 1000; // ms
  maxDelay: number = 10000; // ms
  exponentialBackoff: boolean = true;
  retryableErrors: string[] = [
    'NETWORK_TIMEOUT',
    'MEMORY_PRESSURE',
    'TEMPORARY_UNAVAILABLE'
  ];
}

class CircuitBreaker {
  private failures = 0;
  private lastFailureTime = 0;
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';
  
  constructor(
    private threshold: number = 5,
    private timeout: number = 60000 // 1 minute
  ) {}
  
  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime < this.timeout) {
        throw new Error('Circuit breaker is OPEN');
      }
      this.state = 'HALF_OPEN';
    }
    
    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }
  
  private onSuccess() {
    this.failures = 0;
    this.state = 'CLOSED';
  }
  
  private onFailure() {
    this.failures++;
    this.lastFailureTime = Date.now();
    
    if (this.failures >= this.threshold) {
      this.state = 'OPEN';
    }
  }
}

async function parseWithRetry(
  xmlContent: string,
  config: RetryConfig = new RetryConfig()
): Promise<any> {
  const circuitBreaker = new CircuitBreaker();
  
  return await circuitBreaker.execute(async () => {
    return await retryOperation(
      () => new DDEXParser().parse(xmlContent),
      config
    );
  });
}

async function retryOperation<T>(
  operation: () => Promise<T>,
  config: RetryConfig
): Promise<T> {
  let lastError: Error;
  
  for (let attempt = 0; attempt <= config.maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;
      
      if (attempt === config.maxRetries) {
        break; // Final attempt failed
      }
      
      if (error instanceof SystemError && !config.retryableErrors.includes(error.code)) {
        throw error; // Non-retryable error
      }
      
      const delay = config.exponentialBackoff
        ? Math.min(config.baseDelay * Math.pow(2, attempt), config.maxDelay)
        : config.baseDelay;
      
      console.log(`Attempt ${attempt + 1} failed, retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError;
}
```

### Resource Management and Cleanup

```typescript
class ResourceManager {
  private resources: Set<any> = new Set();
  
  async withResource<T>(
    resourceFactory: () => Promise<any>,
    operation: (resource: any) => Promise<T>
  ): Promise<T> {
    let resource: any;
    
    try {
      resource = await resourceFactory();
      this.resources.add(resource);
      
      return await operation(resource);
    } catch (error) {
      console.error('Operation failed, cleaning up resources:', error);
      throw error;
    } finally {
      if (resource) {
        await this.cleanup(resource);
      }
    }
  }
  
  private async cleanup(resource: any) {
    try {
      this.resources.delete(resource);
      
      if (resource.close) {
        await resource.close();
      } else if (resource.destroy) {
        await resource.destroy();
      }
    } catch (cleanupError) {
      console.error('Cleanup failed:', cleanupError);
    }
  }
  
  async cleanupAll() {
    const promises = Array.from(this.resources).map(resource => this.cleanup(resource));
    await Promise.allSettled(promises);
  }
}

// Usage example
async function processLargeFile(filePath: string) {
  const resourceManager = new ResourceManager();
  
  try {
    return await resourceManager.withResource(
      () => createStreamingParser({ maxMemory: '500MB' }),
      async (parser) => {
        return await parser.parseFile(filePath);
      }
    );
  } catch (error) {
    console.error(`Failed to process ${filePath}:`, error);
    throw error;
  } finally {
    await resourceManager.cleanupAll();
  }
}
```

## Business Logic Error Handling

### Duplicate Detection and Resolution

```typescript
interface DuplicateStrategy {
  onDuplicateRelease: 'error' | 'skip' | 'merge' | 'replace';
  onDuplicateResource: 'error' | 'skip' | 'merge' | 'replace';
  mergeStrategy?: 'newest' | 'manual' | 'custom';
}

class BusinessRuleValidator {
  constructor(private strategy: DuplicateStrategy) {}
  
  async validateReleases(releases: any[]): Promise<any[]> {
    const releaseMap = new Map<string, any[]>();
    
    // Group by identifier
    for (const release of releases) {
      const key = this.getReleaseKey(release);
      if (!releaseMap.has(key)) {
        releaseMap.set(key, []);
      }
      releaseMap.get(key)!.push(release);
    }
    
    const validatedReleases: any[] = [];
    
    for (const [key, duplicates] of releaseMap) {
      if (duplicates.length === 1) {
        validatedReleases.push(duplicates[0]);
        continue;
      }
      
      // Handle duplicates
      switch (this.strategy.onDuplicateRelease) {
        case 'error':
          throw new Error(`Duplicate release found: ${key}`);
          
        case 'skip':
          console.warn(`Skipping duplicate releases for: ${key}`);
          validatedReleases.push(duplicates[0]);
          break;
          
        case 'merge':
          const merged = await this.mergeReleases(duplicates);
          validatedReleases.push(merged);
          break;
          
        case 'replace':
          const latest = this.getLatestRelease(duplicates);
          validatedReleases.push(latest);
          break;
      }
    }
    
    return validatedReleases;
  }
  
  private getReleaseKey(release: any): string {
    // Create unique key from ICPN, UPC, or other identifiers
    return [
      release.icpn,
      release.catalogNumber,
      release.title
    ].filter(Boolean).join('|');
  }
  
  private async mergeReleases(releases: any[]): Promise<any> {
    const merged = { ...releases[0] };
    
    for (let i = 1; i < releases.length; i++) {
      const release = releases[i];
      
      // Merge resources
      merged.resources = [
        ...merged.resources,
        ...release.resources.filter((r: any) => 
          !merged.resources.some((mr: any) => mr.id === r.id)
        )
      ];
      
      // Update metadata with latest values
      if (release.updatedAt > merged.updatedAt) {
        merged.title = release.title || merged.title;
        merged.updatedAt = release.updatedAt;
      }
    }
    
    return merged;
  }
}
```

## Integration Error Handling

### DSP API Error Handling

```typescript
interface DSPConfig {
  baseUrl: string;
  apiKey: string;
  timeout: number;
  retries: number;
}

class DSPIntegrationError extends Error {
  constructor(
    public dsp: string,
    public statusCode: number,
    public response: any,
    message: string
  ) {
    super(message);
  }
}

class DSPClient {
  constructor(private config: DSPConfig) {}
  
  async submitRelease(ddexXml: string): Promise<{ id: string; status: string }> {
    try {
      const response = await this.makeRequest('/releases', {
        method: 'POST',
        body: ddexXml,
        headers: { 'Content-Type': 'application/xml' }
      });
      
      return response.json();
    } catch (error) {
      throw this.handleDSPError(error);
    }
  }
  
  private async makeRequest(endpoint: string, options: any): Promise<Response> {
    const url = `${this.config.baseUrl}${endpoint}`;
    
    const response = await fetch(url, {
      ...options,
      timeout: this.config.timeout,
      headers: {
        ...options.headers,
        'Authorization': `Bearer ${this.config.apiKey}`
      }
    });
    
    if (!response.ok) {
      const errorBody = await response.text();
      throw new DSPIntegrationError(
        'spotify',
        response.status,
        errorBody,
        `HTTP ${response.status}: ${response.statusText}`
      );
    }
    
    return response;
  }
  
  private handleDSPError(error: any): Error {
    if (error instanceof DSPIntegrationError) {
      switch (error.statusCode) {
        case 400:
          return new Error(`Invalid DDEX submission: ${error.response}`);
        case 401:
          return new Error('Authentication failed - check API key');
        case 429:
          return new Error('Rate limit exceeded - retry after delay');
        case 500:
          return new Error('DSP server error - retry later');
        default:
          return error;
      }
    }
    
    return error;
  }
}
```

## Error Monitoring and Alerting

### Structured Logging and Metrics

```typescript
import { createLogger, format, transports } from 'winston';

interface ErrorMetrics {
  errorCount: number;
  errorsByType: Map<string, number>;
  errorsByTime: Map<string, number>;
  lastError?: Error;
}

class ErrorTracker {
  private metrics: ErrorMetrics = {
    errorCount: 0,
    errorsByType: new Map(),
    errorsByTime: new Map()
  };
  
  private logger = createLogger({
    format: format.combine(
      format.timestamp(),
      format.errors({ stack: true }),
      format.json()
    ),
    transports: [
      new transports.File({ filename: 'error.log', level: 'error' }),
      new transports.File({ filename: 'combined.log' })
    ]
  });
  
  trackError(error: Error, context?: any) {
    this.metrics.errorCount++;
    this.metrics.lastError = error;
    
    // Track by type
    const errorType = error.constructor.name;
    this.metrics.errorsByType.set(
      errorType,
      (this.metrics.errorsByType.get(errorType) || 0) + 1
    );
    
    // Track by time (hourly buckets)
    const hourKey = new Date().toISOString().substring(0, 13);
    this.metrics.errorsByTime.set(
      hourKey,
      (this.metrics.errorsByTime.get(hourKey) || 0) + 1
    );
    
    // Log with context
    this.logger.error('DDEX processing error', {
      error: {
        message: error.message,
        stack: error.stack,
        type: errorType
      },
      context,
      metrics: this.getMetricsSummary()
    });
    
    // Check for alerts
    this.checkAlerts();
  }
  
  private checkAlerts() {
    const recentErrors = this.getRecentErrorCount(60 * 1000); // Last minute
    
    if (recentErrors > 10) {
      this.sendAlert('High error rate detected', {
        count: recentErrors,
        period: '1 minute'
      });
    }
  }
  
  private getRecentErrorCount(timeWindowMs: number): number {
    const cutoff = new Date(Date.now() - timeWindowMs);
    const cutoffHour = cutoff.toISOString().substring(0, 13);
    
    return this.metrics.errorsByTime.get(cutoffHour) || 0;
  }
  
  private async sendAlert(message: string, data: any) {
    // Integration with monitoring systems (Slack, PagerDuty, etc.)
    console.error(`ALERT: ${message}`, data);
  }
  
  getMetricsSummary() {
    return {
      totalErrors: this.metrics.errorCount,
      errorTypes: Object.fromEntries(this.metrics.errorsByType),
      lastErrorTime: this.metrics.lastError ? new Date().toISOString() : null
    };
  }
}

// Global error tracker instance
export const errorTracker = new ErrorTracker();

// Global error handler
process.on('uncaughtException', (error) => {
  errorTracker.trackError(error, { type: 'uncaught_exception' });
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  const error = reason instanceof Error ? reason : new Error(String(reason));
  errorTracker.trackError(error, { type: 'unhandled_rejection', promise });
});
```

## Common Pitfalls and Solutions

### Memory Leaks in Error Handling

**Pitfall**: Accumulating error objects in memory without cleanup

```typescript
// DON'T - Memory leak
class BadErrorHandler {
  private allErrors: Error[] = []; // Never cleaned up
  
  handleError(error: Error) {
    this.allErrors.push(error); // Memory grows indefinitely
  }
}

// DO - Bounded error tracking
class GoodErrorHandler {
  private recentErrors: Error[] = [];
  private maxErrors = 100;
  
  handleError(error: Error) {
    this.recentErrors.push(error);
    
    // Keep only recent errors
    if (this.recentErrors.length > this.maxErrors) {
      this.recentErrors = this.recentErrors.slice(-this.maxErrors);
    }
  }
}
```

### Error Swallowing

**Pitfall**: Catching errors without proper handling or logging

```typescript
// DON'T - Silent failures
try {
  await processFile(file);
} catch (error) {
  // Silent failure - error is lost
}

// DO - Explicit error handling
try {
  await processFile(file);
} catch (error) {
  errorTracker.trackError(error, { file: file.name });
  
  if (error instanceof SystemError && error.retryable) {
    await retryProcessing(file);
  } else {
    throw error; // Re-throw if not recoverable
  }
}
```

### Infinite Retry Loops

**Pitfall**: Retrying non-retryable errors indefinitely

```typescript
// DON'T - Infinite retries
async function badRetry(operation: () => Promise<any>) {
  while (true) {
    try {
      return await operation();
    } catch (error) {
      await delay(1000); // Retry forever
    }
  }
}

// DO - Bounded retries with backoff
async function goodRetry(operation: () => Promise<any>) {
  const maxRetries = 3;
  let delay = 1000;
  
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await operation();
    } catch (error) {
      if (i === maxRetries - 1 || !isRetryableError(error)) {
        throw error;
      }
      
      await sleep(delay);
      delay *= 2; // Exponential backoff
    }
  }
}
```

## Performance Considerations

1. **Error Object Creation**: Create error objects lazily to avoid performance overhead
2. **Stack Trace Generation**: Disable stack traces in production for non-critical errors
3. **Logging Overhead**: Use asynchronous logging to avoid blocking operations
4. **Memory Management**: Implement error object pooling for high-frequency errors
5. **Context Collection**: Gather error context efficiently without deep object copying

## Links to API Documentation

- [Parser API Reference](/api/parser/typescript)
- [Builder API Reference](/api/builder/typescript)
- [Error Types Documentation](/api/models/errors)
- [Python Error Handling](/api/parser/python#error-handling)
- [Validation API](/api/builder/validation)

This comprehensive error handling guide ensures robust DDEX processing with proper error recovery, monitoring, and debugging capabilities across all supported platforms.