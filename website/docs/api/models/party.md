# Party Models

Party models represent individuals and organizations involved in music rights, including artists, labels, publishers, and other stakeholders.

## Core Types

### Party

Represents any individual or organization:

```typescript
interface Party {
  partyId: string;
  partyName: PartyName;
  partyType?: PartyType;
  roles?: PartyRole[];
  contactInformation?: ContactInformation[];
  identifiers?: PartyIdentifier[];
}
```

### PartyName

Structured name information:

```typescript
interface PartyName {
  fullName: string;
  fullNameAsciiTranscribed?: string;
  namesBeforeKeyName?: string[];
  keyName?: string;
  namesAfterKeyName?: string[];
}
```

### PartyType

Classification of party entities:

```typescript
enum PartyType {
  Person = 'Person',
  Organisation = 'Organisation',
  Character = 'Character'
}
```

## Roles and Relationships

### PartyRole

Defines the party's function in the context:

```typescript
interface PartyRole {
  role: RoleType;
  instrumentType?: string[];
  contributorRole?: ContributorRole;
}
```

### Common Role Types

- **MainArtist** - Primary performing artist
- **FeaturedArtist** - Guest or featured performer  
- **Producer** - Record producer
- **Composer** - Musical composition author
- **Lyricist** - Lyrics author
- **Publisher** - Music publisher
- **RecordLabel** - Record label/distributor
- **RightsController** - Rights administrator

## Contact Information

### ContactInformation

Contact details for parties:

```typescript
interface ContactInformation {
  emailAddress?: string[];
  telephoneNumber?: string[];
  webPage?: string[];
  physicalAddress?: Address[];
}
```

### Address

Physical address information:

```typescript
interface Address {
  streetAddress?: string[];
  city?: string;
  postalCode?: string;
  territoryCode?: string;
}
```

## Identifiers

### PartyIdentifier

External identification systems:

```typescript
interface PartyIdentifier {
  value: string;
  namespace: string;
  isDPID?: boolean;  // Interested Party ID
  isISNI?: boolean;  // International Standard Name Identifier
}
```

## Usage Examples

### Accessing Party Data

```typescript
const result = await parser.parse(xmlContent);

// Access all parties
const parties = result.flat.parties;

// Find artists
const artists = parties.filter(party => 
  party.roles?.some(role => 
    ['MainArtist', 'FeaturedArtist'].includes(role.role)
  )
);

// Find labels
const labels = parties.filter(party =>
  party.roles?.some(role => role.role === 'RecordLabel')
);

// Get artist names
artists.forEach(artist => {
  console.log(artist.partyName.fullName);
});
```

### Building Party Data

```typescript
const buildRequest = {
  parties: [{
    partyId: 'P123456789',
    partyName: {
      fullName: 'Example Artist',
      fullNameAsciiTranscribed: 'Example Artist'
    },
    partyType: 'Person',
    roles: [{
      role: 'MainArtist'
    }],
    identifiers: [{
      value: '123456789',
      namespace: 'ISNI',
      isISNI: true
    }],
    contactInformation: [{
      emailAddress: ['artist@example.com'],
      webPage: ['https://example-artist.com']
    }]
  }]
};
```

### Working with Contributors

Contributors are parties with specific roles in resource creation:

```typescript
// Access sound recording contributors
const recording = result.flat.resources.soundRecordings[0];
const contributors = recording.contributors || [];

// Group by role
const groupedContributors = contributors.reduce((acc, contributor) => {
  const role = contributor.role;
  if (!acc[role]) acc[role] = [];
  acc[role].push(contributor);
  return acc;
}, {} as Record<string, Contributor[]>);

console.log('Producers:', groupedContributors.Producer?.map(p => p.partyName.fullName));
console.log('Artists:', groupedContributors.MainArtist?.map(p => p.partyName.fullName));
```

## Rights and Ownership

Parties can have different rights relationships:

```typescript
// Rights controllers for a resource
const rightsControllers = recording.rightsController || [];

// Master rights owner
const masterOwner = rightsControllers.find(party =>
  party.roles?.some(role => role.role === 'RightsController')
);

// Publishing rights
const publisher = parties.find(party =>
  party.roles?.some(role => role.role === 'Publisher')
);
```

## See Also

- [Resource Models](./resource) - Contributor relationships
- [Deal Models](./deal) - Rights and licensing
- [Identifiers](./identifiers) - External ID systems