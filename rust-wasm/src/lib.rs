//! Document Converter Library
//! 
//! This library provides document conversion capabilities for competitive exam applications.
//! It supports converting between various formats (PDF, JPEG, PNG, DOCX) with size optimization.

pub mod converter;
pub mod types;

pub use converter::DocumentConverter;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_converter_creation() {
        let converter = DocumentConverter::new();
        // Basic test to ensure converter can be created
        assert!(true);
    }
}