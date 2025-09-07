use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Extensions {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

impl Extensions {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }
    
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}