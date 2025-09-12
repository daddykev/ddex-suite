//! Security module for DDEX Builder
//!
//! This module provides comprehensive security features including:
//! - Entity classification and validation
//! - Cross-platform path validation  
//! - XXE attack prevention
//! - Input sanitization

pub mod entity_classifier;
pub mod path_validator;

pub use entity_classifier::{
    EntityClassifier, EntityClass, Entity, ValidationResult, EntityMetrics,
    AttackType, ClassifierConfig, create_entity, create_parameter_entity, create_external_entity
};

pub use path_validator::{
    PathValidator, PathValidationConfig, ValidatedPath
};