# Message Models

Complete documentation of DDEX message structures, headers, and top-level organization.

## Overview

DDEX messages are the root containers for all metadata exchanges. They define the communication context, control information, and structure for releases, resources, and commercial terms.

## Message Types

| Message Type | Purpose | Common Use Cases |
|--------------|---------|------------------|
| `NewReleaseMessage` | Deliver new release metadata | Album/single launches |
| `ReleaseNotificationMessage` | Notify of release changes | Updated metadata |
| `PurgeReleaseMessage` | Remove release from catalog | Content takedowns |
| `CatalogListMessage` | Provide catalog inventory | Bulk catalog delivery |

## Graph Model

### Message Structure

```typescript
interface Message {
  MessageHeader: MessageHeader;
  UpdateIndicator: UpdateIndicator;
  MessageControlType?: MessageControlType;
  CatalogTransfer?: CatalogTransfer;
  WorkList?: Work[];
  ReleaseList?: Release[];
  ResourceList?: Resource[];
  ChapterList?: Chapter[];
  CueSheetList?: CueSheet[];
  DealList?: Deal[];
  Extensions?: ExtensionData[];
}
```

### MessageHeader

```typescript
interface MessageHeader {
  MessageId: MessageId;
  MessageFileName?: string;
  MessageSender: PartyDescriptor;
  SentOnBehalfOf?: PartyDescriptor;
  MessageRecipient: PartyDescriptor;
  MessageCreatedDateTime: DateTime;
  MessageAuditTrail?: MessageAuditTrail;
  Comment?: string;
  MessageControlType?: MessageControlType;
}
```

#### Core Header Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `MessageId` | `MessageId` | ✓ | Unique message identifier |
| `MessageSender` | `PartyDescriptor` | ✓ | Sender organization details |
| `MessageRecipient` | `PartyDescriptor` | ✓ | Recipient organization details |
| `MessageCreatedDateTime` | `DateTime` | ✓ | Message creation timestamp |
| `SentOnBehalfOf` | `PartyDescriptor` | ○ | Third-party sender |
| `MessageAuditTrail` | `MessageAuditTrail` | ○ | Processing history |

#### Example Graph Structure

```typescript
{
  MessageHeader: {
    MessageId: "MSG_LABEL_20240115_001",
    MessageSender: {
      PartyId: [
        { Value: "DPID:PADPIDA2014101001U", Namespace: "DPID" }
      ],
      PartyName: [
        { FullName: "Independent Records Ltd" }
      ],
      TradingName: "Indie Records"
    },
    MessageRecipient: {
      PartyId: [
        { Value: "DPID:PADPIDA2014101002U", Namespace: "DPID" }
      ],
      PartyName: [
        { FullName: "Global Streaming Platform Inc" }
      ]
    },
    MessageCreatedDateTime: "2024-01-15T10:30:00Z"
  },
  UpdateIndicator: "OriginalMessage",
  MessageControlType: "LiveMessage"
}
```

## Flattened Model

### Simplified Message Info

```typescript
interface FlatMessageInfo {
  messageId: string;
  messageType: string;
  version: string;
  sender: {
    name: string;
    id?: string;
    tradingName?: string;
  };
  recipient: {
    name: string;
    id?: string;
  };
  sentOnBehalfOf?: {
    name: string;
    id?: string;
  };
  createdDateTime: string;
  updateIndicator: string;
  messageControlType: string;
  comment?: string;
}
```

#### Example Flattened Structure

```typescript
{
  messageInfo: {
    messageId: "MSG_LABEL_20240115_001",
    messageType: "NewReleaseMessage",
    version: "4.3",
    sender: {
      name: "Independent Records Ltd",
      id: "DPID:PADPIDA2014101001U",
      tradingName: "Indie Records"
    },
    recipient: {
      name: "Global Streaming Platform Inc", 
      id: "DPID:PADPIDA2014101002U"
    },
    createdDateTime: "2024-01-15T10:30:00Z",
    updateIndicator: "OriginalMessage",
    messageControlType: "LiveMessage"
  }
}
```

## Message Control Types

### Standard Control Types

| Control Type | Purpose | Behavior |
|--------------|---------|----------|
| `TestMessage` | Testing and validation | Not processed in production |
| `LiveMessage` | Production message | Full processing |
| `SandBoxMessage` | Sandbox environment | Limited processing |

### Update Indicators

| Indicator | Purpose | Effect |
|-----------|---------|--------|
| `OriginalMessage` | New content | Create new records |
| `UpdateMessage` | Modify existing | Update existing records |
| `PurgeMessage` | Remove content | Delete records |

## Party Descriptors

### Party Information

```typescript
interface PartyDescriptor {
  PartyId?: PartyId[];
  PartyName?: PartyName[];
  TradingName?: string;
  PartyReference?: PartyReference;
  ArtistRole?: ArtistRole[];
  DisplayArtist?: DisplayArtist[];
}
```

### Party Identifiers

```typescript
interface PartyId {
  Value: string;
  Namespace?: string;
  IsDPID?: boolean;
}

// Common party identifier types
type PartyIdNamespace = 
  | 'DPID'      // DDEX Party Identifier
  | 'ISNI'      // International Standard Name Identifier  
  | 'IPI'       // Interested Parties Information
  | 'Proprietary';
```

### Party Names

```typescript
interface PartyName {
  FullName: string;
  FullNameAsciiTranscribed?: string;
  FullNameIndexed?: string;
  NamesBeforeKeyName?: string;
  KeyName?: string;
  NamesAfterKeyName?: string;
  LanguageAndScriptCode?: string;
}
```

## Message Validation

### Required Fields by Version

#### ERN 3.8.2
- MessageId
- MessageSender (with DPID)
- MessageRecipient
- MessageCreatedDateTime
- UpdateIndicator

#### ERN 4.2/4.3
- All ERN 3.8.2 requirements
- Enhanced party identification
- Optional message audit trail
- Extended control types

### Business Rules

#### Message ID Format
```regex
^[A-Z0-9_-]+$
```
- Must be unique within sender's namespace
- Typically includes sender ID and timestamp
- Maximum length: 50 characters

#### DateTime Requirements
```typescript
// ISO 8601 format with timezone
"2024-01-15T10:30:00Z"        // UTC
"2024-01-15T10:30:00+01:00"   // With timezone offset
```

#### Party Validation
- At least one PartyId or PartyName required
- DPID format: `PADPIDA{YYYY}{MM}{DD}{NNN}{C}`
- ISNI format: 16 digits with optional formatting

## Message Assembly

### TypeScript Example

```typescript
import { MessageBuilder } from 'ddex-builder';

const message = new MessageBuilder()
  .setMessageId('MSG_INDIE_20240115_001')
  .setSender({
    partyId: 'DPID:PADPIDA2014101001U',
    partyName: 'Independent Records Ltd',
    tradingName: 'Indie Records'
  })
  .setRecipient({
    partyId: 'DPID:PADPIDA2014101002U', 
    partyName: 'Global Streaming Platform Inc'
  })
  .setUpdateIndicator('OriginalMessage')
  .setMessageControlType('LiveMessage')
  .addReleases(releases)
  .addResources(resources)
  .addDeals(deals)
  .build();
```

### Python Example

```python
from ddex_builder import MessageBuilder

message = MessageBuilder() \
    .set_message_id('MSG_INDIE_20240115_001') \
    .set_sender(
        party_id='DPID:PADPIDA2014101001U',
        party_name='Independent Records Ltd',
        trading_name='Indie Records'
    ) \
    .set_recipient(
        party_id='DPID:PADPIDA2014101002U',
        party_name='Global Streaming Platform Inc'
    ) \
    .set_update_indicator('OriginalMessage') \
    .set_message_control_type('LiveMessage') \
    .add_releases(releases) \
    .add_resources(resources) \
    .add_deals(deals) \
    .build()
```

## Message Processing Patterns

### Batch Processing

```typescript
// Process multiple messages
interface MessageBatch {
  messages: Message[];
  batchId: string;
  priority: 'High' | 'Normal' | 'Low';
  deliveryDate?: string;
}

class MessageProcessor {
  async processBatch(batch: MessageBatch): Promise<ProcessingResult[]> {
    return Promise.all(
      batch.messages.map(message => this.processMessage(message))
    );
  }
  
  private async processMessage(message: Message): Promise<ProcessingResult> {
    // Validate message structure
    const validation = await this.validateMessage(message);
    if (!validation.isValid) {
      return { success: false, errors: validation.errors };
    }
    
    // Process based on update indicator
    switch (message.UpdateIndicator) {
      case 'OriginalMessage':
        return await this.createNewContent(message);
      case 'UpdateMessage':
        return await this.updateExistingContent(message);
      case 'PurgeMessage':
        return await this.removeContent(message);
      default:
        return { success: false, errors: ['Unknown update indicator'] };
    }
  }
}
```

### Message Routing

```typescript
interface MessageRoute {
  messageType: string;
  sender: string;
  recipient: string;
  processingRules: ProcessingRule[];
}

class MessageRouter {
  private routes: Map<string, MessageRoute> = new Map();
  
  registerRoute(route: MessageRoute): void {
    const key = `${route.sender}:${route.recipient}:${route.messageType}`;
    this.routes.set(key, route);
  }
  
  routeMessage(message: Message): ProcessingRule[] {
    const sender = this.extractPartyId(message.MessageHeader.MessageSender);
    const recipient = this.extractPartyId(message.MessageHeader.MessageRecipient);
    const messageType = this.determineMessageType(message);
    
    const key = `${sender}:${recipient}:${messageType}`;
    const route = this.routes.get(key);
    
    return route?.processingRules || [];
  }
}
```

## Error Handling

### Common Message Errors

| Error Type | Code | Description | Resolution |
|------------|------|-------------|------------|
| Invalid MessageId | `MSG001` | Malformed message identifier | Use valid format |
| Unknown Sender | `MSG002` | Sender not registered | Register sender party |
| Invalid DateTime | `MSG003` | Malformed timestamp | Use ISO 8601 format |
| Missing Required Field | `MSG004` | Required field absent | Add missing field |
| Duplicate MessageId | `MSG005` | MessageId already used | Use unique identifier |

### Error Response Format

```typescript
interface MessageError {
  code: string;
  severity: 'Error' | 'Warning' | 'Info';
  description: string;
  xpath?: string;
  context?: Record<string, any>;
}

interface ProcessingResult {
  messageId: string;
  success: boolean;
  errors: MessageError[];
  warnings: MessageError[];
  processingTime: number;
  recordsProcessed: number;
}
```

## Version Differences

### ERN 3.8.2 Limitations

- Limited party identification options
- Basic message control types
- Simplified audit trail

### ERN 4.2 Enhancements

- Enhanced party descriptors
- Extended message control types  
- Improved namespace handling

### ERN 4.3 Features

- Full streaming platform support
- Advanced message routing
- Enhanced error reporting
- Improved internationalization

## Best Practices

### Message Design

1. **Unique IDs**: Ensure MessageIds are globally unique
2. **Descriptive Names**: Use clear party names and trading names
3. **Appropriate Control**: Choose correct message control type
4. **Audit Trail**: Include audit information for tracking
5. **Extensions**: Use extensions sparingly and document well

### Performance Optimization

1. **Batch Messages**: Group related releases in single messages
2. **Lazy Loading**: Load message components on demand
3. **Compression**: Use compression for large messages
4. **Caching**: Cache frequently accessed party information
5. **Validation**: Pre-validate before full processing

### Security Considerations

1. **Authentication**: Verify sender identity
2. **Authorization**: Validate sender permissions
3. **Integrity**: Check message tampering
4. **Audit**: Log all message processing
5. **Encryption**: Encrypt sensitive party information