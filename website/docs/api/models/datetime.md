# DateTime Models

DateTime models provide standardized date, time, and duration representations used throughout DDEX messages for release dates, licensing periods, and temporal metadata.

## Core Types

### EventDate

Represents dates with varying precision levels:

```typescript
interface EventDate {
  date: string;           // ISO 8601 format
  precision?: DatePrecision;
  isApproximate?: boolean;
  territoryCode?: string;  // Territory-specific dates
}

enum DatePrecision {
  Year = 'Year',
  Month = 'Month', 
  Day = 'Day',
  Hour = 'Hour',
  Minute = 'Minute',
  Second = 'Second'
}
```

### Duration

Represents time durations in ISO 8601 format:

```typescript
interface Duration {
  value: string;  // e.g., "PT3M45S" for 3 minutes 45 seconds
}
```

### Period

Represents time periods with start and end boundaries:

```typescript
interface Period {
  startDate?: EventDate | string;
  endDate?: EventDate | string;
  duration?: Duration;
  isOpenEnded?: boolean;
}
```

## Date Formats

### ISO 8601 Format

All dates follow ISO 8601 standard:

```typescript
// Full precision
"2024-03-15T14:30:00Z"        // UTC with seconds
"2024-03-15T14:30:00-05:00"   // With timezone offset

// Reduced precision  
"2024-03-15"                  // Date only
"2024-03"                     // Month only
"2024"                        // Year only
```

### Duration Format

Durations use ISO 8601 period notation:

```typescript
"PT3M45S"      // 3 minutes 45 seconds
"PT1H30M"      // 1 hour 30 minutes  
"P1Y2M3DT4H5M6S"  // 1 year, 2 months, 3 days, 4 hours, 5 minutes, 6 seconds
```

## Common Use Cases

### Release Dates

```typescript
interface Release {
  releaseDate: EventDate;
  originalReleaseDate?: EventDate;
  pLineDate?: EventDate;
  cLineDate?: EventDate;
}

// Example release date
const releaseDate: EventDate = {
  date: "2024-03-15",
  precision: "Day",
  isApproximate: false
};
```

### Licensing Periods

```typescript
interface Deal {
  validityPeriod?: Period;
  commercialModelType: CommercialModelType[];
}

// Example licensing period
const licensePeriod: Period = {
  startDate: "2024-01-01T00:00:00Z",
  endDate: "2025-12-31T23:59:59Z",
  isOpenEnded: false
};
```

### Content Duration

```typescript
interface SoundRecording {
  duration?: Duration;
  technicalDetails?: {
    duration?: Duration;
    previewStartTime?: Duration;
    previewDuration?: Duration;
  };
}

// Example audio duration
const trackDuration: Duration = {
  value: "PT3M45S"  // 3:45
};
```

## Usage Examples

### Parsing Dates

```typescript
const result = await parser.parse(xmlContent);

// Access release dates
const release = result.flat.releases[0];
console.log('Release date:', release.releaseDate.date);

// Check date precision
if (release.releaseDate.precision === 'Year') {
  console.log('Year-only release date');
}

// Territory-specific dates
const usReleaseDate = release.releaseInformation
  .find(info => info.territories?.some(t => t.code === 'US'))
  ?.salesStartDate;
```

### Building Date Data

```typescript
const buildRequest = {
  releases: [{
    releaseId: 'R123456789',
    releaseDate: {
      date: '2024-03-15',
      precision: 'Day',
      isApproximate: false
    },
    originalReleaseDate: {
      date: '2023',
      precision: 'Year',
      isApproximate: true
    },
    releaseInformation: [{
      territories: [{ code: 'US' }],
      salesStartDate: '2024-03-15T00:00:00-05:00'
    }]
  }]
};
```

### Working with Durations

```typescript
// Parse duration string
function parseDuration(isoDuration: string): number {
  // Convert "PT3M45S" to total seconds
  const match = isoDuration.match(/PT(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?/);
  if (!match) return 0;
  
  const hours = parseInt(match[1] || '0', 10);
  const minutes = parseInt(match[2] || '0', 10);  
  const seconds = parseInt(match[3] || '0', 10);
  
  return hours * 3600 + minutes * 60 + seconds;
}

// Format duration from seconds
function formatDuration(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  
  let duration = 'PT';
  if (hours > 0) duration += `${hours}H`;
  if (minutes > 0) duration += `${minutes}M`;
  if (seconds > 0) duration += `${seconds}S`;
  
  return duration;
}

// Usage
const track = result.flat.resources.soundRecordings[0];
const durationSeconds = parseDuration(track.duration?.value || 'PT0S');
console.log(`Track length: ${Math.floor(durationSeconds / 60)}:${(durationSeconds % 60).toString().padStart(2, '0')}`);
```

### Period Calculations

```typescript
function isPeriodActive(period: Period, date: Date = new Date()): boolean {
  const now = date.toISOString();
  
  // Check start date
  if (period.startDate && now < period.startDate) {
    return false;
  }
  
  // Check end date
  if (period.endDate && !period.isOpenEnded && now > period.endDate) {
    return false;
  }
  
  return true;
}

// Usage with deals
const activeDeal = result.flat.deals.find(deal => 
  !deal.validityPeriod || isPeriodActive(deal.validityPeriod)
);
```

## Timezone Handling

### UTC vs Local Time

```typescript
// Prefer UTC for international distribution
const utcDate = "2024-03-15T00:00:00Z";

// Local time with offset
const localDate = "2024-03-15T00:00:00-05:00";  // EST

// Date only (timezone-agnostic)
const dateOnly = "2024-03-15";
```

### Territory-Specific Dates

Different territories may have different release dates:

```typescript
const releaseInfo = [{
  territories: [{ code: 'US' }],
  salesStartDate: '2024-03-15T00:00:00-05:00'  // EST
}, {
  territories: [{ code: 'JP' }],
  salesStartDate: '2024-03-16T00:00:00+09:00'  // JST (next day)
}];
```

## See Also

- [Deal Models](./deal) - Period usage in licensing
- [Territory Models](./territories) - Territory-specific dates  
- [Release Models](./release) - Release date handling