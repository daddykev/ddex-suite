// packages/ddex-parser/bindings/python/src/lib.rs
use ddex_core::models::flat::ParsedERNMessage as CoreParsedERNMessage;
use ddex_parser::{parser::ParseOptions as CoreParseOptions, DDEXParser as CoreParser};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyModule};
use pyo3::Bound;
use pyo3_async_runtimes;
use pythonize::pythonize;
use std::io::Cursor;

/// Main DDEX Parser class for Python
#[pyclass(name = "DDEXParser", from_py_object)]
#[derive(Clone)]
pub struct PyDDEXParser {
    parser: CoreParser,
}

#[pymethods]
impl PyDDEXParser {
    #[new]
    pub fn new() -> Self {
        PyDDEXParser {
            parser: CoreParser::new(),
        }
    }

    /// Parse DDEX XML synchronously
    #[pyo3(signature = (xml, options=None))]
    pub fn parse(
        &mut self,
        py: Python,
        xml: &Bound<'_, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        // Convert input to string
        let xml_str = extract_xml_string(xml)?;

        // Parse options
        let parse_options = if let Some(opts) = options {
            rust_parse_options_from_dict(opts)?
        } else {
            CoreParseOptions::default()
        };

        // Create a cursor from the string
        let cursor = Cursor::new(xml_str.as_bytes());

        // Parse using the real parser
        let result = self
            .parser
            .parse_with_options(cursor, parse_options)
            .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;

        // Return PyParsedERNMessage wrapper
        let wrapped_result = PyParsedERNMessage::new(result);
        let py_obj = Py::new(py, wrapped_result)?;
        Ok(py_obj.into_any())
    }

    /// Parse DDEX XML asynchronously  
    #[pyo3(signature = (xml, options=None))]
    pub fn parse_async<'p>(
        &self,
        py: Python<'p>,
        xml: &Bound<'_, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let xml_str = extract_xml_string(xml)?;
        let parse_options = if let Some(opts) = options {
            rust_parse_options_from_dict(opts)?
        } else {
            CoreParseOptions::default()
        };

        let mut parser = self.parser.clone();

        // Create async future
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            // Run parsing in a blocking task to avoid blocking the async runtime
            let result = tokio::task::spawn_blocking(move || {
                let cursor = Cursor::new(xml_str.as_bytes());
                parser.parse_with_options(cursor, parse_options)
            })
            .await
            .map_err(|e| PyValueError::new_err(format!("Task join error: {}", e)))?
            .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;

            Python::attach(|py| -> PyResult<Py<PyAny>> {
                let wrapped_result = PyParsedERNMessage::new(result);
                let py_obj = Py::new(py, wrapped_result)?;
                Ok(py_obj.into_any())
            })
        })
    }

    /// Stream parse large files
    #[pyo3(signature = (source, options=None))]
    pub fn stream(
        &mut self,
        _py: Python,
        source: &Bound<'_, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<StreamIterator> {
        // Extract XML content
        let xml_str = extract_xml_string(source)?;

        // Parse options
        let parse_options = if let Some(opts) = options {
            rust_parse_options_from_dict(opts)?
        } else {
            CoreParseOptions::default()
        };

        // Parse the entire document first (for now - in a true streaming implementation,
        // this would parse incrementally)
        let cursor = Cursor::new(xml_str.as_bytes());
        let parsed_result = self
            .parser
            .parse_with_options(cursor, parse_options)
            .map_err(|e| PyValueError::new_err(format!("Stream parse error: {}", e)))?;

        // Create iterator with the parsed releases
        Ok(StreamIterator::from_parsed_result(parsed_result))
    }

    /// Convert DDEX XML to pandas DataFrame
    ///
    /// Args:
    ///     xml: DDEX XML content (string or bytes)
    ///     schema: Output schema format (default "flat")
    ///         - "flat": Mixed schema with message row and release rows
    ///         - "releases": One row per release with release details
    ///         - "tracks": One row per track with full track details
    ///
    /// Returns:
    ///     pandas.DataFrame with DDEX data
    #[pyo3(signature = (xml, schema="flat"))]
    pub fn to_dataframe(
        &mut self,
        py: Python,
        xml: &Bound<'_, PyAny>,
        schema: &str,
    ) -> PyResult<Py<PyAny>> {
        // Parse the XML first
        let xml_str = extract_xml_string(xml)?;
        let cursor = Cursor::new(xml_str.as_bytes());

        let parsed = self
            .parser
            .parse_with_options(cursor, CoreParseOptions::default())
            .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;

        // Same rows as ParsedERNMessage.to_dataframe (one implementation of the columns)
        PyParsedERNMessage::new(parsed).to_dataframe(py, schema)
    }

    /// Detect DDEX version
    pub fn detect_version(&self, xml: &Bound<'_, PyAny>) -> PyResult<String> {
        let xml_str = extract_xml_string(xml)?;
        let cursor = Cursor::new(xml_str.as_bytes());

        match self.parser.detect_version(cursor) {
            Ok(version) => Ok(format!("{:?}", version)),
            Err(e) => Err(PyValueError::new_err(format!(
                "Version detection error: {}",
                e
            ))),
        }
    }

    /// Quick check: ERN version detection and XML well-formedness (not XSD or AVS
    /// validation). Returns {"is_valid", "version", "errors"}; "version" is the detected
    /// version ("V4_3") or None when no ERN version is recognised.
    pub fn sanity_check(&self, py: Python, xml: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let xml_str = extract_xml_string(xml)?;
        let cursor = Cursor::new(xml_str.as_bytes());

        let result = self
            .parser
            .sanity_check(cursor)
            .map_err(|e| PyValueError::new_err(format!("Sanity check error: {}", e)))?;
        Ok(pythonize(py, &result)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?
            .unbind())
    }
}

/// ParsedERNMessage wrapper for Python
#[pyclass(name = "ParsedERNMessage", from_py_object)]
#[derive(Clone)]
pub struct PyParsedERNMessage {
    inner: CoreParsedERNMessage,
}

impl PyParsedERNMessage {
    pub fn new(inner: CoreParsedERNMessage) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyParsedERNMessage {
    /// Convert to pandas DataFrame
    #[pyo3(signature = (schema = "flat"))]
    fn to_dataframe(&self, py: Python, schema: &str) -> PyResult<Py<PyAny>> {
        // Try to import pandas
        let pandas = py.import("pandas").map_err(|_| {
            PyValueError::new_err(
                "pandas is required for to_dataframe(). Install with: pip install pandas",
            )
        })?;

        // Convert based on schema
        let data = match schema {
            "flat" => self.to_flat_dataframe_data(py)?,
            "releases" => self.to_releases_dataframe_data(py)?,
            "tracks" => self.to_tracks_dataframe_data(py)?,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown schema: {}. Use 'flat', 'releases', or 'tracks'",
                    schema
                )))
            }
        };

        // Create DataFrame
        let py_records = PyList::new(py, data)?;
        let df = pandas.call_method1("DataFrame", (py_records,))?;
        Ok(df.into())
    }

    /// Get message ID
    fn message_id(&self) -> String {
        self.inner.flat.message_id.clone()
    }

    /// Get version
    fn version(&self) -> String {
        self.inner.flat.version.clone()
    }

    /// Get number of releases
    fn release_count(&self) -> usize {
        self.inner.flat.releases.len()
    }

    /// `MessageHeader/MessageControlType` as written, or None.
    fn message_control_type(&self) -> Option<String> {
        self.inner.flat.message_control_type.clone()
    }

    /// The deals, one dict per deal: deal_id, releases (release references), start_date,
    /// end_date (RFC 3339, midnight UTC for a date), territories, excluded_territories,
    /// usage_rights (use types), distribution_channels. Absent values are None / [].
    fn deals(&self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .flat
            .deals
            .iter()
            .map(|deal| {
                let d = PyDict::new(py);
                d.set_item("deal_id", &deal.deal_id)?;
                d.set_item("releases", deal.releases.clone())?;
                d.set_item("start_date", deal.validity.start.map(|t| t.to_rfc3339()))?;
                d.set_item("end_date", deal.validity.end.map(|t| t.to_rfc3339()))?;
                d.set_item("territories", deal.territories.included.clone())?;
                d.set_item("excluded_territories", deal.territories.excluded.clone())?;
                d.set_item("usage_rights", deal.usage_rights.clone())?;
                d.set_item(
                    "distribution_channels",
                    deal.distribution_channels.included.clone(),
                )?;
                Ok(d.into_any().unbind())
            })
            .collect()
    }
}

impl PyParsedERNMessage {
    /// Flat schema: a message row, then one row per release. Every row has every column
    /// (None where it does not apply or the value is absent).
    fn to_flat_dataframe_data(&self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
        let flat = &self.inner.flat;
        let mut rows = Vec::new();

        let msg_row = PyDict::new(py);
        msg_row.set_item("message_id", &flat.message_id)?;
        // Sender PartyName is optional per the XSD: None (not "None") when absent.
        msg_row.set_item("sender", &flat.sender.name)?;
        msg_row.set_item("created_date", flat.message_date.to_rfc3339())?;
        msg_row.set_item("message_type", &flat.message_type)?;
        msg_row.set_item("message_control_type", &flat.message_control_type)?;
        msg_row.set_item("type", "message")?;
        for key in FLAT_RELEASE_COLUMNS {
            msg_row.set_item(*key, py.None())?;
        }
        rows.push(msg_row.into_any().into());

        for (idx, release) in flat.releases.iter().enumerate() {
            let row = PyDict::new(py);
            for key in [
                "message_id",
                "sender",
                "created_date",
                "message_type",
                "message_control_type",
            ] {
                row.set_item(key, py.None())?;
            }
            row.set_item("type", "release")?;
            row.set_item("release_index", idx)?;
            row.set_item("release_id", &release.release_id)?;
            row.set_item("title", &release.default_title)?;
            row.set_item("artist", &release.display_artist)?;
            row.set_item("p_line", release.p_line.as_ref().map(|c| c.text.clone()))?;
            row.set_item("c_line", release.c_line.as_ref().map(|c| c.text.clone()))?;
            row.set_item("genre", &release.genre)?;
            row.set_item("sub_genre", &release.sub_genre)?;
            row.set_item("parental_warning", &release.parental_warning)?;
            row.set_item("label_name", &release.label_name)?;
            row.set_item("release_date", release.release_date.map(|d| d.to_rfc3339()))?;
            row.set_item("track_count", release.track_count)?;
            rows.push(row.into_any().into());
        }
        Ok(rows)
    }

    /// Releases schema: one row per release with all release details.
    fn to_releases_dataframe_data(&self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
        let mut rows = Vec::new();
        for release in &self.inner.flat.releases {
            let row = PyDict::new(py);
            row.set_item("release_id", &release.release_id)?;
            row.set_item("title", &release.default_title)?;
            row.set_item("artist", &release.display_artist)?;
            row.set_item("p_line", release.p_line.as_ref().map(|c| c.text.clone()))?;
            row.set_item("genre", &release.genre)?;
            row.set_item("track_count", release.track_count)?;
            row.set_item("release_date", release.release_date.map(|d| d.to_rfc3339()))?;
            set_release_detail_items(py, &row, release)?;
            rows.push(row.into_any().into());
        }
        Ok(rows)
    }

    /// Tracks schema: one row per track with full details.
    fn to_tracks_dataframe_data(&self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
        let mut rows = Vec::new();
        for release in &self.inner.flat.releases {
            for (track_idx, track) in release.tracks.iter().enumerate() {
                let row = PyDict::new(py);
                row.set_item("release_id", &release.release_id)?;
                row.set_item("release_title", &release.default_title)?;
                row.set_item("track_index", track_idx)?;
                row.set_item("track_id", &track.track_id)?;
                row.set_item("track_title", &track.title)?;
                row.set_item("artist", &track.display_artist)?;
                row.set_item("duration", track.duration.map(|d| d.as_secs_f64()))?;
                row.set_item("isrc", &track.isrc)?;
                set_track_position_items(py, &row, track)?;
                set_track_detail_items(py, &row, track)?;
                rows.push(row.into_any().into());
            }
        }
        Ok(rows)
    }
}

/// Release columns of the flat schema (None on the message row).
const FLAT_RELEASE_COLUMNS: &[&str] = &[
    "release_index",
    "release_id",
    "title",
    "artist",
    "p_line",
    "c_line",
    "genre",
    "sub_genre",
    "parental_warning",
    "label_name",
    "release_date",
    "track_count",
];

fn set_scope_items(
    d: &Bound<'_, PyDict>,
    scope: &ddex_core::models::common::TerritoryScope,
) -> PyResult<()> {
    d.set_item("territory_codes", scope.territory_codes.clone())?;
    d.set_item(
        "excluded_territory_codes",
        scope.excluded_territory_codes.clone(),
    )?;
    d.set_item("is_default", scope.is_default)?;
    Ok(())
}

fn dates_list<'py>(
    py: Python<'py>,
    dates: &[ddex_core::models::flat::DateInfo],
) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for date in dates {
        let d = PyDict::new(py);
        d.set_item("date", &date.date)?;
        d.set_item("is_approximate", date.is_approximate)?;
        set_scope_items(&d, &date.scope)?;
        list.append(d)?;
    }
    Ok(list)
}

/// The 0.5.0 release columns (releases schema). Absent values are None / empty lists.
fn set_release_detail_items(
    py: Python,
    row: &Bound<'_, PyDict>,
    release: &ddex_core::models::flat::ParsedRelease,
) -> PyResult<()> {
    row.set_item("icpn", &release.identifiers.upc)?;
    row.set_item("isrc", &release.identifiers.isrc)?;
    row.set_item("grid", &release.identifiers.grid)?;
    let ids = PyList::empty(py);
    for p in &release.identifiers.proprietary {
        let d = PyDict::new(py);
        d.set_item("namespace", &p.namespace)?;
        d.set_item("value", &p.value)?;
        ids.append(d)?;
    }
    row.set_item("proprietary_ids", ids)?;
    row.set_item("label_name", &release.label_name)?;
    row.set_item("sub_genre", &release.sub_genre)?;
    let genres = PyList::empty(py);
    for g in &release.genres {
        let d = PyDict::new(py);
        d.set_item("genre", &g.genre)?;
        d.set_item("sub_genre", &g.sub_genre)?;
        set_scope_items(&d, &g.scope)?;
        genres.append(d)?;
    }
    row.set_item("genres", genres)?;
    row.set_item("parental_warning", &release.parental_warning)?;
    row.set_item("parental_warnings", release.parental_warnings.clone())?;
    row.set_item("p_line_year", release.p_line.as_ref().and_then(|c| c.year))?;
    row.set_item("c_line", release.c_line.as_ref().map(|c| c.text.clone()))?;
    row.set_item("c_line_year", release.c_line.as_ref().and_then(|c| c.year))?;
    row.set_item(
        "original_release_date",
        release.original_release_date.map(|d| d.to_rfc3339()),
    )?;
    row.set_item("release_dates", dates_list(py, &release.release_dates)?)?;
    row.set_item(
        "original_release_dates",
        dates_list(py, &release.original_release_dates)?,
    )?;
    Ok(())
}

/// The 0.5.0 track columns (tracks schema). Absent values are None / empty lists.
fn set_track_detail_items(
    py: Python,
    row: &Bound<'_, PyDict>,
    track: &ddex_core::models::flat::ParsedTrack,
) -> PyResult<()> {
    let artists = PyList::empty(py);
    for a in &track.artists {
        let d = PyDict::new(py);
        d.set_item("name", &a.name)?;
        d.set_item("role", &a.role)?;
        d.set_item("party_reference", &a.party_id)?;
        artists.append(d)?;
    }
    row.set_item("artists", artists)?;
    row.set_item("genre", &track.genre)?;
    row.set_item("sub_genre", &track.sub_genre)?;
    row.set_item("parental_warning", &track.parental_warning)?;
    row.set_item("p_line", track.p_line.as_ref().map(|c| c.text.clone()))?;
    row.set_item("p_line_year", track.p_line.as_ref().and_then(|c| c.year))?;
    row.set_item("c_line", track.c_line.as_ref().map(|c| c.text.clone()))?;
    row.set_item("c_line_year", track.c_line.as_ref().and_then(|c| c.year))?;
    row.set_item("label_name", &track.label_name)?;
    row.set_item("track_release_genre", &track.track_release_genre)?;
    row.set_item("track_release_sub_genre", &track.track_release_sub_genre)?;
    Ok(())
}

/// Stream iterator for large files
#[pyclass]
pub struct StreamIterator {
    position: usize,
    releases: Vec<StreamRelease>,
}

#[derive(Clone)]
struct StreamRelease {
    release_id: String,
    title: Option<String>,
    artist: Option<String>,
    track_count: u32,
}

impl StreamIterator {
    fn from_parsed_result(parsed_result: ddex_core::models::flat::ParsedERNMessage) -> Self {
        let releases = parsed_result
            .flat
            .releases
            .iter()
            .map(|release| StreamRelease {
                release_id: release.release_id.clone(),
                title: release.default_title.clone(),
                artist: release.display_artist.clone(),
                track_count: release.track_count as u32,
            })
            .collect();

        StreamIterator {
            position: 0,
            releases,
        }
    }
}

#[pymethods]
impl StreamIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python) -> Option<Py<PyAny>> {
        if slf.position >= slf.releases.len() {
            return None;
        }

        let release = slf.releases[slf.position].clone();
        slf.position += 1;

        let dict = PyDict::new(py);
        dict.set_item("release_id", &release.release_id).ok()?;
        dict.set_item("title", &release.title).ok()?;
        dict.set_item("artist", &release.artist).ok()?;
        dict.set_item("track_count", release.track_count).ok()?;

        Some(dict.into_any().into())
    }
}

// Helper types and functions

/// Track layout and TrackRelease columns for the tracks DataFrame. Absent values are None.
fn set_track_position_items(
    py: Python,
    row: &Bound<'_, PyDict>,
    track: &ddex_core::models::flat::ParsedTrack,
) -> PyResult<()> {
    row.set_item("position", track.position)?;
    row.set_item("disc_number", track.disc_number)?;
    row.set_item("track_number", track.track_number)?;
    row.set_item("track_release_reference", &track.track_release_reference)?;
    row.set_item("track_release_id", &track.track_release_id)?;
    let path = PyList::empty(py);
    for group in &track.group_path {
        let entry = PyDict::new(py);
        entry.set_item("group_type", &group.group_type)?;
        entry.set_item("sequence_number", group.sequence_number)?;
        path.append(entry)?;
    }
    row.set_item("group_path", path)?;
    Ok(())
}

fn rust_parse_options_from_dict(dict: &Bound<'_, PyDict>) -> PyResult<CoreParseOptions> {
    let mut options = CoreParseOptions::default();

    // Core options matching the actual ParseOptions struct
    if let Some(v) = dict.get_item("resolve_references")? {
        options.resolve_references = v.extract()?;
    }
    if let Some(v) = dict.get_item("max_memory")? {
        options.max_memory = v.extract()?;
    }
    if let Some(v) = dict.get_item("timeout_ms")? {
        options.timeout_ms = v.extract()?;
    }
    if let Some(v) = dict.get_item("chunk_size")? {
        options.chunk_size = v.extract()?;
    }

    Ok(options)
}

fn extract_xml_string(xml: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(s) = xml.extract::<String>() {
        Ok(s)
    } else if let Ok(bytes) = xml.extract::<Bound<'_, PyBytes>>() {
        String::from_utf8(bytes.as_bytes().to_vec())
            .map_err(|e| PyValueError::new_err(format!("Invalid UTF-8: {}", e)))
    } else {
        Err(PyValueError::new_err("xml must be str or bytes"))
    }
}

/// Python module initialization
// gil_used = true keeps the pyo3 0.24 behaviour (0.28 changed the default to declaring
// free-threaded support, which these bindings have not been tested for).
#[pymodule(gil_used = true)]
fn _internal(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Changed from ddex_parser to _internal
    m.add_class::<PyDDEXParser>()?;
    m.add_class::<PyParsedERNMessage>()?;
    m.add_class::<StreamIterator>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
