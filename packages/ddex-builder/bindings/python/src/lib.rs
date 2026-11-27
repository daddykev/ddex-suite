use ::ddex_builder::builder::{
    BuildOptions, BuildRequest, DDEXBuilder, LocalizedStringRequest, ReleaseRequest, ResourceKind,
    TrackRequest,
};
use ::ddex_parser::DDEXParser;
use ddex_core::models::flat::ParsedERNMessage;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};
use std::collections::HashMap;
use std::io::Cursor;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct Release {
    #[pyo3(get, set)]
    pub release_id: String,
    #[pyo3(get, set)]
    pub release_type: String,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get, set)]
    pub artist: String,
    #[pyo3(get, set)]
    pub label: Option<String>,
    #[pyo3(get, set)]
    pub catalog_number: Option<String>,
    #[pyo3(get, set)]
    pub upc: Option<String>,
    #[pyo3(get, set)]
    pub release_date: Option<String>,
    #[pyo3(get, set)]
    pub genre: Option<String>,
    #[pyo3(get, set)]
    pub parental_warning: Option<bool>,
    /// ERN 4.x `ParentalWarningType` (required for 4.x), e.g. "NotExplicit"
    #[pyo3(get, set)]
    pub parental_warning_type: Option<String>,
    #[pyo3(get, set)]
    pub track_ids: Vec<String>,
    #[pyo3(get, set)]
    pub metadata: Option<HashMap<String, String>>,
}

#[pymethods]
impl Release {
    #[new]
    #[pyo3(signature = (release_id, release_type, title, artist, label=None, catalog_number=None, upc=None, release_date=None, genre=None, parental_warning=None, track_ids=None, metadata=None, parental_warning_type=None))]
    pub fn new(
        release_id: String,
        release_type: String,
        title: String,
        artist: String,
        label: Option<String>,
        catalog_number: Option<String>,
        upc: Option<String>,
        release_date: Option<String>,
        genre: Option<String>,
        parental_warning: Option<bool>,
        track_ids: Option<Vec<String>>,
        metadata: Option<HashMap<String, String>>,
        parental_warning_type: Option<String>,
    ) -> Self {
        Release {
            release_id,
            release_type,
            title,
            artist,
            label,
            catalog_number,
            upc,
            release_date,
            genre,
            parental_warning,
            parental_warning_type,
            track_ids: track_ids.unwrap_or_default(),
            metadata,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Release(release_id='{}', title='{}', artist='{}')",
            self.release_id, self.title, self.artist
        )
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct Resource {
    #[pyo3(get, set)]
    pub resource_id: String,
    /// "SoundRecording" or "Video"
    #[pyo3(get, set)]
    pub resource_type: String,
    /// ERN 4.x: the resource's DDEX `Type` (required for 4.x), e.g. "MusicalWorkSoundRecording"
    #[pyo3(get, set)]
    pub ddex_type: Option<String>,
    /// ERN 4.x `ParentalWarningType` (required for 4.x)
    #[pyo3(get, set)]
    pub parental_warning_type: Option<String>,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get, set)]
    pub artist: String,
    #[pyo3(get, set)]
    pub isrc: Option<String>,
    #[pyo3(get, set)]
    pub duration: Option<String>,
    #[pyo3(get, set)]
    pub track_number: Option<i32>,
    #[pyo3(get, set)]
    pub volume_number: Option<i32>,
    #[pyo3(get, set)]
    pub metadata: Option<HashMap<String, String>>,
}

#[pymethods]
impl Resource {
    #[new]
    #[pyo3(signature = (resource_id, resource_type, title, artist, isrc=None, duration=None, track_number=None, volume_number=None, metadata=None, ddex_type=None, parental_warning_type=None))]
    pub fn new(
        resource_id: String,
        resource_type: String,
        title: String,
        artist: String,
        isrc: Option<String>,
        duration: Option<String>,
        track_number: Option<i32>,
        volume_number: Option<i32>,
        metadata: Option<HashMap<String, String>>,
        ddex_type: Option<String>,
        parental_warning_type: Option<String>,
    ) -> Self {
        Resource {
            resource_id,
            resource_type,
            ddex_type,
            parental_warning_type,
            title,
            artist,
            isrc,
            duration,
            track_number,
            volume_number,
            metadata,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Resource(resource_id='{}', title='{}', artist='{}')",
            self.resource_id, self.title, self.artist
        )
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct ValidationResult {
    #[pyo3(get, set)]
    pub is_valid: bool,
    #[pyo3(get, set)]
    pub errors: Vec<String>,
    #[pyo3(get, set)]
    pub warnings: Vec<String>,
}

#[pymethods]
impl ValidationResult {
    #[new]
    pub fn new(is_valid: bool, errors: Vec<String>, warnings: Vec<String>) -> Self {
        ValidationResult {
            is_valid,
            errors,
            warnings,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationResult(is_valid={}, errors={}, warnings={})",
            self.is_valid,
            self.errors.len(),
            self.warnings.len()
        )
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct BuilderStats {
    #[pyo3(get, set)]
    pub releases_count: u32,
    #[pyo3(get, set)]
    pub resources_count: u32,
    #[pyo3(get, set)]
    pub total_build_time_ms: f64,
    #[pyo3(get, set)]
    pub last_build_size_bytes: f64,
    #[pyo3(get, set)]
    pub validation_errors: u32,
    #[pyo3(get, set)]
    pub validation_warnings: u32,
}

#[pymethods]
impl BuilderStats {
    #[new]
    pub fn new(
        releases_count: u32,
        resources_count: u32,
        total_build_time_ms: f64,
        last_build_size_bytes: f64,
        validation_errors: u32,
        validation_warnings: u32,
    ) -> Self {
        BuilderStats {
            releases_count,
            resources_count,
            total_build_time_ms,
            last_build_size_bytes,
            validation_errors,
            validation_warnings,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "BuilderStats(releases={}, resources={}, build_time={}ms)",
            self.releases_count, self.resources_count, self.total_build_time_ms
        )
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct FidelityOptions {
    #[pyo3(get, set)]
    pub canonicalization: String,
    #[pyo3(get, set)]
    pub preserve_comments: bool,
    #[pyo3(get, set)]
    pub preserve_processing_instructions: bool,
    #[pyo3(get, set)]
    pub preserve_extensions: bool,
    #[pyo3(get, set)]
    pub preserve_attribute_order: bool,
    #[pyo3(get, set)]
    pub preserve_namespace_prefixes: bool,
    #[pyo3(get, set)]
    pub enable_verification: bool,
    #[pyo3(get, set)]
    pub collect_statistics: bool,
    #[pyo3(get, set)]
    pub enable_deterministic_ordering: bool,
    #[pyo3(get, set)]
    pub memory_optimization: String,
    #[pyo3(get, set)]
    pub streaming_mode: bool,
    #[pyo3(get, set)]
    pub chunk_size: u32,
    #[pyo3(get, set)]
    pub enable_checksums: bool,
}

#[pymethods]
impl FidelityOptions {
    #[new]
    #[pyo3(signature = (canonicalization="db_c14n".to_string(), preserve_comments=false, preserve_processing_instructions=false, preserve_extensions=true, preserve_attribute_order=true, preserve_namespace_prefixes=true, enable_verification=false, collect_statistics=false, enable_deterministic_ordering=true, memory_optimization="balanced".to_string(), streaming_mode=false, chunk_size=65536, enable_checksums=false))]
    pub fn new(
        canonicalization: String,
        preserve_comments: bool,
        preserve_processing_instructions: bool,
        preserve_extensions: bool,
        preserve_attribute_order: bool,
        preserve_namespace_prefixes: bool,
        enable_verification: bool,
        collect_statistics: bool,
        enable_deterministic_ordering: bool,
        memory_optimization: String,
        streaming_mode: bool,
        chunk_size: u32,
        enable_checksums: bool,
    ) -> Self {
        FidelityOptions {
            canonicalization,
            preserve_comments,
            preserve_processing_instructions,
            preserve_extensions,
            preserve_attribute_order,
            preserve_namespace_prefixes,
            enable_verification,
            collect_statistics,
            enable_deterministic_ordering,
            memory_optimization,
            streaming_mode,
            chunk_size,
            enable_checksums,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "FidelityOptions(canonicalization='{}', collect_statistics={})",
            self.canonicalization, self.collect_statistics
        )
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct BuildStatistics {
    #[pyo3(get, set)]
    pub build_time_ms: f64,
    /// Not measured: always None (before 0.5.1, twice the XML size).
    #[pyo3(get, set)]
    pub memory_used_bytes: Option<u32>,
    #[pyo3(get, set)]
    pub xml_size_bytes: u32,
    #[pyo3(get, set)]
    pub element_count: u32,
    #[pyo3(get, set)]
    pub attribute_count: u32,
    #[pyo3(get, set)]
    pub namespace_count: u32,
    /// Not measured: always None.
    #[pyo3(get, set)]
    pub extension_count: Option<u32>,
    /// Not measured separately (part of the build): always None (before 0.5.1, 2.0).
    #[pyo3(get, set)]
    pub canonicalization_time_ms: Option<f64>,
    #[pyo3(get, set)]
    pub verification_time_ms: Option<f64>,
}

#[pymethods]
impl BuildStatistics {
    #[new]
    pub fn new(
        build_time_ms: f64,
        memory_used_bytes: Option<u32>,
        xml_size_bytes: u32,
        element_count: u32,
        attribute_count: u32,
        namespace_count: u32,
        extension_count: Option<u32>,
        canonicalization_time_ms: Option<f64>,
        verification_time_ms: Option<f64>,
    ) -> Self {
        BuildStatistics {
            build_time_ms,
            memory_used_bytes,
            xml_size_bytes,
            element_count,
            attribute_count,
            namespace_count,
            extension_count,
            canonicalization_time_ms,
            verification_time_ms,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "BuildStatistics(build_time={}ms, xml_size={}bytes)",
            self.build_time_ms, self.xml_size_bytes
        )
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Parse → build → parse found no difference on the compared fields.
    #[pyo3(get, set)]
    pub round_trip_success: bool,
    /// No score is computed: always None (before 0.5.1, a size-ratio heuristic or 1.0).
    #[pyo3(get, set)]
    pub fidelity_score: Option<f64>,
    #[pyo3(get, set)]
    pub canonicalization_consistent: bool,
    #[pyo3(get, set)]
    pub determinism_verified: bool,
    #[pyo3(get, set)]
    pub issues: Vec<String>,
    #[pyo3(get, set)]
    pub checksums_match: Option<bool>,
}

#[pymethods]
impl VerificationResult {
    #[new]
    pub fn new(
        round_trip_success: bool,
        fidelity_score: Option<f64>,
        canonicalization_consistent: bool,
        determinism_verified: bool,
        issues: Vec<String>,
        checksums_match: Option<bool>,
    ) -> Self {
        VerificationResult {
            round_trip_success,
            fidelity_score,
            canonicalization_consistent,
            determinism_verified,
            issues,
            checksums_match,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "VerificationResult(success={}, issues={})",
            self.round_trip_success,
            self.issues.len()
        )
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct BuildResult {
    #[pyo3(get, set)]
    pub xml: String,
    #[pyo3(get, set)]
    pub statistics: Option<BuildStatistics>,
    #[pyo3(get, set)]
    pub verification: Option<VerificationResult>,
}

#[pymethods]
impl BuildResult {
    #[new]
    pub fn new(
        xml: String,
        statistics: Option<BuildStatistics>,
        verification: Option<VerificationResult>,
    ) -> Self {
        BuildResult {
            xml,
            statistics,
            verification,
        }
    }

    fn __repr__(&self) -> String {
        format!("BuildResult(xml_size={}bytes)", self.xml.len())
    }
}

#[pyclass]
pub struct DdexBuilder {
    releases: Vec<Release>,
    resources: Vec<Resource>,
    stats: BuilderStats,
}

#[pymethods]
impl DdexBuilder {
    #[new]
    pub fn new() -> Self {
        DdexBuilder {
            releases: Vec::new(),
            resources: Vec::new(),
            stats: BuilderStats::new(0, 0, 0.0, 0.0, 0, 0),
        }
    }

    pub fn add_release(&mut self, release: Release) {
        self.releases.push(release);
        self.stats.releases_count = self.releases.len() as u32;
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.push(resource);
        self.stats.resources_count = self.resources.len() as u32;
    }

    /// Build DDEX XML with the core builder.
    ///
    /// `request` is a build request dict (the core BuildRequest fields, see
    /// `ddex_builder.build_request`); `header` and `version` are required. When it has no
    /// `releases`, the releases added with `add_release`/`add_resource` are used. Nothing
    /// is filled in: a missing value fails the build with an error naming the field.
    #[pyo3(signature = (request = None))]
    pub fn build(&mut self, request: Option<Bound<'_, PyAny>>) -> PyResult<String> {
        let start_time = std::time::Instant::now();

        let build_request = self.build_request(request, None)?;

        // Use the actual DDEX builder
        let builder = DDEXBuilder::new();
        let options = BuildOptions::default();

        let result = builder.build(build_request, options).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Build failed: {}", e))
        })?;

        self.stats.last_build_size_bytes = result.xml.len() as f64;
        self.stats.total_build_time_ms += start_time.elapsed().as_millis() as f64;

        Ok(result.xml)
    }

    #[pyo3(signature = (fidelity_options = None, request = None))]
    pub fn build_with_fidelity(
        &mut self,
        fidelity_options: Option<&FidelityOptions>,
        request: Option<Bound<'_, PyAny>>,
    ) -> PyResult<BuildResult> {
        // The output is always DB-C14N/1.0; other algorithms are not implemented.
        if let Some(algorithm) = fidelity_options
            .map(|o| o.canonicalization.as_str())
            .filter(|a| !matches!(*a, "none" | "db_c14n"))
        {
            return Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                format!(
                    "canonicalization '{algorithm}' is not implemented in 0.5.x (only db_c14n)"
                ),
            ));
        }
        let start_time = std::time::Instant::now();

        let build_request = self.build_request(request, None)?;

        // Use the actual DDEX builder
        let builder = DDEXBuilder::new();
        let options = BuildOptions::default();

        let result = builder.build(build_request, options).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Build failed: {}", e))
        })?;

        self.stats.last_build_size_bytes = result.xml.len() as f64;
        let build_time = start_time.elapsed().as_millis() as f64;
        self.stats.total_build_time_ms += build_time;

        // Statistics computed from the output; what is not measured is None.
        let mut statistics = if fidelity_options.map_or(false, |o| o.collect_statistics) {
            let (elements, attributes, namespaces) = count_xml(&result.xml);
            Some(BuildStatistics::new(
                build_time,
                None,
                result.xml.len() as u32,
                elements,
                attributes,
                namespaces,
                None,
                None,
                None,
            ))
        } else {
            None
        };

        // A real round trip of the built XML (before 0.5.1: constants, all passing).
        let verification = if fidelity_options.map_or(false, |o| o.enable_verification) {
            let started = std::time::Instant::now();
            let verification = round_trip(&result.xml);
            if let Some(stats) = statistics.as_mut() {
                stats.verification_time_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
            }
            Some(verification)
        } else {
            None
        };

        Ok(BuildResult::new(result.xml, statistics, verification))
    }

    /// Parse → build → parse `original_xml` and compare the two parsed messages on the
    /// fields a build request carries (ddex_builder::round_trip). `issues` lists the
    /// differences; `fidelity_score` is None (no score is computed). The fidelity options
    /// are not applied in 0.5.x. Before 0.5.1 the result was not compared: success was
    /// always true and the score came from the size ratio.
    #[pyo3(signature = (original_xml, fidelity_options = None))]
    pub fn test_round_trip_fidelity(
        &mut self,
        original_xml: String,
        fidelity_options: Option<&FidelityOptions>,
    ) -> PyResult<VerificationResult> {
        let _ = fidelity_options;
        Ok(round_trip(&original_xml))
    }

    /// Runs the build's preflight validation on the added releases and resources (before
    /// 0.5.1 it only checked that a release had been added).
    pub fn validate(&mut self) -> PyResult<ValidationResult> {
        let result = match self
            .build_request(None, None)
            .map_err(|e| e.to_string())
            .and_then(|request| {
                DDEXBuilder::new()
                    .build(request, BuildOptions::default())
                    .map_err(|e| e.to_string())
            }) {
            Ok(built) => ValidationResult::new(
                true,
                vec![],
                built.warnings.iter().map(|w| w.message.clone()).collect(),
            ),
            Err(e) => ValidationResult::new(false, vec![e], vec![]),
        };
        self.stats.validation_errors = result.errors.len() as u32;
        self.stats.validation_warnings = result.warnings.len() as u32;
        Ok(result)
    }

    pub fn get_stats(&self) -> BuilderStats {
        self.stats.clone()
    }

    pub fn reset(&mut self) {
        self.releases.clear();
        self.resources.clear();
        self.stats = BuilderStats::new(0, 0, 0.0, 0.0, 0, 0);
    }

    /// Build DDEX XML from pandas DataFrame
    ///
    /// Args:
    ///     df: pandas DataFrame with DDEX data
    ///     schema: Optional schema hint ('flat', 'releases', or 'tracks')
    ///             If not provided, auto-detects from DataFrame columns
    ///
    /// Returns:
    ///     str: Generated DDEX XML
    #[pyo3(signature = (df, schema = None, request = None))]
    pub fn from_dataframe(
        &mut self,
        df: Bound<'_, PyAny>,
        schema: Option<&str>,
        request: Option<Bound<'_, PyAny>>,
    ) -> PyResult<String> {
        // Import pandas functionality through PyO3
        let pandas = df.py().import("pandas")?;
        let pd_dataframe = pandas.getattr("DataFrame")?;

        // Check if the input is a pandas DataFrame
        if !df.is_instance(&pd_dataframe)? {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Input must be a pandas DataFrame",
            ));
        }

        // Get DataFrame columns for auto-detection
        let columns = df.getattr("columns")?;
        let columns_list: Vec<String> = columns.extract()?;

        // Use provided schema or auto-detect
        let detected_schema = if let Some(s) = schema {
            // Validate provided schema
            match s {
                "flat" | "releases" | "tracks" => s,
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Invalid schema '{}'. Use 'flat', 'releases', or 'tracks'",
                        s
                    )))
                }
            }
        } else {
            // Auto-detect based on columns
            if columns_list.contains(&"type".to_string()) {
                "flat"
            } else if columns_list.contains(&"track_index".to_string()) {
                "tracks"
            } else {
                "releases"
            }
        };

        // Convert the rows to core release requests (only the values present), then build
        // with the core builder; `request` supplies the header, version and the other
        // message-level values.
        let records = df.call_method1("to_dict", ("records",))?;
        let records = records.cast::<PyList>()?;
        let releases = match detected_schema {
            "flat" => releases_from_rows(records, true)?,
            "releases" => releases_from_rows(records, false)?,
            "tracks" => releases_from_track_rows(records)?,
            _ => unreachable!(),
        };
        let build_request = self.build_request(request, Some(releases))?;
        let result = DDEXBuilder::new()
            .build(build_request, BuildOptions::default())
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Build failed: {}", e))
            })?;
        Ok(result.xml)
    }

    fn dict_to_release(&self, record: &Bound<'_, PyDict>) -> PyResult<Release> {
        let release_id: String = record
            .get_item("release_id")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("release_id is required"))?
            .extract()?;

        let release_type: String = record
            .get_item("release_type")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("release_type is required")
            })?
            .extract()?;

        let title: String = record
            .get_item("title")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("title is required"))?
            .extract()?;

        let artist: String = record
            .get_item("artist")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("artist is required"))?
            .extract()?;

        let label: Option<String> = record.get_item("label")?.map(|v| v.extract()).transpose()?;

        let catalog_number: Option<String> = record
            .get_item("catalog_number")?
            .map(|v| v.extract())
            .transpose()?;

        let upc: Option<String> = record.get_item("upc")?.map(|v| v.extract()).transpose()?;

        let release_date: Option<String> = record
            .get_item("release_date")?
            .map(|v| v.extract())
            .transpose()?;

        let genre: Option<String> = record.get_item("genre")?.map(|v| v.extract()).transpose()?;

        let parental_warning: Option<bool> = record
            .get_item("parental_warning")?
            .map(|v| v.extract())
            .transpose()?;

        let track_ids: Vec<String> = record
            .get_item("track_ids")?
            .map(|v| v.extract())
            .transpose()?
            .unwrap_or_default();

        let metadata: Option<HashMap<String, String>> = record
            .get_item("metadata")?
            .map(|v| v.extract())
            .transpose()?;

        let parental_warning_type: Option<String> = record
            .get_item("parental_warning_type")?
            .map(|v| v.extract())
            .transpose()?;

        Ok(Release::new(
            release_id,
            release_type,
            title,
            artist,
            label,
            catalog_number,
            upc,
            release_date,
            genre,
            parental_warning,
            Some(track_ids),
            metadata,
            parental_warning_type,
        ))
    }

    fn dict_to_resource(&self, record: &Bound<'_, PyDict>) -> PyResult<Resource> {
        let resource_id: String = record
            .get_item("resource_id")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("resource_id is required")
            })?
            .extract()?;

        let resource_type: String = record
            .get_item("resource_type")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("resource_type is required")
            })?
            .extract()?;

        let title: String = record
            .get_item("title")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("title is required"))?
            .extract()?;

        let artist: String = record
            .get_item("artist")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("artist is required"))?
            .extract()?;

        let isrc: Option<String> = record.get_item("isrc")?.map(|v| v.extract()).transpose()?;

        let duration: Option<String> = record
            .get_item("duration")?
            .map(|v| v.extract())
            .transpose()?;

        let track_number: Option<i32> = record
            .get_item("track_number")?
            .map(|v| v.extract())
            .transpose()?;

        let volume_number: Option<i32> = record
            .get_item("volume_number")?
            .map(|v| v.extract())
            .transpose()?;

        let metadata: Option<HashMap<String, String>> = record
            .get_item("metadata")?
            .map(|v| v.extract())
            .transpose()?;

        let ddex_type: Option<String> = record
            .get_item("ddex_type")?
            .map(|v| v.extract())
            .transpose()?;
        let parental_warning_type: Option<String> = record
            .get_item("parental_warning_type")?
            .map(|v| v.extract())
            .transpose()?;

        Ok(Resource::new(
            resource_id,
            resource_type,
            title,
            artist,
            isrc,
            duration,
            track_number,
            volume_number,
            metadata,
            ddex_type,
            parental_warning_type,
        ))
    }

    fn __repr__(&self) -> String {
        format!(
            "DdexBuilder(releases={}, resources={})",
            self.releases.len(),
            self.resources.len()
        )
    }
}

/// Build several messages with the core builder; each item is a build request dict (see
/// `DdexBuilder.build`). Fails on the first request that does not build, naming its index.
#[pyfunction]
pub fn batch_build(requests: Vec<Bound<'_, PyAny>>) -> PyResult<Vec<String>> {
    let mut results = Vec::new();
    for (i, item) in requests.into_iter().enumerate() {
        let request = request_from_py(&item)
            .map_err(|e| PyValueError::new_err(format!("batch request {}: {}", i, e)))?;
        let result = DDEXBuilder::new()
            .build(request, BuildOptions::default())
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "batch request {}: Build failed: {}",
                    i, e
                ))
            })?;
        results.push(result.xml);
    }
    Ok(results)
}

#[pyfunction]
pub fn validate_structure(xml: String) -> PyResult<ValidationResult> {
    // Reads the whole document (before 0.5.1, only its first event).
    let errors: Vec<String> = structure_error(&xml).into_iter().collect();
    Ok(ValidationResult::new(errors.is_empty(), errors, vec![]))
}

/// The first problem that makes `xml` not well-formed: a reference to an undefined entity
/// (`&foo;`, there is no DTD), with its line and column, or a syntax error anywhere in the
/// document.
fn structure_error(xml: &str) -> Option<String> {
    if let Err(entity) = ddex_core::xml_refs::check_document(xml.as_bytes()) {
        return Some(format!("XML parsing error: {entity}"));
    }
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Eof) => return None,
            Ok(_) => {}
            Err(e) => return Some(format!("XML parsing error: {e}")),
        }
    }
}

impl DdexBuilder {
    /// The core build request: `request` as given (missing `releases`/`deals` mean none),
    /// with `releases` (from a DataFrame) or else the stored releases when it has none.
    fn build_request(
        &self,
        request: Option<Bound<'_, PyAny>>,
        releases: Option<Vec<ReleaseRequest>>,
    ) -> PyResult<BuildRequest> {
        let mut build_request = match request {
            Some(r) => request_from_py(&r)?,
            None => request_from_json(serde_json::json!({}))?,
        };
        if let Some(releases) = releases {
            if !build_request.releases.is_empty() {
                return Err(PyValueError::new_err(
                    "pass releases either in the DataFrame or in the request, not both",
                ));
            }
            build_request.releases = releases;
        } else if build_request.releases.is_empty() {
            build_request.releases = self.stored_releases()?;
        }
        Ok(build_request)
    }

    /// Releases added with `add_release`/`add_resource`, using only the values given.
    fn stored_releases(&self) -> PyResult<Vec<ReleaseRequest>> {
        let mut releases = Vec::new();
        for release in &self.releases {
            let mut tracks = Vec::new();
            for track_id in &release.track_ids {
                let resource = self
                    .resources
                    .iter()
                    .find(|r| &r.resource_id == track_id)
                    .ok_or_else(|| {
                        PyValueError::new_err(format!(
                            "release {} lists track {} but no resource with that id was added",
                            release.release_id, track_id
                        ))
                    })?;
                tracks.push(TrackRequest {
                    track_id: resource.resource_id.clone(),
                    resource_reference: Some(resource.resource_id.clone()),
                    isrc: resource.isrc.clone().unwrap_or_default(),
                    title: resource.title.clone(),
                    duration: resource.duration.clone().unwrap_or_default(),
                    artist: resource.artist.clone(),
                    resource_kind: resource_kind(&resource.resource_type, &resource.resource_id)?,
                    resource_type: resource.ddex_type.clone().map(Into::into),
                    parental_warning_type: resource.parental_warning_type.clone(),
                    disc_number: resource.volume_number.and_then(|n| u32::try_from(n).ok()),
                    sequence_number: resource.track_number.and_then(|n| u32::try_from(n).ok()),
                    ..Default::default()
                });
            }
            releases.push(ReleaseRequest {
                release_id: release.release_id.clone(),
                // Generated by the core's ID strategy (a release ID such as a GRid is not a
                // valid ERN 4.x ReleaseReference)
                release_reference: None,
                title: vec![LocalizedStringRequest {
                    text: release.title.clone(),
                    language_code: None,
                }],
                artist: release.artist.clone(),
                label: release.label.clone(),
                release_date: release.release_date.clone(),
                upc: release.upc.clone(),
                tracks,
                resource_references: None,
                release_type: Some(release.release_type.clone().into()),
                genre: release.genre.clone(),
                parental_warning_type: release.parental_warning_type.clone(),
                ..Default::default()
            });
        }
        Ok(releases)
    }
}

/// "SoundRecording" or "Video" (the stored/DataFrame resource kind).
fn resource_kind(value: &str, resource_id: &str) -> PyResult<ResourceKind> {
    match value {
        "SoundRecording" => Ok(ResourceKind::SoundRecording),
        "Video" => Ok(ResourceKind::Video),
        other => Err(PyValueError::new_err(format!(
            "resource {}: resource_type must be \"SoundRecording\" or \"Video\", got {:?}",
            resource_id, other
        ))),
    }
}

fn request_from_json(mut value: serde_json::Value) -> PyResult<BuildRequest> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| PyValueError::new_err("the build request must be a dict"))?;
    object
        .entry("releases")
        .or_insert_with(|| serde_json::json!([]));
    object
        .entry("deals")
        .or_insert_with(|| serde_json::json!([]));
    serde_json::from_value(value)
        .map_err(|e| PyValueError::new_err(format!("Invalid build request: {}", e)))
}

/// A build request from a Python dict (via `json.dumps`, so only JSON values are allowed).
fn request_from_py(request: &Bound<'_, PyAny>) -> PyResult<BuildRequest> {
    let json = request.py().import("json")?;
    let text: String = json.call_method1("dumps", (request,))?.extract()?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| PyValueError::new_err(format!("Invalid build request: {}", e)))?;
    request_from_json(value)
}

/// A DataFrame cell as text: `None`/NaN are missing; numbers are written without a
/// trailing `.0` when integral (pandas turns integer columns with gaps into floats).
fn cell(record: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    let Some(value) = record.get_item(key)? else {
        return Ok(None);
    };
    if value.is_none() {
        return Ok(None);
    }
    if let Ok(text) = value.extract::<String>() {
        return Ok(Some(text));
    }
    if let Ok(n) = value.extract::<i64>() {
        return Ok(Some(n.to_string()));
    }
    if let Ok(f) = value.extract::<f64>() {
        if f.is_nan() {
            return Ok(None);
        }
        return Ok(Some(if f.fract() == 0.0 {
            format!("{}", f as i64)
        } else {
            f.to_string()
        }));
    }
    Ok(Some(value.str()?.extract()?))
}

fn cell_u32(record: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<u32>> {
    cell(record, key)?
        .map(|v| {
            v.parse::<u32>().map_err(|_| {
                PyValueError::new_err(format!("column {}: {:?} is not a whole number", key, v))
            })
        })
        .transpose()
}

/// A release date: kept as given, or the date part of an RFC 3339 date-time (the parser's
/// DataFrames carry date-times).
fn date_cell(record: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    Ok(
        cell(record, key)?.map(|v| match chrono::DateTime::parse_from_rfc3339(&v) {
            Ok(dt) => dt.format("%Y-%m-%d").to_string(),
            Err(_) => v,
        }),
    )
}

/// A duration: kept as given (ISO 8601), or numeric seconds (the parser's DataFrames)
/// written as ISO 8601.
fn duration_cell(record: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    Ok(cell(record, key)?.map(|v| match v.parse::<f64>() {
        Ok(seconds) => {
            let whole = seconds.trunc() as u64;
            let fraction = seconds - seconds.trunc();
            if fraction == 0.0 {
                format!("PT{}M{}S", whole / 60, whole % 60)
            } else {
                format!("PT{}M{}S", whole / 60, (whole % 60) as f64 + fraction)
            }
        }
        Err(_) => v,
    }))
}

fn release_from_row(record: &Bound<'_, PyDict>, prefix: &str) -> PyResult<ReleaseRequest> {
    let col = |name: &str| format!("{}{}", prefix, name);
    let release_id = cell(record, "release_id")?
        .ok_or_else(|| PyValueError::new_err("column release_id is required"))?;
    Ok(ReleaseRequest {
        release_id,
        release_reference: cell(record, "release_reference")?,
        title: cell(record, &col("title"))?
            .map(|text| {
                vec![LocalizedStringRequest {
                    text,
                    language_code: None,
                }]
            })
            .unwrap_or_default(),
        artist: cell(record, &col("artist"))?.unwrap_or_default(),
        label: cell(record, &col("label"))?,
        release_date: date_cell(record, &col("date"))?.or(date_cell(record, "release_date")?),
        upc: cell(record, &col("upc"))?.or(cell(record, "upc")?),
        release_type: cell(record, "release_type")?.map(Into::into),
        genre: cell(record, &col("genre"))?,
        sub_genre: cell(record, &col("sub_genre"))?,
        parental_warning_type: cell(record, &col("parental_warning_type"))?,
        ..Default::default()
    })
}

/// 'flat' / 'releases' schema: one release per row (flat: rows with type == "release").
fn releases_from_rows(records: &Bound<'_, PyList>, flat: bool) -> PyResult<Vec<ReleaseRequest>> {
    let mut releases = Vec::new();
    for item in records.iter() {
        let record = item.cast::<PyDict>()?;
        if flat && cell(record, "type")?.as_deref() != Some("release") {
            continue;
        }
        releases.push(release_from_row(record, "")?);
    }
    Ok(releases)
}

/// 'tracks' schema: one track per row, grouped into releases by `release_id` in order of
/// first appearance. Release-level values come from `release_`-prefixed columns
/// (`release_title`, `release_artist`, `release_label`, `release_genre`,
/// `release_parental_warning_type`, ...) and `release_type`.
fn releases_from_track_rows(records: &Bound<'_, PyList>) -> PyResult<Vec<ReleaseRequest>> {
    let mut releases: Vec<ReleaseRequest> = Vec::new();
    for item in records.iter() {
        let record = item.cast::<PyDict>()?;
        let release_id = cell(record, "release_id")?
            .ok_or_else(|| PyValueError::new_err("column release_id is required"))?;
        let index = match releases.iter().position(|r| r.release_id == release_id) {
            Some(i) => i,
            None => {
                releases.push(release_from_row(record, "release_")?);
                releases.len() - 1
            }
        };
        // Track reference: the track_id column, else derived from the row's track_index.
        let track_id = match cell(record, "track_id")? {
            Some(id) => id,
            None => {
                let i = cell_u32(record, "track_index")?.ok_or_else(|| {
                    PyValueError::new_err("column track_id (or track_index) is required")
                })?;
                format!("A{}", i + 1)
            }
        };
        let kind = cell(record, "resource_kind")?;
        releases[index].tracks.push(TrackRequest {
            resource_reference: Some(track_id.clone()),
            isrc: cell(record, "isrc")?.unwrap_or_default(),
            title: cell(record, "track_title")?.unwrap_or_default(),
            duration: duration_cell(record, "duration")?.unwrap_or_default(),
            artist: cell(record, "artist")?.unwrap_or_default(),
            resource_kind: match kind {
                Some(k) => resource_kind(&k, &track_id)?,
                None => ResourceKind::SoundRecording,
            },
            resource_type: cell(record, "resource_type")?.map(Into::into),
            parental_warning_type: cell(record, "parental_warning_type")?,
            disc_number: cell_u32(record, "disc_number")?,
            sequence_number: cell_u32(record, "track_number")?,
            track_id,
            ..Default::default()
        });
    }
    Ok(releases)
}

// gil_used = true keeps the pyo3 0.24 behaviour (0.28 changed the default to declaring
// free-threaded support, which these bindings have not been tested for).
/// Parse with ddex-parser (the parse function the round trip needs).
fn parse_xml(xml: &str) -> Result<ParsedERNMessage, String> {
    DDEXParser::new()
        .parse_with_options(Cursor::new(xml.as_bytes()), Default::default())
        .map_err(|e| e.to_string())
}

/// A real Parse → Build → Parse round trip of `xml` (ddex_builder::round_trip). A failure to
/// parse or build is reported as `round_trip_success: False` with the error as the issue.
fn round_trip(xml: &str) -> VerificationResult {
    match ddex_builder::round_trip::verify_round_trip(xml, parse_xml) {
        Ok(result) => VerificationResult::new(
            result.success,
            None,
            result.canonical,
            result.deterministic,
            result.differences,
            Some(result.checksums_match),
        ),
        Err(e) => VerificationResult::new(
            false,
            None,
            false,
            false,
            vec![format!("round trip failed: {e}")],
            None,
        ),
    }
}

/// (elements, attributes without namespace declarations, namespace declarations) of `xml`.
fn count_xml(xml: &str) -> (u32, u32, u32) {
    use quick_xml::events::Event;
    let (mut elements, mut attributes, mut namespaces) = (0, 0, 0);
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                elements += 1;
                for attr in e.attributes().flatten() {
                    let key = attr.key.as_ref();
                    if key == b"xmlns" || key.starts_with(b"xmlns:") {
                        namespaces += 1;
                    } else {
                        attributes += 1;
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }
    (elements, attributes, namespaces)
}

#[pymodule(gil_used = true)]
fn _internal(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Release>()?;
    m.add_class::<Resource>()?;
    m.add_class::<ValidationResult>()?;
    m.add_class::<BuilderStats>()?;
    m.add_class::<FidelityOptions>()?;
    m.add_class::<BuildStatistics>()?;
    m.add_class::<VerificationResult>()?;
    m.add_class::<BuildResult>()?;
    m.add_class::<DdexBuilder>()?;
    m.add_function(wrap_pyfunction!(batch_build, m)?)?;
    m.add_function(wrap_pyfunction!(validate_structure, m)?)?;
    Ok(())
}
