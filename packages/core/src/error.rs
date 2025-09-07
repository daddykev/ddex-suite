use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    XmlError(String),
    
    #[error("Invalid DDEX structure: {0}")]
    StructureError(String),
    
    #[error("Version detection failed: {0}")]
    VersionError(String),
    
    #[error("Reference resolution failed: {0}")]
    ReferenceError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;

// Convenience re-export
pub use ParseError as Error;
