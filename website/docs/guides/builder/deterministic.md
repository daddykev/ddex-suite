# Deterministic Output

The DDEX Builder ensures consistent, reproducible XML output through deterministic ordering and canonicalization.

## Overview

Deterministic output means that given the same input data, the builder will always produce identical XML byte-for-byte. This is crucial for:

- Version control and diffing
- Digital signatures and checksums
- Reproducible builds
- Testing and validation

## Key Features

### Stable Element Ordering

The builder uses `IndexMap` throughout to maintain consistent ordering:

```rust
// Elements are ordered deterministically
let mut releases = IndexMap::new();
releases.insert("release_1", release_data);
```

### Content-Based IDs

All generated IDs are based on content hashes rather than random values:

```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder({
  deterministicIds: true, // Default: true
  idSeed: "stable-seed"   // Optional: for consistent IDs across runs
});

const result = builder.build(data);
// Same input = same output every time
```

### DB-C14N/1.0 Canonicalization

The builder implements Database Canonical XML 1.0 for standardized output:

- Consistent attribute ordering
- Normalized whitespace
- Standard namespace declarations
- UTF-8 encoding

## Configuration

```python
from ddex_builder import DDEXBuilder

builder = DDEXBuilder(
    deterministic=True,      # Enable deterministic mode
    canonical=True,          # Use DB-C14N/1.0
    preserve_whitespace=False # Normalize whitespace
)

xml = builder.build(data)
```

## Testing Determinism

Verify deterministic output in your tests:

```javascript
const builder = new DDEXBuilder();
const data = { /* your data */ };

const output1 = builder.build(data);
const output2 = builder.build(data);

assert(output1 === output2); // Should always pass
```

## Performance Impact

Deterministic output has minimal performance overhead:

- ~2% slower than non-deterministic mode
- Memory usage remains constant
- Scales linearly with data size

## Best Practices

1. **Always use deterministic mode** in production
2. **Set a stable ID seed** for consistent cross-environment builds
3. **Test determinism** in your CI/CD pipeline
4. **Version your data schemas** to maintain compatibility

## Troubleshooting

Common issues with deterministic output:

### Inconsistent Results

If you're getting different outputs:
- Check that `deterministicIds: true` is set
- Verify input data is identical
- Ensure no external randomness (timestamps, UUIDs)

### Performance Issues

If deterministic mode is slow:
- Profile your data transformation
- Consider batching large datasets
- Use streaming for very large files