// core/src/models/mod.rs
//! DDEX data models

pub mod common;
pub mod graph;
pub mod flat;
pub mod versions;  // Add this line to export the versions module
pub mod attributes;

pub use common::{Identifier, IdentifierType, LocalizedString};
pub use attributes::{AttributeMap, AttributeValue, QName, AttributeType, AttributeInheritance};

pub mod extensions;
pub use extensions::{Extensions, XmlFragment, ProcessingInstruction, Comment, CommentPosition};

pub mod validation;
pub use validation::{
    AttributeValidator, ValidationResult, AttributeValidationError,
    ValidationRule, ValidationPolicy, DependencyCondition,
};