# Round-Trip Fidelity Guide

Comprehensive guide to preserving all data through parse → modify → build cycles with the DDEX Suite, ensuring perfect data integrity for complex workflows.

## Problem Statement

Round-trip fidelity is critical for DDEX processing workflows where you need to:

- **Parse existing DDEX XML** while preserving all original data
- **Make targeted modifications** without losing unrelated information
- **Generate new XML** that maintains all non-modified elements exactly
- **Preserve extension data** that may not be understood by your application
- **Maintain XML formatting** and namespace declarations when possible

Without proper round-trip fidelity, modifications can inadvertently:
- Remove extension elements from third-party tools
- Lose XML comments and processing instructions
- Change namespace prefixes and formatting
- Drop unknown metadata fields
- Alter element ordering in ways that break partner integrations

## Solution Approach

The DDEX Suite provides comprehensive round-trip fidelity through:

1. **Graph Model Preservation**: Maintains the complete DDEX structure
2. **Extension Handling**: Preserves unknown elements and attributes
3. **Raw XML Retention**: Optionally stores original XML for critical sections
4. **Deterministic Building**: Ensures consistent output formatting
5. **Validation-Safe Modifications**: Guarantees schema compliance

## Understanding Graph vs Flattened Models

### Graph Model - Complete Fidelity

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();
const result = await parser.parse(xmlContent, {
  preserveExtensions: true,  // Keep unknown elements
  includeComments: true,     // Preserve XML comments
  rawExtensions: true        // Store raw XML for extensions
});

// Graph model preserves complete structure
console.log(result.graph.message.header.messageId);
console.log(result.graph.message.releaseList.releases[0].releaseReference);

// Extensions are preserved
console.log(result.graph.extensions); // Unknown elements
console.log(result.graph.rawXmlSections); // Raw XML preservation
```

### Flattened Model - Developer Convenience

```typescript
// Flattened model for easier manipulation
console.log(result.flat.releases[0].title);
console.log(result.flat.releases[0].artists);

// But still maintains fidelity links
console.log(result.flat.releases[0]._graphRef); // Link to graph model
console.log(result.flat.releases[0]._extensions); // Preserved extensions
```

## Complete Round-Trip Workflow

### Basic Round-Trip Example

```typescript
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

async function modifyReleaseTitleWithFidelity(
  originalXml: string,
  newTitle: string
): Promise<string> {
  // Step 1: Parse with full fidelity preservation
  const parser = new DDEXParser();
  const parseResult = await parser.parse(originalXml, {
    preserveExtensions: true,
    includeComments: true,
    rawExtensions: true,
    validateReferences: true
  });
  
  // Step 2: Modify only the target field
  parseResult.flat.releases[0].title = newTitle;
  
  // Step 3: Build with fidelity preservation
  const builder = new DDEXBuilder();
  const buildRequest = parseResult.toBuildRequest();
  
  const newXml = await builder.build(buildRequest, {
    preserveExtensions: true,
    maintainFormatting: true,
    deterministicOutput: true
  });
  
  return newXml;
}

// Verify round-trip fidelity
async function verifyRoundTrip(originalXml: string) {
  const parser = new DDEXParser();
  const builder = new DDEXBuilder();
  
  // Parse original
  const original = await parser.parse(originalXml, { preserveExtensions: true });
  
  // Build without modifications
  const rebuilt = await builder.build(original.toBuildRequest());
  
  // Parse rebuilt to compare
  const rebuiltParsed = await parser.parse(rebuilt, { preserveExtensions: true });
  
  // Deep comparison
  const isIdentical = await compareStructures(original, rebuiltParsed);
  console.log(`Round-trip fidelity: ${isIdentical ? 'PASS' : 'FAIL'}`);
  
  return isIdentical;
}
```

### Advanced Extension Preservation

```typescript
interface ExtensionPreservationOptions {
  preserveUnknownElements: boolean;
  preserveUnknownAttributes: boolean;
  preserveNamespaceDeclarations: boolean;
  preserveElementOrder: boolean;
  preserveWhitespace: boolean;
}

async function parseWithFullExtensionSupport(xmlContent: string) {
  const parser = new DDEXParser();
  
  const result = await parser.parse(xmlContent, {
    preserveExtensions: true,
    rawExtensions: true,
    extensionOptions: {
      preserveUnknownElements: true,
      preserveUnknownAttributes: true,
      preserveNamespaceDeclarations: true,
      preserveElementOrder: true,
      preserveWhitespace: false // Usually safe to normalize
    }
  });
  
  // Access preserved extensions
  console.log('Unknown elements:', result.extensions.unknownElements);
  console.log('Unknown attributes:', result.extensions.unknownAttributes);
  console.log('Raw XML sections:', result.rawXmlSections);
  
  return result;
}

// Custom extension handler
class CustomExtensionHandler {
  async processExtensions(extensions: any[]): Promise<any[]> {
    return extensions.map(ext => {
      // Add custom processing while preserving original
      return {
        ...ext,
        _processed: true,
        _originalXml: ext._rawXml
      };
    });
  }
  
  async restoreExtensions(processedExtensions: any[]): Promise<string[]> {
    return processedExtensions.map(ext => ext._originalXml);
  }
}
```

## Python Round-Trip Workflows

### DataFrame Integration with Fidelity

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder
import pandas as pd
from typing import Dict, Any

async def modify_catalog_with_fidelity(
    xml_content: str,
    modifications: Dict[str, Any]
) -> str:
    """Modify catalog data while preserving all other information"""
    
    # Parse with full preservation
    parser = DDEXParser()
    parse_result = await parser.parse(
        xml_content,
        preserve_extensions=True,
        include_comments=True,
        raw_extensions=True
    )
    
    # Convert to DataFrame for bulk operations
    df = parser.to_dataframe(xml_content)
    
    # Apply modifications efficiently
    for release_id, changes in modifications.items():
        mask = df['release_id'] == release_id
        for field, value in changes.items():
            df.loc[mask, field] = value
    
    # Rebuild preserving extensions
    builder = DDEXBuilder()
    
    # Convert back with fidelity preservation
    build_request = await builder.from_dataframe(
        df,
        original_parse_result=parse_result,  # Preserves extensions
        preserve_extensions=True
    )
    
    return await builder.build(build_request)

async def compare_dataframes_for_fidelity(
    original_xml: str,
    modified_xml: str
) -> pd.DataFrame:
    """Compare DataFrames to verify what changed"""
    
    parser = DDEXParser()
    
    original_df = parser.to_dataframe(original_xml)
    modified_df = parser.to_dataframe(modified_xml)
    
    # Identify differences
    comparison = original_df.compare(modified_df, align_axis=1)
    
    return comparison
```

### Extension-Aware Data Processing

```python
import json
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class ExtensionData:
    namespace: str
    element_name: str
    attributes: Dict[str, str]
    content: Optional[str]
    raw_xml: str

class FidelityPreservingProcessor:
    def __init__(self):
        self.preserved_extensions: List[ExtensionData] = []
        self.namespace_mappings: Dict[str, str] = {}
    
    async def process_with_extensions(
        self,
        xml_content: str,
        processor_func: callable
    ) -> str:
        """Process DDEX while preserving all extensions"""
        
        parser = DDEXParser()
        result = await parser.parse(
            xml_content,
            preserve_extensions=True,
            raw_extensions=True
        )
        
        # Store extensions
        self.preserved_extensions = self._extract_extensions(result)
        self.namespace_mappings = result.namespace_mappings
        
        # Process the structured data
        processed_data = await processor_func(result.flat)
        
        # Rebuild with extensions
        builder = DDEXBuilder()
        build_request = self._create_build_request_with_extensions(
            processed_data,
            self.preserved_extensions
        )
        
        return await builder.build(build_request)
    
    def _extract_extensions(self, parse_result) -> List[ExtensionData]:
        extensions = []
        
        for ext in parse_result.extensions.unknown_elements:
            extensions.append(ExtensionData(
                namespace=ext.namespace,
                element_name=ext.local_name,
                attributes=ext.attributes,
                content=ext.text_content,
                raw_xml=ext.raw_xml
            ))
        
        return extensions
    
    def _create_build_request_with_extensions(
        self,
        processed_data,
        extensions: List[ExtensionData]
    ):
        # Create build request that includes extensions
        build_request = {
            'message': processed_data,
            'extensions': [
                {
                    'namespace': ext.namespace,
                    'element_name': ext.element_name,
                    'attributes': ext.attributes,
                    'content': ext.content,
                    'raw_xml': ext.raw_xml
                }
                for ext in extensions
            ],
            'namespace_mappings': self.namespace_mappings
        }
        
        return build_request
```

## Schema Evolution and Versioning

### Handling Version Differences

```typescript
class VersionAwareFidelityHandler {
  async migrateWithFidelity(
    xmlContent: string,
    fromVersion: string,
    toVersion: string
  ): Promise<string> {
    const parser = new DDEXParser();
    const builder = new DDEXBuilder();
    
    // Parse with version-specific handling
    const parseResult = await parser.parse(xmlContent, {
      version: fromVersion,
      preserveExtensions: true,
      versionMigration: {
        targetVersion: toVersion,
        preserveIncompatibleFields: true,
        addVersionExtensions: true
      }
    });
    
    // Version-specific transformations
    const migrated = await this.applyVersionTransformations(
      parseResult,
      fromVersion,
      toVersion
    );
    
    // Build with target version
    return await builder.build(migrated.toBuildRequest(), {
      version: toVersion,
      preserveExtensions: true
    });
  }
  
  private async applyVersionTransformations(
    parseResult: any,
    fromVersion: string,
    toVersion: string
  ): Promise<any> {
    const transformations = this.getVersionTransformations(fromVersion, toVersion);
    
    for (const transformation of transformations) {
      parseResult = await transformation.apply(parseResult);
    }
    
    return parseResult;
  }
  
  private getVersionTransformations(from: string, to: string) {
    const transformationMap = {
      '3.8.2->4.2': [
        new ResourceTypeTransformation(),
        new MetadataFieldTransformation(),
        new IdentifierFormatTransformation()
      ],
      '4.2->4.3': [
        new StreamingMetadataTransformation(),
        new TerritoryCodeTransformation()
      ]
    };
    
    return transformationMap[`${from}->${to}`] || [];
  }
}

class ResourceTypeTransformation {
  async apply(parseResult: any): Promise<any> {
    // Transform resource types while preserving extensions
    for (const resource of parseResult.flat.resources) {
      if (resource.type === 'SoundRecording') {
        // Preserve original in extension
        resource._extensions = resource._extensions || {};
        resource._extensions.originalType = resource.type;
        
        // Apply transformation
        resource.type = 'AudioResource';
      }
    }
    
    return parseResult;
  }
}
```

## Testing Round-Trip Fidelity

### Comprehensive Fidelity Test Suite

```typescript
interface FidelityTestCase {
  name: string;
  inputXml: string;
  modification?: (data: any) => void;
  expectedChanges?: string[];
  preservedElements?: string[];
}

class FidelityTestSuite {
  async runFidelityTests(testCases: FidelityTestCase[]): Promise<TestResult[]> {
    const results: TestResult[] = [];
    
    for (const testCase of testCases) {
      const result = await this.runSingleTest(testCase);
      results.push(result);
    }
    
    return results;
  }
  
  private async runSingleTest(testCase: FidelityTestCase): Promise<TestResult> {
    const parser = new DDEXParser();
    const builder = new DDEXBuilder();
    
    try {
      // Parse original
      const original = await parser.parse(testCase.inputXml, {
        preserveExtensions: true,
        includeComments: true
      });
      
      // Apply modification if specified
      if (testCase.modification) {
        testCase.modification(original);
      }
      
      // Build new XML
      const rebuiltXml = await builder.build(original.toBuildRequest());
      
      // Parse rebuilt for comparison
      const rebuilt = await parser.parse(rebuiltXml, {
        preserveExtensions: true,
        includeComments: true
      });
      
      // Compare structures
      const comparison = await this.compareStructures(original, rebuilt);
      
      return {
        testName: testCase.name,
        passed: comparison.identical,
        differences: comparison.differences,
        preservedExtensions: comparison.preservedExtensions,
        metrics: comparison.metrics
      };
      
    } catch (error) {
      return {
        testName: testCase.name,
        passed: false,
        error: error.message,
        differences: [],
        preservedExtensions: false,
        metrics: {}
      };
    }
  }
  
  private async compareStructures(original: any, rebuilt: any) {
    const differences: string[] = [];
    let preservedExtensions = true;
    
    // Compare graph structures
    const graphDiff = this.deepCompare(original.graph, rebuilt.graph);
    differences.push(...graphDiff);
    
    // Compare extensions
    if (original.extensions.length !== rebuilt.extensions.length) {
      differences.push(`Extension count mismatch: ${original.extensions.length} vs ${rebuilt.extensions.length}`);
      preservedExtensions = false;
    }
    
    // Compare flattened data
    const flatDiff = this.deepCompare(original.flat, rebuilt.flat);
    differences.push(...flatDiff);
    
    return {
      identical: differences.length === 0,
      differences,
      preservedExtensions,
      metrics: {
        totalElements: this.countElements(original.graph),
        extensionCount: original.extensions.length,
        namespaceCount: Object.keys(original.namespaces || {}).length
      }
    };
  }
  
  private deepCompare(obj1: any, obj2: any, path = ''): string[] {
    const differences: string[] = [];
    
    if (typeof obj1 !== typeof obj2) {
      differences.push(`Type mismatch at ${path}: ${typeof obj1} vs ${typeof obj2}`);
      return differences;
    }
    
    if (obj1 === null || obj2 === null) {
      if (obj1 !== obj2) {
        differences.push(`Null mismatch at ${path}: ${obj1} vs ${obj2}`);
      }
      return differences;
    }
    
    if (typeof obj1 === 'object') {
      const keys1 = Object.keys(obj1);
      const keys2 = Object.keys(obj2);
      
      const allKeys = new Set([...keys1, ...keys2]);
      
      for (const key of allKeys) {
        const newPath = path ? `${path}.${key}` : key;
        
        if (!(key in obj1)) {
          differences.push(`Missing key in original: ${newPath}`);
        } else if (!(key in obj2)) {
          differences.push(`Missing key in rebuilt: ${newPath}`);
        } else {
          differences.push(...this.deepCompare(obj1[key], obj2[key], newPath));
        }
      }
    } else if (obj1 !== obj2) {
      differences.push(`Value mismatch at ${path}: ${obj1} vs ${obj2}`);
    }
    
    return differences;
  }
}

// Example test cases
const fidelityTests: FidelityTestCase[] = [
  {
    name: 'No modification round-trip',
    inputXml: originalXml,
    // No modification - should be identical
  },
  {
    name: 'Title modification preserves extensions',
    inputXml: xmlWithExtensions,
    modification: (data) => {
      data.flat.releases[0].title = 'New Title';
    },
    expectedChanges: ['releases[0].title'],
    preservedElements: ['extensions', 'comments', 'namespaces']
  },
  {
    name: 'Add resource preserves structure',
    inputXml: originalXml,
    modification: (data) => {
      data.flat.resources.push({
        id: 'A123456789',
        type: 'SoundRecording',
        duration: 'PT3M45S'
      });
    },
    expectedChanges: ['resources.length'],
    preservedElements: ['message.header', 'releaseList.structure']
  }
];
```

### Automated Fidelity Validation

```python
import asyncio
import hashlib
from typing import List, Dict, Any

class AutomatedFidelityValidator:
    def __init__(self):
        self.test_results = []
    
    async def validate_bulk_processing(
        self,
        xml_files: List[str],
        modification_func: callable
    ) -> Dict[str, Any]:
        """Validate fidelity across multiple files"""
        
        results = {
            'total_files': len(xml_files),
            'passed': 0,
            'failed': 0,
            'failures': []
        }
        
        for file_path in xml_files:
            try:
                with open(file_path, 'r') as f:
                    xml_content = f.read()
                
                # Test round-trip fidelity
                passed = await self._test_file_fidelity(
                    xml_content,
                    modification_func
                )
                
                if passed:
                    results['passed'] += 1
                else:
                    results['failed'] += 1
                    results['failures'].append(file_path)
                    
            except Exception as e:
                results['failed'] += 1
                results['failures'].append(f"{file_path}: {str(e)}")
        
        return results
    
    async def _test_file_fidelity(
        self,
        xml_content: str,
        modification_func: callable
    ) -> bool:
        """Test fidelity for a single file"""
        
        parser = DDEXParser()
        builder = DDEXBuilder()
        
        # Parse original
        original = await parser.parse(
            xml_content,
            preserve_extensions=True
        )
        
        # Create unmodified copy for comparison
        unmodified_copy = copy.deepcopy(original)
        
        # Apply modifications
        if modification_func:
            modification_func(original)
        
        # Build new XML
        rebuilt_xml = await builder.build(original.to_build_request())
        
        # Parse rebuilt
        rebuilt = await parser.parse(
            rebuilt_xml,
            preserve_extensions=True
        )
        
        # Compare preserved elements
        return self._compare_preserved_elements(
            unmodified_copy,
            rebuilt,
            modification_func
        )
    
    def _compare_preserved_elements(
        self,
        original: Any,
        rebuilt: Any,
        modification_func: callable
    ) -> bool:
        """Compare elements that should be preserved"""
        
        # Elements that should always be preserved
        preserved_paths = [
            'graph.message.header.messageId',
            'graph.message.header.sender',
            'extensions',
            'namespace_mappings'
        ]
        
        for path in preserved_paths:
            original_value = self._get_nested_value(original, path)
            rebuilt_value = self._get_nested_value(rebuilt, path)
            
            if original_value != rebuilt_value:
                print(f"Fidelity violation at {path}")
                return False
        
        return True
    
    def _get_nested_value(self, obj: Any, path: str) -> Any:
        """Get value from nested object path"""
        keys = path.split('.')
        current = obj
        
        for key in keys:
            if hasattr(current, key):
                current = getattr(current, key)
            elif isinstance(current, dict) and key in current:
                current = current[key]
            else:
                return None
        
        return current
```

## Common Pitfalls and Solutions

### 1. Extension Loss During Modification

**Pitfall**: Modifying flattened data without preserving graph extensions

```typescript
// DON'T - Extensions lost
result.flat.releases[0] = { ...newReleaseData }; // Overwrites extensions

// DO - Preserve extensions
result.flat.releases[0] = {
  ...result.flat.releases[0],  // Preserve existing data including _extensions
  ...newReleaseData,           // Apply modifications
  _extensions: result.flat.releases[0]._extensions  // Explicitly preserve
};
```

### 2. Namespace Declaration Loss

**Pitfall**: Not preserving namespace prefixes and declarations

```python
# DON'T - Namespace context lost
build_request = {
    'message': modified_data
    # Missing namespace_mappings
}

# DO - Preserve namespace context
build_request = {
    'message': modified_data,
    'namespace_mappings': original_parse_result.namespace_mappings,
    'preserve_prefixes': True
}
```

### 3. Element Order Changes

**Pitfall**: Rebuilding changes element order unintentionally

```typescript
// Configure builder for deterministic order
const xml = await builder.build(buildRequest, {
  preserveElementOrder: true,
  deterministicOutput: true,
  sortingStrategy: 'preserve-original'
});
```

## Performance Considerations

1. **Extension Storage**: Raw XML storage increases memory usage by ~20-30%
2. **Parsing Overhead**: Full fidelity parsing is ~15% slower than basic parsing
3. **Build Complexity**: Preserving extensions adds ~10% to build time
4. **Memory Management**: Use streaming for large files with extensions
5. **Comparison Overhead**: Deep structure comparison is expensive for large documents

## Links to API Documentation

- [Parser TypeScript API](../api/parser/typescript)
- [Builder TypeScript API](../api/builder/typescript)
- [Data Models Overview](../api/models/message)
- [Python Parser API](../api/parser/python)
- [Builder Types Reference](../api/builder/types)

This comprehensive guide ensures perfect round-trip fidelity for all DDEX processing workflows, maintaining data integrity across complex modification cycles.