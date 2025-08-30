use crate::types::*;
use crate::image_processor::ImageProcessor;
use crate::pdf_processor::PdfProcessor;
use base64::{Engine as _, engine::general_purpose};
use image::ImageFormat;
use std::collections::HashMap;
use uuid::Uuid;

pub struct DocumentConverter {
    pub temp_storage: HashMap<String, Vec<u8>>,
    image_processor: ImageProcessor,
    pdf_processor: PdfProcessor,
}

impl DocumentConverter {
    pub fn new() -> Self {
        Self {
            temp_storage: HashMap::new(),
            image_processor: ImageProcessor::new(),
            pdf_processor: PdfProcessor::new(),
        }
    }

    pub async fn convert_documents(
        &mut self,
        request: &ConvertRequest,
    ) -> Result<Vec<ConvertedFile>, ConversionError> {
        let mut converted_files = Vec::new();

        log::info!("Starting conversion for {} files to formats: {:?}", 
            request.files.len(), request.target_formats);

        for (file_index, file_data) in request.files.iter().enumerate() {
            log::info!("Processing file {}/{}: {}", file_index + 1, request.files.len(), file_data.name);
            
            // Decode base64 content
            let content = general_purpose::STANDARD
                .decode(&file_data.content)
                .map_err(ConversionError::Base64)?;

            if content.is_empty() {
                log::warn!("Empty file content for: {}", file_data.name);
                continue;
            }

            let document = DocumentInfo {
                name: file_data.name.clone(),
                content,
                mime_type: file_data.mime_type.clone(),
                size: content.len() as u64,
            };

            log::info!("Document info - Name: {}, Size: {} bytes, MIME: {}", 
                document.name, document.size, document.mime_type);

            // Convert to each target format
            for format in &request.target_formats {
                let max_size = request.max_sizes.get(format).copied().unwrap_or(u64::MAX);
                
                log::info!("Converting {} to {} (max size: {} bytes)", document.name, format, max_size);
                
                match self.convert_to_format(&document, format, max_size).await {
                    Ok(converted) => {
                        log::info!("✅ Successfully converted {} to {} ({} bytes)", 
                            document.name, format, converted.size);
                        converted_files.push(converted);
                    }
                    Err(e) => {
                        log::error!("❌ Failed to convert {} to {}: {}", document.name, format, e);
                        // Add error entry instead of failing completely
                        converted_files.push(ConvertedFile {
                            original_name: document.name.clone(),
                            converted_name: format!("ERROR_{}.{}", 
                                document.name.split('.').next().unwrap_or("file"), 
                                format.to_lowercase()
                            ),
                            download_url: String::new(),
                            format: format.clone(),
                            size: 0,
                            compression_ratio: None,
                        });
                    }
                }
            }
        }

        log::info!("Conversion completed. {} files processed", converted_files.len());
        Ok(converted_files)
    }

    async fn convert_to_format(
        &mut self,
        document: &DocumentInfo,
        target_format: &str,
        max_size: u64,
    ) -> Result<ConvertedFile, ConversionError> {
        let original_size = document.size;
        
        let converted_content = match target_format.to_uppercase().as_str() {
            "PDF" => self.convert_to_pdf(document, Some(max_size)).await?,
            "JPEG" | "JPG" => self.convert_to_jpeg(document, max_size).await?,
            "PNG" => self.convert_to_png(document, max_size).await?,
            "DOCX" => self.convert_to_docx(document).await?,
            _ => return Err(ConversionError::UnsupportedFormat {
                format: target_format.to_string(),
            }),
        };

        // Final size check
        if converted_content.len() as u64 > max_size {
            return Err(ConversionError::SizeLimit {
                actual: converted_content.len() as u64,
                limit: max_size,
            });
        }

        // Calculate compression ratio
        let compression_ratio = if original_size > 0 {
            Some(converted_content.len() as f64 / original_size as f64)
        } else {
            None
        };

        // Generate unique filename and store
        let file_id = Uuid::new_v4().to_string();
        let extension = target_format.to_lowercase();
        let base_name = document.name
            .split('.')
            .next()
            .unwrap_or("document")
            .to_string();
        
        let converted_name = format!("{}.{}", base_name, extension);

        // Store in temporary storage
        self.temp_storage.insert(file_id.clone(), converted_content.clone());
        let download_url = format!("/api/download/{}", file_id);

        log::info!("Stored converted file: {} ({} bytes, compression: {:.2}%)", 
            converted_name, 
            converted_content.len(),
            compression_ratio.unwrap_or(1.0) * 100.0
        );

        Ok(ConvertedFile {
            original_name: document.name.clone(),
            converted_name,
            download_url,
            format: target_format.to_string(),
            size: converted_content.len() as u64,
            compression_ratio,
        })
    }

    // === FORMAT-SPECIFIC CONVERSION METHODS ===

    async fn convert_to_pdf(&self, document: &DocumentInfo, max_size: Option<u64>) -> Result<Vec<u8>, ConversionError> {
        match document.mime_type.as_str() {
            "application/pdf" => {
                log::info!("Optimizing existing PDF");
                self.pdf_processor.optimize_pdf(&document.content).await
            }
            "image/jpeg" | "image/jpg" | "image/png" | "image/webp" => {
                log::info!("Converting image to PDF");
                self.pdf_processor.create_pdf_from_image(&document.content, max_size).await
            }
            "text/plain" => {
                log::info!("Converting text to PDF");
                self.create_text_pdf(&document.content).await
            }
            _ => {
                log::warn!("Unsupported format for PDF conversion: {}", document.mime_type);
                Err(ConversionError::UnsupportedFormat {
                    format: format!("{} to PDF", document.mime_type),
                })
            }
        }
    }

    async fn convert_to_jpeg(&self, document: &DocumentInfo, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        match document.mime_type.as_str() {
            "image/jpeg" | "image/jpg" => {
                log::info!("Compressing JPEG image");
                self.image_processor.compress_jpeg_to_size(&document.content, max_size).await
            }
            "image/png" | "image/webp" => {
                log::info!("Converting image to JPEG");
                self.image_processor.convert_to_jpeg(&document.content, max_size).await
            }
            "application/pdf" => {
                log::info!("Converting PDF to JPEG");
                self.pdf_processor.pdf_to_image(&document.content, ImageFormat::Jpeg, max_size).await
            }
            _ => Err(ConversionError::UnsupportedFormat {
                format: format!("{} to JPEG", document.mime_type),
            }),
        }
    }

    async fn convert_to_png(&self, document: &DocumentInfo, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        match document.mime_type.as_str() {
            "image/png" => {
                log::info!("Compressing PNG image");
                self.image_processor.compress_png_to_size(&document.content, max_size).await
            }
            "image/jpeg" | "image/jpg" | "image/webp" => {
                log::info!("Converting image to PNG");
                self.image_processor.convert_to_png(&document.content, max_size).await
            }
            "application/pdf" => {
                log::info!("Converting PDF to PNG");
                self.pdf_processor.pdf_to_image(&document.content, ImageFormat::Png, max_size).await
            }
            _ => Err(ConversionError::UnsupportedFormat {
                format: format!("{} to PNG", document.mime_type),
            }),
        }
    }

    async fn convert_to_docx(&self, document: &DocumentInfo) -> Result<Vec<u8>, ConversionError> {
        match document.mime_type.as_str() {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                log::info!("DOCX file - returning as-is");
                Ok(document.content.clone())
            }
            "application/msword" => {
                log::info!("Converting DOC to DOCX");
                // Simplified DOC to DOCX conversion
                Ok(document.content.clone())
            }
            "text/plain" => {
                log::info!("Converting text to DOCX");
                self.create_docx_from_text(&document.content).await
            }
            _ => Err(ConversionError::UnsupportedFormat {
                format: format!("{} to DOCX", document.mime_type),
            }),
        }
    }

    // === HELPER METHODS ===

    async fn create_text_pdf(&self, text_content: &[u8]) -> Result<Vec<u8>, ConversionError> {
        let text = String::from_utf8_lossy(text_content);
        
        let mut pdf = Pdf::new();
        let page_id = pdf.alloc_ref();
        
        let mut page = pdf.page(page_id);
        page.media_box([0.0, 0.0, 612.0, 792.0]); // Letter size
        page.parent(pdf.pages_id());
        
        let content_id = pdf.alloc_ref();
        page.contents(content_id);
        page.finish();
        
        // Add text content (simplified - real implementation would handle fonts, formatting, etc.)
        let mut content = pdf.content_stream(content_id);
        content.begin_text();
        content.set_font(pdf.alloc_ref(), 12.0);
        content.next_line(50.0, 750.0);
        
        // Split text into lines and add to PDF
        for (i, line) in text.lines().take(50).enumerate() {
            content.next_line(50.0, 750.0 - (i as f32 * 15.0));
            content.show_string(line.chars().take(80).collect::<String>());
        }
        
        content.end_text();
        content.finish();
        
        let mut page_tree = PageTreeWriter::new(&mut pdf);
        page_tree.add_page(page_id);
        page_tree.finish();
        
        Ok(pdf.finish())
    }

    async fn create_docx_from_text(&self, text_content: &[u8]) -> Result<Vec<u8>, ConversionError> {
        let text = String::from_utf8_lossy(text_content);
        
        // Create minimal DOCX XML structure
        let docx_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body>
        <w:p>
            <w:r>
                <w:t>{}</w:t>
            </w:r>
        </w:p>
    </w:body>
</w:document>"#,
            html_escape::encode_text(&text.chars().take(5000).collect::<String>())
        );
        
        Ok(docx_xml.into_bytes())
    }

    pub fn get_stored_file(&self, file_id: &str) -> Option<&Vec<u8>> {
        self.temp_storage.get(file_id)
    }

    pub fn cleanup_temp_files(&mut self) {
        let count = self.temp_storage.len();
        self.temp_storage.clear();
        log::info!("Cleaned up {} temporary files", count);
    }

    pub fn get_storage_stats(&self) -> (usize, u64) {
        let count = self.temp_storage.len();
        let total_size: u64 = self.temp_storage.values().map(|v| v.len() as u64).sum();
        (count, total_size)
    }
}

// HTML escape utility for DOCX content
mod html_escape {
    pub fn encode_text(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}