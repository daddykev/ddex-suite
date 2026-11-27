# ddex-parser

DDEX ERN XML parser in Rust, part of [DDEX Suite](https://github.com/daddykev/ddex-suite).
It reads ERN 3.8.1, 3.8.2, 4.2 and 4.3 messages into a developer-friendly flat model and
a graph that follows the message structure. Node.js and Python bindings use the same
parser: `npm install ddex-parser`, `pip install ddex-parser`.

> **0.5.0 has breaking changes.** Text is kept exactly as written (no trimming), and
> `parse` always uses the complete parser (it no longer switches to a streaming path above
> 10 MB). The models gained new fields. See the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md).

## Installation

```toml
[dependencies]
ddex-parser = "0.5.1"
```

## Example

```rust,no_run
use ddex_parser::DDEXParser;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = std::fs::read("release.xml")?;
    let message = DDEXParser::new().parse(Cursor::new(xml))?;
    let flat = &message.flat;

    println!("{} (ERN {})", flat.message_id, flat.version);
    if let Some(control_type) = &flat.message_control_type {
        println!("control type: {control_type}");
    }
    for release in &flat.releases {
        println!(
            "{}: {} by {}",
            release.release_id,
            release.default_title.as_deref().unwrap_or("(no title)"),
            release.display_artist.as_deref().unwrap_or("(no artist)"),
        );
        for date in &release.release_dates {
            println!("  release date {} (as written)", date.date);
        }
        for genre in &release.genres {
            println!("  genre {}", genre.genre);
        }
        if let Some(p_line) = &release.p_line {
            println!("  (P) {:?} {}", p_line.year, p_line.text);
        }
        println!("  label {:?}, parental warning {:?}", release.label_name, release.parental_warning);
        for track in &release.tracks {
            println!(
                "  {}. {} [{}] disc {:?}, {} enclosing group(s)",
                track.position,
                track.title.as_deref().unwrap_or("(no title)"),
                track.isrc.as_deref().unwrap_or("no ISRC"),
                track.disc_number,
                track.group_path.len(),
            );
        }
    }
    for deal in &flat.deals {
        println!("deal for {:?}, from {:?} to {:?}", deal.releases, deal.validity.start, deal.validity.end);
    }
    Ok(())
}
```

What you get:

- `message.flat`: releases with their tracks (position, disc and track numbers, the
  enclosing `ResourceGroup`s in `group_path`), resources keyed by resource reference,
  deals (start and end dates, territories, use types) and parties. Release dates, P and C
  lines, genres, parental warnings and labels are read in every ERN version where the XSD
  has them, with their territory scope.
- `message.graph`: the message structure as written.
- Absent values stay `None` or empty: nothing is invented. Text keeps its whitespace,
  following each value's XSD `whiteSpace` facet.

## Security

Every reader rejects a DOCTYPE (and with it any DTD or external entity, XXE) and
references to undefined entities, so no entity is ever expanded. The default configuration
(`SecurityConfig::strict()`) also limits element depth (100) and the number of references
in text (100), and the parser never fetches from the network. It sets no input-size limit. The parser does not validate
against the DDEX XSD or the AVS business rules.

## Streaming

`parse` reads the whole document. The streaming APIs (`ParseMode::Stream` and the
`stream*` iterators) are experimental and opt-in: they read a subset of the fields. The
parallel iterator does not currently scale with the thread count.

## Performance

Criterion benches on a GitHub-hosted runner (Intel Xeon, 4 vCPUs), release profile, 0.5.1:
a typical ERN 4.3 album message (75 KB) parses in 1.7 ms, the official ERN 4.3 samples
(10–330 KB) in 0.27–7.4 ms, and a ~10 MB catalog in 211 ms, about 33–47 MiB/s whatever
the size
([details](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md)).

## Links

- Documentation: [docs.rs/ddex-parser](https://docs.rs/ddex-parser) and [ddex-suite.org](https://ddex-suite.org)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
