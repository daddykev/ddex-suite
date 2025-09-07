//! Localized string support

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedString {
    pub text: String,
    pub language_code: Option<String>,
    pub script: Option<String>,
}

impl LocalizedString {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            language_code: None,
            script: None,
        }
    }
}
