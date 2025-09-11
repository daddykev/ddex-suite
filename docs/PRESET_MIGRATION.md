# DDEX Builder Preset Migration Guide

## Overview

As of DDEX Builder v0.3.0, we have refactored the Partner Presets system to be more realistic and honest about what's actually available. This guide helps you migrate from the old preset system to the new approach.

## What Changed

### ❌ Removed (No Longer Available)
- `spotify_audio_43` / `spotify_album` / `spotify_single` / `spotify_ep`
- `apple_music_43`
- All other platform-specific presets without public DDEX specifications

### ✅ New Available Presets

#### Generic Industry-Standard Presets (Community-maintained)
- `audio_album` - DDEX ERN 4.3 compliant audio album configuration
- `audio_single` - DDEX ERN 4.3 compliant single track configuration  
- `video_single` - DDEX ERN 4.3 compliant video release configuration
- `compilation` - DDEX ERN 4.3 compliant compilation album configuration

#### Platform Presets (Based on Public Documentation)
- `youtube_album` - YouTube Music album requirements
- `youtube_video` - YouTube Music video requirements
- `youtube_single` - YouTube Music single requirements

## Migration Path

### For Spotify Users
**Old:**
```typescript
// ❌ No longer available
builder.applyPreset('spotify_album');
```

**New:**
```typescript
// ✅ Use generic baseline + custom configuration
builder.applyPreset('audio_album');

// Add your own Spotify-specific requirements
builder.addValidation('ExplicitContent', 'Required');
builder.addValidation('AudioQuality', { minBitDepth: 16, minSampleRate: 44100 });
builder.setDefault('DistributionChannel', '01');
```

### For Apple Music Users
**Old:**
```typescript
// ❌ No longer available
builder.applyPreset('apple_music_43');
```

**New:**
```typescript
// ✅ Use generic baseline + custom configuration
builder.applyPreset('audio_album');

// Add your own Apple-specific requirements
builder.addValidation('UPC', 'Required');
builder.addValidation('ISRC', 'Required');
builder.setDefault('ReleaseType', 'Album');
```

### For YouTube Users
**Old:**
```typescript
// ❌ Old naming
builder.applyPreset('youtube_music_43');
```

**New:**
```typescript
// ✅ New naming, based on public documentation
builder.applyPreset('youtube_album');
// or
builder.applyPreset('youtube_video');
```

### CLI Migration

**Old CLI Usage:**
```bash
# ❌ No longer available
ddex-builder build --preset spotify input.json output.xml
ddex-builder build --preset apple input.json output.xml
```

**New CLI Usage:**
```bash
# ✅ Generic presets
ddex-builder build --preset audio-album input.json output.xml
ddex-builder build --preset audio-single input.json output.xml

# ✅ YouTube presets (public documentation)
ddex-builder build --preset youtube-album input.json output.xml
ddx-builder build --preset youtube-video input.json output.xml
```

## Why This Change?

### 🎯 Honesty and Transparency
- **Before**: We provided "Spotify presets" without access to Spotify's actual DDEX requirements
- **After**: We only provide presets for platforms with publicly available specifications

### 🏗️ Better Foundation
- **Generic presets** provide solid DDEX-compliant baselines
- **YouTube presets** are based on actual public Partner documentation
- **Custom framework** allows you to build organization-specific configurations

### ⚖️ Legal and Ethical
- No risk of misrepresenting confidential platform specifications
- Clear provenance and source attribution for all configurations
- Community-maintained approach with proper disclaimers

## Creating Custom Presets

Since most platforms don't publish their DDEX requirements, you'll need to create custom presets for internal use:

### Step 1: Start with Generic Baseline
```typescript
import { generic } from 'ddex-builder/presets';

// Start with a solid foundation
let customPreset = generic.audio_album();
customPreset.name = 'my_platform_preset';
customPreset.description = 'Internal Platform X Requirements';
```

### Step 2: Add Platform-Specific Rules
```typescript
// Add custom validation rules
customPreset.validation_rules.insert(
  'CustomField', 
  ValidationRule::Required
);

// Add custom mappings
customPreset.custom_mappings.insert(
  'PlatformID',
  'CustomIdentifier'
);

// Set platform-specific defaults
customPreset.config.default_values.insert(
  'MessageType',
  'PlatformSpecific'
);
```

### Step 3: Add Territory and Channel Restrictions
```typescript
// Restrict to specific territories
customPreset.config.territory_codes = ['US', 'CA', 'GB'];

// Set distribution channels
customPreset.config.distribution_channels = ['02']; // Streaming only
```

## Best Practices for Custom Presets

### 1. Start with Generic Baseline
Always begin with `audio_album`, `audio_single`, or `video_single` to ensure DDEX compliance.

### 2. Document Your Sources
```typescript
customPreset.source = PresetSource::CustomerFeedback;
customPreset.disclaimer = 'Based on platform integration testing and support feedback. Not an official specification.';
customPreset.provenance_url = 'https://your-internal-wiki.com/platform-x-requirements';
```

### 3. Version Your Presets
```typescript
customPreset.version = '2.1.0';
customPreset.locked = false; // Allow updates
```

### 4. Test Thoroughly
```typescript
// Always test your custom presets
const result = builder.build(testData);
assert(result.isValid());
```

## Preset Template Generator

Use our template generator to create custom presets:

```bash
ddex-builder preset create \
  --name "my_platform" \
  --based-on "audio_album" \
  --output "./presets/my_platform.rs"
```

This generates a well-structured preset template with:
- Proper validation rules
- Default value handling  
- Territory and channel configuration
- Documentation placeholders

## Support and Community

### Getting Help
- **GitHub Issues**: Report bugs or ask questions
- **Documentation**: Full preset architecture guide
- **Examples**: Real-world custom preset examples

### Contributing
- Share generic improvements via pull requests
- Document new platform requirements (when public)
- Help maintain YouTube preset accuracy

## Frequently Asked Questions

### Q: Why can't you just reverse-engineer platform requirements?
**A:** This would be legally and ethically problematic. Platform requirements are often confidential, change frequently, and vary by distributor relationship.

### Q: Will you add presets for Platform X when they publish requirements?
**A:** Yes! If any platform publishes official DDEX specifications, we'll add community-maintained presets based on that public documentation.

### Q: Can I share my custom presets?
**A:** Yes, but be careful about confidential information. Generic improvements are always welcome, but platform-specific details should remain internal to your organization.

### Q: What if I was relying on the old Spotify preset?
**A:** The old preset was based on assumptions, not official requirements. Using the `audio_album` generic preset plus your own tested configuration will be more reliable.

## Migration Checklist

- [ ] Identify which old presets you were using
- [ ] Replace with appropriate generic preset (`audio_album`, `audio_single`, etc.)
- [ ] Add your own platform-specific validation rules
- [ ] Update CLI scripts to use new preset names
- [ ] Test your builds with real platform submissions
- [ ] Document your custom configuration for your team
- [ ] Consider contributing generic improvements back to the community

## Timeline

- **v0.2.x**: Old presets deprecated but still available with warnings
- **v0.3.0**: Old presets removed, migration guide provided
- **v0.3.x**: Enhanced custom preset framework
- **v1.0.0**: Stable API with mature custom preset tooling

---

**Need help with migration?** Open an issue at [ddex-suite/issues](https://github.com/daddykev/ddex-suite/issues) with the `preset-migration` label.