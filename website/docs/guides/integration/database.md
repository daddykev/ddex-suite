# Database Integration

Integrate DDEX Suite with your database systems for automated data processing and storage.

## Overview

Database integration enables you to:
- Process DDEX data directly from database queries
- Store parsed results in structured database tables
- Implement automated ETL pipelines
- Scale processing across multiple database instances

## PostgreSQL Integration

### Setup

```sql
-- Create tables for DDEX data
CREATE TABLE releases (
    id SERIAL PRIMARY KEY,
    ddex_id VARCHAR(255) UNIQUE,
    title VARCHAR(500) NOT NULL,
    artist VARCHAR(500) NOT NULL,
    label VARCHAR(255),
    release_date DATE,
    territory_codes TEXT[],
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE tracks (
    id SERIAL PRIMARY KEY,
    release_id INTEGER REFERENCES releases(id),
    ddex_id VARCHAR(255),
    title VARCHAR(500) NOT NULL,
    artist VARCHAR(500) NOT NULL,
    isrc VARCHAR(12),
    duration_ms INTEGER,
    track_number INTEGER,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_releases_ddex_id ON releases(ddex_id);
CREATE INDEX idx_releases_artist ON releases(artist);
CREATE INDEX idx_releases_release_date ON releases(release_date);
CREATE INDEX idx_tracks_isrc ON tracks(isrc);
CREATE INDEX idx_tracks_release_id ON tracks(release_id);
```

### Data Processing Pipeline

```python
import asyncpg
import asyncio
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

class DDEXDatabaseProcessor:
    def __init__(self, database_url):
        self.database_url = database_url
        self.parser = DDEXParser()
        self.builder = DDEXBuilder()
    
    async def process_incoming_ddex(self, xml_content, source_id):
        """Process DDEX XML and store in database"""
        pool = await asyncpg.create_pool(self.database_url)
        
        try:
            # Parse DDEX content
            parsed = self.parser.parse(xml_content)
            
            async with pool.acquire() as conn:
                async with conn.transaction():
                    # Insert or update release
                    release_id = await self.upsert_release(conn, parsed.flat.releases[0])
                    
                    # Process tracks
                    for track in parsed.flat.tracks:
                        await self.upsert_track(conn, release_id, track)
                    
                    # Log processing
                    await conn.execute("""
                        INSERT INTO processing_log (source_id, status, processed_at)
                        VALUES ($1, 'success', NOW())
                    """, source_id)
                    
        finally:
            await pool.close()
    
    async def upsert_release(self, conn, release):
        """Insert or update release data"""
        return await conn.fetchval("""
            INSERT INTO releases (ddex_id, title, artist, label, release_date, territory_codes, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (ddex_id) 
            DO UPDATE SET
                title = EXCLUDED.title,
                artist = EXCLUDED.artist,
                label = EXCLUDED.label,
                release_date = EXCLUDED.release_date,
                territory_codes = EXCLUDED.territory_codes,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            RETURNING id
        """, release.id, release.title, release.artist, release.label, 
             release.release_date, release.territory_codes, release.metadata)
    
    async def upsert_track(self, conn, release_id, track):
        """Insert or update track data"""
        await conn.execute("""
            INSERT INTO tracks (release_id, ddex_id, title, artist, isrc, duration_ms, track_number, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (release_id, ddex_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                artist = EXCLUDED.artist,
                isrc = EXCLUDED.isrc,
                duration_ms = EXCLUDED.duration_ms,
                track_number = EXCLUDED.track_number,
                metadata = EXCLUDED.metadata
        """, release_id, track.id, track.title, track.artist, 
             track.isrc, track.duration_ms, track.track_number, track.metadata)

# Usage
async def main():
    processor = DDEXDatabaseProcessor('postgresql://user:pass@localhost/ddex')
    
    with open('new_release.xml', 'r') as f:
        xml_content = f.read()
    
    await processor.process_incoming_ddex(xml_content, 'source_123')

asyncio.run(main())
```

## MongoDB Integration

Store DDEX data in MongoDB for flexible document-based storage:

```python
from pymongo import MongoClient
from ddex_parser import DDEXParser
import json

class DDEXMongoProcessor:
    def __init__(self, connection_string):
        self.client = MongoClient(connection_string)
        self.db = self.client.ddex_suite
        self.parser = DDEXParser()
    
    def process_ddex_document(self, xml_content, collection_name='releases'):
        """Process DDEX and store as MongoDB document"""
        
        # Parse DDEX
        parsed = self.parser.parse(xml_content)
        
        # Convert to MongoDB document format
        document = {
            '_id': parsed.flat.releases[0].id,
            'ddex_version': parsed.version,
            'parsed_at': datetime.utcnow(),
            'graph_data': parsed.graph.to_dict(),
            'flat_data': {
                'releases': [r.to_dict() for r in parsed.flat.releases],
                'tracks': [t.to_dict() for t in parsed.flat.tracks],
                'artists': [a.to_dict() for a in parsed.flat.artists]
            },
            'metadata': {
                'source': 'ddex-parser',
                'file_size': len(xml_content),
                'processing_time_ms': parsed.processing_time_ms
            }
        }
        
        # Store in MongoDB
        collection = self.db[collection_name]
        result = collection.replace_one(
            {'_id': document['_id']}, 
            document, 
            upsert=True
        )
        
        return result.upserted_id or result.matched_count

# Usage
processor = DDEXMongoProcessor('mongodb://localhost:27017/')
with open('release.xml', 'r') as f:
    result = processor.process_ddex_document(f.read())
print(f"Stored document: {result}")
```

## SQLite Integration

Lightweight database integration for smaller applications:

```typescript
import sqlite3 from 'sqlite3';
import { DDEXParser } from 'ddex-parser';

class DDEXSQLiteProcessor {
  private db: sqlite3.Database;
  private parser: DDEXParser;

  constructor(dbPath: string) {
    this.db = new sqlite3.Database(dbPath);
    this.parser = new DDEXParser();
    this.initDatabase();
  }

  private initDatabase() {
    const schema = `
      CREATE TABLE IF NOT EXISTS releases (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        artist TEXT NOT NULL,
        release_date TEXT,
        metadata TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      );
      
      CREATE TABLE IF NOT EXISTS tracks (
        id TEXT PRIMARY KEY,
        release_id TEXT REFERENCES releases(id),
        title TEXT NOT NULL,
        isrc TEXT,
        duration_ms INTEGER,
        metadata TEXT
      );
    `;
    
    this.db.exec(schema);
  }

  async processDDEX(xmlContent: string): Promise<void> {
    const parsed = await this.parser.parse(xmlContent);
    
    return new Promise((resolve, reject) => {
      this.db.serialize(() => {
        this.db.run('BEGIN TRANSACTION');
        
        // Insert release
        const release = parsed.flat.releases[0];
        this.db.run(
          'INSERT OR REPLACE INTO releases (id, title, artist, release_date, metadata) VALUES (?, ?, ?, ?, ?)',
          [release.id, release.title, release.artist, release.releaseDate, JSON.stringify(release)],
          function(err) {
            if (err) reject(err);
          }
        );
        
        // Insert tracks
        parsed.flat.tracks.forEach(track => {
          this.db.run(
            'INSERT OR REPLACE INTO tracks (id, release_id, title, isrc, duration_ms, metadata) VALUES (?, ?, ?, ?, ?, ?)',
            [track.id, release.id, track.title, track.isrc, track.durationMs, JSON.stringify(track)]
          );
        });
        
        this.db.run('COMMIT', (err) => {
          if (err) reject(err);
          else resolve();
        });
      });
    });
  }
}
```

## MySQL Integration

```python
import mysql.connector
from mysql.connector import Error
from ddex_parser import DDEXParser

class DDEXMySQLProcessor:
    def __init__(self, config):
        self.config = config
        self.parser = DDEXParser()
    
    def process_batch(self, xml_files):
        """Process multiple DDEX files in batch"""
        try:
            connection = mysql.connector.connect(**self.config)
            cursor = connection.cursor()
            
            # Prepare batch insert statements
            release_data = []
            track_data = []
            
            for xml_file in xml_files:
                with open(xml_file, 'r') as f:
                    parsed = self.parser.parse(f.read())
                
                for release in parsed.flat.releases:
                    release_data.append((
                        release.id,
                        release.title,
                        release.artist,
                        release.label,
                        release.release_date,
                        json.dumps(release.to_dict())
                    ))
                
                for track in parsed.flat.tracks:
                    track_data.append((
                        track.id,
                        track.release_id,
                        track.title,
                        track.isrc,
                        track.duration_ms,
                        json.dumps(track.to_dict())
                    ))
            
            # Batch insert releases
            cursor.executemany("""
                INSERT INTO releases (ddex_id, title, artist, label, release_date, metadata)
                VALUES (%s, %s, %s, %s, %s, %s)
                ON DUPLICATE KEY UPDATE
                title = VALUES(title),
                artist = VALUES(artist),
                label = VALUES(label),
                metadata = VALUES(metadata)
            """, release_data)
            
            # Batch insert tracks
            cursor.executemany("""
                INSERT INTO tracks (ddex_id, release_id, title, isrc, duration_ms, metadata)
                VALUES (%s, %s, %s, %s, %s, %s)
                ON DUPLICATE KEY UPDATE
                title = VALUES(title),
                isrc = VALUES(isrc),
                duration_ms = VALUES(duration_ms),
                metadata = VALUES(metadata)
            """, track_data)
            
            connection.commit()
            print(f"Processed {len(release_data)} releases and {len(track_data)} tracks")
            
        except Error as e:
            print(f"Database error: {e}")
            if connection.is_connected():
                connection.rollback()
        finally:
            if connection.is_connected():
                cursor.close()
                connection.close()

# Usage
config = {
    'host': 'localhost',
    'database': 'ddex_suite',
    'user': 'your_user',
    'password': 'your_password'
}

processor = DDEXMySQLProcessor(config)
processor.process_batch(['release1.xml', 'release2.xml', 'release3.xml'])
```

## Data Warehouse Integration

Integrate with data warehouses for analytics:

```python
from google.cloud import bigquery
from ddex_parser import DDEXParser
import pandas as pd

class DDEXBigQueryProcessor:
    def __init__(self, project_id, dataset_id):
        self.client = bigquery.Client(project=project_id)
        self.dataset_id = dataset_id
        self.parser = DDEXParser()
    
    def create_tables(self):
        """Create BigQuery tables for DDEX data"""
        
        release_schema = [
            bigquery.SchemaField("ddex_id", "STRING", mode="REQUIRED"),
            bigquery.SchemaField("title", "STRING", mode="REQUIRED"),
            bigquery.SchemaField("artist", "STRING", mode="REQUIRED"),
            bigquery.SchemaField("release_date", "DATE"),
            bigquery.SchemaField("territory_codes", "STRING", mode="REPEATED"),
            bigquery.SchemaField("metadata", "JSON"),
            bigquery.SchemaField("processed_at", "TIMESTAMP"),
        ]
        
        track_schema = [
            bigquery.SchemaField("ddex_id", "STRING", mode="REQUIRED"),
            bigquery.SchemaField("release_id", "STRING", mode="REQUIRED"),
            bigquery.SchemaField("title", "STRING", mode="REQUIRED"),
            bigquery.SchemaField("isrc", "STRING"),
            bigquery.SchemaField("duration_ms", "INTEGER"),
            bigquery.SchemaField("metadata", "JSON"),
        ]
        
        # Create tables
        self.create_table("releases", release_schema)
        self.create_table("tracks", track_schema)
    
    def process_to_dataframe(self, xml_content):
        """Process DDEX to DataFrame format suitable for BigQuery"""
        parsed = self.parser.parse(xml_content)
        df = self.parser.to_dataframe(xml_content)
        
        # Add processing metadata
        df['processed_at'] = pd.Timestamp.now()
        df['ddex_version'] = parsed.version
        
        return df
    
    def upload_dataframe(self, df, table_name):
        """Upload DataFrame to BigQuery"""
        table_id = f"{self.dataset_id}.{table_name}"
        
        job_config = bigquery.LoadJobConfig(
            write_disposition="WRITE_APPEND",
            autodetect=True
        )
        
        job = self.client.load_table_from_dataframe(df, table_id, job_config=job_config)
        job.result()  # Wait for the job to complete
        
        print(f"Loaded {len(df)} rows into {table_id}")

# Usage
processor = DDEXBigQueryProcessor('my-project', 'ddex_analytics')
processor.create_tables()

with open('releases.xml', 'r') as f:
    df = processor.process_to_dataframe(f.read())
    processor.upload_dataframe(df, 'releases')
```

## Performance Optimization

### Connection Pooling

```python
from sqlalchemy import create_engine, pool
from sqlalchemy.orm import sessionmaker
from ddex_parser import DDEXParser

class PooledDDEXProcessor:
    def __init__(self, database_url):
        self.engine = create_engine(
            database_url,
            poolclass=pool.QueuePool,
            pool_size=20,
            max_overflow=0,
            pool_pre_ping=True,
            pool_recycle=300
        )
        self.Session = sessionmaker(bind=self.engine)
        self.parser = DDEXParser()
    
    def process_with_pool(self, xml_files):
        """Process files using connection pool"""
        for xml_file in xml_files:
            session = self.Session()
            try:
                with open(xml_file, 'r') as f:
                    parsed = self.parser.parse(f.read())
                
                # Process data with current session
                self.store_parsed_data(session, parsed)
                session.commit()
                
            except Exception as e:
                session.rollback()
                print(f"Error processing {xml_file}: {e}")
            finally:
                session.close()
```

### Bulk Operations

```sql
-- Optimize with bulk operations
COPY releases (ddex_id, title, artist, metadata)
FROM '/path/to/bulk_releases.csv'
WITH (FORMAT csv, HEADER true);

-- Use UPSERT for efficient updates
INSERT INTO releases (ddex_id, title, artist, metadata)
VALUES %s
ON CONFLICT (ddex_id) 
DO UPDATE SET
  title = EXCLUDED.title,
  artist = EXCLUDED.artist,
  metadata = EXCLUDED.metadata;
```

## Best Practices

1. **Use connection pooling** for high-throughput applications
2. **Implement proper indexing** on frequently queried fields
3. **Use transactions** for data consistency
4. **Batch operations** for better performance
5. **Handle errors gracefully** with proper rollback mechanisms
6. **Monitor database performance** and optimize queries
7. **Implement data retention policies** for large datasets
8. **Use appropriate data types** for storage efficiency