use std::fmt;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    MissingField(String),
    InvalidValue {
        field: String,
        value: String,
    },
    XmlError(String),
    StreamError(StreamError),
    InvalidUtf8 {
        message: String,
    },
    SimpleXmlError(String),
    ConversionError {
        from: String,
        to: String,
        message: String,
    },
    IoError(String),
    Timeout {
        message: String,
    },
    DepthLimitExceeded {
        depth: usize,
        limit: usize,
    },
    SecurityViolation {
        message: String,
    },
    MalformedXml {
        message: String,
        position: usize,
    },
    MismatchedTags {
        expected: String,
        found: String,
        position: usize,
    },
    UnexpectedClosingTag {
        tag: String,
        position: usize,
    },
    InvalidAttribute {
        message: String,
        position: usize,
    },
    UnclosedTags {
        tags: Vec<String>,
        position: usize,
    },
}

#[derive(Debug, Clone)]
pub enum StreamError {
    MissingReleaseReference,
    MissingResourceReference,
    MissingDealReference,
    MissingPartyReference,
    IncompleteData(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::MissingField(field) => {
                write!(f, "Required DDEX field missing: {}", field)
            }
            ParseError::InvalidValue { field, value } => {
                write!(f, "Invalid value '{}' for field '{}'", value, field)
            }
            ParseError::XmlError(msg) => write!(f, "XML parsing error: {}", msg),
            ParseError::StreamError(e) => write!(f, "Streaming error: {:?}", e),
            ParseError::InvalidUtf8 { message } => write!(f, "UTF-8 error: {}", message),
            ParseError::SimpleXmlError(msg) => write!(f, "Simple XML error: {}", msg),
            ParseError::ConversionError { from, to, message } => {
                write!(f, "Conversion error from {} to {}: {}", from, to, message)
            }
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::Timeout { message } => write!(f, "Timeout: {}", message),
            ParseError::DepthLimitExceeded { depth, limit } => {
                write!(f, "Depth limit exceeded: {} > {}", depth, limit)
            }
            ParseError::SecurityViolation { message } => {
                write!(f, "Security violation: {}", message)
            }
            ParseError::MalformedXml { message, position } => {
                write!(f, "Malformed XML at position {}: {}", position, message)
            }
            ParseError::MismatchedTags {
                expected,
                found,
                position,
            } => write!(
                f,
                "Mismatched tags at position {}: expected '{}', found '{}'",
                position, expected, found
            ),
            ParseError::UnexpectedClosingTag { tag, position } => write!(
                f,
                "Unexpected closing tag '{}' at position {}",
                tag, position
            ),
            ParseError::InvalidAttribute { message, position } => {
                write!(f, "Invalid attribute at position {}: {}", position, message)
            }
            ParseError::UnclosedTags { tags, position } => {
                write!(f, "Unclosed tags at position {}: {:?}", position, tags)
            }
        }
    }
}

impl std::error::Error for ParseError {}

// From implementations for error conversion
impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        // A checked input (`ddex_core::xml_refs::CheckedRead`) fails its reads after a
        // reference to an undefined entity.
        if let Some(entity) = ddex_core::xml_refs::UndefinedEntity::from_io_error(&err) {
            return crate::xml_text::undefined_entity(entity);
        }
        ParseError::IoError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for ParseError {
    fn from(err: std::str::Utf8Error) -> Self {
        ParseError::InvalidUtf8 {
            message: err.to_string(),
        }
    }
}

impl From<quick_xml::Error> for ParseError {
    fn from(err: quick_xml::Error) -> Self {
        if let quick_xml::Error::Io(io) = &err {
            if let Some(entity) = ddex_core::xml_refs::UndefinedEntity::from_io_error(io) {
                return crate::xml_text::undefined_entity(entity);
            }
        }
        ParseError::XmlError(err.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for ParseError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        ParseError::XmlError(format!("Attribute parsing error: {}", err))
    }
}

impl From<String> for ParseError {
    fn from(err: String) -> Self {
        ParseError::SimpleXmlError(err)
    }
}
