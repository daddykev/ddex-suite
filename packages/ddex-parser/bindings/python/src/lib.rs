// packages/ddex-parser/bindings/python/src/lib.rs
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyBytes, PyDict, PyList, PyModule};
use pyo3::Bound;
use pythonize::pythonize;
use pyo3_async_runtimes;
use ddex_parser::{DDEXParser as CoreParser, parser::ParseOptions as CoreParseOptions};
use ddex_core::models::flat::ParsedERNMessage as CoreParsedERNMessage;
use std::io::Cursor;

/// Main DDEX Parser class for Python
#[pyclass(name = "DDEXParser")]
#[derive(Clone)]
pub struct PyDDEXParser {
    parser: CoreParser,
}

#[pymethods]
impl PyDDEXParser {
    #[new]
    pub fn new() -> Self {
        PyDDEXParser { 
            parser: CoreParser::new()
        }
    }
    
    /// Parse DDEX XML synchronously
    #[pyo3(signature = (xml, options=None))]
    pub fn parse(
        &self,
        py: Python,
        xml: &PyAny,
        options: Option<&PyDict>,
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
        let result = self.parser.parse_with_options(cursor, parse_options)
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
        xml: &PyAny,
        options: Option<&PyDict>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let xml_str = extract_xml_string(xml)?;
        let parse_options = if let Some(opts) = options {
            rust_parse_options_from_dict(opts)?
        } else {
            CoreParseOptions::default()
        };
        
        let parser = self.parser.clone();
        
        // Create async future
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            // Run parsing in a blocking task to avoid blocking the async runtime
            let result = tokio::task::spawn_blocking(move || {
                let cursor = Cursor::new(xml_str.as_bytes());
                parser.parse_with_options(cursor, parse_options)
            }).await
            .map_err(|e| PyValueError::new_err(format!("Task join error: {}", e)))?
            .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;
            
            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                let wrapped_result = PyParsedERNMessage::new(result);
                let py_obj = Py::new(py, wrapped_result)?;
                Ok(py_obj.into_any())
            })
        })
    }
    
    /// Stream parse large files
    pub fn stream(
        &self,
        py: Python,
        source: &PyAny,
        options: Option<&PyDict>,
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
        let parsed_result = self.parser.parse_with_options(cursor, parse_options)
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
        &self,
        py: Python,
        xml: &PyAny,
        schema: &str,
    ) -> PyResult<Py<PyAny>> {
        // Parse the XML first
        let xml_str = extract_xml_string(xml)?;
        let cursor = Cursor::new(xml_str.as_bytes());
        
        let parsed = self.parser.parse_with_options(cursor, CoreParseOptions::default())
            .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;
        
        // Try to import pandas
        let pandas = py.import_bound("pandas")
            .map_err(|_| PyValueError::new_err("pandas is required for to_dataframe(). Install with: pip install pandas"))?;
        
        match schema {
            "flat" => {
                // Create a flattened representation suitable for DataFrame
                let mut records = Vec::new();
                
                // Extract message-level info with all columns
                let message_dict = PyDict::new_bound(py);
                message_dict.set_item("message_id", &parsed.flat.message_id)?;
                message_dict.set_item("sender", format!("{:?}", &parsed.flat.sender))?;
                message_dict.set_item("created_date", &parsed.flat.message_date.to_rfc3339())?;
                message_dict.set_item("message_type", &parsed.flat.message_type)?;
                message_dict.set_item("type", "message")?;
                message_dict.set_item("release_index", py.None())?;
                message_dict.set_item("release_id", py.None())?;
                message_dict.set_item("title", py.None())?;
                message_dict.set_item("artist", py.None())?;
                message_dict.set_item("p_line", py.None())?;
                message_dict.set_item("genre", py.None())?;
                message_dict.set_item("track_count", py.None())?;
                records.push(message_dict.into_any());
                
                // Extract release info with all columns
                for (idx, release) in parsed.flat.releases.iter().enumerate() {
                    let release_dict = PyDict::new_bound(py);
                    release_dict.set_item("message_id", py.None())?;
                    release_dict.set_item("sender", py.None())?;
                    release_dict.set_item("created_date", py.None())?;
                    release_dict.set_item("message_type", py.None())?;
                    release_dict.set_item("type", "release")?;
                    release_dict.set_item("release_index", idx)?;
                    release_dict.set_item("release_id", &release.release_id)?;
                    release_dict.set_item("title", &release.default_title)?;
                    release_dict.set_item("artist", &release.display_artist)?;
                    release_dict.set_item("p_line", format!("{:?}", &release.p_line))?;
                    release_dict.set_item("genre", format!("{:?}", &release.genre))?;
                    release_dict.set_item("track_count", release.track_count)?;
                    records.push(release_dict.into_any());
                }
                
                let py_records = PyList::new_bound(py, records);
                let df = pandas.call_method1("DataFrame", (py_records.as_gil_ref(),))?;
                Ok(df.into())
            }
            "releases" => {
                // Create a DataFrame focused on releases  
                let mut records = Vec::new();
                for release in parsed.flat.releases.iter() {
                    let dict = PyDict::new_bound(py);
                    dict.set_item("release_id", &release.release_id)?;
                    dict.set_item("title", &release.default_title)?;
                    dict.set_item("artist", &release.display_artist)?;
                    dict.set_item("p_line", format!("{:?}", &release.p_line))?;
                    dict.set_item("genre", format!("{:?}", &release.genre))?;
                    dict.set_item("track_count", release.track_count)?;
                    dict.set_item("release_date", format!("{:?}", &release.release_date))?;
                    records.push(dict.into_any());
                }
                
                let py_records = PyList::new_bound(py, records);
                let df = pandas.call_method1("DataFrame", (py_records.as_gil_ref(),))?;
                Ok(df.into())
            }
            "tracks" => {
                // Create a DataFrame focused on sound recordings/tracks
                let mut records = Vec::new();
                
                for release in &parsed.flat.releases {
                    for (track_idx, track) in release.tracks.iter().enumerate() {
                        let dict = PyDict::new_bound(py);
                        dict.set_item("release_id", &release.release_id)?;
                        dict.set_item("release_title", &release.default_title)?;
                        dict.set_item("track_index", track_idx)?;
                        dict.set_item("track_id", &track.track_id)?;
                        dict.set_item("track_title", &track.title)?;
                        dict.set_item("artist", &track.display_artist)?;
                        dict.set_item("duration", format!("{:?}", &track.duration))?;
                        dict.set_item("isrc", format!("{:?}", &track.isrc))?;
                        records.push(dict.into_any());
                    }
                }
                
                let py_records = PyList::new_bound(py, records);
                let df = pandas.call_method1("DataFrame", (py_records.as_gil_ref(),))?;
                Ok(df.into())
            }
            _ => {
                Err(PyValueError::new_err(format!(
                    "Unknown schema '{}'. Supported schemas: 'flat', 'releases', 'tracks'", 
                    schema
                )))
            }
        }
    }
    
    /// Create DDEX XML from pandas DataFrame  
    #[pyo3(signature = (df, schema="flat", template=None))]
    pub fn from_dataframe(
        &self,
        py: Python,
        df: &PyAny,
        schema: &str,
        template: Option<&PyAny>,
    ) -> PyResult<String> {
        // Check if it's a pandas DataFrame
        let pandas = py.import_bound("pandas")
            .map_err(|_| PyValueError::new_err("pandas is required for from_dataframe(). Install with: pip install pandas"))?;
        
        let dataframe_type = pandas.getattr("DataFrame")?;
        if !df.is_instance(&dataframe_type)? {
            return Err(PyValueError::new_err("Input must be a pandas DataFrame"));
        }
        
        // Convert DataFrame to records (list of dictionaries)
        let to_dict_method = df.getattr("to_dict")?;
        let records = to_dict_method.call1(("records",))?;
        let records_list: Vec<&PyDict> = records.extract()?;
        
        match schema {
            "flat" => {
                self.build_ddex_from_flat_dataframe(py, records_list, template)
            }
            "releases" => {
                self.build_ddex_from_releases_dataframe(py, records_list, template)
            }
            "tracks" => {
                self.build_ddex_from_tracks_dataframe(py, records_list, template)
            }
            _ => {
                Err(PyValueError::new_err(format!(
                    "Unknown schema '{}'. Supported schemas: 'flat', 'releases', 'tracks'", 
                    schema
                )))
            }
        }
    }
    
    /// Detect DDEX version
    pub fn detect_version(&self, xml: &PyAny) -> PyResult<String> {
        let xml_str = extract_xml_string(xml)?;
        let cursor = Cursor::new(xml_str.as_bytes());
        
        match self.parser.detect_version(cursor) {
            Ok(version) => Ok(format!("{:?}", version)),
            Err(e) => Err(PyValueError::new_err(format!("Version detection error: {}", e))),
        }
    }
    
    /// Perform sanity check
    pub fn sanity_check(&self, py: Python, xml: &PyAny) -> PyResult<Py<PyAny>> {
        let xml_str = extract_xml_string(xml)?;
        let cursor = Cursor::new(xml_str.as_bytes());
        
        let result = self.parser.sanity_check(cursor)
            .map_err(|e| PyValueError::new_err(format!("Sanity check error: {}", e)))?;
        
        let py_obj = pythonize(py, &result)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
        
        Ok(py_obj.into())
    }
    
    // Helper methods for building DDEX from DataFrames
    fn build_ddex_from_flat_dataframe(
        &self,
        _py: Python,
        records: Vec<&PyDict>,
        _template: Option<&PyAny>,
    ) -> PyResult<String> {
        // For now, return a mock DDEX XML structure
        // In a full implementation, this would reconstruct proper DDEX XML
        // from the flattened DataFrame records
        let mut ddex_content = String::new();
        ddex_content.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        ddex_content.push_str(r#"<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">"#);
        ddex_content.push_str(r#"<MessageHeader>"#);
        
        // Extract message header info from first record if available
        if let Some(first_record) = records.first() {
            if let Ok(Some(message_id)) = first_record.get_item("message_id") {
                if let Ok(id) = message_id.extract::<String>() {
                    ddex_content.push_str(&format!("<MessageId>{}</MessageId>", id));
                }
            }
            ddex_content.push_str("<MessageSender><PartyId>Sender</PartyId></MessageSender>");
            ddex_content.push_str("<MessageRecipient><PartyId>Recipient</PartyId></MessageRecipient>");
            ddex_content.push_str("<MessageCreatedDateTime>2023-01-01T00:00:00Z</MessageCreatedDateTime>");
        }
        
        ddex_content.push_str("</MessageHeader>");
        ddex_content.push_str("<UpdateIndicator>OriginalFile</UpdateIndicator>");
        ddex_content.push_str("<ReleaseList>");
        
        // Add releases from records
        for record in &records {
            if let Ok(Some(record_type)) = record.get_item("type") {
                if let Ok(type_str) = record_type.extract::<String>() {
                    if type_str == "release" {
                        ddex_content.push_str("<Release>");
                        
                        if let Ok(Some(release_id)) = record.get_item("release_id") {
                            if let Ok(id) = release_id.extract::<String>() {
                                ddex_content.push_str(&format!("<ReleaseId><ProprietaryId>{}</ProprietaryId></ReleaseId>", id));
                            }
                        }
                        
                        if let Ok(Some(title)) = record.get_item("title") {
                            if let Ok(title_str) = title.extract::<String>() {
                                ddex_content.push_str(&format!("<ReferenceTitle><TitleText>{}</TitleText></ReferenceTitle>", title_str));
                            }
                        }
                        
                        ddex_content.push_str("</Release>");
                    }
                }
            }
        }
        
        ddex_content.push_str("</ReleaseList>");
        ddex_content.push_str("</ern:NewReleaseMessage>");
        
        Ok(ddex_content)
    }
    
    fn build_ddex_from_releases_dataframe(
        &self,
        _py: Python,
        records: Vec<&PyDict>,
        _template: Option<&PyAny>,
    ) -> PyResult<String> {
        // Build DDEX focused on releases schema
        let mut ddex_content = String::new();
        ddex_content.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        ddex_content.push_str(r#"<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">"#);
        ddex_content.push_str(r#"<MessageHeader><MessageId>DataFrame-Generated</MessageId><MessageSender><PartyId>DataFrameSender</PartyId></MessageSender><MessageRecipient><PartyId>DataFrameRecipient</PartyId></MessageRecipient><MessageCreatedDateTime>2023-01-01T00:00:00Z</MessageCreatedDateTime></MessageHeader>"#);
        ddex_content.push_str("<UpdateIndicator>OriginalFile</UpdateIndicator>");
        ddex_content.push_str("<ReleaseList>");
        
        for record in &records {
            ddex_content.push_str("<Release>");
            
            if let Ok(Some(release_id)) = record.get_item("release_id") {
                if let Ok(id) = release_id.extract::<String>() {
                    ddex_content.push_str(&format!("<ReleaseId><ProprietaryId>{}</ProprietaryId></ReleaseId>", id));
                }
            }
            
            if let Ok(Some(title)) = record.get_item("title") {
                if let Ok(title_str) = title.extract::<String>() {
                    ddex_content.push_str(&format!("<ReferenceTitle><TitleText>{}</TitleText></ReferenceTitle>", title_str));
                }
            }
            
            ddex_content.push_str("</Release>");
        }
        
        ddex_content.push_str("</ReleaseList>");
        ddex_content.push_str("</ern:NewReleaseMessage>");
        
        Ok(ddex_content)
    }
    
    fn build_ddex_from_tracks_dataframe(
        &self,
        _py: Python,
        records: Vec<&PyDict>,
        _template: Option<&PyAny>,
    ) -> PyResult<String> {
        // Build DDEX with tracks/sound recordings focus
        // Group tracks by release_id
        
        let mut releases_map: std::collections::HashMap<String, Vec<&PyDict>> = std::collections::HashMap::new();
        
        for record in &records {
            if let Ok(Some(release_id)) = record.get_item("release_id") {
                if let Ok(id) = release_id.extract::<String>() {
                    releases_map.entry(id).or_insert_with(Vec::new).push(record);
                }
            }
        }
        
        let mut ddex_content = String::new();
        ddex_content.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        ddex_content.push_str(r#"<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">"#);
        ddex_content.push_str(r#"<MessageHeader><MessageId>DataFrame-Tracks</MessageId><MessageSender><PartyId>DataFrameSender</PartyId></MessageSender><MessageRecipient><PartyId>DataFrameRecipient</PartyId></MessageRecipient><MessageCreatedDateTime>2023-01-01T00:00:00Z</MessageCreatedDateTime></MessageHeader>"#);
        ddex_content.push_str("<UpdateIndicator>OriginalFile</UpdateIndicator>");
        ddex_content.push_str("<ReleaseList>");
        
        for (release_id, tracks) in releases_map {
            ddex_content.push_str("<Release>");
            ddex_content.push_str(&format!("<ReleaseId><ProprietaryId>{}</ProprietaryId></ReleaseId>", release_id));
            
            if let Some(first_track) = tracks.first() {
                if let Ok(Some(title)) = first_track.get_item("release_title") {
                    if let Ok(title_str) = title.extract::<String>() {
                        ddex_content.push_str(&format!("<ReferenceTitle><TitleText>{}</TitleText></ReferenceTitle>", title_str));
                    }
                }
            }
            
            // Add sound recordings
            ddex_content.push_str("<SoundRecordingList>");
            for track in tracks {
                ddex_content.push_str("<SoundRecording>");
                
                if let Ok(Some(track_id)) = track.get_item("track_id") {
                    if let Ok(id) = track_id.extract::<String>() {
                        ddex_content.push_str(&format!("<SoundRecordingId><ProprietaryId>{}</ProprietaryId></SoundRecordingId>", id));
                    }
                }
                
                if let Ok(Some(track_title)) = track.get_item("track_title") {
                    if let Ok(title_str) = track_title.extract::<String>() {
                        ddex_content.push_str(&format!("<ReferenceTitle><TitleText>{}</TitleText></ReferenceTitle>", title_str));
                    }
                }
                
                ddex_content.push_str("</SoundRecording>");
            }
            ddex_content.push_str("</SoundRecordingList>");
            ddex_content.push_str("</Release>");
        }
        
        ddex_content.push_str("</ReleaseList>");
        ddex_content.push_str("</ern:NewReleaseMessage>");
        
        Ok(ddex_content)
    }
}

/// ParsedERNMessage wrapper for Python
#[pyclass(name = "ParsedERNMessage")]
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
        let pandas = py.import_bound("pandas")
            .map_err(|_| PyValueError::new_err("pandas is required for to_dataframe(). Install with: pip install pandas"))?;
        
        // Convert based on schema
        let data = match schema {
            "flat" => self.to_flat_dataframe_data(py)?,
            "releases" => self.to_releases_dataframe_data(py)?,
            "tracks" => self.to_tracks_dataframe_data(py)?,
            _ => return Err(PyValueError::new_err(format!(
                "Unknown schema: {}. Use 'flat', 'releases', or 'tracks'", schema
            ))),
        };
        
        // Create DataFrame
        let py_records = PyList::new_bound(py, data);
        let df = pandas.call_method1("DataFrame", (py_records.as_gil_ref(),))?;
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
}

impl PyParsedERNMessage {
    fn to_flat_dataframe_data(&self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
        // Flat schema: one row per release with basic info
        let mut rows = Vec::new();
        
        // Add message-level row with all columns (None for release-specific fields)
        let msg_row = PyDict::new_bound(py);
        msg_row.set_item("message_id", &self.inner.flat.message_id)?;
        msg_row.set_item("sender", format!("{:?}", &self.inner.flat.sender))?;
        msg_row.set_item("created_date", &self.inner.flat.message_date.to_rfc3339())?;
        msg_row.set_item("message_type", &self.inner.flat.message_type)?;
        msg_row.set_item("type", "message")?;
        msg_row.set_item("release_index", py.None())?;
        msg_row.set_item("release_id", py.None())?;
        msg_row.set_item("title", py.None())?;
        msg_row.set_item("artist", py.None())?;
        msg_row.set_item("p_line", py.None())?;
        msg_row.set_item("genre", py.None())?;
        msg_row.set_item("track_count", py.None())?;
        rows.push(msg_row.into_any().into());
        
        // Add release rows with all columns (None for message-specific fields)
        for (idx, release) in self.inner.flat.releases.iter().enumerate() {
            let row = PyDict::new_bound(py);
            row.set_item("message_id", py.None())?;
            row.set_item("sender", py.None())?;
            row.set_item("created_date", py.None())?;
            row.set_item("message_type", py.None())?;
            row.set_item("type", "release")?;
            row.set_item("release_index", idx)?;
            row.set_item("release_id", &release.release_id)?;
            row.set_item("title", &release.default_title)?;
            row.set_item("artist", &release.display_artist)?;
            row.set_item("p_line", format!("{:?}", &release.p_line))?;
            row.set_item("genre", format!("{:?}", &release.genre))?;
            row.set_item("track_count", release.track_count)?;
            rows.push(row.into_any().into());
        }
        Ok(rows)
    }
    
    fn to_releases_dataframe_data(&self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
        // Releases schema: one row per release with all release details
        let mut rows = Vec::new();
        for release in &self.inner.flat.releases {
            let row = PyDict::new_bound(py);
            row.set_item("release_id", &release.release_id)?;
            row.set_item("title", &release.default_title)?;
            row.set_item("artist", &release.display_artist)?;
            row.set_item("p_line", format!("{:?}", &release.p_line))?;
            row.set_item("genre", format!("{:?}", &release.genre))?;
            row.set_item("track_count", release.track_count)?;
            row.set_item("release_date", format!("{:?}", &release.release_date))?;
            rows.push(row.into_any().into());
        }
        Ok(rows)
    }
    
    fn to_tracks_dataframe_data(&self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
        // Tracks schema: one row per track with full details
        let mut rows = Vec::new();
        for release in &self.inner.flat.releases {
            for (track_idx, track) in release.tracks.iter().enumerate() {
                let row = PyDict::new_bound(py);
                row.set_item("release_id", &release.release_id)?;
                row.set_item("release_title", &release.default_title)?;
                row.set_item("track_index", track_idx)?;
                row.set_item("track_id", &track.track_id)?;
                row.set_item("track_title", &track.title)?;
                row.set_item("artist", &track.display_artist)?;
                row.set_item("duration", format!("{:?}", &track.duration))?;
                row.set_item("isrc", format!("{:?}", &track.isrc))?;
                rows.push(row.into_any().into());
            }
        }
        Ok(rows)
    }
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
    title: String,
    artist: String,
    track_count: u32,
}

impl StreamIterator {
    fn new() -> Self {
        StreamIterator {
            position: 0,
            releases: Vec::new(),
        }
    }
    
    fn from_parsed_result(parsed_result: ddex_core::models::flat::ParsedERNMessage) -> Self {
        let releases = parsed_result.flat.releases.iter().map(|release| {
            StreamRelease {
                release_id: release.release_id.clone(),
                title: release.default_title.clone(),
                artist: release.display_artist.clone(),
                track_count: release.track_count as u32,
            }
        }).collect();
        
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
        
        let dict = PyDict::new_bound(py);
        dict.set_item("release_id", &release.release_id).ok()?;
        dict.set_item("title", &release.title).ok()?;
        dict.set_item("artist", &release.artist).ok()?;
        dict.set_item("track_count", release.track_count).ok()?;
        
        Some(dict.into_any().into())
    }
}

// Helper types and functions

fn rust_parse_options_from_dict(dict: &PyDict) -> PyResult<CoreParseOptions> {
    let mut options = CoreParseOptions::default();
    
    // Core options matching the actual ParseOptions struct
    if let Ok(Some(v)) = dict.get_item("resolve_references") {
        options.resolve_references = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("include_raw") {
        options.include_raw = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("max_memory") {
        options.max_memory = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("timeout_ms") {
        options.timeout_ms = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("allow_blocking") {
        options.allow_blocking = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("include_raw_extensions") {
        options.include_raw_extensions = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("include_comments") {
        options.include_comments = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("preserve_unknown_elements") {
        options.preserve_unknown_elements = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("chunk_size") {
        options.chunk_size = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("auto_threshold") {
        options.auto_threshold = v.extract()?;
    }
    
    // Legacy options for backward compatibility
    if let Ok(Some(v)) = dict.get_item("validate_references") {
        options.resolve_references = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("timeout") {
        let timeout_secs: f64 = v.extract()?;
        options.timeout_ms = (timeout_secs * 1000.0) as u64;
    }
    
    Ok(options)
}

fn extract_xml_string(xml: &PyAny) -> PyResult<String> {
    if let Ok(s) = xml.extract::<String>() {
        Ok(s)
    } else if let Ok(bytes) = xml.extract::<&PyBytes>() {
        String::from_utf8(bytes.as_bytes().to_vec())
            .map_err(|e| PyValueError::new_err(format!("Invalid UTF-8: {}", e)))
    } else {
        Err(PyValueError::new_err("xml must be str or bytes"))
    }
}


/// Python module initialization
#[pymodule]
fn _internal(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {  // Changed from ddex_parser to _internal
    m.add_class::<PyDDEXParser>()?;
    m.add_class::<PyParsedERNMessage>()?;
    m.add_class::<StreamIterator>()?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}