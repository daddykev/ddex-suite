---
sidebar_position: 6
---

# Examples

Real-world examples of using DDEX Suite for common music industry workflows.

## Quick Examples

### Round-Trip Processing

Parse, modify, and rebuild DDEX XML with perfect fidelity:

```typescript
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

// Parse existing DDEX
const parser = new DDEXParser();
const result = await parser.parse(originalXml);

// Modify release information
result.flat.releases[0].title = 'Remastered Edition';
result.flat.releases[0].releaseDate = '2024-01-01';

// Add new territories
result.flat.deals[0].territories.push('US', 'CA', 'GB');

// Build back to XML
const builder = new DDEXBuilder();
const newXml = await builder.build(result.toBuildRequest());
```

### Batch Processing

Process multiple DDEX files efficiently:

```typescript
import { DDEXParser, DDEXBuilder } from 'ddex-suite';
import glob from 'glob';

async function processBatch(pattern: string) {
  const files = glob.sync(pattern);
  const parser = new DDEXParser();
  const builder = new DDEXBuilder({ preset: 'spotify' });
  
  const results = [];
  
  for (const file of files) {
    try {
      const xml = fs.readFileSync(file, 'utf8');
      const parsed = await parser.parse(xml);
      
      // Apply business logic
      const modified = applyBusinessRules(parsed);
      
      // Build output
      const output = await builder.build(modified.toBuildRequest());
      results.push({ file, output, status: 'success' });
      
    } catch (error) {
      results.push({ file, error: error.message, status: 'error' });
    }
  }
  
  return results;
}
```

## Data Analysis Examples

### Release Analytics (Python)

```python
import pandas as pd
from ddex_parser import DDEXParser
import matplotlib.pyplot as plt

def analyze_catalog(ddex_files):
    parser = DDEXParser()
    all_releases = []
    
    # Parse all files to DataFrames
    for file in ddex_files:
        dfs = parser.to_dataframe(file)
        all_releases.append(dfs.releases)
    
    # Combine all releases
    catalog_df = pd.concat(all_releases, ignore_index=True)
    
    # Analyze by genre
    genre_counts = catalog_df.explode('genres').groupby('genres').size()
    genre_counts.plot(kind='bar', title='Releases by Genre')
    plt.show()
    
    # Analyze release dates
    catalog_df['release_date'] = pd.to_datetime(catalog_df['release_date'])
    yearly_releases = catalog_df.groupby(catalog_df['release_date'].dt.year).size()
    yearly_releases.plot(title='Releases by Year')
    plt.show()
    
    return catalog_df
```

### Territory Coverage Analysis

```python
def analyze_territory_coverage(ddex_files):
    parser = DDEXParser()
    territory_data = []
    
    for file in ddex_files:
        dfs = parser.to_dataframe(file)
        
        # Explode territories for analysis
        deals_exploded = dfs.deals.explode('territories')
        territory_counts = deals_exploded.groupby('territories').size()
        territory_data.append(territory_counts)
    
    # Combine territory data
    all_territories = pd.concat(territory_data).groupby(level=0).sum()
    
    # Top 20 territories
    top_territories = all_territories.nlargest(20)
    top_territories.plot(kind='barh', title='Top 20 Territories by Release Count')
    plt.show()
    
    return all_territories
```

## Business Logic Examples

### Label Processing Workflow

```typescript
class LabelProcessor {
  private parser = new DDEXParser();
  private builder = new DDEXBuilder({ preset: 'generic' });
  
  async processRelease(xmlContent: string, labelRules: LabelRules) {
    // Parse the DDEX
    const result = await this.parser.parse(xmlContent);
    
    // Apply label-specific rules
    this.applyLabelRules(result, labelRules);
    
    // Validate business rules
    this.validateBusinessRules(result);
    
    // Generate output for each partner
    const outputs = await this.generatePartnerOutputs(result);
    
    return outputs;
  }
  
  private applyLabelRules(result: ParseResult, rules: LabelRules) {
    // Apply pricing rules
    result.flat.deals.forEach(deal => {
      deal.priceInformation = this.calculatePricing(deal, rules.pricing);
    });
    
    // Apply territory restrictions
    result.flat.deals.forEach(deal => {
      deal.territories = this.filterTerritories(deal.territories, rules.territories);
    });
    
    // Apply metadata enrichment
    result.flat.releases.forEach(release => {
      release.labelName = rules.labelName;
      release.genres = this.enrichGenres(release.genres, rules.genreMapping);
    });
  }
  
  private async generatePartnerOutputs(result: ParseResult) {
    const partners = ['spotify', 'apple', 'youtube'] as const;
    const outputs: Record<string, string> = {};
    
    for (const partner of partners) {
      const partnerBuilder = new DDEXBuilder({ preset: partner });
      outputs[partner] = await partnerBuilder.build(result.toBuildRequest());
    }
    
    return outputs;
  }
}
```

### Compliance Validation

```typescript
class ComplianceValidator {
  async validateRelease(ddexXml: string): Promise<ComplianceReport> {
    const parser = new DDEXParser({ validation: 'strict' });
    const builder = new DDEXBuilder({ validation: 'strict' });
    
    try {
      // Parse and validate structure
      const result = await parser.parse(ddexXml);
      
      // Validate business rules
      const businessValidation = this.validateBusinessRules(result);
      
      // Test round-trip fidelity
      const roundTripXml = await builder.build(result.toBuildRequest());
      const roundTripResult = await parser.parse(roundTripXml);
      const fidelityCheck = this.compareParsedResults(result, roundTripResult);
      
      return {
        isValid: businessValidation.isValid && fidelityCheck.isIdentical,
        businessRules: businessValidation,
        roundTrip: fidelityCheck,
        recommendations: this.generateRecommendations(result),
      };
      
    } catch (error) {
      return {
        isValid: false,
        error: error.message,
        recommendations: ['Fix XML structure errors'],
      };
    }
  }
  
  private validateBusinessRules(result: ParseResult) {
    const errors = [];
    
    // Validate required fields
    result.flat.releases.forEach((release, index) => {
      if (!release.title) errors.push(`Release ${index}: Missing title`);
      if (!release.releaseDate) errors.push(`Release ${index}: Missing release date`);
    });
    
    // Validate ISRC format
    result.flat.soundRecordings.forEach((recording, index) => {
      if (recording.isrc && !this.validateISRC(recording.isrc)) {
        errors.push(`Recording ${index}: Invalid ISRC format`);
      }
    });
    
    // Validate territory codes
    result.flat.deals.forEach((deal, index) => {
      const invalidTerritories = deal.territories.filter(t => !this.isValidTerritoryCode(t));
      if (invalidTerritories.length > 0) {
        errors.push(`Deal ${index}: Invalid territories: ${invalidTerritories.join(', ')}`);
      }
    });
    
    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}
```

## Integration Examples

### Express.js API

```typescript
import express from 'express';
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

const app = express();
const parser = new DDEXParser();
const builder = new DDEXBuilder();

app.post('/parse', async (req, res) => {
  try {
    const { xml } = req.body;
    const result = await parser.parse(xml);
    res.json(result.flat);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

app.post('/build', async (req, res) => {
  try {
    const { buildRequest } = req.body;
    const xml = await builder.build(buildRequest);
    res.set('Content-Type', 'application/xml');
    res.send(xml);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

app.post('/transform', async (req, res) => {
  try {
    const { xml, transformations } = req.body;
    
    // Parse
    const result = await parser.parse(xml);
    
    // Apply transformations
    const transformed = applyTransformations(result, transformations);
    
    // Build
    const newXml = await builder.build(transformed.toBuildRequest());
    
    res.set('Content-Type', 'application/xml');
    res.send(newXml);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});
```

### Database Integration

```typescript
import { DDEXParser } from 'ddex-parser';
import { Database } from 'your-db-library';

class DDEXImporter {
  constructor(private db: Database) {}
  
  async importDDEXFile(filePath: string) {
    const parser = new DDEXParser();
    const xml = fs.readFileSync(filePath, 'utf8');
    const result = await parser.parse(xml);
    
    // Import releases
    for (const release of result.flat.releases) {
      await this.db.releases.upsert({
        where: { releaseId: release.releaseId },
        create: {
          ...release,
          importedAt: new Date(),
        },
        update: {
          ...release,
          updatedAt: new Date(),
        },
      });
    }
    
    // Import sound recordings
    for (const recording of result.flat.soundRecordings) {
      await this.db.soundRecordings.upsert({
        where: { soundRecordingId: recording.soundRecordingId },
        create: {
          ...recording,
          importedAt: new Date(),
        },
        update: {
          ...recording,
          updatedAt: new Date(),
        },
      });
    }
    
    // Import deals
    for (const deal of result.flat.deals) {
      await this.db.deals.upsert({
        where: { dealId: deal.dealId },
        create: {
          ...deal,
          importedAt: new Date(),
        },
        update: {
          ...deal,
          updatedAt: new Date(),
        },
      });
    }
  }
}
```

## Testing Examples

### Unit Testing

```typescript
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

describe('DDEX Processing', () => {
  const parser = new DDEXParser();
  const builder = new DDEXBuilder();
  
  test('round-trip fidelity', async () => {
    const originalXml = fs.readFileSync('test-data/sample.xml', 'utf8');
    
    // Parse
    const result = await parser.parse(originalXml);
    
    // Build
    const rebuiltXml = await builder.build(result.toBuildRequest());
    
    // Parse again
    const rebuiltResult = await parser.parse(rebuiltXml);
    
    // Compare
    expect(rebuiltResult.flat).toEqual(result.flat);
  });
  
  test('validation catches errors', async () => {
    const invalidXml = '<invalid>xml</invalid>';
    
    await expect(parser.parse(invalidXml)).rejects.toThrow();
  });
  
  test('builder validates input', async () => {
    const invalidRequest = {
      version: '4.3',
      releases: [{ /* missing required fields */ }],
    };
    
    await expect(builder.build(invalidRequest)).rejects.toThrow();
  });
});
```

## Next Steps

- **[Guides](../guides/)** - Step-by-step guides for common tasks
- **[API Reference](../api/)** - Complete API documentation
- **[GitHub Repository](https://github.com/daddykev/ddex-suite)** - Source code and more examples