use crate::types::*;
use lopdf::{Document as PdfDocument, Object, Stream};
use pdf_writer::{Pdf, Ref, writers::PageTreeWriter};
use image::DynamicImage;
use std::io::Cursor;

pub struct PdfProcessor;

impl PdfProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Optimize existing PDF by removing unnecessary elements and compressing
    pub async fn optimize_pdf(&self, content: &[u8]) -> Result<Vec<u8>, ConversionError> {
        match PdfDocument::load_mem(content) {
            Ok(mut doc) => {
                // Remove unnecessary objects
                self.remove_unused_objects(&mut doc)?;
                
                // Compress streams
                self.compress_streams(&mut doc)?;
                
                // Save optimized PDF
                let mut output = Vec::new();
                doc.save_to(&mut output)
                    .map_err(|e| ConversionError::Pdf(format!("Failed to save optimized PDF: {}", e)))?;
                
                log::info!("PDF optimized: {} -> {} bytes ({:.1}% reduction)", 
                    content.len(), 
                    output.len(),
                    (1.0 - output.len() as f64 / content.len() as f64) * 100.0
                );
                
                Ok(output)
            }
            Err(e) => {
                log::warn!("PDF optimization failed, returning original: {}", e);
                Ok(content.to_vec())
            }
        }
    }

    /// Create PDF from image with proper sizing
    pub async fn create_pdf_from_image(&self, image_content: &[u8], target_size: Option<u64>) -> Result<Vec<u8>, ConversionError> {
        let img = image::load_from_memory(image_content)?;
        let (width, height) = img.dimensions();
        
        // Convert to RGB for PDF embedding
        let rgb_img = img.to_rgb8();
        
        // Create new PDF document
        let mut pdf = Pdf::new();
        
        // Calculate page size (A4 proportions or image proportions)
        let (page_width, page_height) = self.calculate_page_size(width, height);
        
        // Create page
        let page_id = pdf.alloc_ref();
        let mut page = pdf.page(page_id);
        page.media_box([0.0, 0.0, page_width, page_height]);
        page.parent(pdf.pages_id());
        
        // Create image XObject
        let image_id = pdf.alloc_ref();
        let mut image_obj = pdf.image_xobject(image_id);
        image_obj.width(width as i32);
        image_obj.height(height as i32);
        image_obj.color_space().device_rgb();
        image_obj.bits_per_component(8);
        image_obj.data(rgb_img.as_raw());
        image_obj.finish();
        
        // Create content stream
        let content_id = pdf.alloc_ref();
        page.contents(content_id);
        page.finish();
        
        let mut content = pdf.content_stream(content_id);
        content.save_state();
        content.transform([page_width, 0.0, 0.0, page_height, 0.0, 0.0]);
        content.x_object(image_id);
        content.restore_state();
        content.finish();
        
        // Create page tree
        let mut page_tree = PageTreeWriter::new(&mut pdf);
        page_tree.add_page(page_id);
        page_tree.finish();
        
        let pdf_bytes = pdf.finish();
        
        // Check size constraint if specified
        if let Some(max_size) = target_size {
            if pdf_bytes.len() as u64 > max_size {
                // Try with compressed image
                return self.create_compressed_pdf_from_image(image_content, max_size).await;
            }
        }
        
        log::info!("Created PDF from image: {} bytes", pdf_bytes.len());
        Ok(pdf_bytes)
    }

    /// Create PDF with compressed image to meet size requirements
    async fn create_compressed_pdf_from_image(&self, image_content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        let img = image::load_from_memory(image_content)?;
        let mut quality = 85u8;
        
        for _ in 0..5 {
            // Compress image first
            let mut compressed_img = Vec::new();
            let mut cursor = Cursor::new(&mut compressed_img);
            img.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(quality))?;
            
            // Create PDF with compressed image
            let pdf_result = self.create_pdf_from_image(&compressed_img, None).await?;
            
            if pdf_result.len() as u64 <= max_size {
                log::info!("Created compressed PDF: {} bytes with {}% JPEG quality", pdf_result.len(), quality);
                return Ok(pdf_result);
            }
            
            quality = std::cmp::max(20, quality - 15);
        }
        
        Err(ConversionError::CompressionFailed {
            message: format!("Could not create PDF under {} bytes", max_size),
        })
    }

    /// Extract first page of PDF as image
    pub async fn pdf_to_image(&self, content: &[u8], format: ImageFormat, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        // This is a simplified implementation
        // In production, you'd use a proper PDF rendering library like pdf2image
        
        match PdfDocument::load_mem(content) {
            Ok(doc) => {
                // For now, create a placeholder image representing the PDF
                let placeholder = self.create_pdf_placeholder_image()?;
                
                match format {
                    ImageFormat::Jpeg => {
                        let mut output = Vec::new();
                        let mut cursor = Cursor::new(&mut output);
                        placeholder.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(85))?;
                        
                        if output.len() as u64 <= max_size {
                            Ok(output)
                        } else {
                            // Use image processor to compress further
                            let processor = crate::image_processor::ImageProcessor::new();
                            processor.compress_jpeg_to_size(&output, max_size).await
                        }
                    }
                    ImageFormat::Png => {
                        let mut output = Vec::new();
                        let mut cursor = Cursor::new(&mut output);
                        placeholder.write_to(&mut cursor, image::ImageOutputFormat::Png)?;
                        
                        if output.len() as u64 <= max_size {
                            Ok(output)
                        } else {
                            let processor = crate::image_processor::ImageProcessor::new();
                            processor.compress_png_to_size(&output, max_size).await
                        }
                    }
                    _ => Err(ConversionError::UnsupportedFormat {
                        format: format!("PDF to {:?}", format),
                    }),
                }
            }
            Err(e) => Err(ConversionError::Pdf(format!("Failed to load PDF: {}", e))),
        }
    }

    /// Remove unused objects from PDF to reduce size
    fn remove_unused_objects(&self, doc: &mut PdfDocument) -> Result<(), ConversionError> {
        // Remove unused references and compress
        doc.prune_objects();
        Ok(())
    }

    /// Compress streams in PDF
    fn compress_streams(&self, doc: &mut PdfDocument) -> Result<(), ConversionError> {
        // Iterate through objects and compress streams
        for (_, object) in doc.objects.iter_mut() {
            if let Object::Stream(ref mut stream) = object {
                // Apply compression if not already compressed
                if !stream.dict.has(b"Filter") {
                    // Add FlateDecode filter for compression
                    stream.compress();
                }
            }
        }
        Ok(())
    }

    /// Calculate appropriate page size for PDF
    fn calculate_page_size(&self, img_width: u32, img_height: u32) -> (f32, f32) {
        // A4 size in points (72 DPI)
        const A4_WIDTH: f32 = 595.0;
        const A4_HEIGHT: f32 = 842.0;
        
        let img_ratio = img_width as f32 / img_height as f32;
        let a4_ratio = A4_WIDTH / A4_HEIGHT;
        
        if img_ratio > a4_ratio {
            // Image is wider, fit to width
            (A4_WIDTH, A4_WIDTH / img_ratio)
        } else {
            // Image is taller, fit to height
            (A4_HEIGHT * img_ratio, A4_HEIGHT)
        }
    }

    /// Create placeholder image for PDF content
    fn create_pdf_placeholder_image(&self) -> Result<DynamicImage, ConversionError> {
        use image::{Rgb, RgbImage};
        
        let width = 800;
        let height = 600;
        let mut img = RgbImage::new(width, height);
        
        // Fill with light gray background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([240, 240, 240]);
        }
        
        // Add border
        for x in 0..width {
            img.put_pixel(x, 0, Rgb([100, 100, 100]));
            img.put_pixel(x, height - 1, Rgb([100, 100, 100]));
        }
        for y in 0..height {
            img.put_pixel(0, y, Rgb([100, 100, 100]));
            img.put_pixel(width - 1, y, Rgb([100, 100, 100]));
        }
        
        // Add "PDF" text representation (simplified)
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw simple "PDF" indicator
        for x in (center_x - 50)..(center_x + 50) {
            for y in (center_y - 20)..(center_y + 20) {
                if x < width && y < height {
                    img.put_pixel(x, y, Rgb([200, 200, 200]));
                }
            }
        }
        
        Ok(DynamicImage::ImageRgb8(img))
    }
}