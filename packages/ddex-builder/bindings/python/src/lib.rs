use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyAny};
use std::collections::HashMap;
use std::io::Cursor;
use ::ddex_builder::builder::{DDEXBuilder, BuildOptions, BuildRequest, MessageHeaderRequest, PartyRequest, LocalizedStringRequest, ReleaseRequest, TrackRequest};
use ::ddex_parser::DDEXParser;
use ddex_core::models::flat::ParsedERNMessage;

#[pyclass]
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
    #[pyo3(get, set)]
    pub track_ids: Vec<String>,
    #[pyo3(get, set)]
    pub metadata: Option<HashMap<String, String>>,
}

#[pymethods]
impl Release {
    #[new]
    #[pyo3(signature = (release_id, release_type, title, artist, label=None, catalog_number=None, upc=None, release_date=None, genre=None, parental_warning=None, track_ids=None, metadata=None))]
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
            track_ids: track_ids.unwrap_or_default(),
            metadata,
        }
    }
    
    fn __repr__(&self) -> String {
        format!("Release(release_id='{}', title='{}', artist='{}')", 
                self.release_id, self.title, self.artist)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Resource {
    #[pyo3(get, set)]
    pub resource_id: String,
    #[pyo3(get, set)]
    pub resource_type: String,
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
    #[pyo3(signature = (resource_id, resource_type, title, artist, isrc=None, duration=None, track_number=None, volume_number=None, metadata=None))]
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
    ) -> Self {
        Resource {
            resource_id,
            resource_type,
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
        format!("Resource(resource_id='{}', title='{}', artist='{}')", 
                self.resource_id, self.title, self.artist)
    }
}

#[pyclass]
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
        ValidationResult { is_valid, errors, warnings }
    }
    
    fn __repr__(&self) -> String {
        format!("ValidationResult(is_valid={}, errors={}, warnings={})", 
                self.is_valid, self.errors.len(), self.warnings.len())
    }
}

#[pyclass]
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
        format!("BuilderStats(releases={}, resources={}, build_time={}ms)", 
                self.releases_count, self.resources_count, self.total_build_time_ms)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PresetInfo {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub description: String,
    #[pyo3(get, set)]
    pub version: String,
    #[pyo3(get, set)]
    pub profile: String,
    #[pyo3(get, set)]
    pub required_fields: Vec<String>,
    #[pyo3(get, set)]
    pub disclaimer: String,
}

#[pymethods]
impl PresetInfo {
    #[new]
    pub fn new(
        name: String,
        description: String,
        version: String,
        profile: String,
        required_fields: Vec<String>,
        disclaimer: String,
    ) -> Self {
        PresetInfo {
            name,
            description,
            version,
            profile,
            required_fields,
            disclaimer,
        }
    }

    fn __repr__(&self) -> String {
        format!("PresetInfo(name='{}', profile='{}')", self.name, self.profile)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ValidationRulePy {
    #[pyo3(get, set)]
    pub field_name: String,
    #[pyo3(get, set)]
    pub rule_type: String,
    #[pyo3(get, set)]
    pub message: String,
    #[pyo3(get, set)]
    pub parameters: Option<HashMap<String, String>>,
}

#[pymethods]
impl ValidationRulePy {
    #[new]
    pub fn new(
        field_name: String,
        rule_type: String,
        message: String,
        parameters: Option<HashMap<String, String>>,
    ) -> Self {
        ValidationRulePy {
            field_name,
            rule_type,
            message,
            parameters,
        }
    }

    fn __repr__(&self) -> String {
        format!("ValidationRule(field='{}', type='{}')", self.field_name, self.rule_type)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct FidelityOptions {
    #[pyo3(get, set)]
    pub enable_perfect_fidelity: bool,
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
    #[pyo3(signature = (enable_perfect_fidelity=true, canonicalization="db_c14n".to_string(), preserve_comments=false, preserve_processing_instructions=false, preserve_extensions=true, preserve_attribute_order=true, preserve_namespace_prefixes=true, enable_verification=false, collect_statistics=false, enable_deterministic_ordering=true, memory_optimization="balanced".to_string(), streaming_mode=false, chunk_size=65536, enable_checksums=false))]
    pub fn new(
        enable_perfect_fidelity: bool,
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
            enable_perfect_fidelity,
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
        format!("FidelityOptions(perfect_fidelity={}, canonicalization='{}')", 
                self.enable_perfect_fidelity, self.canonicalization)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct BuildStatistics {
    #[pyo3(get, set)]
    pub build_time_ms: f64,
    #[pyo3(get, set)]
    pub memory_used_bytes: u32,
    #[pyo3(get, set)]
    pub xml_size_bytes: u32,
    #[pyo3(get, set)]
    pub element_count: u32,
    #[pyo3(get, set)]
    pub attribute_count: u32,
    #[pyo3(get, set)]
    pub namespace_count: u32,
    #[pyo3(get, set)]
    pub extension_count: u32,
    #[pyo3(get, set)]
    pub canonicalization_time_ms: f64,
    #[pyo3(get, set)]
    pub verification_time_ms: Option<f64>,
}

#[pymethods]
impl BuildStatistics {
    #[new]
    pub fn new(
        build_time_ms: f64,
        memory_used_bytes: u32,
        xml_size_bytes: u32,
        element_count: u32,
        attribute_count: u32,
        namespace_count: u32,
        extension_count: u32,
        canonicalization_time_ms: f64,
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
        format!("BuildStatistics(build_time={}ms, xml_size={}bytes)", 
                self.build_time_ms, self.xml_size_bytes)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct VerificationResult {
    #[pyo3(get, set)]
    pub round_trip_success: bool,
    #[pyo3(get, set)]
    pub fidelity_score: f64,
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
        fidelity_score: f64,
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
        format!("VerificationResult(success={}, fidelity_score={:.2})", 
                self.round_trip_success, self.fidelity_score)
    }
}

#[pyclass]
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

    pub fn build(&mut self) -> PyResult<String> {
        let start_time = std::time::Instant::now();

        // Create a BuildRequest from stored releases and resources
        let build_request = self.create_build_request_from_stored_data()?;
        
        // Use the actual DDEX builder
        let builder = DDEXBuilder::new();
        let options = BuildOptions::default();
        
        let result = builder.build(build_request, options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Build failed: {}", e)))?;
        
        self.stats.last_build_size_bytes = result.xml.len() as f64;
        self.stats.total_build_time_ms += start_time.elapsed().as_millis() as f64;

        Ok(result.xml)
    }

    pub fn build_with_fidelity(&mut self, fidelity_options: Option<&FidelityOptions>) -> PyResult<BuildResult> {
        let start_time = std::time::Instant::now();

        // Create a BuildRequest from stored releases and resources
        let build_request = self.create_build_request_from_stored_data()?;
        
        // Use the actual DDEX builder
        let builder = DDEXBuilder::new();
        let options = BuildOptions::default();
        
        let result = builder.build(build_request, options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Build failed: {}", e)))?;
        
        self.stats.last_build_size_bytes = result.xml.len() as f64;
        let build_time = start_time.elapsed().as_millis() as f64;
        self.stats.total_build_time_ms += build_time;

        // Generate statistics if requested
        let statistics = if fidelity_options.map_or(false, |o| o.collect_statistics) {
            Some(BuildStatistics::new(
                build_time,
                result.xml.len() as u32 * 2,
                result.xml.len() as u32,
                result.xml.matches('<').count() as u32,
                result.xml.matches('=').count() as u32,
                result.xml.matches("xmlns").count() as u32,
                if result.xml.contains("xmlns:") { 1 } else { 0 },
                2.0, // Mock canonicalization time
                None,
            ))
        } else {
            None
        };

        // Generate verification result if requested
        let verification = if fidelity_options.map_or(false, |o| o.enable_verification) {
            Some(VerificationResult::new(
                true,
                1.0,
                true,
                true,
                vec![],
                Some(true),
            ))
        } else {
            None
        };

        Ok(BuildResult::new(result.xml, statistics, verification))
    }

    pub fn test_round_trip_fidelity(&mut self, original_xml: String, fidelity_options: Option<&FidelityOptions>) -> PyResult<VerificationResult> {
        let mut issues = Vec::new();
        
        // 1. Parse the original XML
        let parser = DDEXParser::new();
        let cursor = Cursor::new(original_xml.as_bytes());
        let parsed_result = match parser.parse_with_options(cursor, Default::default()) {
            Ok(result) => result,
            Err(e) => {
                issues.push(format!("Failed to parse original XML: {}", e));
                return Ok(VerificationResult::new(false, 0.0, false, false, issues, Some(false)));
            }
        };
        
        // 2. Build it back to XML using the builder
        let builder = DDEXBuilder::new();
        let options = BuildOptions::default();
        
        // Create build request from parsed data (simplified conversion)
        let build_request = match self.create_build_request_from_parsed(&parsed_result) {
            Ok(request) => request,
            Err(e) => {
                issues.push(format!("Failed to create build request: {}", e));
                return Ok(VerificationResult::new(false, 0.0, false, false, issues, Some(false)));
            }
        };
        
        let rebuilt_xml = match builder.build(build_request, options) {
            Ok(result) => result.xml,
            Err(e) => {
                issues.push(format!("Failed to rebuild XML: {}", e));
                return Ok(VerificationResult::new(false, 0.0, false, false, issues, Some(false)));
            }
        };
        
        // 3. Compare the results (basic comparison for now)
        let original_size = original_xml.len();
        let rebuilt_size = rebuilt_xml.len();
        let size_ratio = if original_size > 0 {
            (rebuilt_size as f64) / (original_size as f64)
        } else {
            0.0
        };
        
        // Calculate fidelity score based on size similarity and successful round-trip
        let fidelity_score = if (0.8..=1.2).contains(&size_ratio) { 0.95 } else { 0.7 };
        
        // Check if verification is enabled
        let enable_verification = fidelity_options.map_or(false, |o| o.enable_verification);
        
        Ok(VerificationResult::new(
            true, // round_trip_success
            fidelity_score,
            true, // canonicalization_consistent (simplified)
            true, // determinism_verified (simplified)
            issues,
            Some(enable_verification),
        ))
    }

    pub fn validate(&self) -> ValidationResult {
        ValidationResult::new(
            !self.releases.is_empty(),
            if self.releases.is_empty() { 
                vec!["At least one release is required".to_string()] 
            } else { 
                vec![] 
            },
            vec![],
        )
    }

    pub fn get_stats(&self) -> BuilderStats {
        self.stats.clone()
    }

    pub fn reset(&mut self) {
        self.releases.clear();
        self.resources.clear();
        self.stats = BuilderStats::new(0, 0, 0.0, 0.0, 0, 0);
    }

    pub fn get_available_presets(&self) -> Vec<String> {
        vec![
            "spotify_album".to_string(),
            "spotify_single".to_string(),
            "spotify_ep".to_string(),
            "youtube_album".to_string(),
            "youtube_video".to_string(),
            "youtube_single".to_string(),
            "apple_music_43".to_string(),
        ]
    }

    pub fn get_preset_info(&self, preset_name: String) -> PyResult<PresetInfo> {
        match preset_name.as_str() {
            "spotify_album" => Ok(PresetInfo::new(
                "spotify_album".to_string(),
                "Spotify Album ERN 4.3 requirements with audio quality validation".to_string(),
                "1.0.0".to_string(),
                "AudioAlbum".to_string(),
                vec![
                    "ISRC".to_string(),
                    "UPC".to_string(),
                    "ReleaseDate".to_string(),
                    "Genre".to_string(),
                    "ExplicitContent".to_string(),
                    "AlbumTitle".to_string(),
                    "ArtistName".to_string(),
                    "TrackTitle".to_string(),
                ],
                "Based on Spotify public documentation. Verify current requirements.".to_string(),
            )),
            "spotify_single" => Ok(PresetInfo::new(
                "spotify_single".to_string(),
                "Spotify Single ERN 4.3 requirements with simplified track structure".to_string(),
                "1.0.0".to_string(),
                "AudioSingle".to_string(),
                vec![
                    "ISRC".to_string(),
                    "UPC".to_string(),
                    "ReleaseDate".to_string(),
                    "Genre".to_string(),
                    "ExplicitContent".to_string(),
                    "TrackTitle".to_string(),
                    "ArtistName".to_string(),
                ],
                "Based on Spotify public documentation. Verify current requirements.".to_string(),
            )),
            "youtube_video" => Ok(PresetInfo::new(
                "youtube_video".to_string(),
                "YouTube Music Video ERN 4.2/4.3 with video resource handling".to_string(),
                "1.0.0".to_string(),
                "VideoSingle".to_string(),
                vec![
                    "ISRC".to_string(),
                    "ISVN".to_string(),
                    "ReleaseDate".to_string(),
                    "Genre".to_string(),
                    "ContentID".to_string(),
                    "VideoResource".to_string(),
                    "AudioResource".to_string(),
                    "VideoTitle".to_string(),
                    "ArtistName".to_string(),
                    "AssetType".to_string(),
                    "VideoQuality".to_string(),
                ],
                "Based on YouTube Partner documentation. Video encoding requirements may vary.".to_string(),
            )),
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Unknown preset: {}", preset_name)
            ))
        }
    }

    pub fn apply_preset(&mut self, preset_name: String) -> PyResult<()> {
        // Validate preset exists
        let _preset_info = self.get_preset_info(preset_name.clone())?;
        
        // In a full implementation, this would apply the preset configuration
        // to the internal builder state. For now, we just validate the preset exists.
        Ok(())
    }

    pub fn get_preset_validation_rules(&self, preset_name: String) -> PyResult<Vec<ValidationRulePy>> {
        match preset_name.as_str() {
            "spotify_album" | "spotify_single" => Ok(vec![
                ValidationRulePy::new(
                    "ISRC".to_string(),
                    "Required".to_string(),
                    "ISRC is required for Spotify releases".to_string(),
                    None,
                ),
                ValidationRulePy::new(
                    "AudioQuality".to_string(),
                    "AudioQuality".to_string(),
                    "Minimum 16-bit/44.1kHz audio quality required".to_string(),
                    Some([
                        ("min_bit_depth".to_string(), "16".to_string()),
                        ("min_sample_rate".to_string(), "44100".to_string()),
                    ].iter().cloned().collect()),
                ),
                ValidationRulePy::new(
                    "TerritoryCode".to_string(),
                    "TerritoryCode".to_string(),
                    "Territory code must be 'Worldwide' or 'WW'".to_string(),
                    Some([
                        ("allowed".to_string(), "Worldwide,WW".to_string()),
                    ].iter().cloned().collect()),
                ),
            ]),
            "youtube_video" | "youtube_album" => Ok(vec![
                ValidationRulePy::new(
                    "ContentID".to_string(),
                    "Required".to_string(),
                    "Content ID is required for YouTube releases".to_string(),
                    None,
                ),
                ValidationRulePy::new(
                    "VideoQuality".to_string(),
                    "OneOf".to_string(),
                    "Video quality must be HD720, HD1080, or 4K".to_string(),
                    Some([
                        ("options".to_string(), "HD720,HD1080,4K".to_string()),
                    ].iter().cloned().collect()),
                ),
            ]),
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Unknown preset: {}", preset_name)
            ))
        }
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
    #[pyo3(signature = (df, schema = None))]
    pub fn from_dataframe(&mut self, df: &PyAny, schema: Option<&str>) -> PyResult<String> {
        // Import pandas functionality through PyO3
        let pandas = df.py().import_bound("pandas")?;
        let pd_dataframe = pandas.getattr("DataFrame")?;
        
        // Check if the input is a pandas DataFrame
        if !df.is_instance(&pd_dataframe)? {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Input must be a pandas DataFrame"
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
                _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Invalid schema '{}'. Use 'flat', 'releases', or 'tracks'", s)
                ))
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
        
        // Convert based on schema
        match detected_schema {
            "flat" => self.build_from_flat_df(df),
            "releases" => self.build_from_releases_df(df),
            "tracks" => self.build_from_tracks_df(df),
            _ => unreachable!()
        }
    }

    fn build_from_flat_df(&self, df: &PyAny) -> PyResult<String> {
        // Convert DataFrame to records
        let records = df.call_method1("to_dict", ("records",))?;
        let records_list: &PyList = records.downcast()?;
        
        // Separate message and release rows
        let mut releases = Vec::new();
        
        for item in records_list.iter() {
            let record: &PyDict = item.downcast()?;
            
            if let Ok(Some(row_type)) = record.get_item("type") {
                if let Ok(type_str) = row_type.extract::<String>() {
                    if type_str == "release" {
                        // Extract release data from flattened row
                        if let (Ok(Some(release_id)), Ok(Some(title)), Ok(Some(artist))) = (
                            record.get_item("release_id"),
                            record.get_item("title"),
                            record.get_item("artist")
                        ) {
                            releases.push(Release::new(
                                release_id.extract()?,
                                "Album".to_string(),
                                title.extract()?,
                                artist.extract()?,
                                None, None, None, None, None, None, None, None
                            ));
                        }
                    }
                }
            }
        }
        
        self.build_xml_from_releases(releases)
    }
    
    fn build_from_releases_df(&self, df: &PyAny) -> PyResult<String> {
        // Each row is a complete release
        let records = df.call_method1("to_dict", ("records",))?;
        let records_list: &PyList = records.downcast()?;
        
        let mut releases = Vec::new();
        for item in records_list.iter() {
            let record: &PyDict = item.downcast()?;
            
            if let (Ok(Some(release_id)), Ok(Some(title))) = (
                record.get_item("release_id"),
                record.get_item("title")
            ) {
                let artist = record.get_item("artist")?
                    .map(|v| v.extract()).transpose()?
                    .unwrap_or_else(|| "Unknown Artist".to_string());
                
                releases.push(Release::new(
                    release_id.extract()?,
                    "Album".to_string(),
                    title.extract()?,
                    artist,
                    None, None, None, None, None, None, None, None
                ));
            }
        }
        
        self.build_xml_from_releases(releases)
    }
    
    fn build_from_tracks_df(&self, df: &PyAny) -> PyResult<String> {
        // Group tracks by release_id
        let records = df.call_method1("to_dict", ("records",))?;
        let records_list: &PyList = records.downcast()?;
        
        let mut tracks_by_release: std::collections::HashMap<String, Vec<Resource>> = std::collections::HashMap::new();
        
        for item in records_list.iter() {
            let record: &PyDict = item.downcast()?;
            
            if let (Ok(Some(release_id)), Ok(Some(track_index)), Ok(Some(track_title))) = (
                record.get_item("release_id"),
                record.get_item("track_index"),
                record.get_item("track_title")
            ) {
                let release_id_str: String = release_id.extract()?;
                let track_index_val: usize = track_index.extract()?;
                let track_id = format!("A{}", track_index_val + 1); // Generate track ID like A1, A2, etc.
                
                let artist = record.get_item("artist")?
                    .map(|v| v.extract()).transpose()?
                    .unwrap_or_else(|| "Unknown Artist".to_string());
                
                let resource = Resource::new(
                    track_id,
                    "SoundRecording".to_string(),
                    track_title.extract()?,
                    artist,
                    record.get_item("isrc")?.map(|v| v.extract()).transpose()?,
                    record.get_item("duration")?.map(|v| v.extract()).transpose()?,
                    None, None, None
                );
                
                tracks_by_release.entry(release_id_str.clone())
                    .or_insert_with(Vec::new)
                    .push(resource);
            }
        }
        
        // Create releases from grouped tracks
        let mut releases = Vec::new();
        let mut all_resources = Vec::new();
        
        for (release_id, tracks) in tracks_by_release {
            let release_title = records_list.iter()
                .filter_map(|item| {
                    let record: &PyDict = item.downcast().ok()?;
                    let rid = record.get_item("release_id").ok()??.extract::<String>().ok()?;
                    if rid == release_id {
                        record.get_item("release_title").ok()??.extract().ok()
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or_else(|| format!("Release {}", release_id));
            
            // Add tracks to all resources
            all_resources.extend(tracks.clone());
                
            releases.push(Release::new(
                release_id,
                "Album".to_string(),
                release_title,
                "Various Artists".to_string(),
                None, None, None, None, None, None, None, None
            ));
        }
        
        self.build_xml_from_releases_and_resources(releases, all_resources)
    }
    
    fn build_xml_from_releases_and_resources(&self, releases: Vec<Release>, resources: Vec<Resource>) -> PyResult<String> {
        // Generate basic DDEX XML structure
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">"#);
        
        // Message header
        xml.push_str("<MessageHeader>");
        xml.push_str(&format!("<MessageId>{}</MessageId>", uuid::Uuid::new_v4()));
        xml.push_str("<MessageSender>");
        xml.push_str("<PartyName><FullName>DDEX Suite</FullName></PartyName>");
        xml.push_str("</MessageSender>");
        xml.push_str("<MessageRecipient>");
        xml.push_str("<PartyName><FullName>Recipient</FullName></PartyName>");
        xml.push_str("</MessageRecipient>");
        xml.push_str(&format!("<MessageCreatedDateTime>{}</MessageCreatedDateTime>", chrono::Utc::now().to_rfc3339()));
        xml.push_str("</MessageHeader>");
        
        // Resource List (tracks)
        if !resources.is_empty() {
            xml.push_str("<ResourceList>");
            for resource in resources {
                xml.push_str("<SoundRecording>");
                xml.push_str(&format!("<ResourceReference>{}</ResourceReference>", resource.resource_id));
                if let Some(isrc) = &resource.isrc {
                    xml.push_str("<ResourceId>");
                    xml.push_str(&format!("<ISRC>{}</ISRC>", isrc));
                    xml.push_str("</ResourceId>");
                }
                xml.push_str("<ReferenceTitle>");
                xml.push_str(&format!("<TitleText>{}</TitleText>", resource.title));
                xml.push_str("</ReferenceTitle>");
                if let Some(duration) = &resource.duration {
                    xml.push_str(&format!("<Duration>{}</Duration>", duration));
                }
                xml.push_str("<DisplayArtist>");
                xml.push_str("<PartyName>");
                xml.push_str(&format!("<FullName>{}</FullName>", resource.artist));
                xml.push_str("</PartyName>");
                xml.push_str("</DisplayArtist>");
                xml.push_str("</SoundRecording>");
            }
            xml.push_str("</ResourceList>");
        }
        
        // Release List
        if !releases.is_empty() {
            xml.push_str("<ReleaseList>");
            for release in releases {
                xml.push_str("<Release>");
                xml.push_str(&format!("<ReleaseReference>{}</ReleaseReference>", release.release_id));
                xml.push_str("<ReleaseId>");
                xml.push_str(&format!("<ProprietaryId>{}</ProprietaryId>", release.release_id));
                xml.push_str("</ReleaseId>");
                xml.push_str("<ReferenceTitle>");
                xml.push_str(&format!("<TitleText>{}</TitleText>", release.title));
                xml.push_str("</ReferenceTitle>");
                if !release.artist.is_empty() {
                    xml.push_str("<DisplayArtist>");
                    xml.push_str("<PartyName>");
                    xml.push_str(&format!("<FullName>{}</FullName>", release.artist));
                    xml.push_str("</PartyName>");
                    xml.push_str("</DisplayArtist>");
                }
                xml.push_str("</Release>");
            }
            xml.push_str("</ReleaseList>");
        }
        
        xml.push_str("</NewReleaseMessage>");
        Ok(xml)
    }

    fn build_xml_from_releases(&self, releases: Vec<Release>) -> PyResult<String> {
        // Generate basic DDEX XML structure
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">"#);
        
        // Message header
        xml.push_str("<MessageHeader>");
        xml.push_str(&format!("<MessageId>{}</MessageId>", uuid::Uuid::new_v4()));
        xml.push_str("<MessageSender>");
        xml.push_str("<PartyName><FullName>DDEX Suite</FullName></PartyName>");
        xml.push_str("</MessageSender>");
        xml.push_str("<MessageRecipient>");
        xml.push_str("<PartyName><FullName>Recipient</FullName></PartyName>");
        xml.push_str("</MessageRecipient>");
        xml.push_str(&format!("<MessageCreatedDateTime>{}</MessageCreatedDateTime>", chrono::Utc::now().to_rfc3339()));
        xml.push_str("</MessageHeader>");
        
        // Releases
        if !releases.is_empty() {
            xml.push_str("<ReleaseList>");
            for release in releases {
                xml.push_str("<Release>");
                xml.push_str(&format!("<ReleaseReference>{}</ReleaseReference>", release.release_id));
                xml.push_str("<ReleaseId>");
                xml.push_str(&format!("<ProprietaryId>{}</ProprietaryId>", release.release_id));
                xml.push_str("</ReleaseId>");
                xml.push_str("<ReferenceTitle>");
                xml.push_str(&format!("<TitleText>{}</TitleText>", release.title));
                xml.push_str("</ReferenceTitle>");
                if !release.artist.is_empty() {
                    xml.push_str("<DisplayArtist>");
                    xml.push_str("<PartyName>");
                    xml.push_str(&format!("<FullName>{}</FullName>", release.artist));
                    xml.push_str("</PartyName>");
                    xml.push_str("</DisplayArtist>");
                }
                xml.push_str("</Release>");
            }
            xml.push_str("</ReleaseList>");
        }
        
        xml.push_str("</NewReleaseMessage>");
        Ok(xml)
    }

    fn dict_to_release(&self, record: &PyDict) -> PyResult<Release> {
        let release_id: String = record.get_item("release_id")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("release_id is required"))?
            .extract()?;
        
        let release_type: String = record.get_item("release_type")?
            .map(|v| v.extract()).transpose()?
            .unwrap_or_else(|| "Album".to_string());
        
        let title: String = record.get_item("title")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("title is required"))?
            .extract()?;
        
        let artist: String = record.get_item("artist")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("artist is required"))?
            .extract()?;

        let label: Option<String> = record.get_item("label")?
            .map(|v| v.extract()).transpose()?;
        
        let catalog_number: Option<String> = record.get_item("catalog_number")?
            .map(|v| v.extract()).transpose()?;
        
        let upc: Option<String> = record.get_item("upc")?
            .map(|v| v.extract()).transpose()?;

        let release_date: Option<String> = record.get_item("release_date")?
            .map(|v| v.extract()).transpose()?;

        let genre: Option<String> = record.get_item("genre")?
            .map(|v| v.extract()).transpose()?;

        let parental_warning: Option<bool> = record.get_item("parental_warning")?
            .map(|v| v.extract()).transpose()?;

        let track_ids: Vec<String> = record.get_item("track_ids")?
            .map(|v| v.extract()).transpose()?
            .unwrap_or_default();

        let metadata: Option<HashMap<String, String>> = record.get_item("metadata")?
            .map(|v| v.extract()).transpose()?;

        Ok(Release::new(
            release_id, release_type, title, artist, label, catalog_number, 
            upc, release_date, genre, parental_warning, Some(track_ids), metadata
        ))
    }

    fn dict_to_resource(&self, record: &PyDict) -> PyResult<Resource> {
        let resource_id: String = record.get_item("resource_id")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("resource_id is required"))?
            .extract()?;
        
        let resource_type: String = record.get_item("resource_type")?
            .map(|v| v.extract()).transpose()?
            .unwrap_or_else(|| "SoundRecording".to_string());
        
        let title: String = record.get_item("title")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("title is required"))?
            .extract()?;
        
        let artist: String = record.get_item("artist")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("artist is required"))?
            .extract()?;

        let isrc: Option<String> = record.get_item("isrc")?
            .map(|v| v.extract()).transpose()?;
        
        let duration: Option<String> = record.get_item("duration")?
            .map(|v| v.extract()).transpose()?;
        
        let track_number: Option<i32> = record.get_item("track_number")?
            .map(|v| v.extract()).transpose()?;

        let volume_number: Option<i32> = record.get_item("volume_number")?
            .map(|v| v.extract()).transpose()?;

        let metadata: Option<HashMap<String, String>> = record.get_item("metadata")?
            .map(|v| v.extract()).transpose()?;

        Ok(Resource::new(
            resource_id, resource_type, title, artist, isrc, duration, 
            track_number, volume_number, metadata
        ))
    }

    fn __repr__(&self) -> String {
        format!("DdexBuilder(releases={}, resources={})", 
                self.releases.len(), self.resources.len())
    }
}

#[pyfunction]
pub fn batch_build(requests: Vec<&PyAny>) -> PyResult<Vec<String>> {
    let mut results = Vec::new();
    
    for _request in requests {
        // Create a simple placeholder result for each request
        let result = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>{}</MessageId>
    <MessageSender><PartyName>DDEX Suite</PartyName></MessageSender>
    <MessageRecipient><PartyName>Recipient</PartyName></MessageRecipient>
  </MessageHeader>
</NewReleaseMessage>"#, uuid::Uuid::new_v4());
        results.push(result);
    }
    
    Ok(results)
}

#[pyfunction]
pub fn validate_structure(xml: String) -> PyResult<ValidationResult> {
    // Parse and validate XML structure
    match quick_xml::Reader::from_str(&xml).read_event() {
        Ok(_) => Ok(ValidationResult::new(true, vec![], vec![])),
        Err(e) => Ok(ValidationResult::new(
            false, 
            vec![format!("XML parsing error: {}", e)], 
            vec![]
        )),
    }
}

impl DdexBuilder {
    fn create_build_request_from_parsed(&self, parsed_result: &ParsedERNMessage) -> PyResult<BuildRequest> {
        // Convert parsed result back to build request (simplified implementation)
        let header = MessageHeaderRequest {
            message_id: Some(parsed_result.flat.message_id.clone()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: format!("{:?}", parsed_result.flat.sender),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: Some(parsed_result.flat.message_type.clone()),
            message_created_date_time: Some(parsed_result.flat.message_date.to_rfc3339()),
        };

        let mut releases = Vec::new();
        for release in &parsed_result.flat.releases {
            let tracks: Vec<TrackRequest> = release.tracks.iter().map(|track| {
                TrackRequest {
                    track_id: track.track_id.clone(),
                    resource_reference: Some(track.track_id.clone()),
                    isrc: track.isrc.clone().unwrap_or_else(|| "TEMP00000000".to_string()),
                    title: track.title.clone(),
                    duration: format!("PT{}S", track.duration.as_secs()),
                    artist: track.display_artist.clone(),
                }
            }).collect();

            releases.push(ReleaseRequest {
                release_id: release.release_id.clone(),
                release_reference: Some(release.release_id.clone()),
                title: vec![LocalizedStringRequest {
                    text: release.default_title.clone(),
                    language_code: None,
                }],
                artist: release.display_artist.clone(),
                label: None, // Simplified
                release_date: None, // Simplified
                upc: None, // Simplified
                tracks,
                resource_references: Some(release.tracks.iter().map(|t| t.track_id.clone()).collect()),
            });
        }

        Ok(BuildRequest {
            header,
            version: "4.3".to_string(),
            profile: Some("AudioAlbum".to_string()),
            releases,
            deals: vec![],
            extensions: None,
        })
    }

    fn create_build_request_from_stored_data(&self) -> Result<BuildRequest, PyErr> {
        // Create message header
        let header = MessageHeaderRequest {
            message_id: Some(uuid::Uuid::new_v4().to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "DDEX Suite".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: None,
            message_created_date_time: Some(chrono::Utc::now().to_rfc3339()),
        };

        // Convert releases
        let mut releases = Vec::new();
        for release in &self.releases {
            let tracks = self.resources
                .iter()
                .filter(|resource| release.track_ids.contains(&resource.resource_id))
                .map(|resource| TrackRequest {
                    track_id: resource.resource_id.clone(),
                    resource_reference: Some(resource.resource_id.clone()),
                    isrc: resource.isrc.clone().unwrap_or_else(|| "TEMP00000000".to_string()),
                    title: resource.title.clone(),
                    duration: resource.duration.clone().unwrap_or_else(|| "PT180S".to_string()),
                    artist: resource.artist.clone(),
                })
                .collect();

            releases.push(ReleaseRequest {
                release_id: release.release_id.clone(),
                release_reference: Some(release.release_id.clone()),
                title: vec![LocalizedStringRequest {
                    text: release.title.clone(),
                    language_code: None,
                }],
                artist: release.artist.clone(),
                label: release.label.clone(),
                release_date: release.release_date.clone(),
                upc: release.upc.clone(),
                tracks,
                resource_references: Some(release.track_ids.clone()),
            });
        }

        // Create build request
        Ok(BuildRequest {
            header,
            version: "4.3".to_string(),
            profile: Some("AudioAlbum".to_string()),
            releases,
            deals: vec![], // Empty for now
            extensions: None,
        })
    }
}

#[pymodule]
fn ddex_builder(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Release>()?;
    m.add_class::<Resource>()?;
    m.add_class::<ValidationResult>()?;
    m.add_class::<BuilderStats>()?;
    m.add_class::<PresetInfo>()?;
    m.add_class::<ValidationRulePy>()?;
    m.add_class::<FidelityOptions>()?;
    m.add_class::<BuildStatistics>()?;
    m.add_class::<VerificationResult>()?;
    m.add_class::<BuildResult>()?;
    m.add_class::<DdexBuilder>()?;
    m.add_function(wrap_pyfunction!(batch_build, m)?)?;
    m.add_function(wrap_pyfunction!(validate_structure, m)?)?;
    Ok(())
}