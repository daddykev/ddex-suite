//! Vendor quirks: messages that are valid except for a known vendor deviation.

use crate::common::{ern43_release, parse_str};

#[test]
fn missing_message_thread_id_is_accepted() {
    // MessageThreadId is optional (minOccurs=0); everything else is present.
    let xml = format!(
        r#"<?xml version="1.0"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>MSG123</MessageId>
        <MessageSender><PartyId>PADPIDA0000000001</PartyId></MessageSender>
        <MessageRecipient><PartyId>PADPIDA0000000002</PartyId></MessageRecipient>
        <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
    </MessageHeader>
    <ReleaseList>{}</ReleaseList>
</ern:NewReleaseMessage>"#,
        ern43_release("R1", "Title", "Artist")
    );
    let parsed = parse_str(&xml).expect("missing MessageThreadId must be accepted");
    assert_eq!(parsed.flat.message_id, "MSG123");
}

#[test]
fn empty_message_audit_trail_is_accepted() {
    // An empty MessageAuditTrail violates the 4.2 XSD (it needs at least one event), but
    // vendors send it; the parser is lenient here.
    let xml = format!(
        r#"<?xml version="1.0"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42">
    <MessageHeader>
        <MessageId>MSG123</MessageId>
        <MessageSender><PartyId>PADPIDA0000000001</PartyId></MessageSender>
        <MessageRecipient><PartyId>PADPIDA0000000002</PartyId></MessageRecipient>
        <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
        <MessageAuditTrail/>
    </MessageHeader>
    <ReleaseList>{}</ReleaseList>
</ern:NewReleaseMessage>"#,
        ern43_release("R1", "Title", "Artist")
    );
    let parsed = parse_str(&xml).expect("empty MessageAuditTrail must be accepted");
    assert_eq!(parsed.flat.message_id, "MSG123");
}
