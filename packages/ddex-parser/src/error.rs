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
    InvalidUtf8 {
        message: String,
    },
    SimpleXmlError(String),
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
            ParseError::InvalidUtf8 { message } => write!(f, "UTF-8 error: {}", message),
            ParseError::SimpleXmlError(msg) => write!(f, "Simple XML error: {}", msg),
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
        if let Some(doctype) = ddex_core::xml_refs::Doctype::from_io_error(&err) {
            return crate::xml_text::doctype_error(doctype);
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

/// A quick-xml error as a [`ParseError`]: an undefined entity or a DOCTYPE found by the
/// checked reader gets its own error, anything else is [`ParseError::XmlError`] with
/// quick-xml's message (which includes the position when quick-xml gives one). quick-xml
/// types are not part of the public API (before 0.6.0 this was a `From` impl).
pub(crate) fn xml_error(err: quick_xml::Error) -> ParseError {
    if let quick_xml::Error::Io(io) = &err {
        if let Some(entity) = ddex_core::xml_refs::UndefinedEntity::from_io_error(io) {
            return crate::xml_text::undefined_entity(entity);
        }
        if let Some(doctype) = ddex_core::xml_refs::Doctype::from_io_error(io) {
            return crate::xml_text::doctype_error(doctype);
        }
    }
    ParseError::XmlError(err.to_string())
}

/// A quick-xml attribute error as [`ParseError::XmlError`] (before 0.6.0 a `From` impl).
pub(crate) fn attr_error(err: quick_xml::events::attributes::AttrError) -> ParseError {
    ParseError::XmlError(format!("Attribute parsing error: {}", err))
}

impl From<String> for ParseError {
    fn from(err: String) -> Self {
        ParseError::SimpleXmlError(err)
    }
}
