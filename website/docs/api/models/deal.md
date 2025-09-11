# Deal Models

Deal models represent licensing agreements, usage rights, and commercial terms for DDEX resources and releases.

## Core Types

### Deal

Represents a commercial agreement:

```typescript
interface Deal {
  dealId: string;
  dealType: DealType;
  dealScope: DealScope;
  usageRights: UsageRights[];
  territory: Territory[];
  validityPeriod?: Period;
  commercialModelType?: CommercialModelType[];
}
```

### DealType

Specifies the nature of the commercial agreement:

```typescript
enum DealType {
  License = 'License',
  Assignment = 'Assignment',
  ServiceProvider = 'ServiceProvider'
}
```

### UsageRights

Defines permitted uses of the content:

```typescript
interface UsageRights {
  useType: UseType[];
  territory?: Territory[];
  period?: Period;
  conditions?: Condition[];
}
```

## Deal Scope

### DealScope

Defines what content is covered by the deal:

```typescript
interface DealScope {
  referenceType: 'Release' | 'Resource';
  referenceId: string;
  excludedTerritories?: Territory[];
  excludedUsageRights?: UsageRights[];
}
```

## Commercial Models

### CommercialModelType

Supported business models:

- **Subscription** - Monthly/annual subscription access
- **PayAsYouGo** - Per-stream/download payment  
- **FreeOfCharge** - Free access (ad-supported)
- **PurchaseAsPhysicalProduct** - Physical sales
- **PurchaseAsDigitalProduct** - Digital sales

## Usage Examples

### Accessing Deal Information

```typescript
const result = await parser.parse(xmlContent);

// Access all deals
const deals = result.flat.deals;
deals.forEach(deal => {
  console.log(`Deal ${deal.dealId}: ${deal.dealType}`);
  console.log(`Territories: ${deal.territory.map(t => t.code).join(', ')}`);
  console.log(`Rights: ${deal.usageRights.map(r => r.useType).join(', ')}`);
});

// Filter by territory
const usDeals = deals.filter(deal => 
  deal.territory.some(t => t.code === 'US')
);
```

### Building Deal Data

```typescript
const buildRequest = {
  deals: [{
    dealId: 'DEAL001',
    dealType: 'License',
    dealScope: {
      referenceType: 'Release',
      referenceId: 'R123456789'
    },
    usageRights: [{
      useType: ['Stream', 'PermanentDownload'],
      territory: [{ code: 'Worldwide' }],
      period: {
        startDate: '2024-01-01',
        endDate: '2025-12-31'
      }
    }],
    commercialModelType: ['Subscription', 'PayAsYouGo']
  }]
};
```

## Territory Handling

Deals can specify territories at multiple levels:
- Deal level (applies to entire deal)
- Usage rights level (specific to certain uses)
- Exclusions (territories where deal doesn't apply)

```typescript
// Global deal with regional exclusions
const globalDeal = {
  territory: [{ code: 'Worldwide' }],
  dealScope: {
    excludedTerritories: [{ code: 'CN' }, { code: 'KP' }]
  }
};
```

## See Also

- [Territory Models](./territories) - Geographic and market definitions
- [Party Models](./party) - Rights holders and licensees  
- [DateTime Models](./datetime) - Period and date specifications