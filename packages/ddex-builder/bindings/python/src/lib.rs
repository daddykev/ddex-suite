use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyAny};
use std::collections::HashMap;

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

        // Generate a basic DDEX-like XML structure for demonstration
        let xml_output = self.generate_placeholder_xml()?;
        
        self.stats.last_build_size_bytes = xml_output.len() as f64;
        self.stats.total_build_time_ms += start_time.elapsed().as_millis() as f64;

        Ok(xml_output)
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

    #[pyo3(signature = (df, profile="AudioAlbum"))]
    pub fn from_dataframe(&mut self, df: &PyAny, profile: &str) -> PyResult<()> {
        // Import pandas functionality through PyO3
        let pandas = PyModule::import(df.py(), "pandas")?;
        let pd_dataframe = pandas.getattr("DataFrame")?;
        
        // Check if the input is a pandas DataFrame
        if !df.is_instance(&pd_dataframe)? {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Input must be a pandas DataFrame"
            ));
        }
        
        // Convert DataFrame to dict for easier processing
        let df_dict = df.call_method0("to_dict")?;
        let records = df.call_method1("to_dict", ("records",))?;
        
        // Process each row as a release or resource
        let records_list: &PyList = records.downcast()?;
        
        for item in records_list.iter() {
            let record: &PyDict = item.downcast()?;
            
            // Determine if this row represents a release or resource
            // This is a simplified approach - in practice you'd have more sophisticated logic
            if record.contains("release_id")? {
                let release = self.dict_to_release(record)?;
                self.add_release(release);
            } else if record.contains("resource_id")? {
                let resource = self.dict_to_resource(record)?;
                self.add_resource(resource);
            }
        }
        
        Ok(())
    }

    fn generate_placeholder_xml(&self) -> PyResult<String> {
        // Generate a basic DDEX-like XML structure for demonstration
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">"#);
        xml.push('\n');
        
        // Message header
        xml.push_str("  <MessageHeader>\n");
        xml.push_str(&format!("    <MessageId>{}</MessageId>\n", uuid::Uuid::new_v4()));
        xml.push_str("    <MessageSender>\n");
        xml.push_str("      <PartyName>DDEX Suite</PartyName>\n");
        xml.push_str("    </MessageSender>\n");
        xml.push_str("    <MessageRecipient>\n");
        xml.push_str("      <PartyName>Recipient</PartyName>\n");
        xml.push_str("    </MessageRecipient>\n");
        xml.push_str(&format!("    <MessageCreatedDateTime>{}</MessageCreatedDateTime>\n", chrono::Utc::now().to_rfc3339()));
        xml.push_str("  </MessageHeader>\n");

        // Releases
        for release in &self.releases {
            xml.push_str("  <ReleaseList>\n");
            xml.push_str("    <Release>\n");
            xml.push_str(&format!("      <ReleaseId>{}</ReleaseId>\n", release.release_id));
            xml.push_str(&format!("      <Title>{}</Title>\n", release.title));
            xml.push_str(&format!("      <Artist>{}</Artist>\n", release.artist));
            if let Some(ref label) = release.label {
                xml.push_str(&format!("      <Label>{}</Label>\n", label));
            }
            xml.push_str("    </Release>\n");
            xml.push_str("  </ReleaseList>\n");
        }

        // Resources
        for resource in &self.resources {
            xml.push_str("  <ResourceList>\n");
            xml.push_str("    <SoundRecording>\n");
            xml.push_str(&format!("      <ResourceId>{}</ResourceId>\n", resource.resource_id));
            xml.push_str(&format!("      <Title>{}</Title>\n", resource.title));
            xml.push_str(&format!("      <Artist>{}</Artist>\n", resource.artist));
            if let Some(ref isrc) = resource.isrc {
                xml.push_str(&format!("      <ISRC>{}</ISRC>\n", isrc));
            }
            xml.push_str("    </SoundRecording>\n");
            xml.push_str("  </ResourceList>\n");
        }
        
        xml.push_str("</NewReleaseMessage>\n");
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

#[pymodule]
fn ddex_builder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Release>()?;
    m.add_class::<Resource>()?;
    m.add_class::<ValidationResult>()?;
    m.add_class::<BuilderStats>()?;
    m.add_class::<PresetInfo>()?;
    m.add_class::<ValidationRulePy>()?;
    m.add_class::<DdexBuilder>()?;
    m.add_function(wrap_pyfunction!(batch_build, m)?)?;
    m.add_function(wrap_pyfunction!(validate_structure, m)?)?;
    Ok(())
}