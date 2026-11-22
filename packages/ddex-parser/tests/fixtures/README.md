# ddex-parser test fixtures

Tracked inputs for `tests/it` (the single integration-test binary) and the Node
binding smoke tests (`bindings/node/test-*.js`). Everything here is synthetic.

| File | Used by |
|---|---|
| `ern_381.xml`, `ern382_basic.xml`, `ern42_basic.xml`, `ern43_basic.xml` | `version_parsing`: one minimal message per ERN version. The 4.x files were adapted from the gitignored `test-suite/` corpus, adding the PartyId/ReleaseType fields the parser requires |
| `deal_without_reference.xml`, `deals_linked_to_releases.xml`, `image_without_reference_title.xml`, `sound_recording_without_duration.xml` | `community_prs_regression`, Node `test-community-prs.js` |
| `ern43_multi_disc.xml`, `ern43_main_track_release.xml` | `ern4_tracks`, Node `test-ern4-tracks.js` |
| `ern43_details.xml`, `ern382_details.xml` | The 0.5.0 fields (dates, P/C lines, genres, parental warnings, labels, MessageControlType, deal EndDate; 3.8.2 details blocks, ReleaseId ProprietaryId, track artists and ResourceGroup): `serialization`, Node `test-parser-fields.js`, Python `tests/test_parser_fields.py`; `ern382_details.xml` is also a source of ddex-builder's `ern38_round_trip`. Both are XSD-valid |

The official DDEX samples live in `examples/Samples42` and `examples/Samples43` at
the repo root (tracked) and are read directly by `ern4_tracks`.

## lxml oracle for ERN 4.x tracks

`scripts/ern4_oracle.py` is an independent lxml implementation of the ResourceGroup
walk. `scripts/compare_ern4_oracle.py` diffs it against the parser CLI per track.
`official_samples_track_counts_match_oracle` in `tests/it/ern4_tracks.rs` hard-codes
the oracle's release/track counts, so `cargo test` covers the regression without
Python. To re-run the full per-track comparison:

```bash
python3 -m venv .venv && .venv/bin/pip install lxml
cargo build --release -p ddex-parser --bin ddex-parser
.venv/bin/python scripts/compare_ern4_oracle.py examples/Samples42/*.xml examples/Samples43/*.xml
```

It prints one Markdown row per file and exits non-zero on any mismatch.
