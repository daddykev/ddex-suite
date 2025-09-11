# Territory Models

Territory models define geographic regions and markets for rights management, distribution, and licensing in DDEX messages.

## Core Types

### Territory

Represents a geographic region or market:

```typescript
interface Territory {
  code: string;
  excludedTerritories?: Territory[];
  applicableTerritoryCode?: string;
}
```

### Territory Codes

DDEX supports multiple territory coding systems:

- **ISO 3166-1 Alpha-2** - Two-letter country codes (US, GB, JP)
- **ISO 3166-1 Alpha-3** - Three-letter country codes (USA, GBR, JPN)  
- **CISAC TIS** - Specialized music industry territory codes
- **Custom codes** - Worldwide, Europe, etc.

## Common Territory Definitions

### Standard Territories

```typescript
// Individual countries
{ code: 'US' }     // United States
{ code: 'GB' }     // United Kingdom  
{ code: 'DE' }     // Germany
{ code: 'FR' }     // France
{ code: 'JP' }     // Japan

// Regional groupings
{ code: 'Worldwide' }
{ code: 'Europe' }
{ code: 'NorthAmerica' }
{ code: 'Asia' }
```

### Territory Exclusions

Territories can exclude specific regions:

```typescript
// Worldwide except certain countries
{
  code: 'Worldwide',
  excludedTerritories: [
    { code: 'US' },
    { code: 'CN' }
  ]
}

// European Union excluding UK
{
  code: 'EuropeanUnion',
  excludedTerritories: [
    { code: 'GB' }
  ]
}
```

## Territory Usage Contexts

### Deal Territories

Defines where licensing agreements apply:

```typescript
interface Deal {
  territory: Territory[];
  dealScope: {
    excludedTerritories?: Territory[];
  };
}
```

### Release Territories

Specifies where releases are available:

```typescript
interface Release {
  territories: Territory[];
  releaseInformation: {
    salesStartDate?: string;
    territories?: Territory[];
  }[];
}
```

### Rights Territories  

Defines geographic scope of usage rights:

```typescript
interface UsageRights {
  territory?: Territory[];
  useType: UseType[];
  period?: Period;
}
```

## Usage Examples

### Accessing Territory Data

```typescript
const result = await parser.parse(xmlContent);

// Get all territories from deals
const dealTerritories = result.flat.deals
  .flatMap(deal => deal.territory)
  .map(t => t.code);

console.log('Deal territories:', [...new Set(dealTerritories)]);

// Check if territory is included
const hasUSRights = result.flat.deals.some(deal =>
  deal.territory.some(t => 
    t.code === 'US' || t.code === 'Worldwide'
  ) && !deal.territory.some(t =>
    t.excludedTerritories?.some(excluded => excluded.code === 'US')
  )
);
```

### Building Territory Data

```typescript
// Simple territory list
const territories = [
  { code: 'US' },
  { code: 'CA' },
  { code: 'MX' }
];

// Global with exclusions
const globalTerritory = {
  code: 'Worldwide',
  excludedTerritories: [
    { code: 'CN' },
    { code: 'RU' },
    { code: 'KP' }
  ]
};

// In a deal context
const buildRequest = {
  deals: [{
    dealId: 'DEAL001',
    territory: [globalTerritory],
    usageRights: [{
      useType: ['Stream'],
      territory: [{ code: 'US' }]  // More restrictive than deal level
    }]
  }]
};
```

### Territory Validation

Check territory coverage and conflicts:

```typescript
function isTerritoryCovered(territory: string, coverageTerritories: Territory[]): boolean {
  return coverageTerritories.some(t => {
    // Check direct match
    if (t.code === territory || t.code === 'Worldwide') {
      // Check exclusions
      return !t.excludedTerritories?.some(excluded => excluded.code === territory);
    }
    return false;
  });
}

// Usage
const isUSCovered = isTerritoryCovered('US', deal.territory);
```

### Regional Groupings

Common regional territory definitions:

```typescript
const territoryGroups = {
  northAmerica: ['US', 'CA', 'MX'],
  europe: ['GB', 'DE', 'FR', 'IT', 'ES', 'NL', 'BE', 'AT', 'CH'],
  asia: ['JP', 'KR', 'CN', 'IN', 'SG', 'HK', 'TH', 'MY'],
  oceania: ['AU', 'NZ'],
  latinAmerica: ['BR', 'AR', 'CL', 'CO', 'PE']
};

// Build territories for a region
const europeTerritory = {
  code: 'Europe',
  applicableTerritoryCode: territoryGroups.europe.join(',')
};
```

## Territory Hierarchy

Territories can have hierarchical relationships:

```typescript
// Country is part of larger region
const nestedTerritories = {
  worldwide: {
    code: 'Worldwide',
    excludedTerritories: []
  },
  northAmerica: {
    code: 'NorthAmerica',
    includes: ['US', 'CA', 'MX']
  },
  unitedStates: {
    code: 'US',
    parentTerritory: 'NorthAmerica'
  }
};
```

## See Also

- [Deal Models](./deal) - Territory usage in licensing
- [DateTime Models](./datetime) - Territory-specific dates and periods
- [Technical Models](./technical) - Territory-specific technical requirements