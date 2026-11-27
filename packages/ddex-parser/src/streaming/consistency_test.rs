//! Test to ensure parsing consistency between different parsers

use crate::error::ParseError;
use crate::streaming::{ParallelStreamingIterator, WorkingStreamIterator};
use ddex_core::models::versions::ERNVersion;
use std::io::Cursor;

pub fn test_parser_consistency() -> Result<(), ParseError> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddx.net/xml/ern/43">
    <MessageHeader>
        <MessageId>CONSISTENCY-TEST-MSG</MessageId>
        <CreatedDateTime>2024-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
    <Release ReleaseReference="CONS-REL-001">
        <ReferenceTitle>
            <TitleText>Consistency Test Release</TitleText>
        </ReferenceTitle>
    </Release>
    <SoundRecording ResourceReference="CONS-RES-001">
        <ResourceId>
            <ISRC>CONS12345678</ISRC>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Consistency Test Track</TitleText>
        </ReferenceTitle>
        <Duration>PT3M45S</Duration>
    </SoundRecording>
</ern:NewReleaseMessage>"#;

    // Test working parser
    let cursor = Cursor::new(xml.as_bytes());
    let working_iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);
    let working_elements: Result<Vec<_>, _> = working_iterator.collect();
    let working_elements = working_elements?;

    println!("Working parser found {} elements:", working_elements.len());
    for (i, element) in working_elements.iter().enumerate() {
        println!("  {}: {:?}", i, std::mem::discriminant(element));
    }

    // Test parallel parser
    let cursor = Cursor::new(xml.as_bytes());
    let parallel_iterator = ParallelStreamingIterator::new(cursor, ERNVersion::V4_3);
    let parallel_elements: Result<Vec<_>, _> = parallel_iterator.collect();
    let parallel_elements = parallel_elements?;

    println!(
        "Parallel parser found {} elements:",
        parallel_elements.len()
    );
    for (i, element) in parallel_elements.iter().enumerate() {
        println!("  {}: {:?}", i, std::mem::discriminant(element));
    }

    // Check if counts match
    if working_elements.len() != parallel_elements.len() {
        println!(
            "❌ Element count mismatch: {} vs {}",
            working_elements.len(),
            parallel_elements.len()
        );

        // Find differences
        let max_len = working_elements.len().max(parallel_elements.len());
        for i in 0..max_len {
            let working_type = working_elements.get(i).map(std::mem::discriminant);
            let parallel_type = parallel_elements.get(i).map(std::mem::discriminant);

            if working_type != parallel_type {
                println!(
                    "  Difference at index {}: working={:?}, parallel={:?}",
                    i, working_type, parallel_type
                );
            }
        }

        return Err(ParseError::InvalidValue {
            field: "element count".to_string(),
            value: "differs between the working and parallel parsers".to_string(),
        });
    }

    println!("✅ Element counts match!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency() {
        test_parser_consistency().unwrap();
    }
}
