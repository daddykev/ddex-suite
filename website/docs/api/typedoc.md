# TypeDoc Integration

Automated TypeScript API documentation generation from source code using TypeDoc.

## Overview

The DDEX Suite uses TypeDoc to automatically generate comprehensive API documentation from TypeScript source code, ensuring that documentation stays synchronized with the actual implementation.

## Setup and Configuration

### Installation

```bash
# Install TypeDoc globally
npm install -g typedoc

# Or install locally in the project
npm install --save-dev typedoc
```

### TypeDoc Configuration

Create `typedoc.json` in the project root:

```json
{
  "entryPoints": [
    "packages/ddex-parser/bindings/node/index.ts",
    "packages/ddex-builder/bindings/node/index.ts"
  ],
  "out": "website/docs/api/generated",
  "name": "DDEX Suite API Documentation",
  "includeVersion": true,
  "excludeExternals": true,
  "excludePrivate": true,
  "excludeProtected": false,
  "disableSources": false,
  "theme": "default",
  "readme": "none",
  "plugin": [
    "typedoc-plugin-markdown"
  ],
  "githubPages": false,
  "hideGenerator": true,
  "sort": ["source-order"],
  "categorizeByGroup": true,
  "defaultCategory": "Other",
  "categoryOrder": [
    "Classes",
    "Interfaces", 
    "Type Aliases",
    "Functions",
    "Enumerations"
  ]
}
```

### Package.json Scripts

```json
{
  "scripts": {
    "docs:generate": "typedoc",
    "docs:watch": "typedoc --watch",
    "docs:serve": "typedoc --serve",
    "docs:clean": "rm -rf website/docs/api/generated"
  }
}
```

## TypeDoc Markdown Plugin

### Installation

```bash
npm install --save-dev typedoc-plugin-markdown
```

### Configuration for Docusaurus

```json
{
  "plugin": ["typedoc-plugin-markdown"],
  "out": "website/docs/api/generated",
  "entryDocument": "README.md",
  "hideBreadcrumbs": true,
  "hideInPageTOC": true,
  "readme": "none",
  "allReflectionsHaveOwnDocument": true,
  "fileExtension": ".md",
  "publicPath": "/api/generated/",
  "gitRevision": "main"
}
```

## Source Code Documentation Standards

### TSDoc Comments

```typescript
/**
 * Parses DDEX XML content into structured data.
 * 
 * @remarks
 * This parser supports ERN 3.8.2, 4.2, and 4.3 formats and provides
 * both graph (faithful) and flattened (developer-friendly) representations.
 * 
 * @param xml - The DDEX XML content to parse
 * @param options - Optional parsing configuration
 * @returns Promise resolving to parsed DDEX data
 * 
 * @example
 * ```typescript
 * const parser = new DDEXParser();
 * const result = await parser.parse(xmlContent, {
 *   includeRawExtensions: true,
 *   validateReferences: true
 * });
 * console.log(`Parsed ${result.flat.releases.length} releases`);
 * ```
 * 
 * @throws {@link ValidationError}
 * Thrown when XML is malformed or doesn't conform to DDEX schema
 * 
 * @throws {@link UnsupportedVersionError}
 * Thrown when DDEX version is not supported
 * 
 * @public
 */
async parse(xml: string, options?: DDEXParserOptions): Promise<ParseResult> {
  // Implementation...
}
```

### Interface Documentation

```typescript
/**
 * Configuration options for DDEX parsing behavior.
 * 
 * @remarks
 * These options control how the parser processes DDEX XML and what
 * information is included in the output.
 * 
 * @public
 */
interface DDEXParserOptions {
  /**
   * Include raw XML for extension elements.
   * 
   * @remarks
   * When enabled, preserves original XML content for extension elements
   * to maintain round-trip fidelity. Increases memory usage.
   * 
   * @defaultValue false
   */
  includeRawExtensions?: boolean;
  
  /**
   * Include XML comments in parsed output.
   * 
   * @defaultValue false
   */
  includeComments?: boolean;
  
  /**
   * Validate that all resource references are resolvable.
   * 
   * @remarks
   * When enabled, ensures all resource IDs referenced in releases
   * exist in the resource list. Slower but more thorough validation.
   * 
   * @defaultValue true
   */
  validateReferences?: boolean;
  
  /**
   * Enable streaming mode for large files.
   * 
   * @remarks
   * Streaming mode reduces memory usage for large DDEX files but
   * may limit some processing capabilities.
   * 
   * @defaultValue false
   */
  streaming?: boolean;
}
```

### Error Documentation

```typescript
/**
 * Base class for all DDEX parsing errors.
 * 
 * @public
 */
abstract class DDEXError extends Error {
  /**
   * Error code for programmatic handling.
   */
  readonly code: string;
  
  /**
   * Optional context information about the error.
   */
  readonly context?: Record<string, any>;
  
  constructor(message: string, code: string, context?: Record<string, any>) {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
    this.context = context;
  }
}

/**
 * Thrown when DDEX XML fails validation.
 * 
 * @example
 * ```typescript
 * try {
 *   await parser.parse(invalidXml);
 * } catch (error) {
 *   if (error instanceof ValidationError) {
 *     console.error('Validation failed:', error.validationErrors);
 *   }
 * }
 * ```
 * 
 * @public
 */
class ValidationError extends DDEXError {
  /**
   * Detailed validation error information.
   */
  readonly validationErrors: ValidationResult[];
  
  constructor(message: string, validationErrors: ValidationResult[]) {
    super(message, 'VALIDATION_FAILED', { validationErrors });
    this.validationErrors = validationErrors;
  }
}
```

## Custom TypeDoc Plugin

### Creating a Custom Plugin

```typescript
// typedoc-ddex-plugin.ts
import { Application, Converter, ParameterType } from 'typedoc';

/**
 * Custom TypeDoc plugin for DDEX-specific documentation features.
 */
export function load(app: Application) {
  // Add custom options
  app.options.addDeclaration({
    name: 'ddexExamples',
    help: 'Include DDEX-specific examples',
    type: ParameterType.Boolean,
    defaultValue: true
  });
  
  // Custom converter for DDEX annotations
  app.converter.on(Converter.EVENT_CREATE_DECLARATION, (context, reflection) => {
    if (reflection.comment?.tags) {
      // Process @ddexVersion tags
      const versionTags = reflection.comment.tags.filter(tag => tag.tag === 'ddexversion');
      if (versionTags.length > 0) {
        reflection.comment.shortText = `[DDEX ${versionTags[0].text}] ${reflection.comment.shortText}`;
      }
      
      // Process @platform tags
      const platformTags = reflection.comment.tags.filter(tag => tag.tag === 'platform');
      if (platformTags.length > 0) {
        const platforms = platformTags.map(tag => tag.text).join(', ');
        reflection.comment.text += `\n\n**Supported Platforms:** ${platforms}`;
      }
    }
  });
}
```

### Using Custom Tags

```typescript
/**
 * Applies Spotify-specific preset configuration.
 * 
 * @param presetName - The preset name to apply
 * 
 * @ddexversion 4.3
 * @platform Spotify
 * @since 0.2.0
 * 
 * @example
 * ```typescript
 * const builder = new DdexBuilder();
 * builder.applyPreset('spotify');
 * ```
 */
applyPreset(presetName: string): void {
  // Implementation...
}
```

## Integration with Docusaurus

### Sidebar Configuration

```javascript
// sidebars.js
module.exports = {
  apiSidebar: [
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api/parser/index',
        'api/builder/index',
        'api/models/index',
        {
          type: 'category',
          label: 'Generated Documentation',
          items: [
            {
              type: 'autogenerated',
              dirName: 'api/generated'
            }
          ]
        }
      ]
    }
  ]
};
```

### Build Integration

```javascript
// docusaurus.config.js
module.exports = {
  plugins: [
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'api',
        path: 'docs/api',
        routeBasePath: 'api',
        sidebarPath: require.resolve('./sidebars.js'),
        beforeDefaultRemarkPlugins: [
          // Process TypeDoc markdown
          require('./plugins/typedoc-processor')
        ]
      }
    ]
  ],
  scripts: [
    {
      src: '/js/api-search.js',
      async: true
    }
  ]
};
```

## Automated Generation Workflow

### GitHub Actions

```yaml
# .github/workflows/docs.yml
name: Generate API Documentation

on:
  push:
    branches: [main]
    paths: ['packages/*/bindings/node/**/*.ts']
  pull_request:
    paths: ['packages/*/bindings/node/**/*.ts']

jobs:
  generate-docs:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: |
        npm ci
        npm install -g typedoc
    
    - name: Generate TypeDoc documentation
      run: |
        npm run docs:clean
        npm run docs:generate
    
    - name: Verify generated docs
      run: |
        if [ ! -d "website/docs/api/generated" ]; then
          echo "Documentation generation failed"
          exit 1
        fi
        
        # Check that key files were generated
        if [ ! -f "website/docs/api/generated/README.md" ]; then
          echo "Main documentation file missing"
          exit 1
        fi
    
    - name: Commit generated documentation
      if: github.ref == 'refs/heads/main'
      run: |
        git config --local user.email "action@github.com"
        git config --local user.name "GitHub Action"
        git add website/docs/api/generated/
        git diff --staged --quiet || git commit -m "docs: update generated API documentation"
        git push
```

### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Check if TypeScript files changed
if git diff --cached --name-only | grep -E "packages/.*/bindings/node/.*\.ts$"; then
  echo "TypeScript files changed, regenerating API documentation..."
  
  # Generate documentation
  npm run docs:generate
  
  # Add generated files to commit
  git add website/docs/api/generated/
  
  echo "API documentation updated"
fi
```

## Documentation Quality Assurance

### Validation Scripts

```typescript
// scripts/validate-docs.ts
import { existsSync, readFileSync } from 'fs';
import { glob } from 'glob';

interface DocValidationRule {
  name: string;
  check: (content: string, filePath: string) => string[];
}

const validationRules: DocValidationRule[] = [
  {
    name: 'Has description',
    check: (content) => {
      return content.includes('## Description') ? [] : ['Missing description section'];
    }
  },
  {
    name: 'Has examples',
    check: (content) => {
      return content.includes('```typescript') || content.includes('```javascript')
        ? [] : ['Missing code examples'];
    }
  },
  {
    name: 'Has parameters documentation',
    check: (content) => {
      const hasParams = content.includes('### Parameters') || content.includes('#### Parameters');
      const hasParamAnnotations = content.includes('@param');
      return hasParams || hasParamAnnotations ? [] : ['Missing parameter documentation'];
    }
  }
];

async function validateDocumentation() {
  const docFiles = await glob('website/docs/api/generated/**/*.md');
  const issues: Array<{ file: string; rule: string; issues: string[] }> = [];
  
  for (const filePath of docFiles) {
    const content = readFileSync(filePath, 'utf-8');
    
    for (const rule of validationRules) {
      const ruleIssues = rule.check(content, filePath);
      if (ruleIssues.length > 0) {
        issues.push({
          file: filePath,
          rule: rule.name,
          issues: ruleIssues
        });
      }
    }
  }
  
  if (issues.length > 0) {
    console.error('Documentation validation failed:');
    issues.forEach(issue => {
      console.error(`${issue.file} (${issue.rule}): ${issue.issues.join(', ')}`);
    });
    process.exit(1);
  } else {
    console.log('Documentation validation passed');
  }
}

validateDocumentation().catch(console.error);
```

### Coverage Analysis

```typescript
// scripts/doc-coverage.ts
import { Project } from 'ts-morph';

function analyzeDocumentationCoverage() {
  const project = new Project({
    tsConfigFilePath: 'packages/ddex-parser/bindings/node/tsconfig.json'
  });
  
  const sourceFiles = project.getSourceFiles();
  let totalSymbols = 0;
  let documentedSymbols = 0;
  
  sourceFiles.forEach(sourceFile => {
    sourceFile.getExportedDeclarations().forEach((declarations, name) => {
      declarations.forEach(declaration => {
        totalSymbols++;
        
        const jsDoc = declaration.getJsDocs();
        if (jsDoc.length > 0) {
          documentedSymbols++;
        } else {
          console.warn(`Missing documentation: ${sourceFile.getBaseName()}::${name}`);
        }
      });
    });
  });
  
  const coverage = (documentedSymbols / totalSymbols) * 100;
  console.log(`Documentation coverage: ${coverage.toFixed(1)}% (${documentedSymbols}/${totalSymbols})`);
  
  if (coverage < 90) {
    console.error('Documentation coverage below 90% threshold');
    process.exit(1);
  }
}

analyzeDocumentationCoverage();
```

## Best Practices

### Documentation Standards

1. **Completeness**: Document all public APIs with descriptions, parameters, and examples
2. **Consistency**: Use consistent formatting and terminology across all documentation
3. **Examples**: Provide practical, working examples for all major features
4. **Error Handling**: Document all possible errors and their handling
5. **Versioning**: Track API changes and deprecations clearly

### TypeDoc Configuration

1. **Entry Points**: Carefully select entry points to include only public APIs
2. **Exclusions**: Exclude internal implementation details and test files
3. **Themes**: Use appropriate themes for your documentation style
4. **Plugins**: Leverage plugins for enhanced functionality
5. **Output**: Configure output format for your documentation platform

### Maintenance

1. **Automation**: Automate documentation generation as part of CI/CD
2. **Validation**: Implement documentation quality checks
3. **Review**: Include documentation review in code review process
4. **Updates**: Keep documentation synchronized with code changes
5. **Feedback**: Collect and act on documentation user feedback

This TypeDoc integration ensures that the DDEX Suite API documentation remains accurate, comprehensive, and automatically updated with each code change.