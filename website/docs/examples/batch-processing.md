# Batch Processing Multiple DDEX Files

Learn how to efficiently process multiple DDEX files using both the parser and builder for high-throughput scenarios.

## Overview

Batch processing is essential for:
- **Catalog ingestion**: Processing thousands of releases from distributors
- **Data migration**: Converting between DDEX versions or formats
- **Quality assurance**: Validating large sets of DDEX files
- **Analytics**: Extracting metadata from entire catalogs
- **Compliance checking**: Ensuring files meet platform requirements

## JavaScript/TypeScript Examples

### Batch Parsing with Error Handling

```typescript
import { DDEXParser } from 'ddex-parser';
import { promises as fs } from 'fs';
import path from 'path';

interface BatchResult {
  file: string;
  success: boolean;
  data?: any;
  error?: string;
  processTime: number;
}

class BatchProcessor {
  private parser: DDEXParser;
  private results: BatchResult[] = [];

  constructor() {
    this.parser = new DDEXParser({
      preserveExtensions: true,
      includeComments: false,
      maxMemoryMB: 100 // Limit memory per file
    });
  }

  async processDirectory(directoryPath: string): Promise<BatchResult[]> {
    const files = await this.getXmlFiles(directoryPath);
    console.log(`Processing ${files.length} DDEX files...`);

    // Process in chunks to avoid memory overload
    const chunkSize = 10;
    for (let i = 0; i < files.length; i += chunkSize) {
      const chunk = files.slice(i, i + chunkSize);
      await Promise.all(chunk.map(file => this.processFile(file)));
      
      // Log progress
      console.log(`Processed ${Math.min(i + chunkSize, files.length)}/${files.length} files`);
    }

    return this.results;
  }

  private async processFile(filePath: string): Promise<void> {
    const startTime = Date.now();
    const fileName = path.basename(filePath);

    try {
      const xmlContent = await fs.readFile(filePath, 'utf-8');
      const result = await this.parser.parse(xmlContent);
      
      this.results.push({
        file: fileName,
        success: true,
        data: result,
        processTime: Date.now() - startTime
      });

    } catch (error) {
      this.results.push({
        file: fileName,
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        processTime: Date.now() - startTime
      });
    }
  }

  private async getXmlFiles(dir: string): Promise<string[]> {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    const files: string[] = [];

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        files.push(...await this.getXmlFiles(fullPath));
      } else if (entry.name.endsWith('.xml')) {
        files.push(fullPath);
      }
    }

    return files;
  }

  generateReport(): void {
    const successful = this.results.filter(r => r.success);
    const failed = this.results.filter(r => !r.success);
    const avgTime = successful.reduce((sum, r) => sum + r.processTime, 0) / successful.length;

    console.log('\n📊 Batch Processing Report');
    console.log('========================');
    console.log(`Total files: ${this.results.length}`);
    console.log(`Successful: ${successful.length}`);
    console.log(`Failed: ${failed.length}`);
    console.log(`Average processing time: ${avgTime.toFixed(2)}ms`);
    
    if (failed.length > 0) {
      console.log('\n❌ Failed files:');
      failed.forEach(f => console.log(`  ${f.file}: ${f.error}`));
    }
  }
}

// Usage
async function main() {
  const processor = new BatchProcessor();
  await processor.processDirectory('./ddex-files');
  processor.generateReport();
}
```

### Batch Building with Templates

```typescript
import { DDEXBuilder } from 'ddex-builder';
import { promises as fs } from 'fs';

interface ReleaseData {
  title: string;
  artist: string;
  isrc: string;
  label: string;
  releaseDate: string;
  genre: string;
}

class BatchBuilder {
  private builder: DDEXBuilder;

  constructor() {
    this.builder = new DDEXBuilder({
      preset: 'youtube_album'',
      validateOutput: true
    });
  }

  async buildFromCsv(csvPath: string, outputDir: string): Promise<void> {
    const csvContent = await fs.readFile(csvPath, 'utf-8');
    const releases = this.parseCsvData(csvContent);

    console.log(`Building ${releases.length} DDEX files...`);

    // Ensure output directory exists
    await fs.mkdir(outputDir, { recursive: true });

    // Process releases in parallel batches
    const batchSize = 5;
    for (let i = 0; i < releases.length; i += batchSize) {
      const batch = releases.slice(i, i + batchSize);
      await Promise.all(batch.map(release => this.buildRelease(release, outputDir)));
      
      console.log(`Built ${Math.min(i + batchSize, releases.length)}/${releases.length} files`);
    }
  }

  private async buildRelease(release: ReleaseData, outputDir: string): Promise<void> {
    try {
      const buildRequest = this.createBuildRequest(release);
      const xml = await this.builder.build(buildRequest);
      
      const filename = `${release.isrc}_${release.title.replace(/[^a-zA-Z0-9]/g, '_')}.xml`;
      const outputPath = path.join(outputDir, filename);
      
      await fs.writeFile(outputPath, xml, 'utf-8');
      
    } catch (error) {
      console.error(`Failed to build ${release.title}: ${error}`);
    }
  }

  private createBuildRequest(release: ReleaseData): any {
    return {
      version: '4.3',
      messageHeader: {
        messageId: `MSG_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        sentOnBehalfOf: 'YourLabel',
        messageRecipient: 'Spotify'
      },
      updateIndicator: 'OriginalMessage',
      messageControlType: 'LiveMessage',
      dealList: [{
        dealId: `DEAL_${release.isrc}`,
        dealType: 'License',
        commercialTerms: {
          territory: 'Worldwide',
          distributionChannel: 'OnDemandStream'
        }
      }],
      releaseList: [{
        releaseId: `REL_${release.isrc}`,
        releaseType: 'Single',
        releaseReference: release.isrc,
        releaseDetailsByTerritory: [{
          territory: 'Worldwide',
          displayArtist: release.artist,
          labelName: release.label,
          title: release.title,
          releaseDate: release.releaseDate,
          genre: release.genre
        }],
        soundRecordings: [{
          soundRecordingId: `SR_${release.isrc}`,
          isrc: release.isrc,
          title: release.title,
          displayArtist: release.artist,
          duration: 'PT3M30S' // Default 3:30
        }]
      }]
    };
  }

  private parseCsvData(csv: string): ReleaseData[] {
    const lines = csv.trim().split('\n');
    const headers = lines[0].split(',');
    
    return lines.slice(1).map(line => {
      const values = line.split(',');
      return {
        title: values[0]?.trim() || '',
        artist: values[1]?.trim() || '',
        isrc: values[2]?.trim() || '',
        label: values[3]?.trim() || '',
        releaseDate: values[4]?.trim() || '',
        genre: values[5]?.trim() || ''
      };
    });
  }
}
```

### Stream Processing for Large Files

```typescript
import { DDEXParser } from 'ddex-parser';
import { createReadStream, createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';
import { Transform } from 'stream';

class StreamingBatchProcessor {
  private parser: DDEXParser;

  constructor() {
    this.parser = new DDEXParser({
      streaming: true,
      maxMemoryMB: 50
    });
  }

  async processLargeFile(inputPath: string, outputPath: string): Promise<void> {
    const extractTransform = new Transform({
      objectMode: true,
      transform(chunk, encoding, callback) {
        try {
          // Extract key metadata from each parsed element
          const metadata = {
            timestamp: new Date().toISOString(),
            releaseId: chunk.releaseId,
            title: chunk.flat?.releases?.[0]?.title,
            artist: chunk.flat?.releases?.[0]?.displayArtist,
            isrc: chunk.flat?.soundRecordings?.[0]?.isrc,
            territory: chunk.flat?.dealTerms?.[0]?.territory
          };
          
          this.push(JSON.stringify(metadata) + '\n');
          callback();
        } catch (error) {
          callback(error);
        }
      }
    });

    await pipeline(
      createReadStream(inputPath),
      this.parser.createParseStream(),
      extractTransform,
      createWriteStream(outputPath)
    );
  }
}
```

## Python Examples

### Batch Processing with Multiprocessing

```python
import os
import json
import time
from pathlib import Path
from multiprocessing import Pool, cpu_count
from ddex_parser import DDEXParser
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class ProcessResult:
    file_path: str
    success: bool
    data: Optional[dict] = None
    error: Optional[str] = None
    process_time: float = 0.0

def process_single_file(file_path: str) -> ProcessResult:
    """Process a single DDEX file - designed for multiprocessing"""
    start_time = time.time()
    parser = DDEXParser()
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            xml_content = f.read()
        
        result = parser.parse(xml_content)
        
        return ProcessResult(
            file_path=file_path,
            success=True,
            data=result.to_dict(),
            process_time=time.time() - start_time
        )
        
    except Exception as e:
        return ProcessResult(
            file_path=file_path,
            success=False,
            error=str(e),
            process_time=time.time() - start_time
        )

class BatchProcessor:
    def __init__(self, max_workers: Optional[int] = None):
        self.max_workers = max_workers or cpu_count()
        self.results: List[ProcessResult] = []
    
    def process_directory(self, directory: Path) -> List[ProcessResult]:
        """Process all XML files in directory using multiprocessing"""
        xml_files = list(directory.rglob("*.xml"))
        print(f"Found {len(xml_files)} XML files")
        
        # Process files in parallel
        with Pool(processes=self.max_workers) as pool:
            self.results = pool.map(process_single_file, [str(f) for f in xml_files])
        
        return self.results
    
    def generate_report(self) -> dict:
        """Generate processing statistics"""
        successful = [r for r in self.results if r.success]
        failed = [r for r in self.results if not r.success]
        
        avg_time = sum(r.process_time for r in successful) / len(successful) if successful else 0
        
        report = {
            'total_files': len(self.results),
            'successful': len(successful),
            'failed': len(failed),
            'success_rate': len(successful) / len(self.results) * 100,
            'average_processing_time': avg_time,
            'failed_files': [{'file': r.file_path, 'error': r.error} for r in failed]
        }
        
        return report
    
    def export_successful_data(self, output_file: Path):
        """Export all successfully parsed data to JSON"""
        successful_data = [
            {'file': r.file_path, 'data': r.data} 
            for r in self.results if r.success
        ]
        
        with open(output_file, 'w') as f:
            json.dump(successful_data, f, indent=2, default=str)

# Usage example
if __name__ == "__main__":
    processor = BatchProcessor(max_workers=4)
    results = processor.process_directory(Path("./ddex_files"))
    
    # Generate and print report
    report = processor.generate_report()
    print(f"Processed {report['total_files']} files")
    print(f"Success rate: {report['success_rate']:.1f}%")
    print(f"Average time: {report['average_processing_time']:.3f}s")
    
    # Export successful data
    processor.export_successful_data(Path("./batch_results.json"))
```

### DataFrame Batch Processing

```python
import pandas as pd
from ddex_parser import DDEXParser
from pathlib import Path
from typing import List

class DataFrameBatchProcessor:
    def __init__(self):
        self.parser = DDEXParser()
    
    def process_to_dataframe(self, file_paths: List[str]) -> pd.DataFrame:
        """Process multiple DDEX files and combine into a single DataFrame"""
        all_dataframes = []
        
        for file_path in file_paths:
            try:
                with open(file_path, 'r') as f:
                    xml_content = f.read()
                
                # Convert to DataFrame
                df = self.parser.to_dataframe(xml_content)
                df['source_file'] = Path(file_path).name
                all_dataframes.append(df)
                
            except Exception as e:
                print(f"Failed to process {file_path}: {e}")
        
        # Combine all DataFrames
        if all_dataframes:
            combined_df = pd.concat(all_dataframes, ignore_index=True)
            return combined_df
        else:
            return pd.DataFrame()
    
    def analyze_catalog(self, df: pd.DataFrame) -> dict:
        """Perform catalog-wide analysis"""
        analysis = {
            'total_releases': df['release_id'].nunique(),
            'total_tracks': df['sound_recording_id'].nunique(),
            'unique_artists': df['display_artist'].nunique(),
            'unique_labels': df['label_name'].nunique(),
            'genres': df['genre'].value_counts().to_dict(),
            'release_years': df['release_date'].str[:4].value_counts().to_dict(),
            'territories': df['territory'].value_counts().to_dict()
        }
        
        return analysis

# Usage
processor = DataFrameBatchProcessor()
file_paths = list(Path("./catalog").glob("*.xml"))
catalog_df = processor.process_to_dataframe([str(f) for f in file_paths])
analysis = processor.analyze_catalog(catalog_df)

print(f"Catalog contains {analysis['total_releases']} releases")
print(f"Top genres: {list(analysis['genres'].keys())[:5]}")
```

## Performance Optimization Tips

### Memory Management

```typescript
// Configure parser for batch processing
const parser = new DDEXParser({
  maxMemoryMB: 100,        // Limit memory per file
  streaming: true,         // Use streaming for large files
  preserveExtensions: false // Skip if not needed for round-trip
});

// Process in chunks to avoid memory buildup
const chunkSize = 10;
for (let i = 0; i < files.length; i += chunkSize) {
  const chunk = files.slice(i, i + chunkSize);
  await Promise.all(chunk.map(processFile));
  
  // Force garbage collection if available
  if (global.gc) global.gc();
}
```

### Error Recovery Strategies

```typescript
class RobustBatchProcessor {
  async processWithRetry(filePath: string, maxRetries = 3): Promise<any> {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await this.processFile(filePath);
      } catch (error) {
        if (attempt === maxRetries) {
          throw error;
        }
        
        // Exponential backoff
        await new Promise(resolve => 
          setTimeout(resolve, Math.pow(2, attempt) * 1000)
        );
      }
    }
  }
}
```

### Monitoring and Progress Tracking

```python
from tqdm import tqdm
import logging

# Setup logging for batch operations
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('batch_processing.log'),
        logging.StreamHandler()
    ]
)

def process_with_progress(file_paths: List[str]):
    """Process files with progress bar and logging"""
    results = []
    
    with tqdm(total=len(file_paths), desc="Processing DDEX files") as pbar:
        for file_path in file_paths:
            try:
                result = process_single_file(file_path)
                results.append(result)
                logging.info(f"Successfully processed {file_path}")
                
            except Exception as e:
                logging.error(f"Failed to process {file_path}: {e}")
                
            pbar.update(1)
    
    return results
```

## Best Practices

1. **Chunk Processing**: Process files in small batches to manage memory
2. **Error Isolation**: Don't let one bad file stop the entire batch
3. **Progress Tracking**: Provide visibility into long-running operations
4. **Resource Limits**: Set memory and time limits for individual files
5. **Parallel Processing**: Use multiprocessing/threading appropriately
6. **Graceful Degradation**: Continue processing even when some files fail
7. **Comprehensive Logging**: Track successes, failures, and performance metrics

This approach ensures robust, scalable batch processing for any size DDEX catalog.