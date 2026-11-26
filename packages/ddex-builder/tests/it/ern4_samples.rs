//! Official DDEX samples (examples/Samples42, examples/Samples43): parse -> build (same
//! version) -> XSD validation -> re-parse, comparing the re-parsed flat output with the
//! original on: release title / ICPN / type / display artist, track count, per track
//! ISRC / title / duration / disc / position, deal count and deal release references.
//!
//! Data the build request cannot carry yet (contributors, P/C lines, images, ...) is not
//! compared; CHANGELOG.md lists it. Run with `--nocapture` to print the results table.

use crate::ern4_support::{parse, request_from_parsed, validate_xsd, xsd_skipped};
use ddex_builder::round_trip::compare_flat as compare;
use ddex_builder::{BuildOptions, DDEXBuilder};
use std::path::PathBuf;

fn samples() -> Vec<(&'static str, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut out = Vec::new();
    for (version, dir) in [("4.2", "Samples42"), ("4.3", "Samples43")] {
        let mut files: Vec<PathBuf> = std::fs::read_dir(root.join(dir))
            .expect("samples directory")
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "xml"))
            .collect();
        files.sort();
        out.extend(files.into_iter().map(|p| (version, p)));
    }
    out
}

#[test]
fn official_samples_round_trip() {
    let mut table = vec![
        "| sample | XSD valid | fields equal | differences |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    let mut failures = Vec::new();
    for (version, path) in samples() {
        let name = format!(
            "{}/{}",
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy(),
            path.file_name().unwrap().to_string_lossy()
        );
        let original = parse(&std::fs::read_to_string(&path).unwrap());
        let request = request_from_parsed(&original, version);
        let xml = match DDEXBuilder::new().build(request, BuildOptions::default()) {
            Ok(result) => result.xml,
            Err(e) => {
                failures.push(format!("{name}: build failed: {e}"));
                table.push(format!("| {name} | build failed | - | {e} |"));
                continue;
            }
        };
        // DDEX_SAMPLES_OUT=<dir> keeps the rebuilt documents for inspection
        if let Ok(dir) = std::env::var("DDEX_SAMPLES_OUT") {
            let out = PathBuf::from(dir).join(name.replace('/', "_"));
            std::fs::create_dir_all(out.parent().unwrap()).unwrap();
            std::fs::write(out, &xml).unwrap();
        }
        let xsd = if xsd_skipped() {
            "skipped".to_string()
        } else {
            match validate_xsd(&xml, version) {
                Ok(()) => "yes".to_string(),
                Err(errors) => {
                    failures.push(format!("{name}: not schema-valid:\n{errors}"));
                    "**no**".to_string()
                }
            }
        };
        let diffs = compare(&original.flat, &parse(&xml).flat);
        if !diffs.is_empty() {
            failures.push(format!("{name}: {}", diffs.join("; ")));
        }
        let tracks = original
            .flat
            .releases
            .iter()
            .map(|r| r.tracks.len())
            .sum::<usize>();
        table.push(format!(
            "| {name} | {xsd} | {} | {} |",
            if diffs.is_empty() {
                format!("yes ({tracks} tracks, {} deals)", original.flat.deals.len())
            } else {
                "**no**".to_string()
            },
            if diffs.is_empty() {
                "none".to_string()
            } else {
                diffs.join("; ")
            }
        ));
    }
    println!("{}", table.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
