# Testing

Comprehensive testing strategies for DDEX Suite integrations and applications.

## Overview

Effective testing ensures:
- Reliable DDEX processing
- Data integrity validation
- Performance requirements
- Error handling coverage
- Integration compatibility

## Unit Testing

### JavaScript/TypeScript

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';
import fs from 'fs/promises';

describe('DDEX Parser', () => {
  let parser: DDEXParser;
  let sampleXML: string;

  beforeEach(async () => {
    parser = new DDEXParser();
    sampleXML = await fs.readFile('test/fixtures/sample-release.xml', 'utf-8');
  });

  it('should parse valid DDEX XML', async () => {
    const result = await parser.parse(sampleXML);
    
    expect(result.version).toBe('4.3');
    expect(result.flat.releases).toHaveLength(1);
    expect(result.flat.releases[0].title).toBe('Test Album');
    expect(result.flat.tracks).toHaveLength(3);
  });

  it('should handle invalid XML gracefully', async () => {
    const invalidXML = '<invalid>xml</invalid>';
    
    await expect(parser.parse(invalidXML)).rejects.toThrow();
  });

  it('should validate ISRC format', async () => {
    const xmlWithInvalidISRC = sampleXML.replace(
      'USRC17607839',
      'INVALID_ISRC'
    );
    
    await expect(parser.parse(xmlWithInvalidISRC)).rejects.toThrow(/Invalid ISRC/);
  });

  it('should preserve round-trip fidelity', async () => {
    const parser = new DDEXParser();
    const builder = new DDEXBuilder();
    
    const parsed = await parser.parse(sampleXML);
    const rebuilt = await builder.build(parsed.toBuildRequest());
    const reparsed = await parser.parse(rebuilt);
    
    expect(reparsed.flat.releases[0].title).toBe(parsed.flat.releases[0].title);
    expect(reparsed.flat.tracks).toHaveLength(parsed.flat.tracks.length);
  });
});

describe('DDEX Builder', () => {
  let builder: DDEXBuilder;
  let sampleData: any;

  beforeEach(async () => {
    builder = new DDEXBuilder();
    sampleData = JSON.parse(
      await fs.readFile('test/fixtures/sample-data.json', 'utf-8')
    );
  });

  it('should build valid DDEX XML', async () => {
    const xml = await builder.build(sampleData);
    
    expect(xml).toContain('<?xml version="1.0" encoding="UTF-8"?>');
    expect(xml).toContain('<NewReleaseMessage');
    expect(xml).toContain('MessageSchemaVersionId="ern/43"');
  });

  it('should validate required fields', async () => {
    const incompleteData = { ...sampleData };
    delete incompleteData.releases[0].title;
    
    await expect(builder.build(incompleteData)).rejects.toThrow(/Required field: title/);
  });

  it('should produce deterministic output', async () => {
    const xml1 = await builder.build(sampleData);
    const xml2 = await builder.build(sampleData);
    
    expect(xml1).toBe(xml2);
  });

  it('should handle batch processing', async () => {
    const batch = [sampleData, { ...sampleData, id: 'release-2' }];
    const results = await builder.buildBatch(batch);
    
    expect(results.success).toHaveLength(2);
    expect(results.errors).toHaveLength(0);
  });
});
```

### Python Testing

```python
import pytest
import json
from pathlib import Path
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

class TestDDEXParser:
    def setup_method(self):
        self.parser = DDEXParser()
        
        # Load test fixtures
        self.sample_xml = (Path(__file__).parent / 'fixtures' / 'sample-release.xml').read_text()
        
    def test_parse_valid_xml(self):
        result = self.parser.parse(self.sample_xml)
        
        assert result.version == '4.3'
        assert len(result.flat.releases) == 1
        assert result.flat.releases[0].title == 'Test Album'
        assert len(result.flat.tracks) == 3
        
    def test_parse_invalid_xml(self):
        invalid_xml = '<invalid>xml</invalid>'
        
        with pytest.raises(Exception):
            self.parser.parse(invalid_xml)
            
    def test_parse_to_dataframe(self):
        df = self.parser.to_dataframe(self.sample_xml)
        
        assert not df.empty
        assert 'title' in df.columns
        assert 'artist' in df.columns
        assert len(df) > 0
        
    def test_performance_large_file(self):
        import time
        
        # Create large XML content (simulate 10MB file)
        large_xml = self.sample_xml * 100
        
        start_time = time.time()
        result = self.parser.parse(large_xml)
        processing_time = time.time() - start_time
        
        assert processing_time < 1.0  # Should process in under 1 second
        assert result is not None

class TestDDEXBuilder:
    def setup_method(self):
        self.builder = DDEXBuilder()
        
        with open(Path(__file__).parent / 'fixtures' / 'sample-data.json') as f:
            self.sample_data = json.load(f)
            
    def test_build_valid_xml(self):
        xml = self.builder.build(self.sample_data)
        
        assert xml.startswith('<?xml version="1.0" encoding="UTF-8"?>')
        assert '<NewReleaseMessage' in xml
        assert 'MessageSchemaVersionId="ern/43"' in xml
        
    def test_validation_errors(self):
        incomplete_data = self.sample_data.copy()
        del incomplete_data['releases'][0]['title']
        
        with pytest.raises(Exception) as exc_info:
            self.builder.build(incomplete_data)
            
        assert 'Required field: title' in str(exc_info.value)
        
    def test_deterministic_output(self):
        xml1 = self.builder.build(self.sample_data)
        xml2 = self.builder.build(self.sample_data)
        
        assert xml1 == xml2
        
    def test_partner_presets(self):
        spotify_xml = self.builder.build(self.sample_data, partner_preset='spotify')
        youtube_xml = self.builder.build(self.sample_data, partner_preset='youtube')
        
        assert spotify_xml != youtube_xml
        assert 'spotify' not in spotify_xml.lower()  # No partner branding in XML
        
    def test_batch_validation(self):
        batch_data = [self.sample_data, self.sample_data.copy()]
        batch_data[1]['id'] = 'release-2'
        
        validation = self.builder.validate_batch(batch_data)
        
        assert len(validation.valid) == 2
        assert len(validation.invalid) == 0

@pytest.fixture(scope="session")
def sample_ddex_files():
    """Provide sample DDEX files for testing"""
    fixtures_dir = Path(__file__).parent / 'fixtures'
    return list(fixtures_dir.glob('*.xml'))

@pytest.mark.parametrize("ddex_file", sample_ddex_files())
def test_parse_all_fixtures(ddex_file):
    """Test parser against all fixture files"""
    parser = DDEXParser()
    
    with open(ddex_file) as f:
        xml_content = f.read()
        
    result = parser.parse(xml_content)
    assert result is not None
    assert result.flat.releases  # Should have at least one release
```

### Rust Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ddex_parser::DDEXParser;
    use ddex_builder::DDEXBuilder;
    use std::fs;
    
    #[test]
    fn test_parse_valid_ddex() {
        let parser = DDEXParser::new();
        let xml_content = fs::read_to_string("tests/fixtures/sample-release.xml")
            .expect("Failed to read test file");
            
        let result = parser.parse(&xml_content).expect("Failed to parse");
        
        assert_eq!(result.version(), "4.3");
        assert_eq!(result.flat().releases().len(), 1);
        assert_eq!(result.flat().releases()[0].title(), "Test Album");
    }
    
    #[test]
    fn test_build_deterministic() {
        let builder = DDEXBuilder::new();
        let data = serde_json::json!({
            "releases": [{
                "id": "R001",
                "title": "Test Album",
                "artist": "Test Artist"
            }]
        });
        
        let xml1 = builder.build(&data).expect("First build failed");
        let xml2 = builder.build(&data).expect("Second build failed");
        
        assert_eq!(xml1, xml2, "Output should be deterministic");
    }
    
    #[test]
    fn test_round_trip_fidelity() {
        let parser = DDEXParser::new();
        let builder = DDEXBuilder::new();
        
        let original_xml = fs::read_to_string("tests/fixtures/sample-release.xml")
            .expect("Failed to read test file");
            
        let parsed = parser.parse(&original_xml).expect("Parse failed");
        let rebuilt_xml = builder.build(&parsed.to_build_request()).expect("Build failed");
        let reparsed = parser.parse(&rebuilt_xml).expect("Re-parse failed");
        
        assert_eq!(
            parsed.flat().releases()[0].title(),
            reparsed.flat().releases()[0].title(),
            "Round-trip should preserve data"
        );
    }
    
    #[tokio::test]
    async fn test_concurrent_processing() {
        use tokio::task;
        
        let parser = DDEXParser::new();
        let xml_content = fs::read_to_string("tests/fixtures/sample-release.xml")
            .expect("Failed to read test file");
            
        let tasks: Vec<_> = (0..10).map(|_| {
            let parser = parser.clone();
            let xml = xml_content.clone();
            
            task::spawn(async move {
                parser.parse(&xml)
            })
        }).collect();
        
        let results = futures::future::join_all(tasks).await;
        
        for result in results {
            assert!(result.unwrap().is_ok(), "Concurrent parsing should succeed");
        }
    }
    
    #[bench]
    fn bench_parse_performance(b: &mut test::Bencher) {
        let parser = DDEXParser::new();
        let xml_content = fs::read_to_string("tests/fixtures/large-release.xml")
            .expect("Failed to read test file");
            
        b.iter(|| {
            parser.parse(&xml_content).expect("Parse failed")
        });
    }
}
```

## Integration Testing

### Database Integration Tests

```python
import pytest
import asyncpg
import asyncio
from testcontainers import compose
from ddex_parser import DDEXParser
from your_app.database import DDEXDatabaseProcessor

@pytest.fixture(scope="session")
def postgres_container():
    """Start PostgreSQL container for testing"""
    with compose.DockerCompose("tests/docker-compose.test.yml") as composition:
        postgres_url = composition.get_service_host("postgres", 5432)
        yield f"postgresql://test:test@{postgres_url}/test_ddex"

@pytest.mark.asyncio
async def test_database_integration(postgres_container):
    processor = DDEXDatabaseProcessor(postgres_container)
    
    # Load test data
    with open('tests/fixtures/sample-release.xml') as f:
        xml_content = f.read()
    
    # Process and store
    await processor.process_incoming_ddex(xml_content, 'test_source_1')
    
    # Verify data was stored
    pool = await asyncpg.create_pool(postgres_container)
    async with pool.acquire() as conn:
        releases = await conn.fetch("SELECT * FROM releases")
        tracks = await conn.fetch("SELECT * FROM tracks")
        
        assert len(releases) == 1
        assert releases[0]['title'] == 'Test Album'
        assert len(tracks) == 3
    
    await pool.close()

@pytest.mark.asyncio
async def test_batch_processing(postgres_container):
    processor = DDEXDatabaseProcessor(postgres_container)
    
    # Process multiple files
    xml_files = [
        'tests/fixtures/release-1.xml',
        'tests/fixtures/release-2.xml',
        'tests/fixtures/release-3.xml'
    ]
    
    for i, xml_file in enumerate(xml_files):
        with open(xml_file) as f:
            await processor.process_incoming_ddex(f.read(), f'batch_test_{i}')
    
    # Verify all data was stored
    pool = await asyncpg.create_pool(postgres_container)
    async with pool.acquire() as conn:
        count = await conn.fetchval("SELECT COUNT(*) FROM releases")
        assert count == 3
    
    await pool.close()
```

### API Integration Tests

```typescript
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import { createApp } from '../src/app';
import fs from 'fs/promises';

describe('DDEX API Integration', () => {
  let app: any;
  let server: any;

  beforeAll(async () => {
    app = createApp();
    server = app.listen(0); // Random port
  });

  afterAll(async () => {
    await server.close();
  });

  it('should parse uploaded DDEX file', async () => {
    const xmlContent = await fs.readFile('test/fixtures/sample-release.xml');
    
    const response = await request(app)
      .post('/api/ddex/parse')
      .attach('ddex', xmlContent, 'test-release.xml')
      .expect(200);

    expect(response.body.success).toBe(true);
    expect(response.body.data.version).toBe('4.3');
    expect(response.body.data.flat.releases).toHaveLength(1);
  });

  it('should build DDEX from JSON data', async () => {
    const buildData = JSON.parse(
      await fs.readFile('test/fixtures/sample-data.json', 'utf-8')
    );

    const response = await request(app)
      .post('/api/ddex/build')
      .send({ data: buildData })
      .expect(200);

    expect(response.body.success).toBe(true);
    expect(response.body.data.xml).toContain('<?xml');
    expect(response.body.data.xml).toContain('<NewReleaseMessage');
  });

  it('should handle validation errors', async () => {
    const invalidData = {
      releases: [{
        // Missing required title field
        artist: 'Test Artist'
      }]
    };

    const response = await request(app)
      .post('/api/ddex/build')
      .send({ data: invalidData })
      .expect(400);

    expect(response.body.success).toBe(false);
    expect(response.body.error).toContain('Validation failed');
  });

  it('should process batch operations', async () => {
    const batchData = [
      JSON.parse(await fs.readFile('test/fixtures/sample-data.json', 'utf-8')),
      JSON.parse(await fs.readFile('test/fixtures/sample-data-2.json', 'utf-8'))
    ];

    const response = await request(app)
      .post('/api/ddex/batch')
      .send({
        operation: 'build',
        items: batchData
      })
      .expect(200);

    expect(response.body.success).toBe(true);
    expect(response.body.data.successful).toBe(2);
    expect(response.body.data.failed).toBe(0);
  });

  it('should handle file upload errors', async () => {
    const response = await request(app)
      .post('/api/ddex/parse')
      .attach('ddex', Buffer.from('<invalid>xml</invalid>'), 'invalid.xml')
      .expect(400);

    expect(response.body.success).toBe(false);
    expect(response.body.error).toBeDefined();
  });
});
```

## Performance Testing

### Load Testing with Artillery

```yaml
# artillery-config.yml
config:
  target: 'http://localhost:3000'
  phases:
    - duration: 60
      arrivalRate: 10
      name: "Warm up"
    - duration: 300
      arrivalRate: 50
      name: "Sustained load"
    - duration: 120
      arrivalRate: 100
      name: "Peak load"

scenarios:
  - name: "Parse DDEX files"
    weight: 70
    flow:
      - post:
          url: "/api/ddex/parse/text"
          json:
            xml: "{{ $processTemplateString(sampleXML) }}"
      - think: 1

  - name: "Build DDEX files"
    weight: 30
    flow:
      - post:
          url: "/api/ddex/build"
          json:
            data: "{{ sampleData }}"
            options:
              validation_level: "standard"
      - think: 2
```

```bash
# Run load test
artillery run artillery-config.yml --output report.json
artillery report report.json
```

### Benchmark Tests

```python
import pytest
import time
import statistics
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

class TestPerformance:
    def setup_method(self):
        self.parser = DDEXParser()
        self.builder = DDEXBuilder()
        
        # Load test data
        with open('tests/fixtures/large-release.xml') as f:
            self.large_xml = f.read()
        
        with open('tests/fixtures/complex-data.json') as f:
            import json
            self.complex_data = json.load(f)
    
    def test_parse_performance(self):
        """Test parsing performance meets requirements"""
        times = []
        
        for _ in range(10):
            start = time.time()
            result = self.parser.parse(self.large_xml)
            end = time.time()
            times.append(end - start)
        
        avg_time = statistics.mean(times)
        p95_time = statistics.quantiles(times, n=20)[18]  # 95th percentile
        
        # Performance requirements
        assert avg_time < 0.5, f"Average parse time {avg_time:.3f}s exceeds 0.5s limit"
        assert p95_time < 1.0, f"95th percentile {p95_time:.3f}s exceeds 1.0s limit"
        
    def test_build_performance(self):
        """Test building performance meets requirements"""
        times = []
        
        for _ in range(10):
            start = time.time()
            xml = self.builder.build(self.complex_data)
            end = time.time()
            times.append(end - start)
        
        avg_time = statistics.mean(times)
        p95_time = statistics.quantiles(times, n=20)[18]
        
        assert avg_time < 0.3, f"Average build time {avg_time:.3f}s exceeds 0.3s limit"
        assert p95_time < 0.6, f"95th percentile {p95_time:.3f}s exceeds 0.6s limit"
    
    def test_memory_usage(self):
        """Test memory usage stays within limits"""
        import psutil
        import os
        
        process = psutil.Process(os.getpid())
        baseline_memory = process.memory_info().rss
        
        # Process large file
        result = self.parser.parse(self.large_xml)
        peak_memory = process.memory_info().rss
        
        memory_increase = peak_memory - baseline_memory
        memory_mb = memory_increase / (1024 * 1024)
        
        assert memory_mb < 100, f"Memory usage {memory_mb:.1f}MB exceeds 100MB limit"
```

## Property-Based Testing

```python
from hypothesis import given, strategies as st
from hypothesis.extra import django
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Generate test data strategies
release_strategy = st.builds(
    dict,
    id=st.text(min_size=1, max_size=50),
    title=st.text(min_size=1, max_size=200),
    artist=st.text(min_size=1, max_size=100),
    release_date=st.dates().map(str),
    territory_codes=st.lists(st.sampled_from(['US', 'GB', 'DE', 'FR', 'JP']), min_size=1)
)

@given(release_data=release_strategy)
def test_roundtrip_property(release_data):
    """Property: Parse -> Build -> Parse should preserve data"""
    parser = DDEXParser()
    builder = DDEXBuilder()
    
    try:
        # Build from data
        xml = builder.build({'releases': [release_data]})
        
        # Parse back
        parsed = parser.parse(xml)
        
        # Verify key properties preserved
        assert len(parsed.flat.releases) == 1
        assert parsed.flat.releases[0].title == release_data['title']
        assert parsed.flat.releases[0].artist == release_data['artist']
        
    except Exception as e:
        # Some generated data might be invalid, which is ok
        pytest.assume(False, f"Generated invalid data: {e}")

@given(st.text())
def test_parser_robustness(xml_content):
    """Property: Parser should handle any string input gracefully"""
    parser = DDEXParser()
    
    try:
        result = parser.parse(xml_content)
        # If parsing succeeds, result should be valid
        assert result is not None
        assert hasattr(result, 'flat')
        assert hasattr(result, 'graph')
    except Exception:
        # Parsing can fail for invalid XML, which is expected
        pass
```

## Test Data Management

### Fixture Generation

```python
import json
from faker import Faker
from ddex_builder import DDEXBuilder

fake = Faker()

def generate_test_release():
    """Generate realistic test release data"""
    return {
        'id': f"R{fake.random_int(10000, 99999)}",
        'title': fake.catch_phrase(),
        'artist': fake.name(),
        'label': fake.company(),
        'release_date': fake.date_between(start_date='-5y', end_date='today').isoformat(),
        'territory_codes': fake.random_choices(['US', 'GB', 'DE', 'FR', 'JP', 'CA'], length=3),
        'tracks': [
            {
                'id': f"T{fake.random_int(10000, 99999)}",
                'title': fake.catch_phrase(),
                'artist': fake.name(),
                'isrc': f"{fake.country_code()}{fake.random_int(10, 99)}{fake.random_int(10000000, 99999999)}",
                'duration_ms': fake.random_int(120000, 360000),  # 2-6 minutes
                'track_number': i + 1
            }
            for i in range(fake.random_int(3, 12))
        ]
    }

def create_test_fixtures():
    """Create a variety of test fixtures"""
    builder = DDEXBuilder()
    
    fixtures = {
        'simple_release': {
            'releases': [generate_test_release()]
        },
        'multi_release': {
            'releases': [generate_test_release() for _ in range(3)]
        },
        'large_release': {
            'releases': [{
                **generate_test_release(),
                'tracks': [generate_test_track() for _ in range(50)]
            }]
        }
    }
    
    for name, data in fixtures.items():
        # Save JSON data
        with open(f'tests/fixtures/{name}.json', 'w') as f:
            json.dump(data, f, indent=2)
        
        # Generate corresponding XML
        xml = builder.build(data)
        with open(f'tests/fixtures/{name}.xml', 'w') as f:
            f.write(xml)

if __name__ == '__main__':
    create_test_fixtures()
```

## Continuous Integration

### GitHub Actions Test Workflow

```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: [16, 18, 20]
        python-version: [3.9, 3.10, 3.11]
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test_ddex
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: ${{ matrix.node-version }}
          
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
      
      - name: Install dependencies
        run: |
          npm ci
          pip install -r requirements-test.txt
          
      - name: Run JavaScript tests
        run: npm test
        
      - name: Run Python tests
        env:
          DATABASE_URL: postgresql://postgres:test@localhost:5432/test_ddex
        run: pytest -v --cov=ddex_suite
        
      - name: Run integration tests
        env:
          DATABASE_URL: postgresql://postgres:test@localhost:5432/test_ddex
        run: pytest tests/integration/ -v
        
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Best Practices

1. **Test Pyramid**: More unit tests, fewer integration tests, minimal E2E
2. **Realistic Test Data**: Use representative DDEX files and data
3. **Performance Testing**: Include benchmarks in your test suite
4. **Error Scenarios**: Test error conditions and edge cases
5. **Round-Trip Testing**: Verify Parse → Build → Parse fidelity
6. **Concurrent Testing**: Test thread safety and concurrent access
7. **Database Testing**: Use test containers for database integration
8. **Mocking**: Mock external dependencies appropriately
9. **Test Coverage**: Aim for high coverage but focus on critical paths
10. **CI/CD Integration**: Run tests on every commit and deployment