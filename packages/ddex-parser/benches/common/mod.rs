//! Bench inputs shared by the parser benches and the perf-tests sanity limits
//! (`tests/it/perf.rs` includes this file).

#![allow(dead_code)]

use std::path::PathBuf;

/// `examples/` at the repository root.
pub fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// The official ERN 4.3 samples, as (file name, contents).
pub fn official_43_samples() -> Vec<(String, String)> {
    let dir = examples_dir().join("Samples43");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "xml"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&p).unwrap())
        })
        .collect()
}

/// `examples/Samples43/1 Audio.xml` (75 KB, 21 sound recordings, one album).
pub fn audio_album_43() -> String {
    std::fs::read_to_string(examples_dir().join("Samples43/1 Audio.xml")).unwrap()
}

/// A large ERN 4.3 catalog of about `target_bytes`: `1 Audio.xml` with its 21
/// `SoundRecording`s repeated (references renamed `A1` → `A1_2`, ...) until the target
/// size is reached.
pub fn large_catalog_43(target_bytes: usize) -> String {
    let sample = audio_album_43();
    let start = sample.find("<SoundRecording>").expect("SoundRecording");
    let end =
        sample.rfind("</SoundRecording>").expect("/SoundRecording") + "</SoundRecording>".len();
    let recordings = &sample[start..end];
    let mut out = String::with_capacity(target_bytes + sample.len());
    out.push_str(&sample[..end]);
    let mut copy = 2;
    while out.len() + (sample.len() - end) < target_bytes {
        out.push('\n');
        out.push_str(&rename_references(recordings, copy));
        copy += 1;
    }
    out.push_str(&sample[end..]);
    out
}

fn rename_references(block: &str, copy: usize) -> String {
    let mut out = String::with_capacity(block.len() + 256);
    let mut rest = block;
    let tag = "<ResourceReference>";
    while let Some(i) = rest.find(tag) {
        let value_start = i + tag.len();
        let value_end = value_start + rest[value_start..].find('<').unwrap();
        out.push_str(&rest[..value_end]);
        out.push_str(&format!("_{copy}"));
        rest = &rest[value_end..];
    }
    out.push_str(rest);
    out
}
