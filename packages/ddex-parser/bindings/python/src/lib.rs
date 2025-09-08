// packages/ddex-parser/bindings/python/src/lib.rs
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyBytes, PyDict, PyList};
use pythonize::pythonize;

/// Main DDEX Parser class for Python
#[pyclass(name = "DDEXParser")]
#[derive(Clone)]
pub struct PyDDEXParser {
    _private: (),
}

#[pymethods]
impl PyDDEXParser {
    #[new]
    pub fn new() -> Self {
        PyDDEXParser { _private: () }
    }
    
    /// Parse DDEX XML synchronously
    #[pyo3(signature = (xml, options=None))]
    pub fn parse(
        &self,
        py: Python,
        xml: &PyAny,
        options: Option<&PyDict>,
    ) -> PyResult<Py<PyDict>> {
        // Convert input to string
        let xml_str = if let Ok(s) = xml.extract::<String>() {
            s
        } else if let Ok(bytes) = xml.extract::<&PyBytes>() {
            String::from_utf8(bytes.as_bytes().to_vec())
                .map_err(|e| PyValueError::new_err(format!("Invalid UTF-8: {}", e)))?
        } else {
            return Err(PyValueError::new_err("xml must be str or bytes"));
        };
        
        // Parse options (for future use)
        let _parse_options = if let Some(opts) = options {
            parse_dict_options(opts)?
        } else {
            ParseOptions::default()
        };
        
        // Mock implementation - return structured data
        let result = mock_parse_result(&xml_str);
        
        // Convert to Python dict
        let dict = PyDict::new(py);
        dict.set_item("message_id", result.message_id)?;
        dict.set_item("version", result.version)?;
        dict.set_item("release_count", result.release_count)?;
        dict.set_item("releases", pythonize(py, &result.releases).unwrap())?;
        
        Ok(dict.into())
    }
    
    /// Parse DDEX XML asynchronously
    #[pyo3(signature = (xml, options=None))]
    pub fn parse_async<'p>(
        &self,
        py: Python<'p>,
        xml: &PyAny,
        options: Option<&PyDict>,
    ) -> PyResult<&'p PyAny> {
        let xml_str = extract_xml_string(xml)?;
        let _parse_options = if let Some(opts) = options {
            parse_dict_options(opts)?
        } else {
            ParseOptions::default()
        };
        
        // Create async future
        pyo3_asyncio::tokio::future_into_py(py, async move {
            // Simulate async parsing
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            let result = mock_parse_result(&xml_str);
            
            Python::with_gil(|py| -> PyResult<Py<PyDict>> {
                let dict = PyDict::new(py);
                dict.set_item("message_id", result.message_id)?;
                dict.set_item("version", result.version)?;
                dict.set_item("release_count", result.release_count)?;
                dict.set_item("releases", pythonize(py, &result.releases).unwrap())?;
                Ok(dict.into())
            })
        })
    }
    
    /// Stream parse large files
    pub fn stream(
        &self,
        _py: Python,
        _source: &PyAny,
        _options: Option<&PyDict>,
    ) -> PyResult<StreamIterator> {
        // Return a Python iterator
        Ok(StreamIterator::new())
    }
    
    /// Convert to pandas DataFrame
    #[pyo3(signature = (xml, schema="flat"))]
    pub fn to_dataframe(
        &self,
        py: Python,
        xml: &PyAny,
        schema: &str,
    ) -> PyResult<Py<PyAny>> {
        let xml_str = extract_xml_string(xml)?;
        let _schema_str = schema; // Use it to avoid warning
        let result = mock_parse_result(&xml_str);
        
        // Try to import pandas
        match py.import("pandas") {
            Ok(pandas) => {
                // Convert releases to records
                let records = PyList::new(py, 
                    result.releases.iter().map(|r| {
                        let dict = PyDict::new(py);
                        dict.set_item("release_id", &r.release_id).unwrap();
                        dict.set_item("title", &r.title).unwrap();
                        dict.set_item("artist", &r.artist).unwrap();
                        dict.set_item("track_count", r.track_count).unwrap();
                        dict
                    })
                );
                
                let df = pandas.call_method1("DataFrame", (records,))?;
                Ok(df.into())
            }
            Err(_) => {
                // Pandas not available, return a dict instead
                let dict = PyDict::new(py);
                dict.set_item("error", "pandas not installed")?;
                dict.set_item("data", pythonize(py, &result.releases).unwrap())?;
                Ok(dict.into())
            }
        }
    }
    
    /// Detect DDEX version
    pub fn detect_version(&self, xml: &PyAny) -> PyResult<String> {
        let xml_str = extract_xml_string(xml)?;
        
        if xml_str.contains("ern/43") || xml_str.contains("xml/ern/43") {
            Ok("4.3".to_string())
        } else if xml_str.contains("ern/42") || xml_str.contains("xml/ern/42") {
            Ok("4.2".to_string())
        } else if xml_str.contains("ern/382") || xml_str.contains("xml/ern/382") {
            Ok("3.8.2".to_string())
        } else {
            Ok("Unknown".to_string())
        }
    }
    
    /// Perform sanity check
    pub fn sanity_check(&self, py: Python, xml: &PyAny) -> PyResult<Py<PyDict>> {
        let xml_str = extract_xml_string(xml)?;
        
        let dict = PyDict::new(py);
        dict.set_item("is_valid", !xml_str.is_empty())?;
        dict.set_item("version", self.detect_version(xml)?)?;
        dict.set_item("errors", PyList::empty(py))?;
        dict.set_item("warnings", PyList::empty(py))?;
        
        Ok(dict.into())
    }
}

/// Stream iterator for large files
#[pyclass]
pub struct StreamIterator {
    position: usize,
    max_items: usize,
}

impl StreamIterator {
    fn new() -> Self {
        StreamIterator {
            position: 0,
            max_items: 3,
        }
    }
}

#[pymethods]
impl StreamIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    
    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python) -> Option<Py<PyDict>> {
        if slf.position >= slf.max_items {
            return None;
        }
        
        slf.position += 1;
        
        let dict = PyDict::new(py);
        dict.set_item("release_id", format!("R{:03}", slf.position)).ok()?;
        dict.set_item("title", format!("Release {}", slf.position)).ok()?;
        dict.set_item("artist", "Test Artist").ok()?;
        dict.set_item("track_count", 10).ok()?;
        
        Some(dict.into())
    }
}

// Helper types and functions
#[derive(Default)]
struct ParseOptions {
    include_raw_extensions: bool,
    include_comments: bool,
    validate_references: bool,
    streaming: bool,
}

fn parse_dict_options(dict: &PyDict) -> PyResult<ParseOptions> {
    let mut options = ParseOptions::default();
    
    if let Ok(Some(v)) = dict.get_item("include_raw_extensions") {
        options.include_raw_extensions = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("include_comments") {
        options.include_comments = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("validate_references") {
        options.validate_references = v.extract()?;
    }
    if let Ok(Some(v)) = dict.get_item("streaming") {
        options.streaming = v.extract()?;
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

// Mock implementation for testing
#[derive(serde::Serialize)]
struct MockParseResult {
    message_id: String,
    version: String,
    release_count: usize,
    releases: Vec<MockRelease>,
}

#[derive(serde::Serialize)]
struct MockRelease {
    release_id: String,
    title: String,
    artist: String,
    track_count: usize,
}

fn mock_parse_result(xml: &str) -> MockParseResult {
    let version = if xml.contains("ern/43") { 
        "4.3" 
    } else if xml.contains("ern/42") { 
        "4.2" 
    } else { 
        "3.8.2" 
    };
    
    MockParseResult {
        message_id: "MSG001".to_string(),
        version: version.to_string(),
        release_count: 1,
        releases: vec![
            MockRelease {
                release_id: "REL001".to_string(),
                title: "Test Album".to_string(),
                artist: "Test Artist".to_string(),
                track_count: 12,
            }
        ],
    }
}

/// Python module initialization
#[pymodule]
fn _internal(_py: Python, m: &PyModule) -> PyResult<()> {  // Changed from ddex_parser to _internal
    m.add_class::<PyDDEXParser>()?;
    m.add_class::<StreamIterator>()?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}