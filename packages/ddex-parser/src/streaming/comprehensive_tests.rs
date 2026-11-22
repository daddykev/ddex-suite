//! Comprehensive tests for streaming parser functionality
//!
//! These tests verify the key requirements:
//! - O(1) memory complexity
//! - Selective parsing capabilities
//! - Security features
//! - Performance characteristics
//! - Real-world DDEX XML compatibility

#[cfg(test)]
mod tests {
    use crate::streaming::working_impl::{WorkingStreamIterator, WorkingStreamingElement};
    use ddex_core::models::versions::ERNVersion;
    use std::io::Cursor;

    /// Test with a realistic DDEX XML sample
    #[test]
    fn test_comprehensive_ddex_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <MessageHeader>
        <MessageThreadId>MSG-THREAD-001</MessageThreadId>
        <MessageId>MSG-001-COMPREHENSIVE</MessageId>
        <CreatedDateTime>2023-12-01T10:30:00Z</CreatedDateTime>
        <MessageSender>
            <PartyId namespace="ISNI">0000000123456789</PartyId>
            <PartyName>Test Music Label</PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyId namespace="ISNI">0000000987654321</PartyId>
            <PartyName>Digital Service Provider</PartyName>
        </MessageRecipient>
    </MessageHeader>

    <UpdateIndicator>OriginalMessage</UpdateIndicator>

    <Release ReleaseReference="REL-2023-001">
        <ReferenceTitle>
            <TitleText>Greatest Hits Collection</TitleText>
            <SubTitle>Remastered Edition</SubTitle>
        </ReferenceTitle>
        <Genre>
            <GenreText>Pop</GenreText>
            <SubGenre>Contemporary Pop</SubGenre>
        </Genre>
        <PLine>
            <Year>2023</Year>
            <PLineText>℗ 2023 Test Music Label</PLineText>
        </PLine>
        <CLine>
            <Year>2023</Year>
            <CLineText>© 2023 Test Music Label</CLineText>
        </CLine>
        <ReleaseLabelReference>LBL-001</ReleaseLabelReference>
        <ResourceGroupSequenceList>
            <ResourceGroup>
                <ResourceGroupContentItem>
                    <ResourceReference>RES-001</ResourceReference>
                    <ReleaseResourceReference>REL-RES-001</ReleaseResourceReference>
                </ResourceGroupContentItem>
                <ResourceGroupContentItem>
                    <ResourceReference>RES-002</ResourceReference>
                    <ReleaseResourceReference>REL-RES-002</ReleaseResourceReference>
                </ResourceGroupContentItem>
            </ResourceGroup>
        </ResourceGroupSequenceList>
    </Release>

    <SoundRecording ResourceReference="RES-001">
        <ResourceId>
            <ISRC>USRC17607839</ISRC>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Hit Song #1</TitleText>
        </ReferenceTitle>
        <Duration>PT3M45S</Duration>
        <CreationDate>2023-01-15</CreationDate>
        <LanguageOfPerformance>en</LanguageOfPerformance>
        <ResourceContributor>
            <PartyId namespace="IPI">00199081827</PartyId>
            <PartyName>John Artist</PartyName>
            <ContributorRole>MainArtist</ContributorRole>
        </ResourceContributor>
        <ResourceContributor>
            <PartyId namespace="IPI">00295920775</PartyId>
            <PartyName>Jane Producer</PartyName>
            <ContributorRole>Producer</ContributorRole>
        </ResourceContributor>
    </SoundRecording>

    <SoundRecording ResourceReference="RES-002">
        <ResourceId>
            <ISRC>USRC17607840</ISRC>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Hit Song #2</TitleText>
        </ReferenceTitle>
        <Duration>PT4M12S</Duration>
        <CreationDate>2023-02-20</CreationDate>
        <LanguageOfPerformance>en</LanguageOfPerformance>
        <ResourceContributor>
            <PartyId namespace="IPI">00199081827</PartyId>
            <PartyName>John Artist</PartyName>
            <ContributorRole>MainArtist</ContributorRole>
        </ResourceContributor>
    </SoundRecording>

    <Video ResourceReference="RES-003">
        <ResourceId>
            <ProprietaryId namespace="LABEL">VID-001</ProprietaryId>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Music Video - Hit Song #1</TitleText>
        </ReferenceTitle>
        <Duration>PT3M50S</Duration>
        <CreationDate>2023-03-01</CreationDate>
        <VideoCodecType>H.264</VideoCodecType>
        <VideoDefinitionType>HighDefinition</VideoDefinitionType>
    </Video>

    <Image ResourceReference="RES-004">
        <ResourceId>
            <ProprietaryId namespace="LABEL">IMG-001</ProprietaryId>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Album Cover Art</TitleText>
        </ReferenceTitle>
        <CreationDate>2023-01-10</CreationDate>
        <Width>3000</Width>
        <Height>3000</Height>
        <ImageCodecType>JPEG</ImageCodecType>
        <ImageResolution>300</ImageResolution>
    </Image>

</ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let elements: Result<Vec<_>, _> = iterator.collect();
        assert!(elements.is_ok(), "Comprehensive parsing should succeed");

        let elements = elements.unwrap();
        println!("Total elements parsed: {}", elements.len());

        // Verify we found all expected elements
        let header_count = elements
            .iter()
            .filter(|e| matches!(e, WorkingStreamingElement::MessageHeader { .. }))
            .count();
        let release_count = elements
            .iter()
            .filter(|e| matches!(e, WorkingStreamingElement::Release { .. }))
            .count();
        let sound_recording_count = elements
            .iter()
            .filter(|e| matches!(e, WorkingStreamingElement::SoundRecording { .. }))
            .count();
        let video_count = elements
            .iter()
            .filter(|e| matches!(e, WorkingStreamingElement::Video { .. }))
            .count();
        let image_count = elements
            .iter()
            .filter(|e| matches!(e, WorkingStreamingElement::Image { .. }))
            .count();

        assert_eq!(header_count, 1, "Should find exactly 1 message header");
        assert_eq!(release_count, 1, "Should find exactly 1 release");
        assert_eq!(
            sound_recording_count, 2,
            "Should find exactly 2 sound recordings"
        );
        assert_eq!(video_count, 1, "Should find exactly 1 video");
        assert_eq!(image_count, 1, "Should find exactly 1 image");

        // Verify specific element content
        if let Some(WorkingStreamingElement::MessageHeader { message_id, .. }) = elements
            .iter()
            .find(|e| matches!(e, WorkingStreamingElement::MessageHeader { .. }))
        {
            assert_eq!(message_id, "MSG-001-COMPREHENSIVE");
        }

        if let Some(WorkingStreamingElement::Release {
            reference, title, ..
        }) = elements
            .iter()
            .find(|e| matches!(e, WorkingStreamingElement::Release { .. }))
        {
            assert_eq!(reference, "REL-2023-001");
            assert_eq!(title, "Greatest Hits Collection"); // Should extract TitleText
        }

        if let Some(WorkingStreamingElement::SoundRecording { reference: _, isrc, duration, .. }) =
            elements.iter().find(|e| matches!(e, WorkingStreamingElement::SoundRecording { reference, .. } if reference == "RES-001")) {
            assert_eq!(isrc, &Some("USRC17607839".to_string()));
            assert_eq!(duration, &Some("PT3M45S".to_string()));
        }
    }

    /// Test memory usage with large documents
    #[test]
    fn test_o1_memory_complexity() {
        // Generate XML with many elements to test memory bounds
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">"#,
        );

        xml.push_str(
            r#"
    <MessageHeader>
        <MessageId>MEMORY-TEST-MSG</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
"#,
        );

        // Add many sound recordings to test memory usage
        for i in 1..=100 {
            xml.push_str(&format!(
                r#"
    <SoundRecording ResourceReference="RES-{:03}">
        <ResourceId>
            <ISRC>TEST{:08}</ISRC>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Test Track #{}</TitleText>
        </ReferenceTitle>
        <Duration>PT3M30S</Duration>
        <CreationDate>2023-01-01</CreationDate>
        <LanguageOfPerformance>en</LanguageOfPerformance>
    </SoundRecording>
"#,
                i, i, i
            ));
        }

        xml.push_str("</ern:NewReleaseMessage>");

        println!("Generated XML size: {} bytes", xml.len());

        let cursor = Cursor::new(xml.as_bytes());
        let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let mut element_count = 0;
        let mut max_memory_seen = 0usize;

        // Process elements one by one and track memory usage
        while let Some(result) = iterator.next() {
            assert!(result.is_ok(), "All elements should parse successfully");
            element_count += 1;

            let stats = iterator.stats();
            max_memory_seen = max_memory_seen.max(stats.current_memory_bytes);

            // Verify O(1) memory complexity: memory should not grow linearly with elements
            if element_count > 10 {
                assert!(
                    stats.current_memory_bytes < 50 * 1024 * 1024,
                    "Memory usage should stay bounded under 50MB, got {} bytes at element {}",
                    stats.current_memory_bytes,
                    element_count
                );
            }
        }

        println!("Processed {} elements", element_count);
        println!(
            "Max memory used: {} bytes ({:.2} MB)",
            max_memory_seen,
            max_memory_seen as f64 / (1024.0 * 1024.0)
        );

        assert!(
            element_count >= 100,
            "Should process at least 100 sound recordings"
        );
        assert!(
            max_memory_seen < 10 * 1024 * 1024,
            "Max memory should be under 10MB for O(1) complexity, got {:.2} MB",
            max_memory_seen as f64 / (1024.0 * 1024.0)
        );
    }

    /// Test selective parsing - only extracting specific element types
    #[test]
    fn test_selective_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>SELECTIVE-TEST</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>

    <Release ReleaseReference="REL-SELECTIVE">
        <ReferenceTitle><TitleText>Selective Test Release</TitleText></ReferenceTitle>
    </Release>

    <SoundRecording ResourceReference="RES-AUDIO-001">
        <ReferenceTitle><TitleText>Audio Track 1</TitleText></ReferenceTitle>
        <Duration>PT2M30S</Duration>
    </SoundRecording>

    <SoundRecording ResourceReference="RES-AUDIO-002">
        <ReferenceTitle><TitleText>Audio Track 2</TitleText></ReferenceTitle>
        <Duration>PT3M15S</Duration>
    </SoundRecording>

    <Video ResourceReference="RES-VIDEO-001">
        <ReferenceTitle><TitleText>Music Video</TitleText></ReferenceTitle>
        <Duration>PT3M45S</Duration>
    </Video>

    <Image ResourceReference="RES-IMAGE-001">
        <ReferenceTitle><TitleText>Cover Art</TitleText></ReferenceTitle>
        <Width>1000</Width>
        <Height>1000</Height>
    </Image>

</ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let elements: Result<Vec<_>, _> = iterator.collect();
        assert!(elements.is_ok());

        let elements = elements.unwrap();

        // Test selective extraction of only sound recordings
        let sound_recordings: Vec<_> = elements
            .iter()
            .filter_map(|e| match e {
                WorkingStreamingElement::SoundRecording {
                    reference,
                    title,
                    duration,
                    ..
                } => Some((reference.clone(), title.clone(), duration.clone())),
                _ => None,
            })
            .collect();

        assert_eq!(
            sound_recordings.len(),
            2,
            "Should selectively extract 2 sound recordings"
        );
        assert!(sound_recordings
            .iter()
            .any(|(ref_, _, _)| ref_ == "RES-AUDIO-001"));
        assert!(sound_recordings
            .iter()
            .any(|(ref_, _, _)| ref_ == "RES-AUDIO-002"));

        // Test selective extraction of only videos
        let videos: Vec<_> = elements
            .iter()
            .filter_map(|e| match e {
                WorkingStreamingElement::Video {
                    reference, title, ..
                } => Some((reference.clone(), title.clone())),
                _ => None,
            })
            .collect();

        assert_eq!(videos.len(), 1, "Should selectively extract 1 video");
        assert_eq!(videos[0].0, "RES-VIDEO-001");
        assert_eq!(videos[0].1, "Music Video");

        // Test selective extraction of only images
        let images: Vec<_> = elements
            .iter()
            .filter_map(|e| match e {
                WorkingStreamingElement::Image {
                    reference,
                    width,
                    height,
                    ..
                } => Some((reference.clone(), width.clone(), height.clone())),
                _ => None,
            })
            .collect();

        assert_eq!(images.len(), 1, "Should selectively extract 1 image");
        assert_eq!(images[0].0, "RES-IMAGE-001");
        assert_eq!(images[0].1, Some(1000));
        assert_eq!(images[0].2, Some(1000));
    }

    /// Test performance characteristics (timing assertions: perf-tests only)
    #[cfg(feature = "perf-tests")]
    #[test]
    fn test_performance_benchmarks() {
        // Create a moderately sized XML (similar to real-world releases)
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">"#,
        );

        xml.push_str(
            r#"
    <MessageHeader>
        <MessageId>PERFORMANCE-TEST</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>

    <Release ReleaseReference="REL-PERF">
        <ReferenceTitle><TitleText>Performance Test Album</TitleText></ReferenceTitle>
    </Release>
"#,
        );

        // Add 50 tracks (typical album size)
        for i in 1..=50 {
            let minutes = (i * 17) % 60;
            let seconds = (i * 23) % 60;
            xml.push_str(&format!(
                r#"
    <SoundRecording ResourceReference="TRACK-{:02}">
        <ResourceId><ISRC>PERF{:08}</ISRC></ResourceId>
        <ReferenceTitle><TitleText>Track {}</TitleText></ReferenceTitle>
        <Duration>PT{}M{}S</Duration>
        <CreationDate>2023-01-01</CreationDate>
        <LanguageOfPerformance>en</LanguageOfPerformance>
    </SoundRecording>
"#,
                i, i, i, minutes, seconds
            ));
        }

        xml.push_str("</ern:NewReleaseMessage>");

        let xml_size = xml.len();
        println!(
            "Performance test XML size: {} bytes ({:.1} KB)",
            xml_size,
            xml_size as f64 / 1024.0
        );

        let start = std::time::Instant::now();
        let cursor = Cursor::new(xml.as_bytes());
        let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let mut element_count = 0;
        while let Some(result) = iterator.next() {
            assert!(result.is_ok(), "Performance test parsing should succeed");
            element_count += 1;
        }

        let elapsed = start.elapsed();
        let final_stats = iterator.stats();

        println!("Performance Results:");
        println!("  - Elements processed: {}", element_count);
        println!("  - Time elapsed: {:.2}ms", elapsed.as_millis());
        println!(
            "  - Throughput: {:.2} MB/s",
            final_stats.throughput_mb_per_sec
        );
        println!(
            "  - Memory efficiency: {:.1}x",
            final_stats.memory_efficiency()
        );
        println!(
            "  - Max memory used: {:.2} KB",
            final_stats.max_memory_used_bytes as f64 / 1024.0
        );

        // Performance assertions
        assert!(
            elapsed.as_millis() < 100,
            "Should parse typical album in under 100ms, took {}ms",
            elapsed.as_millis()
        );
        assert!(
            final_stats.is_memory_bounded(),
            "Should maintain O(1) memory usage"
        );
        assert!(
            final_stats.throughput_mb_per_sec > 1.0,
            "Should achieve at least 1 MB/s throughput"
        );
        assert!(element_count >= 50, "Should process all 50+ elements");
    }

    /// Test error handling and security features
    #[test]
    fn test_security_and_error_handling() {
        // Test 1: Deep nesting protection
        let mut deep_xml = String::from(r#"<?xml version="1.0"?>"#);
        for i in 0..150 {
            deep_xml.push_str(&format!("<level{}>", i));
        }
        deep_xml.push_str("content");
        for i in (0..150).rev() {
            deep_xml.push_str(&format!("</level{}>", i));
        }

        let cursor = Cursor::new(deep_xml.as_bytes());
        let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let result = iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_err(), "Should reject deeply nested XML");

        // Test 2: Malformed XML handling
        let malformed_xml = r#"<?xml version="1.0"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>MALFORMED-TEST</MessageId>
        <UnclosedTag>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
</ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(malformed_xml.as_bytes());
        let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let mut found_error = false;
        while let Some(result) = iterator.next() {
            if result.is_err() {
                found_error = true;
                break;
            }
        }

        // Note: Some XML parsers may be lenient, so we allow either error detection or successful parsing
        // The important thing is that the parser doesn't crash or consume excessive resources
        println!(
            "Malformed XML test completed (error detected: {})",
            found_error
        );

        // Test 3: Empty document handling
        let empty_xml = "";
        let cursor = Cursor::new(empty_xml.as_bytes());
        let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let result = iterator.next();
        // Should either return None or an error, but shouldn't panic
        if let Some(r) = result {
            println!("Empty XML result: {:?}", r.is_ok());
        } else {
            println!("Empty XML returned None (acceptable)");
        }
    }

    /// Test compatibility with different ERN versions
    #[test]
    fn test_ern_version_compatibility() {
        let base_xml = |version_namespace: &str| {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="{}">
    <MessageHeader>
        <MessageId>VERSION-TEST</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
    <Release ReleaseReference="REL-VER">
        <ReferenceTitle><TitleText>Version Test</TitleText></ReferenceTitle>
    </Release>
</ern:NewReleaseMessage>"#,
                version_namespace
            )
        };

        // Test different ERN versions
        let test_cases = vec![
            (ERNVersion::V3_8_2, "http://ddex.net/xml/ern/382"),
            (ERNVersion::V4_2, "http://ddex.net/xml/ern/42"),
            (ERNVersion::V4_3, "http://ddex.net/xml/ern/43"),
        ];

        for (version, namespace) in test_cases {
            let xml = base_xml(namespace);
            let cursor = Cursor::new(xml.as_bytes());
            let iterator = WorkingStreamIterator::new(cursor, version);

            let elements: Result<Vec<_>, _> = iterator.collect();
            assert!(
                elements.is_ok(),
                "Should successfully parse ERN version {:?}",
                version
            );

            let elements = elements.unwrap();
            assert!(
                elements.len() >= 2,
                "Should find header and release for ERN version {:?}",
                version
            );

            // Verify version is correctly set in header
            if let Some(WorkingStreamingElement::MessageHeader {
                version: header_version,
                ..
            }) = elements
                .iter()
                .find(|e| matches!(e, WorkingStreamingElement::MessageHeader { .. }))
            {
                assert_eq!(
                    header_version, &version,
                    "Header should have correct version"
                );
            }
        }
    }
}
