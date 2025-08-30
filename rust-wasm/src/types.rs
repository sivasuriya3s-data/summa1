use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),
    
    #[error("PDF processing error: {0}")]
    Pdf(String),
    
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    
    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("File size {actual} exceeds limit {limit}")]
    SizeLimit { actual: u64, limit: u64 },
    
    #[error("Invalid file content: {message}")]
    InvalidContent { message: String },
    
    #[error("Compression failed: {message}")]
    CompressionFailed { message: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct ConvertedFile {
    pub original_name: String,
    pub converted_name: String,
    pub download_url: String,
    pub format: String,
    pub size: u64,
    pub compression_ratio: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct FileData {
    pub name: String,
    pub content: String, // base64 encoded
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub files: Vec<FileData>,
    pub exam_type: String,
    pub target_formats: Vec<String>,
    pub max_sizes: HashMap<String, u64>,
}

#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub success: bool,
    pub files: Vec<ConvertedFile>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompressionSettings {
    pub quality: u8,        // 1-100 for JPEG
    pub png_compression: u8, // 0-9 for PNG
    pub max_iterations: u32, // Maximum compression attempts
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            quality: 85,
            png_compression: 6,
            max_iterations: 5,
        }
    }
}