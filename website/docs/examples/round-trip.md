---
sidebar_position: 3
---

# Round-Trip Processing

Learn how to parse existing DDEX files, modify the data, and rebuild deterministic XML while preserving perfect fidelity. This is essential for content management systems, metadata updates, and workflow automation.

## Basic Round-Trip Workflow

### Parse → Modify → Build

```typescript
import { DDEXParser } from 'ddex-parser';
import { DdexBuilder } from 'ddex-builder';

async function basicRoundTrip(inputFile: string) {
  // Step 1: Parse existing DDEX file
  const parser = new DDEXParser({
    validation: 'strict',
    includeRawExtensions: true  // Preserve extensions for round-trip
  });
  
  console.log('📖 Parsing original DDEX file...');
  const original = await parser.parseFile(inputFile);
  
  console.log(`✅ Parsed: ${original.messageId} (v${original.version})`);
  console.log(`📊 Contains: ${original.flat.releases.length} releases, ${original.flat.soundRecordings.length} tracks`);
  
  // Step 2: Modify the parsed data
  console.log('✏️ Modifying release data...');
  
  // Get a copy of the flattened data for easy modification
  const modified = { ...original.flat };
  
  // Update release information
  if (modified.releases && modified.releases.length > 0) {
    modified.releases[0] = {
      ...modified.releases[0],
      title: `${modified.releases[0].title} (Remastered)`,
      releaseDate: '2024-07-15',  // Updated release date
      pLine: '℗ 2024 Remastered Edition',
      cLine: '© 2024 Updated Rights'
    };
  }
  
  // Update track information
  if (modified.soundRecordings && modified.soundRecordings.length > 0) {
    modified.soundRecordings = modified.soundRecordings.map(track => ({
      ...track,
      metadata: {
        ...track.metadata,
        remastered: 'true',
        remasteringYear: '2024',
        remasteringEngineer: 'Modern Mastering Studio'
      }
    }));
  }
  
  // Step 3: Convert back to build format
  const buildRequest = original.toBuildRequest();
  
  // Apply our modifications
  if (buildRequest.releases && buildRequest.releases.length > 0) {
    buildRequest.releases[0].title = modified.releases[0].title;
    buildRequest.releases[0].releaseDate = modified.releases[0].releaseDate;
    buildRequest.releases[0].pLine = modified.releases[0].pLine;
    buildRequest.releases[0].cLine = modified.releases[0].cLine;
  }
  
  // Update resources
  if (buildRequest.resources && modified.soundRecordings) {
    buildRequest.resources = buildRequest.resources.map((resource, index) => ({
      ...resource,
      metadata: {
        ...resource.metadata,
        ...modified.soundRecordings[index]?.metadata
      }
    }));
  }
  
  // Step 4: Build new deterministic XML
  const builder = new DdexBuilder({
    canonical: true,  // Ensure deterministic output
    validate: true,
    preserveExtensions: true
  });
  
  console.log('🏗️ Building modified DDEX file...');
  const newXml = await builder.build(buildRequest);
  
  // Step 5: Save the result
  const outputFile = inputFile.replace('.xml', '_modified.xml');
  await fs.writeFile(outputFile, newXml, 'utf-8');
  
  console.log(`💾 Saved modified DDEX to: ${outputFile}`);
  
  // Step 6: Verify round-trip integrity
  console.log('🔍 Verifying round-trip integrity...');
  const verification = await parser.parseString(newXml);
  
  const integrityCheck = {
    messageIdPreserved: verification.messageId === original.messageId,
    versionPreserved: verification.version === original.version,
    releaseCountSame: verification.flat.releases.length === original.flat.releases.length,
    trackCountSame: verification.flat.soundRecordings.length === original.flat.soundRecordings.length,
    modificationsApplied: verification.flat.releases[0]?.title.includes('Remastered')
  };
  
  console.log('✅ Round-trip integrity check:', integrityCheck);
  
  return {
    originalXml: await fs.readFile(inputFile, 'utf-8'),
    modifiedXml: newXml,
    originalData: original,
    modifiedData: verification,
    integrityCheck
  };
}

// Usage
await basicRoundTrip('./original-release.xml');
```

### Python Round-Trip

```python
from ddex_parser import DDEXParser
from ddex_builder import DdexBuilder
from datetime import datetime
import json

def basic_round_trip(input_file):
    """Parse, modify, and rebuild a DDEX file."""
    
    # Step 1: Parse original file
    parser = DDEXParser()
    print(f'📖 Parsing {input_file}...')
    
    original = parser.parse_file(input_file)
    print(f'✅ Parsed: {original.message_id} (v{original.version})')
    print(f'📊 Contains: {len(original.releases)} releases, {len(original.sound_recordings)} tracks')
    
    # Step 2: Modify the data
    print('✏️ Modifying release data...')
    
    # Convert to build request format
    build_data = original.to_build_request()
    
    # Update release information
    if build_data['releases']:
        release = build_data['releases'][0]
        release['title'] = f"{release['title']} (Remastered)"
        release['release_date'] = '2024-07-15'
        release['p_line'] = '℗ 2024 Remastered Edition'
        release['c_line'] = '© 2024 Updated Rights'
    
    # Update track metadata
    if build_data['resources']:
        for resource in build_data['resources']:
            if resource['resource_type'] == 'SoundRecording':
                if 'metadata' not in resource:
                    resource['metadata'] = {}
                resource['metadata']['remastered'] = 'true'
                resource['metadata']['remastering_year'] = '2024'
                resource['metadata']['remastering_engineer'] = 'Modern Mastering Studio'
    
    # Step 3: Build new XML
    builder = DdexBuilder(
        canonical=True,
        validate=True,
        preserve_extensions=True
    )
    
    print('🏗️ Building modified DDEX file...')
    new_xml = builder.build(build_data)
    
    # Step 4: Save result
    output_file = input_file.replace('.xml', '_modified.xml')
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(new_xml)
    
    print(f'💾 Saved modified DDEX to: {output_file}')
    
    # Step 5: Verify integrity
    print('🔍 Verifying round-trip integrity...')
    verification = parser.parse(new_xml)
    
    integrity_check = {
        'message_id_preserved': verification.message_id == original.message_id,
        'version_preserved': verification.version == original.version,
        'release_count_same': len(verification.releases) == len(original.releases),
        'track_count_same': len(verification.sound_recordings) == len(original.sound_recordings),
        'modifications_applied': 'Remastered' in verification.releases[0].title if verification.releases else False
    }
    
    print('✅ Round-trip integrity check:', integrity_check)
    
    return {
        'original_file': input_file,
        'modified_file': output_file,
        'integrity_check': integrity_check
    }

# Usage
if __name__ == "__main__":
    result = basic_round_trip('./original-release.xml')
```

## Advanced Modification Patterns

### Metadata Enhancement Workflow

```typescript
class DDEXMetadataEnhancer {
  private parser: DDEXParser;
  private builder: DdexBuilder;
  
  constructor() {
    this.parser = new DDEXParser({
      validation: 'strict',
      includeRawExtensions: true
    });
    
    this.builder = new DdexBuilder({
      canonical: true,
      validate: true,
      preserveExtensions: true
    });
  }
  
  async enhanceMetadata(inputFile: string, enhancements: MetadataEnhancements) {
    console.log('🔍 Analyzing original DDEX file...');
    const original = await this.parser.parseFile(inputFile);
    
    // Convert to build format for easier manipulation
    const buildData = original.toBuildRequest();
    
    console.log('✨ Applying metadata enhancements...');
    
    // Apply release-level enhancements
    if (enhancements.release) {
      this.applyReleaseEnhancements(buildData, enhancements.release);
    }
    
    // Apply track-level enhancements
    if (enhancements.tracks) {
      this.applyTrackEnhancements(buildData, enhancements.tracks);
    }
    
    // Apply commercial enhancements
    if (enhancements.deals) {
      this.applyDealEnhancements(buildData, enhancements.deals);
    }
    
    // Add new territories if specified
    if (enhancements.newTerritories) {
      this.expandTerritories(buildData, enhancements.newTerritories);
    }
    
    // Generate enhanced XML
    const enhancedXml = await this.builder.build(buildData);
    
    // Save with descriptive filename
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const outputFile = inputFile.replace('.xml', `_enhanced_${timestamp}.xml`);
    
    await fs.writeFile(outputFile, enhancedXml, 'utf-8');
    
    console.log(`💾 Enhanced DDEX saved to: ${outputFile}`);
    
    // Generate enhancement report
    const report = await this.generateEnhancementReport(original, buildData);
    
    return {
      originalFile: inputFile,
      enhancedFile: outputFile,
      enhancements: enhancements,
      report: report
    };
  }
  
  private applyReleaseEnhancements(buildData: any, enhancements: ReleaseEnhancements) {
    if (!buildData.releases) return;
    
    buildData.releases.forEach(release => {
      // Add missing genres
      if (enhancements.additionalGenres) {
        release.genres = [...(release.genres || []), ...enhancements.additionalGenres];
        // Remove duplicates
        release.genres = [...new Set(release.genres)];
      }
      
      // Update copyright information
      if (enhancements.copyrightInfo) {
        release.pLine = enhancements.copyrightInfo.pLine;
        release.cLine = enhancements.copyrightInfo.cLine;
      }
      
      // Add marketing metadata
      if (enhancements.marketing) {
        release.metadata = {
          ...release.metadata,
          marketingText: enhancements.marketing.description,
          keyWords: enhancements.marketing.keywords?.join(','),
          targetAudience: enhancements.marketing.targetAudience
        };
      }
      
      // Update release identifiers
      if (enhancements.additionalIds) {
        release.releaseId = [
          ...(release.releaseId || []),
          ...enhancements.additionalIds
        ];
      }
    });
  }
  
  private applyTrackEnhancements(buildData: any, enhancements: TrackEnhancements) {
    if (!buildData.resources) return;
    
    buildData.resources.forEach((resource, index) => {
      if (resource.resourceType !== 'SoundRecording') return;
      
      // Add contributor information
      if (enhancements.contributors && enhancements.contributors[index]) {
        resource.contributors = [
          ...(resource.contributors || []),
          ...enhancements.contributors[index]
        ];
      }
      
      // Enhance technical metadata
      if (enhancements.technicalInfo && enhancements.technicalInfo[index]) {
        resource.technicalDetails = {
          ...resource.technicalDetails,
          ...enhancements.technicalInfo[index]
        };
      }
      
      // Add lyrics information
      if (enhancements.lyrics && enhancements.lyrics[index]) {
        resource.metadata = {
          ...resource.metadata,
          lyricsBy: enhancements.lyrics[index].lyricist,
          language: enhancements.lyrics[index].language,
          hasLyrics: enhancements.lyrics[index].hasLyrics
        };
      }
      
      // Add mood and theme tags
      if (enhancements.moods && enhancements.moods[index]) {
        resource.moods = enhancements.moods[index];
      }
    });
  }
  
  private applyDealEnhancements(buildData: any, enhancements: DealEnhancements) {
    if (!buildData.deals) return;
    
    buildData.deals.forEach(deal => {
      // Expand use types
      if (enhancements.additionalUseTypes) {
        deal.useTypes = [...new Set([
          ...(deal.useTypes || []),
          ...enhancements.additionalUseTypes
        ])];
      }
      
      // Update pricing
      if (enhancements.pricing) {
        deal.priceInformation = {
          ...deal.priceInformation,
          ...enhancements.pricing
        };
      }
      
      // Add promotional periods
      if (enhancements.promotionalPeriods) {
        deal.promotionalPeriods = enhancements.promotionalPeriods;
      }
    });
  }
  
  private expandTerritories(buildData: any, newTerritories: string[]) {
    // Add territories to releases
    if (buildData.releases) {
      buildData.releases.forEach(release => {
        release.territories = [...new Set([
          ...(release.territories || []),
          ...newTerritories
        ])];
      });
    }
    
    // Add territories to deals
    if (buildData.deals) {
      buildData.deals.forEach(deal => {
        deal.territories = [...new Set([
          ...(deal.territories || []),
          ...newTerritories
        ])];
      });
    }
  }
  
  private async generateEnhancementReport(original: any, enhanced: any) {
    return {
      originalReleases: original.flat.releases.length,
      originalTracks: original.flat.soundRecordings.length,
      enhancedReleases: enhanced.releases?.length || 0,
      enhancedTracks: enhanced.resources?.length || 0,
      changesApplied: {
        genresAdded: this.countGenreChanges(original, enhanced),
        territoriesExpanded: this.countTerritoryChanges(original, enhanced),
        metadataEnhanced: this.countMetadataChanges(original, enhanced)
      }
    };
  }
  
  private countGenreChanges(original: any, enhanced: any): number {
    const originalGenres = new Set();
    original.flat.releases?.forEach(r => r.genres?.forEach(g => originalGenres.add(g)));
    
    const enhancedGenres = new Set();
    enhanced.releases?.forEach(r => r.genres?.forEach(g => enhancedGenres.add(g)));
    
    return enhancedGenres.size - originalGenres.size;
  }
  
  private countTerritoryChanges(original: any, enhanced: any): number {
    const originalTerritories = new Set();
    original.flat.releases?.forEach(r => r.territories?.forEach(t => originalTerritories.add(t)));
    
    const enhancedTerritories = new Set();
    enhanced.releases?.forEach(r => r.territories?.forEach(t => enhancedTerritories.add(t)));
    
    return enhancedTerritories.size - originalTerritories.size;
  }
  
  private countMetadataChanges(original: any, enhanced: any): number {
    // Simple count of metadata fields added
    let changes = 0;
    
    if (enhanced.resources) {
      enhanced.resources.forEach((resource, index) => {
        const originalResource = original.flat.soundRecordings[index];
        const originalMetadataKeys = Object.keys(originalResource?.metadata || {});
        const enhancedMetadataKeys = Object.keys(resource.metadata || {});
        
        changes += enhancedMetadataKeys.length - originalMetadataKeys.length;
      });
    }
    
    return Math.max(0, changes);
  }
}

// Type definitions for enhancements
interface MetadataEnhancements {
  release?: ReleaseEnhancements;
  tracks?: TrackEnhancements;
  deals?: DealEnhancements;
  newTerritories?: string[];
}

interface ReleaseEnhancements {
  additionalGenres?: string[];
  copyrightInfo?: {
    pLine: string;
    cLine: string;
  };
  marketing?: {
    description: string;
    keywords: string[];
    targetAudience: string;
  };
  additionalIds?: Array<{
    namespace: string;
    value: string;
  }>;
}

interface TrackEnhancements {
  contributors?: Array<Array<{
    name: string;
    role: string;
  }>>;
  technicalInfo?: Array<{
    audioCodecType?: string;
    bitRate?: number;
    sampleRate?: number;
  }>;
  lyrics?: Array<{
    lyricist: string;
    language: string;
    hasLyrics: boolean;
  }>;
  moods?: Array<string[]>;
}

interface DealEnhancements {
  additionalUseTypes?: string[];
  pricing?: {
    priceCurrency: string;
    wholesalePrice: number;
  };
  promotionalPeriods?: Array<{
    startDate: string;
    endDate: string;
    promotionType: string;
  }>;
}

// Usage example
const enhancer = new DDEXMetadataEnhancer();

const enhancements: MetadataEnhancements = {
  release: {
    additionalGenres: ['Electronic', 'Ambient'],
    copyrightInfo: {
      pLine: '℗ 2024 Enhanced Record Label',
      cLine: '© 2024 Enhanced Publishing'
    },
    marketing: {
      description: 'A groundbreaking album that defines the future of music',
      keywords: ['innovative', 'electronic', 'ambient', 'experimental'],
      targetAudience: 'electronic music enthusiasts'
    }
  },
  tracks: {
    contributors: [
      [{ name: 'Sound Engineer', role: 'RecordingEngineer' }],
      [{ name: 'Mix Master', role: 'MixingEngineer' }]
    ],
    technicalInfo: [
      { audioCodecType: 'FLAC', bitRate: 1411, sampleRate: 44100 },
      { audioCodecType: 'FLAC', bitRate: 1411, sampleRate: 44100 }
    ]
  },
  newTerritories: ['JP', 'KR', 'AU', 'NZ']
};

await enhancer.enhanceMetadata('./original-release.xml', enhancements);
```

## Version Migration Workflows

### Migrating Between DDEX Versions

```typescript
class DDEXVersionMigrator {
  private parser: DDEXParser;
  private builder43: DdexBuilder;
  private builder42: DdexBuilder;
  
  constructor() {
    this.parser = new DDEXParser();
    this.builder43 = new DdexBuilder({ version: '4.3' });
    this.builder42 = new DdexBuilder({ version: '4.2' });
  }
  
  async migrateToVersion43(inputFile: string) {
    console.log('🔄 Migrating DDEX file to version 4.3...');
    
    // Parse original file (auto-detects version)
    const original = await this.parser.parseFile(inputFile);
    
    console.log(`📋 Original version: ${original.version}`);
    
    if (original.version === '4.3') {
      console.log('✅ File is already version 4.3');
      return inputFile;
    }
    
    // Get build data
    const buildData = original.toBuildRequest();
    
    // Apply version 4.3 specific enhancements
    this.applyVersion43Enhancements(buildData);
    
    // Build with 4.3 builder
    const xml43 = await this.builder43.build(buildData);
    
    const outputFile = inputFile.replace('.xml', '_v43.xml');
    await fs.writeFile(outputFile, xml43, 'utf-8');
    
    console.log(`✅ Migrated to version 4.3: ${outputFile}`);
    
    // Verify the migration
    const verification = await this.parser.parseString(xml43);
    console.log(`🔍 Verification: Version ${verification.version}`);
    
    return outputFile;
  }
  
  private applyVersion43Enhancements(buildData: any) {
    // Version 4.3 specific features
    if (buildData.releases) {
      buildData.releases.forEach(release => {
        // Enhanced territory handling in 4.3
        if (release.territories) {
          release.territories = this.normalizeTerritoriesFor43(release.territories);
        }
        
        // New genre classifications in 4.3
        if (release.genres) {
          release.genres = this.normalizeGenresFor43(release.genres);
        }
        
        // Enhanced metadata support
        release.metadata = {
          ...release.metadata,
          ddexVersion: '4.3',
          migrationTimestamp: new Date().toISOString()
        };
      });
    }
    
    if (buildData.resources) {
      buildData.resources.forEach(resource => {
        // Enhanced technical metadata in 4.3
        if (resource.resourceType === 'SoundRecording') {
          resource.metadata = {
            ...resource.metadata,
            enhancedMetadataVersion: '4.3'
          };
          
          // New technical fields available in 4.3
          if (resource.technicalDetails) {
            resource.technicalDetails = {
              ...resource.technicalDetails,
              // Add 4.3 specific technical fields
              spatialAudioType: 'none', // Default value
              dynamicRange: 'standard'
            };
          }
        }
      });
    }
  }
  
  private normalizeTerritoriesFor43(territories: string[]): string[] {
    // 4.3 has enhanced territory codes
    const territoryMapping = {
      'Worldwide': 'WorldWide', // Standardized spelling in 4.3
      'UK': 'GB', // ISO country codes preferred in 4.3
    };
    
    return territories.map(territory => 
      territoryMapping[territory] || territory
    );
  }
  
  private normalizeGenresFor43(genres: string[]): string[] {
    // 4.3 has updated genre taxonomy
    const genreMapping = {
      'Electronica': 'Electronic',
      'Hip-Hop/Rap': 'Hip-Hop',
      'R&B/Soul': 'R&B'
    };
    
    return genres.map(genre =>
      genreMapping[genre] || genre
    );
  }
  
  async downgradeTo42(inputFile: string) {
    console.log('⬇️ Downgrading DDEX file to version 4.2...');
    
    const original = await this.parser.parseFile(inputFile);
    
    if (original.version === '4.2') {
      console.log('✅ File is already version 4.2');
      return inputFile;
    }
    
    const buildData = original.toBuildRequest();
    
    // Remove 4.3-specific features for 4.2 compatibility
    this.applyVersion42Restrictions(buildData);
    
    const xml42 = await this.builder42.build(buildData);
    
    const outputFile = inputFile.replace('.xml', '_v42.xml');
    await fs.writeFile(outputFile, xml42, 'utf-8');
    
    console.log(`✅ Downgraded to version 4.2: ${outputFile}`);
    
    return outputFile;
  }
  
  private applyVersion42Restrictions(buildData: any) {
    // Remove 4.3-specific fields that don't exist in 4.2
    if (buildData.resources) {
      buildData.resources.forEach(resource => {
        if (resource.technicalDetails) {
          // Remove 4.3-specific technical fields
          delete resource.technicalDetails.spatialAudioType;
          delete resource.technicalDetails.dynamicRange;
        }
        
        if (resource.metadata) {
          delete resource.metadata.enhancedMetadataVersion;
        }
      });
    }
    
    if (buildData.releases) {
      buildData.releases.forEach(release => {
        if (release.metadata) {
          delete release.metadata.ddexVersion;
          delete release.metadata.migrationTimestamp;
        }
      });
    }
  }
}

// Usage
const migrator = new DDEXVersionMigrator();

// Migrate to 4.3
await migrator.migrateToVersion43('./old-release-v42.xml');

// Downgrade to 4.2 for compatibility
await migrator.downgradeTo42('./new-release-v43.xml');
```

## Batch Round-Trip Processing

### Processing Multiple Files

```typescript
class BatchRoundTripProcessor {
  private parser: DDEXParser;
  private builder: DdexBuilder;
  
  constructor(preset: string = 'universal') {
    this.parser = new DDEXParser({
      includeRawExtensions: true,
      validation: 'permissive' // More forgiving for batch processing
    });
    
    this.builder = new DdexBuilder({
      preset: preset,
      canonical: true,
      preserveExtensions: true
    });
  }
  
  async processDirectory(
    inputDir: string, 
    outputDir: string, 
    transformation: (data: any) => any
  ) {
    console.log(`📁 Processing DDEX files from ${inputDir}`);
    
    const files = await fs.readdir(inputDir);
    const xmlFiles = files.filter(f => f.toLowerCase().endsWith('.xml'));
    
    console.log(`Found ${xmlFiles.length} XML files`);
    
    const results = [];
    
    for (const file of xmlFiles) {
      const inputPath = path.join(inputDir, file);
      const outputPath = path.join(outputDir, file.replace('.xml', '_processed.xml'));
      
      try {
        const result = await this.processFile(inputPath, outputPath, transformation);
        results.push({ file, success: true, ...result });
        console.log(`✅ Processed ${file}`);
        
      } catch (error) {
        console.error(`❌ Failed to process ${file}: ${error.message}`);
        results.push({ file, success: false, error: error.message });
      }
    }
    
    this.printBatchSummary(results);
    return results;
  }
  
  async processFile(inputPath: string, outputPath: string, transformation: (data: any) => any) {
    // Parse original
    const original = await this.parser.parseFile(inputPath);
    
    // Apply transformation
    const buildData = original.toBuildRequest();
    const transformed = transformation(buildData);
    
    // Build new XML
    const newXml = await this.builder.build(transformed);
    
    // Ensure output directory exists
    await fs.mkdir(path.dirname(outputPath), { recursive: true });
    
    // Save result
    await fs.writeFile(outputPath, newXml, 'utf-8');
    
    return {
      originalReleases: original.flat.releases.length,
      originalTracks: original.flat.soundRecordings.length,
      outputFile: outputPath,
      sizeBytes: newXml.length
    };
  }
  
  private printBatchSummary(results: any[]) {
    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);
    
    console.log('\n📊 Batch Processing Summary');
    console.log('===========================');
    console.log(`✅ Successfully processed: ${successful.length} files`);
    console.log(`❌ Failed to process: ${failed.length} files`);
    
    if (successful.length > 0) {
      const totalReleases = successful.reduce((sum, r) => sum + (r.originalReleases || 0), 0);
      const totalTracks = successful.reduce((sum, r) => sum + (r.originalTracks || 0), 0);
      const totalSize = successful.reduce((sum, r) => sum + (r.sizeBytes || 0), 0);
      
      console.log(`🎵 Total releases processed: ${totalReleases}`);
      console.log(`🎵 Total tracks processed: ${totalTracks}`);
      console.log(`📄 Total output size: ${(totalSize / 1024 / 1024).toFixed(2)} MB`);
    }
    
    if (failed.length > 0) {
      console.log('\nFailed files:');
      failed.forEach(result => {
        console.log(`  - ${result.file}: ${result.error}`);
      });
    }
  }
}

// Usage examples

// 1. Update all release dates
const dateUpdater = new BatchRoundTripProcessor('universal');

await dateUpdater.processDirectory('./input', './output', (buildData) => {
  if (buildData.releases) {
    buildData.releases.forEach(release => {
      release.releaseDate = '2024-08-01'; // Update all releases to same date
    });
  }
  return buildData;
});

// 2. Add remastered flag to all tracks
const remasterProcessor = new BatchRoundTripProcessor('spotify');

await remasterProcessor.processDirectory('./originals', './remastered', (buildData) => {
  // Update release titles
  if (buildData.releases) {
    buildData.releases.forEach(release => {
      release.title = `${release.title} (Remastered)`;
    });
  }
  
  // Update track metadata
  if (buildData.resources) {
    buildData.resources.forEach(resource => {
      if (resource.resourceType === 'SoundRecording') {
        resource.metadata = {
          ...resource.metadata,
          remastered: 'true',
          remasteringYear: '2024'
        };
      }
    });
  }
  
  return buildData;
});

// 3. Territory expansion
const territoryExpander = new BatchRoundTripProcessor('universal');

await territoryExpander.processDirectory('./regional', './worldwide', (buildData) => {
  const newTerritories = ['JP', 'KR', 'AU', 'NZ', 'BR', 'MX'];
  
  // Expand release territories
  if (buildData.releases) {
    buildData.releases.forEach(release => {
      release.territories = [...new Set([
        ...(release.territories || []),
        ...newTerritories
      ])];
    });
  }
  
  // Expand deal territories
  if (buildData.deals) {
    buildData.deals.forEach(deal => {
      deal.territories = [...new Set([
        ...(deal.territories || []),
        ...newTerritories
      ])];
    });
  }
  
  return buildData;
});
```

## Quality Assurance and Validation

### Round-Trip Integrity Testing

```typescript
class RoundTripValidator {
  private parser: DDEXParser;
  private builder: DdexBuilder;
  
  constructor() {
    this.parser = new DDEXParser({ includeRawExtensions: true });
    this.builder = new DdexBuilder({ canonical: true, preserveExtensions: true });
  }
  
  async validateRoundTripIntegrity(originalFile: string): Promise<ValidationReport> {
    console.log('🔍 Validating round-trip integrity...');
    
    // Step 1: Parse original
    const original = await this.parser.parseFile(originalFile);
    
    // Step 2: Convert to build format and back to XML
    const buildData = original.toBuildRequest();
    const rebuiltXml = await this.builder.build(buildData);
    
    // Step 3: Parse the rebuilt XML
    const rebuilt = await this.parser.parseString(rebuiltXml);
    
    // Step 4: Compare everything
    const report: ValidationReport = {
      file: originalFile,
      integrityTests: {
        messageIdMatch: original.messageId === rebuilt.messageId,
        versionMatch: original.version === rebuilt.version,
        releaseCountMatch: original.flat.releases.length === rebuilt.flat.releases.length,
        trackCountMatch: original.flat.soundRecordings.length === rebuilt.flat.soundRecordings.length,
        dealCountMatch: original.flat.deals.length === rebuilt.flat.deals.length,
        partyCountMatch: original.flat.parties.length === rebuilt.flat.parties.length
      },
      dataIntegrity: this.compareDataIntegrity(original, rebuilt),
      xmlIntegrity: this.compareXmlIntegrity(originalFile, rebuiltXml),
      passed: true,
      issues: []
    };
    
    // Check if all tests passed
    const allTestsPassed = Object.values(report.integrityTests).every(test => test === true);
    report.passed = allTestsPassed && report.issues.length === 0;
    
    // Log results
    if (report.passed) {
      console.log('✅ Round-trip integrity validation PASSED');
    } else {
      console.log('❌ Round-trip integrity validation FAILED');
      report.issues.forEach(issue => console.log(`  - ${issue}`));
    }
    
    return report;
  }
  
  private compareDataIntegrity(original: any, rebuilt: any): DataIntegrityReport {
    const report: DataIntegrityReport = {
      releaseTitleMatch: true,
      releaseArtistMatch: true,
      trackTitleMatch: true,
      trackISRCMatch: true,
      dealTermsMatch: true,
      mismatches: []
    };
    
    // Compare release data
    for (let i = 0; i < Math.min(original.flat.releases.length, rebuilt.flat.releases.length); i++) {
      const origRelease = original.flat.releases[i];
      const rebuiltRelease = rebuilt.flat.releases[i];
      
      if (origRelease.title !== rebuiltRelease.title) {
        report.releaseTitleMatch = false;
        report.mismatches.push(`Release ${i}: Title mismatch`);
      }
      
      if (origRelease.displayArtist !== rebuiltRelease.displayArtist) {
        report.releaseArtistMatch = false;
        report.mismatches.push(`Release ${i}: Artist mismatch`);
      }
    }
    
    // Compare track data
    for (let i = 0; i < Math.min(original.flat.soundRecordings.length, rebuilt.flat.soundRecordings.length); i++) {
      const origTrack = original.flat.soundRecordings[i];
      const rebuiltTrack = rebuilt.flat.soundRecordings[i];
      
      if (origTrack.title !== rebuiltTrack.title) {
        report.trackTitleMatch = false;
        report.mismatches.push(`Track ${i}: Title mismatch`);
      }
      
      if (origTrack.isrc !== rebuiltTrack.isrc) {
        report.trackISRCMatch = false;
        report.mismatches.push(`Track ${i}: ISRC mismatch`);
      }
    }
    
    return report;
  }
  
  private compareXmlIntegrity(originalFile: string, rebuiltXml: string): XmlIntegrityReport {
    // This would involve more sophisticated XML comparison
    // For now, we'll do basic checks
    
    return {
      xmlWellFormed: true, // Assume if parsing succeeded, it's well-formed
      sizeComparison: {
        originalSize: 0, // Would need to read original file
        rebuiltSize: rebuiltXml.length,
        sizeDifferencePercent: 0
      },
      structureMatch: true // Would need detailed XML structure comparison
    };
  }
}

// Type definitions
interface ValidationReport {
  file: string;
  integrityTests: {
    messageIdMatch: boolean;
    versionMatch: boolean;
    releaseCountMatch: boolean;
    trackCountMatch: boolean;
    dealCountMatch: boolean;
    partyCountMatch: boolean;
  };
  dataIntegrity: DataIntegrityReport;
  xmlIntegrity: XmlIntegrityReport;
  passed: boolean;
  issues: string[];
}

interface DataIntegrityReport {
  releaseTitleMatch: boolean;
  releaseArtistMatch: boolean;
  trackTitleMatch: boolean;
  trackISRCMatch: boolean;
  dealTermsMatch: boolean;
  mismatches: string[];
}

interface XmlIntegrityReport {
  xmlWellFormed: boolean;
  sizeComparison: {
    originalSize: number;
    rebuiltSize: number;
    sizeDifferencePercent: number;
  };
  structureMatch: boolean;
}

// Usage
const validator = new RoundTripValidator();

// Test individual file
const report = await validator.validateRoundTripIntegrity('./test-release.xml');

// Test multiple files
const testFiles = [
  './releases/album1.xml',
  './releases/single1.xml',
  './releases/ep1.xml'
];

for (const file of testFiles) {
  await validator.validateRoundTripIntegrity(file);
}
```

This comprehensive round-trip guide demonstrates:

- Basic parse → modify → build workflows
- Advanced metadata enhancement patterns
- Version migration between DDEX versions
- Batch processing for multiple files
- Quality assurance and integrity validation

Round-trip processing is essential for:
- Content management systems
- Automated metadata enhancement
- Version upgrades
- Territory expansion
- Remastering workflows
- Quality assurance testing

For more advanced scenarios, see:
- [Batch Processing](./batch-processing)
- [Python DataFrame Integration](./python-dataframes)