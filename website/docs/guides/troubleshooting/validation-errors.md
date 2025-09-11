# Validation Errors

Common DDEX validation errors and how to resolve them.

## Common Validation Errors

### Missing Required Fields

**Error**: `Required field 'title' is missing`

**Solution**: Ensure all required fields are present
```typescript
const releaseData = {
  id: "R123",
  title: "Album Title",     // Required
  artist: "Artist Name",    // Required
  release_date: "2023-01-01" // Required for most partners
};
```

### Invalid ISRC Format

**Error**: `Invalid ISRC format: ABC123456789`

**Solution**: Use proper ISRC format (2 country letters + 3 registrant code + 7 digits)
```python
# Correct ISRC format
track_data = {
    "isrc": "USRC17607839"  # US-RC1-76-07839
}
```

### Territory Code Issues

**Error**: `Invalid territory code: USA`

**Solution**: Use ISO 3166-1 alpha-2 codes
```typescript
const territoryMapping = {
  "USA": "US",
  "United Kingdom": "GB", 
  "Germany": "DE"
};
```

### Date Format Errors

**Error**: `Invalid date format: 01/01/2023`

**Solution**: Use ISO 8601 date format
```python
# Correct date formats
release_data = {
    "release_date": "2023-01-01",           # Date only
    "created_at": "2023-01-01T10:30:00Z"    # Full datetime
}
```

## Validation Error Categories

### Schema Errors
- Missing required elements
- Invalid element structure
- Namespace issues
- Version incompatibilities

### Data Type Errors
- String instead of number
- Invalid date formats
- Boolean format issues
- Array vs single value

### Business Rule Violations
- Invalid duration ranges
- Missing ISRCs for commercial releases
- Territory restrictions
- Content rating inconsistencies

## Error Resolution Strategies

### 1. Use Validation Before Building
```typescript
const builder = new DDEXBuilder();

// Validate first
const validation = await builder.validate(data);
if (!validation.isValid) {
  console.log("Validation errors:", validation.errors);
  // Fix errors before building
}

const xml = await builder.build(data);
```

### 2. Implement Data Sanitization
```python
def sanitize_release_data(data):
    """Clean and validate release data"""
    
    # Ensure required fields
    if not data.get('title'):
        raise ValueError("Release title is required")
    
    # Clean ISRC format
    if 'tracks' in data:
        for track in data['tracks']:
            if 'isrc' in track and track['isrc']:
                track['isrc'] = clean_isrc(track['isrc'])
    
    # Normalize territory codes
    if 'territory_codes' in data:
        data['territory_codes'] = [
            normalize_territory_code(code) 
            for code in data['territory_codes']
        ]
    
    return data
```

### 3. Validation Patterns
```typescript
// Common validation patterns
const validators = {
  isrc: /^[A-Z]{2}[A-Z0-9]{3}[0-9]{7}$/,
  upc: /^[0-9]{12}$/,
  territory: /^[A-Z]{2}$/,
  duration: (ms: number) => ms >= 1000 && ms <= 3600000
};
```

## Partner-Specific Validation

### Spotify Requirements
```python
def validate_for_spotify(release_data):
    """Validate data for Spotify submission"""
    
    errors = []
    
    # Spotify requires UPC for releases
    if not release_data.get('upc'):
        errors.append("Spotify requires UPC for all releases")
    
    # Track duration requirements
    for track in release_data.get('tracks', []):
        duration_ms = track.get('duration_ms', 0)
        
        if duration_ms < 30000:  # 30 seconds minimum
            errors.append(f"Track '{track['title']}' too short for Spotify")
        
        if not track.get('isrc'):
            errors.append(f"Track '{track['title']}' missing ISRC")
    
    return errors
```

## Best Practices

1. **Validate Early**: Check data before processing
2. **Use Partner Presets**: Leverage built-in partner validation
3. **Sanitize Input**: Clean data before validation
4. **Provide Context**: Include meaningful error messages
5. **Log Validation Issues**: Track common validation problems
6. **Document Requirements**: Maintain validation requirement docs