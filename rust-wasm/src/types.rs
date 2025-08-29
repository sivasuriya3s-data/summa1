use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamConfig {
    pub name: String,
    pub formats: Vec<String>,
    pub max_sizes: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct DocumentInfo {
    pub name: String,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversionError {
    pub message: String,
    pub code: String,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ConversionError {}