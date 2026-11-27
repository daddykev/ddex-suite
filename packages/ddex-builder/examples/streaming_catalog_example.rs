//! Streaming Catalog Example
//!
//! Writes a large catalog with `ddex_builder::streaming::StreamingBuilder`, which
//! emits resources and releases one at a time to any `std::io::Write` instead of
//! building the whole message in memory. A progress callback reports throughput.
//!
//! Run with: `cargo run -p ddex-builder --example streaming_catalog_example [releases]`

use ddex_builder::builder::{LocalizedStringRequest, MessageHeaderRequest, PartyRequest};
use ddex_builder::streaming::{StreamingBuilder, StreamingConfig};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

const TRACKS_PER_RELEASE: usize = 10;

fn party(name: &str) -> PartyRequest {
    PartyRequest {
        party_name: vec![LocalizedStringRequest {
            text: name.to_string(),
            language_code: None,
        }],
        party_id: None,
        party_reference: None,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let release_count: usize = std::env::args()
        .nth(1)
        .and_then(|n| n.parse().ok())
        .unwrap_or(1_000);
    let output_path = std::env::temp_dir().join("streaming_catalog_example.xml");

    let config = StreamingConfig {
        // Flush to the writer whenever 1 MB is buffered.
        max_buffer_size: 1024 * 1024,
        progress_callback_frequency: 2_500,
        ..StreamingConfig::default()
    };
    let writer = BufWriter::new(File::create(&output_path)?);
    let mut builder = StreamingBuilder::new_with_config(writer, config)?;
    builder.set_estimated_total(release_count * (TRACKS_PER_RELEASE + 1));
    builder.set_progress_callback(Box::new(|p| {
        println!(
            "  progress: {} resources, {} releases, {:.1} MB written{}",
            p.resources_written,
            p.releases_written,
            p.bytes_written as f64 / (1024.0 * 1024.0),
            p.estimated_completion_percent
                .map(|pct| format!(" ({pct:.0}%)"))
                .unwrap_or_default(),
        );
    }));

    let header = MessageHeaderRequest {
        message_id: "CATALOG-0001".to_string(),
        message_sender: party("Example Label"),
        message_recipient: party("Example DSP"),
        message_control_type: None,
        message_created_date_time: "2024-01-01T00:00:00Z".to_string(),
    };

    let start = Instant::now();
    println!(
        "Streaming {release_count} releases x {TRACKS_PER_RELEASE} tracks to {}",
        output_path.display()
    );
    // The streaming writer does not support ERN 4.x yet (use DDEXBuilder::build for 4.2/4.3)
    builder.start_message(&header, "3.8.2")?;

    // ERN puts all resources before all releases, so write every sound
    // recording first and remember the references for each release.
    let mut release_resources: Vec<Vec<String>> = Vec::with_capacity(release_count);
    for r in 0..release_count {
        let mut refs = Vec::with_capacity(TRACKS_PER_RELEASE);
        for t in 0..TRACKS_PER_RELEASE {
            let reference = builder.write_resource(
                &format!("SR{r:06}{t:02}"),
                &format!("Track {} of Album {}", t + 1, r + 1),
                "Catalog Artist",
                Some(&format!("USCAT{:07}", r * TRACKS_PER_RELEASE + t)),
                Some("PT3M30S"),
                None,
            )?;
            refs.push(reference);
        }
        release_resources.push(refs);
    }

    builder.finish_resources_start_releases()?;
    for (r, refs) in release_resources.iter().enumerate() {
        builder.write_release(
            &format!("REL{r:06}"),
            &format!("Album {}", r + 1),
            "Catalog Artist",
            Some("Example Label"),
            Some(&format!("{:012}", 100_000_000_000u64 + r as u64)),
            Some("2024-01-01"),
            Some("Pop"),
            refs,
        )?;
    }

    let stats = builder.finish_message()?;
    let elapsed = start.elapsed();

    println!("\nDone in {:.2?}", elapsed);
    println!("  releases written:  {}", stats.releases_written);
    println!("  resources written: {}", stats.resources_written);
    println!(
        "  output size:       {:.1} MB",
        stats.bytes_written as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  peak buffer usage: {:.1} MB",
        stats.peak_memory_usage as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  throughput:        {:.0} releases/s",
        stats.releases_written as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}
