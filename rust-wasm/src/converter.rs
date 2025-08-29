use crate::types::*;
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;
use uuid::Uuid;

pub struct DocumentConverter {
    temp_storage: HashMap<String, Vec<u8>>,
}

impl DocumentConverter {
    pub fn new() -> Self {
        Self {
            temp_storage: HashMap::new(),
        }
    }

    pub async fn convert_documents(
        &self,
        request: &crate::ConvertRequest,
    ) -> Result<Vec<crate::ConvertedFile>, ConversionError> {
        let mut converted_files = Vec::new();

        for file_data in &request.files {
            // Decode base64 content
            let content = general_purpose::STANDARD
                .decode(&file_data.content)
                .map_err(|e| ConversionError {
                    message: format!("Failed to decode file content: {}", e),
                    code: "DECODE_ERROR".to_string(),
                })?;

            let document = DocumentInfo {
                name: file_data.name.clone(),
                content,
                mime_type: file_data.mime_type.clone(),
                size: content.len() as u64,
            };

            // Convert to each target format
            for format in &request.target_formats {
                let max_size = request.max_sizes.get(format).copied().unwrap_or(u64::MAX);
                
                let converted = self.convert_to_format(&document, format, max_size).await?;
                converted_files.push(converted);
            }
        }

        Ok(converted_files)
    }

    async fn convert_to_format(
        &self,
        document: &DocumentInfo,
        target_format: &str,
        max_size: u64,
    ) -> Result<crate::ConvertedFile, ConversionError> {
        let converted_content = match target_format.to_uppercase().as_str() {
            "PDF" => self.convert_to_pdf(document).await?,
            "JPEG" | "JPG" => self.convert_to_jpeg(document, max_size).await?,
            "PNG" => self.convert_to_png(document, max_size).await?,
            "DOCX" => self.convert_to_docx(document).await?,
            _ => return Err(ConversionError {
                message: format!("Unsupported format: {}", target_format),
                code: "UNSUPPORTED_FORMAT".to_string(),
            }),
        };

        // Check size constraint
        if converted_content.len() as u64 > max_size {
            return Err(ConversionError {
                message: format!(
                    "Converted file size ({} bytes) exceeds maximum allowed size ({} bytes)",
                    converted_content.len(),
                    max_size
                ),
                code: "SIZE_LIMIT_EXCEEDED".to_string(),
            });
        }

        // Generate unique filename and store
        let file_id = Uuid::new_v4().to_string();
        let extension = target_format.to_lowercase();
        let converted_name = format!(
            "{}.{}",
            document.name.rsplit('.').nth(1).unwrap_or(&document.name),
            extension
        );

        // In a real implementation, you would store this in a file system or cloud storage
        // For now, we'll create a mock download URL
        let download_url = format!("/api/download/{}", file_id);

        Ok(crate::ConvertedFile {
            original_name: document.name.clone(),
            converted_name,
            download_url,
            format: target_format.to_string(),
            size: converted_content.len() as u64,
        })
    }

    async fn convert_to_pdf(&self, document: &DocumentInfo) -> Result<Vec<u8>, ConversionError> {
        // Mock PDF conversion - in reality, you'd use a PDF library
        // For images, you might embed them in a PDF
        // For documents, you might convert them to PDF format
        
        match document.mime_type.as_str() {
            "application/pdf" => Ok(document.content.clone()),
            "image/jpeg" | "image/jpg" | "image/png" => {
                // Mock: Create a simple PDF with the image
                self.create_pdf_with_image(&document.content).await
            }
            _ => Err(ConversionError {
                message: "Cannot convert this file type to PDF".to_string(),
                code: "CONVERSION_NOT_SUPPORTED".to_string(),
            }),
        }
    }

    async fn convert_to_jpeg(&self, document: &DocumentInfo, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        match document.mime_type.as_str() {
            "image/jpeg" | "image/jpg" => {
                self.compress_image(&document.content, "jpeg", max_size).await
            }
            "image/png" => {
                self.convert_png_to_jpeg(&document.content, max_size).await
            }
            "application/pdf" => {
                self.pdf_to_jpeg(&document.content, max_size).await
            }
            _ => Err(ConversionError {
                message: "Cannot convert this file type to JPEG".to_string(),
                code: "CONVERSION_NOT_SUPPORTED".to_string(),
            }),
        }
    }

    async fn convert_to_png(&self, document: &DocumentInfo, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        match document.mime_type.as_str() {
            "image/png" => {
                self.compress_image(&document.content, "png", max_size).await
            }
            "image/jpeg" | "image/jpg" => {
                self.convert_jpeg_to_png(&document.content, max_size).await
            }
            _ => Err(ConversionError {
                message: "Cannot convert this file type to PNG".to_string(),
                code: "CONVERSION_NOT_SUPPORTED".to_string(),
            }),
        }
    }

    async fn convert_to_docx(&self, document: &DocumentInfo) -> Result<Vec<u8>, ConversionError> {
        // Mock DOCX conversion
        match document.mime_type.as_str() {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                Ok(document.content.clone())
            }
            _ => Err(ConversionError {
                message: "Cannot convert this file type to DOCX".to_string(),
                code: "CONVERSION_NOT_SUPPORTED".to_string(),
            }),
        }
    }

    // Helper methods (mock implementations)
    async fn create_pdf_with_image(&self, _image_content: &[u8]) -> Result<Vec<u8>, ConversionError> {
        // Mock PDF creation
        Ok(b"Mock PDF content with embedded image".to_vec())
    }

    async fn compress_image(&self, content: &[u8], _format: &str, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        // Mock image compression
        if content.len() as u64 <= max_size {
            Ok(content.to_vec())
        } else {
            // Simulate compression by reducing size
            let compression_ratio = max_size as f64 / content.len() as f64;
            let compressed_size = (content.len() as f64 * compression_ratio) as usize;
            Ok(content[..compressed_size.min(content.len())].to_vec())
        }
    }

    async fn convert_png_to_jpeg(&self, content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        // Mock PNG to JPEG conversion
        self.compress_image(content, "jpeg", max_size).await
    }

    async fn convert_jpeg_to_png(&self, content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        // Mock JPEG to PNG conversion
        self.compress_image(content, "png", max_size).await
    }

    async fn pdf_to_jpeg(&self, _content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        // Mock PDF to JPEG conversion
        let mock_jpeg = b"Mock JPEG content from PDF";
        if mock_jpeg.len() as u64 <= max_size {
            Ok(mock_jpeg.to_vec())
        } else {
            Err(ConversionError {
                message: "PDF to JPEG conversion resulted in file too large".to_string(),
                code: "SIZE_LIMIT_EXCEEDED".to_string(),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Starting Rust Document Converter on port 8002");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .route("/health", web::get().to(health))
            .route("/convert", web::post().to(convert_documents))
    })
    .bind("0.0.0.0:8002")?
    .run()
    .await
}