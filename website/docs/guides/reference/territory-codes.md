# Territory Codes

ISO 3166-1 alpha-2 territory codes used in DDEX Suite.

## Common Territory Codes

| Code | Country/Territory | Common Usage |
|------|-------------------|--------------|
| **AD** | Andorra | Europe |
| **AE** | United Arab Emirates | Middle East |
| **AR** | Argentina | Latin America |
| **AT** | Austria | Europe |
| **AU** | Australia | Asia Pacific |
| **BE** | Belgium | Europe |
| **BR** | Brazil | Latin America |
| **CA** | Canada | North America |
| **CH** | Switzerland | Europe |
| **CN** | China | Asia Pacific |
| **DE** | Germany | Europe |
| **DK** | Denmark | Europe |
| **ES** | Spain | Europe |
| **FI** | Finland | Europe |
| **FR** | France | Europe |
| **GB** | United Kingdom | Europe |
| **IE** | Ireland | Europe |
| **IN** | India | Asia Pacific |
| **IT** | Italy | Europe |
| **JP** | Japan | Asia Pacific |
| **KR** | South Korea | Asia Pacific |
| **MX** | Mexico | Latin America |
| **NL** | Netherlands | Europe |
| **NO** | Norway | Europe |
| **NZ** | New Zealand | Asia Pacific |
| **PL** | Poland | Europe |
| **PT** | Portugal | Europe |
| **RU** | Russia | Europe |
| **SE** | Sweden | Europe |
| **US** | United States | North America |
| **WW** | Worldwide | Global |

## Regional Groupings

### North America
- **US** - United States
- **CA** - Canada
- **MX** - Mexico

### Europe
- **GB** - United Kingdom
- **DE** - Germany
- **FR** - France
- **ES** - Spain
- **IT** - Italy
- **NL** - Netherlands
- **SE** - Sweden
- **NO** - Norway
- **DK** - Denmark
- **FI** - Finland

### Asia Pacific
- **JP** - Japan
- **AU** - Australia
- **NZ** - New Zealand
- **KR** - South Korea
- **CN** - China
- **IN** - India

## Usage in DDEX

```typescript
// Specify territories for a release
const releaseData = {
  id: "R12345",
  title: "Global Release",
  artist: "Artist Name",
  territory_codes: ["US", "GB", "DE", "FR", "JP", "AU"]
};

// Worldwide release
const worldwideRelease = {
  id: "R67890",
  title: "Worldwide Release",
  artist: "Global Artist",
  territory_codes: ["WW"]
};
```

## Validation

```python
# Validate territory codes
VALID_TERRITORIES = {
    'AD', 'AE', 'AF', 'AG', 'AI', 'AL', 'AM', 'AO', 'AQ', 'AR', 'AS', 'AT',
    'AU', 'AW', 'AX', 'AZ', 'BA', 'BB', 'BD', 'BE', 'BF', 'BG', 'BH', 'BI',
    'BJ', 'BL', 'BM', 'BN', 'BO', 'BQ', 'BR', 'BS', 'BT', 'BV', 'BW', 'BY',
    'BZ', 'CA', 'CC', 'CD', 'CF', 'CG', 'CH', 'CI', 'CK', 'CL', 'CM', 'CN',
    'CO', 'CR', 'CU', 'CV', 'CW', 'CX', 'CY', 'CZ', 'DE', 'DJ', 'DK', 'DM',
    'DO', 'DZ', 'EC', 'EE', 'EG', 'EH', 'ER', 'ES', 'ET', 'FI', 'FJ', 'FK',
    'FM', 'FO', 'FR', 'GA', 'GB', 'GD', 'GE', 'GF', 'GG', 'GH', 'GI', 'GL',
    'GM', 'GN', 'GP', 'GQ', 'GR', 'GS', 'GT', 'GU', 'GW', 'GY', 'HK', 'HM',
    'HN', 'HR', 'HT', 'HU', 'ID', 'IE', 'IL', 'IM', 'IN', 'IO', 'IQ', 'IR',
    'IS', 'IT', 'JE', 'JM', 'JO', 'JP', 'KE', 'KG', 'KH', 'KI', 'KM', 'KN',
    'KP', 'KR', 'KW', 'KY', 'KZ', 'LA', 'LB', 'LC', 'LI', 'LK', 'LR', 'LS',
    'LT', 'LU', 'LV', 'LY', 'MA', 'MC', 'MD', 'ME', 'MF', 'MG', 'MH', 'MK',
    'ML', 'MM', 'MN', 'MO', 'MP', 'MQ', 'MR', 'MS', 'MT', 'MU', 'MV', 'MW',
    'MX', 'MY', 'MZ', 'NA', 'NC', 'NE', 'NF', 'NG', 'NI', 'NL', 'NO', 'NP',
    'NR', 'NU', 'NZ', 'OM', 'PA', 'PE', 'PF', 'PG', 'PH', 'PK', 'PL', 'PM',
    'PN', 'PR', 'PS', 'PT', 'PW', 'PY', 'QA', 'RE', 'RO', 'RS', 'RU', 'RW',
    'SA', 'SB', 'SC', 'SD', 'SE', 'SG', 'SH', 'SI', 'SJ', 'SK', 'SL', 'SM',
    'SN', 'SO', 'SR', 'SS', 'ST', 'SV', 'SX', 'SY', 'SZ', 'TC', 'TD', 'TF',
    'TG', 'TH', 'TJ', 'TK', 'TL', 'TM', 'TN', 'TO', 'TR', 'TT', 'TV', 'TW',
    'TZ', 'UA', 'UG', 'UM', 'US', 'UY', 'UZ', 'VA', 'VC', 'VE', 'VG', 'VI',
    'VN', 'VU', 'WF', 'WS', 'YE', 'YT', 'ZA', 'ZM', 'ZW', 'WW'
}

def validate_territory_codes(codes):
    """Validate territory codes"""
    invalid_codes = [code for code in codes if code not in VALID_TERRITORIES]
    
    if invalid_codes:
        raise ValueError(f"Invalid territory codes: {invalid_codes}")
    
    # Check for WW + other codes conflict
    if 'WW' in codes and len(codes) > 1:
        raise ValueError("WW (Worldwide) cannot be combined with specific territories")
    
    return True
```

## Common Mistakes

1. **Using 3-letter codes**: Use `US` not `USA`
2. **Mixing WW with specific codes**: `WW` should be used alone
3. **Case sensitivity**: Always use uppercase (`GB` not `gb`)
4. **Invalid combinations**: Some partners don't accept certain combinations

## Partner-Specific Notes

### Spotify
- Accepts most ISO 3166-1 codes
- Supports WW for worldwide releases
- Some restrictions on certain territories

### Apple Music
- Requires specific territory codes per release
- WW not always accepted
- Different availability by territory

### YouTube Music
- WW cannot be combined with specific territories
- Some territories require additional metadata

## Best Practices

1. **Validate Codes**: Always validate territory codes before submission
2. **Check Partner Requirements**: Different partners have different requirements
3. **Use Appropriate Groupings**: Consider regional releases vs worldwide
4. **Document Territories**: Keep track of which territories are used for each release
5. **Handle Conflicts**: Be aware of territory conflicts and restrictions