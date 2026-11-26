---
sidebar_position: 8
title: Python and DataFrames
description: The Python parser's accessors and DataFrame schemas, deals(), message_control_type(), building from a DataFrame, and missing values in pandas 2 and 3.
---

# Python and DataFrames

The Python packages are native extensions over the Rust parser and builder. The DataFrame
functions need pandas:

```bash
pip install "ddex-parser[pandas]" "ddex-builder[pandas]"
```

CI runs the examples on this page with pandas 3; they also work with pandas 2.

## The parsed message

`DDEXParser().parse(xml)` takes `str` or `bytes` and returns a `ParsedERNMessage`. Its
accessors are **methods**:

| Method | Returns |
|---|---|
| `message_id()` | `MessageHeader/MessageId` |
| `version()` | The version enum's name: `"V3_8_1"`, `"V3_8_2"`, `"V4_2"` or `"V4_3"` |
| `release_count()` | Number of releases |
| `message_control_type()` | `MessageHeader/MessageControlType` as written (`"LiveMessage"`, `"TestMessage"`), or `None` |
| `deals()` | One dict per deal (below) |
| `to_dataframe(schema="flat")` | A pandas DataFrame: `"flat"`, `"releases"` or `"tracks"` |

`DDEXParser().to_dataframe(xml, schema)` parses and converts in one call.
`parse_async(xml)` is the awaitable form of `parse`.

```python
from ddex_parser import DDEXParser

with open("release.xml", encoding="utf-8") as f:
    message = DDEXParser().parse(f.read())

print(message.message_id(), message.version(), message.message_control_type())
for deal in message.deals():
    print(deal["releases"], deal["territories"], deal["start_date"], deal["end_date"])

tracks = message.to_dataframe("tracks")
print(tracks[["track_index", "track_title", "isrc", "duration", "disc_number", "group_path"]])
```

### `deals()`

| Key | Value |
|---|---|
| `deal_id` | `DealReference`, or `None` |
| `releases` | The release references the deal applies to (`"R0"`, track releases `"R1"`, ...) |
| `start_date`, `end_date` | RFC 3339 strings (midnight UTC for a date), or `None` |
| `territories` | `TerritoryCode`s |
| `excluded_territories` | `ExcludedTerritoryCode`s |
| `usage_rights` | Use types |
| `distribution_channels` | Distribution channels |

The [parse output reference](./parse-output.mdx#deal) explains each value.

## DataFrame schemas

Every row of a schema has every column; a value the message does not have is missing (see
[below](#missing-values-pandas-2-and-3)). Durations are in seconds. Columns holding lists
contain Python lists of dicts.

### `"flat"`: a message row, then one row per release

| Column | On the message row | On release rows |
|---|---|---|
| `type` | `"message"` | `"release"` |
| `message_id`, `sender`, `created_date`, `message_type`, `message_control_type` | The header (`sender` is the sender's name; `created_date` RFC 3339) | missing |
| `release_index` | missing | 0-based index |
| `release_id` | missing | The release reference |
| `title`, `artist` | missing | Default title, display artist |
| `p_line`, `c_line` | missing | The default P / C line text |
| `genre`, `sub_genre` | missing | The default genre |
| `parental_warning` | missing | The default parental warning |
| `label_name` | missing | The label |
| `release_date` | missing | The default release date (RFC 3339), only when it is a full date |
| `track_count` | missing | Number of tracks |

### `"releases"`: one row per release

| Column | Value |
|---|---|
| `release_id` | The release reference (`R0`, ...) |
| `icpn`, `isrc`, `grid` | `ReleaseId` identifiers (`isrc` is ERN 3.8.x only) |
| `proprietary_ids` | `[{"namespace", "value"}]` |
| `title`, `artist` | Default title, display artist |
| `genre`, `sub_genre` | The default genre |
| `genres` | `[{"genre", "sub_genre", "territory_codes", "excluded_territory_codes", "is_default"}]` |
| `parental_warning` | The default parental warning |
| `parental_warnings` | Every parental warning value |
| `label_name` | The label |
| `p_line`, `p_line_year`, `c_line`, `c_line_year` | The default P and C lines |
| `release_date`, `original_release_date` | The default dates (RFC 3339), only when they are full dates |
| `release_dates`, `original_release_dates` | Every date as written: `[{"date", "is_approximate", "territory_codes", "excluded_territory_codes", "is_default"}]` |
| `track_count` | Number of tracks |

There is no release type column.

### `"tracks"`: one row per track

| Column | Value |
|---|---|
| `release_id`, `release_title` | The track's release (reference and default title) |
| `track_index` | 0-based index within the release |
| `track_id` | The resource reference (`A1`, ...) |
| `track_title`, `artist` | Title, display artist |
| `artists` | `[{"name", "role", "party_reference"}]` |
| `isrc` | ISRC |
| `duration` | Seconds (float) |
| `position`, `disc_number`, `track_number` | The [track layout](./parse-output.mdx#track-layout) |
| `group_path` | `[{"group_type", "sequence_number"}]`, outermost first |
| `track_release_reference`, `track_release_id` | ERN 4.x [track release](./parse-output.mdx#track-releases) |
| `track_release_genre`, `track_release_sub_genre` | ERN 4.x track release genre |
| `genre`, `sub_genre` | The resource's default genre (3.8.x) |
| `parental_warning` | The resource's default parental warning |
| `p_line`, `p_line_year`, `c_line`, `c_line_year` | The resource's P and C lines |
| `label_name` | The resource's label (3.8.x) |

## Missing values: pandas 2 and 3

The parser hands pandas `None` for every absent value (and `[]` for an empty list). How
pandas stores it depends on the column and on the pandas version:

| Column | pandas 2 | pandas 3 |
|---|---|---|
| Text column with some values (e.g. `title` on the flat message row) | `None` (object dtype) | `NaN` (the default `str` dtype) |
| Column with no values at all | `None` (object) | `None` (object) |
| Number column with gaps (e.g. `track_count` on the flat message row) | `NaN` (float64) | `NaN` (float64) |

So test for a missing value with `pd.isna(value)`, not `value is None`. Whole-number columns
with gaps become floats (`release_index` `0.0`); the builder accepts those back.

```python
import pandas as pd
from ddex_parser import DDEXParser

with open("release.xml", encoding="utf-8") as f:
    flat = DDEXParser().parse(f.read()).to_dataframe("flat")

message_row = flat.iloc[0]
assert message_row["type"] == "message"
assert pd.isna(message_row["title"])        # None in pandas 2, NaN in pandas 3
assert pd.isna(message_row["track_count"])  # NaN in both
print(flat[flat["type"] == "release"][["release_id", "title", "track_count"]])
```

## Build from a DataFrame

`DdexBuilder().from_dataframe(df, schema, request)` builds one message from DataFrame rows.
`request` is a build request without releases: it supplies `header`, `version` and the
other message-level values (`avs_version_id`, `language_and_script_code`, `deals`, ...).
The rows become the releases, and the build runs the same checks as
[`build`](./build-request.mdx#required-fields-by-ern-version).

With `schema="tracks"` each row is a track, and rows with the same `release_id` form one
release. The columns read are:

| Column | Build request field |
|---|---|
| `release_id` | **Required.** `release_id` (written as GRid unless empty) and the grouping key |
| `release_reference` | `release_reference` |
| `release_title`, `release_artist`, `release_label` | `title`, `artist`, `label` |
| `release_date` | `release_date` (an RFC 3339 date-time is cut to its date) |
| `release_upc` or `upc` | `upc` (written as ICPN) |
| `release_type` | `release_type` |
| `release_genre`, `release_sub_genre` | `genre`, `sub_genre` |
| `release_parental_warning_type` | `parental_warning_type` |
| `track_id` | `track_id` and `resource_reference` (else `A<track_index + 1>`) |
| `track_title`, `artist`, `isrc` | track `title`, `artist`, `isrc` |
| `duration` | `duration`: ISO 8601 (`PT3M42S`) or seconds |
| `resource_kind`, `resource_type` | `resource_kind`, `resource_type` |
| `parental_warning_type` | track `parental_warning_type` |
| `disc_number`, `track_number` | `disc_number`, `sequence_number` |

Missing values (`None`, `NaN`) are treated as absent, and whole-number floats are written
without `.0`. With `schema="releases"` or `"flat"` (release rows only) each row is a release
with the same columns **without** the `release_` prefix (`title`, `artist`, `label`, `genre`,
...) and no tracks. Without `schema`, a `type` column selects `"flat"` and a `track_index`
column `"tracks"`.

The parser's tracks DataFrame already uses the builder's track column names, so a parsed
message can be rebuilt by adding the release columns and the values the DataFrames do not
carry:

```python
from ddex_builder import DdexBuilder
from ddex_parser import DDEXParser

parser = DDEXParser()
with open("release.xml", encoding="utf-8") as f:
    message = parser.parse(f.read())
release = message.to_dataframe("releases").iloc[0]
tracks = message.to_dataframe("tracks")

frame = tracks.rename(columns={"parental_warning": "parental_warning_type"}).assign(
    release_id="",  # no GRid: the release is identified by its ICPN (release_upc)
    release_upc=release["icpn"],
    release_title=f"{release['title']} (Remastered)",
    release_artist=release["artist"],
    release_label=release["label_name"],
    release_genre=release["genre"],
    release_parental_warning_type=release["parental_warning"],
    # Not in the DataFrames: the release type and the resources' DDEX Type
    release_type="Album",
    resource_type="MusicalWorkSoundRecording",
)

header = {
    "message_sender": {"party_name": [{"text": "Example Label"}], "party_id": "PADPIDA0000000001"},
    "message_recipient": {"party_name": [{"text": "Example DSP"}], "party_id": "PADPIDA0000000002"},
}
xml = DdexBuilder().from_dataframe(
    frame,
    schema="tracks",
    request={"header": header, "version": "4.3", "avs_version_id": "12", "language_and_script_code": "en"},
)

rebuilt = parser.parse(xml).to_dataframe("tracks")
assert list(rebuilt["isrc"]) == list(tracks["isrc"])
assert list(rebuilt["duration"]) == list(tracks["duration"])
print(rebuilt[["track_index", "track_title", "isrc", "duration"]])
```

A failed build raises `RuntimeError` with the same message as `build`; a missing
`release_id` column raises `ValueError`.

## Other Python APIs

- `DDEXParser().stream(xml)` parses the whole document, then yields one dict per release
  (`release_id`, `title`, `artist`, `track_count`). It does not reduce memory use.
- `detect_version(xml)` returns `"V4_3"` (the enum spelling); `sanity_check(xml)` returns
  `{"is_valid", "version", "errors", "warnings"}`: it detects the ERN version (`None` when
  there is no ERN namespace) and checks that the XML is well-formed. It is not XSD
  validation, and `parse` can still fail on a document that passes.
- `ParseOptions` (`include_raw_extensions`, `include_comments`, `validate_references`,
  `streaming`, `timeout`) is accepted for forward compatibility; none of its fields
  currently changes the result.
- The builder's preset methods (`apply_preset`, ...) do not change the output in 0.5.x.
- `ddex_parser.__version__` and `ddex_builder.__version__` report the installed package
  version.
