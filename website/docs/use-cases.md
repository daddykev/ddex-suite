# Use Cases

Explore scenarios where DDEX Suite fits into music industry workflows, from catalog migrations to streaming platform ingestion pipelines.

:::note Illustrative scenarios
The organizations and volumes described on this page are fictional and for illustration only. Helper functions such as `db.streamCatalog()`, `uploadToDsp()` or `filter_by_territory_rights()` stand in for your own systems and are not part of DDEX Suite.
:::

## Major Record Labels

### Major Label Group - Catalog Migration

**Scenario (illustrative)**: A major label group needs to migrate its back catalog (millions of recordings) from a legacy system to a new distribution platform that requires DDEX ERN 4.3.

```typescript
const { DdexBuilder } = require('ddex-builder');

const db = new DatabaseConnection(); // your own data layer

// Stream from the legacy database and emit one ERN message per release
for await (const batch of db.streamCatalog({ batchSize: 1000 })) {
  for (const legacyRelease of batch) {
    const builder = new DdexBuilder();

    builder.addRelease({
      releaseId: legacyRelease.upc,
      releaseType: 'Album',
      title: legacyRelease.title,
      artist: legacyRelease.artist_name,
      label: legacyRelease.label_name,
      catalogNumber: legacyRelease.catalog_no,
      upc: legacyRelease.upc,
      releaseDate: legacyRelease.release_date, // 'YYYY-MM-DD'
      trackIds: legacyRelease.tracks.map(track => track.isrc),
    });

    for (const track of legacyRelease.tracks) {
      builder.addResource({
        resourceId: track.isrc,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: track.artist || legacyRelease.artist_name,
        isrc: track.isrc,
        duration: `PT${track.duration_seconds}S`, // ISO 8601 duration
        trackNumber: track.sequence,
      });
    }

    const validation = await builder.validate();
    if (!validation.isValid) {
      await db.logMigrationErrors(legacyRelease.upc, validation.errors);
      continue;
    }

    const xml = await builder.build(); // returns the ERN XML as a string
    await saveToDistributionQueue(legacyRelease.upc, xml);
  }
}
```

### Major Label - Weekly New Release Feed

**Scenario (illustrative)**: A label needs to generate weekly DDEX feeds of new releases for a large number of DSP partners.

```python
from datetime import datetime, timedelta

import pandas as pd
from ddex_builder import DdexBuilder

# Load this week's approved releases from the data warehouse
releases_df = pd.read_sql("""
    SELECT release_id, release_type, title, artist, label, upc,
           catalog_number, release_date, genre
    FROM releases
    WHERE release_date BETWEEN %s AND %s
      AND status = 'APPROVED'
    ORDER BY priority DESC, release_date
""", warehouse_connection, params=[datetime.now(), datetime.now() + timedelta(days=7)])

for dsp, dsp_config in DSP_CONFIGS.items():
    # Filter releases for this DSP based on territory rights
    dsp_releases = filter_by_territory_rights(releases_df, dsp_config['territories'])

    # Build one ERN message from the DataFrame (one row per release)
    builder = DdexBuilder()
    xml = builder.from_dataframe(dsp_releases, schema='releases')

    # Upload to the DSP's FTP/API
    upload_to_dsp(dsp, xml)
```

## Digital Distributors

### Independent Distributor - New Release Pipeline

**Scenario (illustrative)**: An independent distributor receives a high volume of new releases every day from independent artists and needs to generate DDEX feeds for multiple platforms.

```typescript
const { DdexBuilder } = require('ddex-builder');
const Queue = require('bull');

const releaseQueue = new Queue('releases');

releaseQueue.process(async (job) => {
  const { artistSubmission } = job.data;
  const builder = new DdexBuilder();

  // Transform the artist's simple form data into DDEX
  const isrcs = await Promise.all(
    artistSubmission.tracks.map(track => track.isrc || generateISRC(track))
  );

  builder.addRelease({
    releaseId: artistSubmission.releaseId,
    releaseType: artistSubmission.releaseType, // e.g. 'Single', 'EP', 'Album'
    title: artistSubmission.albumTitle,
    artist: artistSubmission.artistName,
    upc: await generateUPC(artistSubmission),
    releaseDate: artistSubmission.releaseDate,
    genre: mapToAVSGenre(artistSubmission.genre),
    parentalWarning: artistSubmission.tracks.some(track => track.hasExplicitLyrics),
    trackIds: isrcs,
  });

  artistSubmission.tracks.forEach((track, idx) => {
    builder.addResource({
      resourceId: isrcs[idx],
      resourceType: 'SoundRecording',
      title: track.title,
      artist: track.featuring
        ? `${artistSubmission.artistName} feat. ${track.featuring}`
        : artistSubmission.artistName,
      isrc: isrcs[idx],
      duration: `PT${track.durationSeconds}S`,
      trackNumber: idx + 1,
    });
  });

  const xml = await builder.build();

  // Deliver the same message to each target platform
  for (const platform of ['platform-a', 'platform-b', 'platform-c']) {
    await queueForDelivery(platform, xml);
  }
});
```

## Streaming Platforms

### Streaming Platform - Ingestion Pipeline

**Scenario (illustrative)**: A streaming service receives a large number of DDEX messages every day from many suppliers and needs to normalize them for internal processing.

```python
from ddex_parser import DDEXParser
from ddex_builder import DdexBuilder, Release

parser = DDEXParser()

def normalize_incoming_ddex(raw_xml: str) -> dict:
    """Normalize supplier DDEX into an internal ERN message and searchable rows."""

    # Detect the ERN version (3.8.2, 4.2 or 4.3) before parsing
    version = parser.detect_version(raw_xml)

    # Parse into a DataFrame with one row per release
    releases_df = parser.to_dataframe(raw_xml, schema='releases')

    # Apply platform-specific business rules
    releases_df['content_rating'] = releases_df.apply(derive_content_rating, axis=1)

    # Rebuild a normalized message for internal systems
    builder = DdexBuilder()
    for row in releases_df.to_dict('records'):
        builder.add_release(Release(
            release_id=row['release_id'],
            release_type='Album',
            title=row['title'],
            artist=row['artist'],
            metadata={'content_rating': row['content_rating']},
        ))

    return {
        'source_version': version,
        'normalized_xml': builder.build(),
        'metadata': extract_searchable_metadata(releases_df),
    }
```

## Enterprise Catalog Management

### Major Label Group - Multi-Format Delivery

**Scenario (illustrative)**: A label group needs to deliver the same release in different formats (streaming, download, physical) with format-specific metadata.

```python
from enum import Enum

from ddex_builder import DdexBuilder, Release, Resource

class ReleaseFormat(Enum):
    STREAMING = "streaming"
    DOWNLOAD = "download"
    PHYSICAL_CD = "physical_cd"
    VINYL = "vinyl"

class MultiFormatBuilder:
    def build_format_specific_release(self, master_release: dict, format_type: ReleaseFormat) -> str:
        """Generate format-specific DDEX from a master release record."""
        builder = DdexBuilder()

        metadata = {'format': format_type.value}
        if format_type == ReleaseFormat.VINYL:
            metadata.update({'configuration': '2xLP', 'speed': '33RPM'})

        builder.add_release(Release(
            release_id=f"{master_release['upc']}-{format_type.value}",
            release_type=master_release['release_type'],
            title=master_release['title'],
            artist=master_release['artist'],
            label=master_release['label'],
            upc=master_release['upc'],
            release_date=master_release['release_date'],
            track_ids=[track['isrc'] for track in master_release['tracks']],
            metadata=metadata,
        ))

        for position, track in enumerate(master_release['tracks'], start=1):
            builder.add_resource(Resource(
                resource_id=track['isrc'],
                resource_type='SoundRecording',
                title=track['title'],
                artist=track.get('artist', master_release['artist']),
                isrc=track['isrc'],
                duration=track['duration'],  # ISO 8601, e.g. 'PT3M45S'
                track_number=position,
            ))

        return builder.build()
```

## Trifecta - The "Parse → Modify → Build" Workflow

This is the primary use case, demonstrating the full suite:

```typescript
const { DdexParser } = require('ddex-parser');
const { DdexBuilder } = require('ddex-builder');
const fs = require('fs/promises');

const parser = new DdexParser();

// 1. PARSE an existing message
const originalXml = await fs.readFile('path/to/original.xml', 'utf8');
const parsed = await parser.parse(originalXml);

// 2. MODIFY the data in a simple, programmatic way
const builder = new DdexBuilder();

for (const release of parsed.releases) {
  const trackIds = release.tracks.map(track => track.trackId);
  if (release === parsed.releases[0]) {
    trackIds.push('USXYZ2600001'); // add a bonus track to the first release
  }

  builder.addRelease({
    releaseId: release.releaseId,
    releaseType: release.releaseType,
    title: release.title,
    artist: release.displayArtist,
    label: release.labelName,
    releaseDate: '2026-03-01', // move the release date
    trackIds,
  });

  for (const track of release.tracks) {
    builder.addResource({
      resourceId: track.trackId,
      resourceType: 'SoundRecording',
      title: track.title,
      artist: track.artist,
      isrc: track.isrc,
      duration: track.duration,
      trackNumber: track.position,
    });
  }
}

builder.addResource({
  resourceId: 'USXYZ2600001',
  resourceType: 'SoundRecording',
  title: 'New Bonus Track',
  artist: parsed.releases[0].displayArtist,
  isrc: 'USXYZ2600001',
  duration: 'PT3M0S',
});

// 3. BUILD a new XML message from the modified data
const validation = await builder.validate();
if (validation.warnings.length > 0) {
  console.warn('Build warnings:', validation.warnings);
}

const xml = await builder.build();
await fs.writeFile('path/to/updated.xml', xml);
```

## Next Steps

Ready to implement these use cases in your organization?

- **Get Started**: Follow our [Getting Started Guide](/docs/getting-started) for installation
- **API Reference**: Explore the complete [API Documentation](/docs/api)
- **Examples**: See more detailed [Examples](/docs/examples) with full code samples
- **Performance**: Learn about [Performance Optimization](/docs/guides/performance-tuning) for large-scale deployments
