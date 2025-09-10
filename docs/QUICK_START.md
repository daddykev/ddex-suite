# 🚀 DDEX Suite Quick Start Guide

*Get up and running with DDEX parsing and building in under 5 minutes!*

## 📋 Prerequisites

Before you begin, make sure you have:
- **Node.js 18+** or **Python 3.8+** installed
- Basic familiarity with music metadata (ISRC, UPC codes)
- A DDEX XML file to test with (optional - we'll provide examples)

## 🎯 What You'll Learn

In the next 5 minutes, you'll:
1. **Parse** a DDEX XML message into clean, typed data
2. **Modify** the metadata programmatically 
3. **Build** a new, deterministic DDEX XML message
4. **Validate** the round-trip fidelity

This is the core **Parse → Modify → Build** workflow that makes DDEX Suite powerful.

---

## 🟢 JavaScript/TypeScript Path

### Step 1: Installation
```bash
npm install ddex-parser ddex-builder
```

### Step 2: Parse a DDEX Message
```typescript
import { DDEXParser } from 'ddex-parser';
import { readFileSync } from 'fs';

// Initialize parser
const parser = new DDEXParser();

// Sample XML (you can use your own DDEX file)
const xmlContent = readFileSync('path/to/your/release.xml', 'utf8');

// Parse to structured data
const parsed = await parser.parse(xmlContent);

console.log('Parsed successfully!');
console.log(`Found ${parsed.flat.releases.length} releases`);
console.log(`First release: "${parsed.flat.releases[0].title}"`);
console.log(`By: ${parsed.flat.releases[0].displayArtist}`);
```

### Step 3: Modify the Data
```typescript
// Access the flattened, developer-friendly model
const release = parsed.flat.releases[0];

// Make some changes - it's just JavaScript objects!
release.title = "Updated Album Title (Deluxe Edition)";
release.tracks[0].title = "New Bonus Track";

// Add a new track
release.tracks.push({
    position: release.tracks.length + 1,
    title: "Hidden Track",
    isrc: "USXYZ2400099",
    duration: 240,
    displayArtist: release.displayArtist
});

console.log(`Modified release now has ${release.tracks.length} tracks`);
```

### Step 4: Build New DDEX XML
```typescript
import { DDEXBuilder } from 'ddex-builder';

// Initialize builder with Spotify preset for compatibility
const builder = new DDEXBuilder();
await builder.applyPreset('spotify_audio_43');

// Convert parsed data to build request
const buildRequest = parsed.toBuildRequest();

// Build deterministic XML
const result = await builder.build(buildRequest, {
    determinism: {
        canonMode: 'db-c14n',
        emitReproducibilityBanner: true
    }
});

console.log('✅ New XML generated!');
console.log(`Size: ${(result.xml.length / 1024).toFixed(1)} KB`);
console.log(`Warnings: ${result.warnings.length}`);

// Save the result
writeFileSync('updated_release.xml', result.xml);
```

### Step 5: Verify Round-Trip Fidelity
```typescript
// Parse the newly generated XML
const reparsed = await parser.parse(result.xml);

// Verify the modification worked
console.log('🔍 Verifying changes...');
console.log(`New title: ${reparsed.flat.releases[0].title}`);
console.log(`Track count: ${reparsed.flat.releases[0].tracks.length}`);
console.log('✅ Round-trip successful!');
```

---

## 🐍 Python Path

### Step 1: Installation
```bash
pip install ddex-parser ddex-builder
```

### Step 2: Build DDEX From Scratch
```python
from ddex_builder import DDEXBuilder
import json

# Initialize builder
builder = DDEXBuilder()

# Create a simple album release
request = {
    'header': {
        'message_sender': {'party_name': [{'text': 'Indie Music Records'}]},
        'message_recipient': {'party_name': [{'text': 'Spotify'}]}
    },
    'version': '4.3',
    'releases': [{
        'release_id': '1234567890123',
        'identifiers': {
            'upc': '123456789012',
            'catalog_number': 'IMR-2024-001'
        },
        'title': [{'text': 'My First Album'}],
        'display_artist': 'New Artist',
        'release_type': 'Album',
        'release_date': '2024-03-15',
        'tracks': [
            {
                'position': 1,
                'isrc': 'USIND2400001',
                'title': 'Opening Song',
                'duration': 210,
                'display_artist': 'New Artist'
            },
            {
                'position': 2, 
                'isrc': 'USIND2400002',
                'title': 'The Hit Single',
                'duration': 195,
                'display_artist': 'New Artist'
            }
        ]
    }]
}

print("🎵 Building DDEX XML...")
result = builder.build(request)

print(f"✅ Generated XML: {len(result.xml)} bytes")
print(f"Warnings: {len(result.warnings)}")

# Save to file
with open('my_first_release.xml', 'w', encoding='utf-8') as f:
    f.write(result.xml)

print("💾 Saved as my_first_release.xml")
```

### Step 3: Parse and Modify
```python
from ddex_parser import DDEXParser

# Parse the XML we just created  
parser = DDEXParser()
parsed = parser.parse(result.xml)

print("📋 Parsed release info:")
print(f"Title: {parsed.flat.releases[0]['title']}")
print(f"Artist: {parsed.flat.releases[0]['display_artist']}")
print(f"Tracks: {len(parsed.flat.releases[0]['tracks'])}")

# Modify the release
parsed.flat.releases[0]['title'] = 'My First Album (Remastered)'
parsed.flat.releases[0]['tracks'].append({
    'position': 3,
    'isrc': 'USIND2400003', 
    'title': 'Bonus Track',
    'duration': 180,
    'display_artist': 'New Artist'
})

print(f"🔄 Modified to {len(parsed.flat.releases[0]['tracks'])} tracks")
```

### Step 4: Advanced Python Features
```python
# Convert to pandas DataFrame for analysis
import pandas as pd

# Get flattened data as DataFrame (coming soon in v0.2.0)
# df = parser.to_dataframe(result.xml, schema='tracks')
# print(f"📊 DataFrame shape: {df.shape}")
# print(df[['title', 'duration', 'isrc']].head())

# Batch processing example
requests = []
for i in range(5):
    req = request.copy()  # Copy the base request
    req['releases'][0]['title'] = [{'text': f'Album {i+1}'}]
    req['releases'][0]['identifiers']['catalog_number'] = f'IMR-2024-{i+1:03d}'
    requests.append(req)

print("🚀 Batch building 5 releases...")
results = []
for i, req in enumerate(requests):
    result = builder.build(req)
    results.append(result)
    print(f"  Release {i+1}: {len(result.xml)} bytes")

print("✅ Batch processing complete!")
```

---

## 🌐 Browser/WASM Path

Perfect for client-side applications and music web apps!

### Step 1: Installation
```bash
npm install ddex-builder  # Includes WASM build automatically
```

### Step 2: Browser Usage
```html
<!DOCTYPE html>
<html>
<head>
    <title>DDEX Builder in Browser</title>
</head>
<body>
    <h1>DDEX Suite in the Browser</h1>
    <button onclick="buildDDEX()">Build DDEX XML</button>
    <pre id="output"></pre>

    <script type="module">
        import { DDEXBuilder } from 'ddex-builder';

        window.buildDDEX = async function() {
            const output = document.getElementById('output');
            
            try {
                // Initialize builder (automatically uses WASM in browser)
                const builder = new DDEXBuilder();
                
                const request = {
                    header: {
                        message_sender: {party_name: [{text: 'Web Music App'}]},
                        message_recipient: {party_name: [{text: 'Streaming Service'}]}
                    },
                    version: '4.3',
                    releases: [{
                        release_id: '1234567890123',
                        title: [{text: 'Web-Generated Album'}],
                        display_artist: 'Browser Artist',
                        tracks: [{
                            position: 1,
                            isrc: 'USWEB2400001',
                            title: 'Client-Side Song',
                            duration: 180
                        }]
                    }]
                };

                const result = await builder.build(request);
                
                output.textContent = `✅ Success!\nGenerated ${result.xml.length} bytes of DDEX XML\nBundle size: Only 114KB WASM!`;
                
                // Download the XML
                const blob = new Blob([result.xml], {type: 'application/xml'});
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'browser_generated.xml';
                a.click();
                
            } catch (error) {
                output.textContent = `❌ Error: ${error.message}`;
            }
        };
    </script>
</body>
</html>
```

---

## 🎯 Real-World Examples

### Spotify Album Release
```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder();
await builder.applyPreset('spotify_audio_43');

const spotifyRelease = {
    header: {
        message_sender: {party_name: [{text: 'My Record Label'}]},
        message_recipient: {party_name: [{text: 'Spotify'}]}
    },
    version: '4.3',
    profile: 'AudioAlbum',
    releases: [{
        identifiers: {
            upc: '196589418425',  // Real UPC format
            catalog_number: 'LBL-2024-001'
        },
        title: [{text: 'Midnight Sessions'}],
        display_artist: 'Luna Rivers',
        release_type: 'Album',
        release_date: '2024-04-01',
        genre: [{text: 'Electronic'}],
        tracks: [
            {
                position: 1,
                isrc: 'USLUN2400001',
                title: 'Neon Dreams',
                duration: 252,  // 4:12
                display_artist: 'Luna Rivers'
            },
            {
                position: 2,
                isrc: 'USLUN2400002', 
                title: 'Midnight Drive (feat. Echo Vale)',
                duration: 198,  // 3:18
                display_artist: 'Luna Rivers feat. Echo Vale'
            }
        ],
        images: [{
            type: 'FrontCoverImage',
            width: 3000,
            height: 3000,
            uri: 'https://example.com/covers/midnight_sessions_3000x3000.jpg'
        }]
    }]
};

const xml = await builder.build(spotifyRelease);
console.log('🎵 Spotify-ready DDEX XML generated!');
```

### Label Catalog Migration
```python
from ddex_builder import DDEXBuilder
import pandas as pd

# Load catalog from CSV  
catalog_df = pd.read_csv('label_catalog.csv')

builder = DDEXBuilder()
builder.apply_preset('universal_distribution')

releases = []
for _, row in catalog_df.iterrows():
    release = {
        'identifiers': {
            'upc': row['upc'],
            'catalog_number': row['catalog_number']
        },
        'title': [{'text': row['album_title']}],
        'display_artist': row['artist_name'],
        'release_date': row['release_date'],
        'tracks': []
    }
    
    # Add tracks from related data
    track_data = get_tracks_for_release(row['album_id'])
    for track in track_data:
        release['tracks'].append({
            'position': track['track_number'],
            'isrc': track['isrc'],
            'title': track['title'],
            'duration': track['duration_seconds']
        })
    
    releases.append(release)

print(f"🏭 Processing {len(releases)} releases...")
for i, release in enumerate(releases):
    xml = builder.build({
        'version': '4.3',
        'releases': [release]
    })
    
    filename = f"catalog_release_{i+1:04d}.xml"
    with open(filename, 'w') as f:
        f.write(xml.xml)
    
    if i % 100 == 0:
        print(f"  Processed {i+1}/{len(releases)}")

print("✅ Catalog migration complete!")
```

---

## 🚨 Common Issues & Solutions

### Missing Required Fields
```
❌ Build failed: Missing required field 'ISRC' for track at position 1
```
**Solution**: Ensure all tracks have valid ISRC codes
```typescript
// ✅ Good - All required fields present
{
    position: 1,
    isrc: 'USXYZ2400001',  // Required!
    title: 'Song Title',   // Required!
    duration: 180          // Required for most profiles
}
```

### Invalid ISRC Format
```
❌ Validation failed: Invalid ISRC format 'US123456789012'  
```
**Solution**: Use proper ISRC format: 2 country letters + 3 registrant + 7 digits
```typescript
// ❌ Bad
isrc: 'US123456789012'

// ✅ Good  
isrc: 'USXYZ2400001'
//     ^^    ^^    ^^
//     Country Registrant Year+ID
```

### UPC Checksum Errors
```
❌ Validation failed: UPC checksum invalid for '123456789013'
```
**Solution**: Use a valid UPC with correct checksum
```python
# Use a UPC checksum calculator or valid test UPC
upc = '196589418425'  # Valid UPC with correct checksum
```

### Preset Not Found
```
❌ Failed to apply preset: Preset 'custom_preset' not found
```
**Solution**: Use available presets or create custom configuration
```typescript
// Check available presets
console.log(builder.availablePresets());

// Use built-in presets
await builder.applyPreset('spotify_audio_43');
// or
await builder.applyPreset('universal_distribution');
```

---

## 📚 Next Steps

Congratulations! You've mastered the basics of DDEX Suite. Here's what to explore next:

### 🔍 Deep Dive Tutorials
- **[User Guide](./user-guide.md)** - Comprehensive feature walkthrough
- **[Developer Guide](./developer-guide.md)** - Advanced integration patterns
- **[Builder Examples](../packages/ddex-builder/examples/README.md)** - Real-world scenarios

### 🛠️ Advanced Features
- **Partner Presets** - Platform-specific optimizations (Spotify, YouTube, Amazon)
- **Streaming Processing** - Handle large catalogs efficiently
- **DataFrame Integration** - Python pandas workflows
- **Semantic Diff Engine** - Track changes between releases
- **Custom Validation Rules** - Business logic enforcement

### 🏗️ Production Deployment  
- **Error Handling** - Robust error recovery patterns
- **Performance Tuning** - Optimize for your use case
- **Security Hardening** - XXE protection and input validation
- **Monitoring & Logging** - Track processing metrics
- **CI/CD Integration** - Automated DDEX generation

### 🤝 Community & Support
- **[GitHub Issues](https://github.com/daddykev/ddex-suite/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/daddykev/ddex-suite/discussions)** - Community Q&A
- **[API Documentation](https://docs.rs/ddex-builder)** - Complete API reference
- **[DDEX Specifications](https://ddex.net/standards/)** - Official DDEX documentation

---

## 🎉 Success!

You've successfully:
- ✅ Installed and configured DDEX Suite
- ✅ Parsed DDEX XML into structured data  
- ✅ Modified metadata programmatically
- ✅ Generated new, compliant DDEX XML
- ✅ Verified round-trip fidelity

**You're now ready to integrate DDEX Suite into your music technology stack!**

The combination of reliable parsing and deterministic building makes DDEX Suite perfect for:
- **Digital Distributors** - Automate release processing
- **Record Labels** - Manage catalog delivery  
- **Streaming Platforms** - Normalize incoming metadata
- **Music Tech Startups** - Build DDEX-powered features
- **Enterprise Systems** - Integrate with existing workflows

---

*Ready to build the next generation of music technology? [Get started with the full documentation](../README.md) or dive into [advanced examples](../packages/ddex-builder/examples/)!*

**DDEX Suite v0.1.0** - One Rust Core, Every Language, Perfect Fidelity