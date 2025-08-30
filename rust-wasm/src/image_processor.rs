use crate::types::*;
use image::{DynamicImage, ImageFormat, ImageOutputFormat};
use std::io::Cursor;

pub struct ImageProcessor {
    compression_settings: CompressionSettings,
}

impl ImageProcessor {
    pub fn new() -> Self {
        Self {
            compression_settings: CompressionSettings::default(),
        }
    }

    pub fn with_settings(settings: CompressionSettings) -> Self {
        Self {
            compression_settings: settings,
        }
    }

    /// Compress JPEG image to meet size requirements
    pub async fn compress_jpeg_to_size(&self, content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        let img = image::load_from_memory(content)?;
        let mut quality = self.compression_settings.quality;
        let mut iterations = 0;

        while iterations < self.compression_settings.max_iterations {
            let compressed = self.encode_jpeg(&img, quality)?;
            
            if compressed.len() as u64 <= max_size || quality <= 10 {
                log::info!("JPEG compressed to {} bytes with {}% quality", compressed.len(), quality);
                return Ok(compressed);
            }
            
            // Reduce quality for next iteration
            quality = std::cmp::max(10, (quality as f32 * 0.85) as u8);
            iterations += 1;
        }

        // If still too large, try resizing
        self.resize_and_compress_jpeg(&img, max_size).await
    }

    /// Compress PNG image to meet size requirements
    pub async fn compress_png_to_size(&self, content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        let img = image::load_from_memory(content)?;
        
        // PNG is lossless, so we can only resize to reduce size
        let compressed = self.encode_png(&img)?;
        
        if compressed.len() as u64 <= max_size {
            log::info!("PNG size: {} bytes (within limit)", compressed.len());
            return Ok(compressed);
        }

        // Resize image to meet size requirements
        self.resize_and_compress_png(&img, max_size).await
    }

    /// Convert any image format to JPEG with size constraint
    pub async fn convert_to_jpeg(&self, content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        let img = image::load_from_memory(content)?;
        self.compress_jpeg_to_size(&self.encode_jpeg(&img, self.compression_settings.quality)?, max_size).await
    }

    /// Convert any image format to PNG with size constraint
    pub async fn convert_to_png(&self, content: &[u8], max_size: u64) -> Result<Vec<u8>, ConversionError> {
        let img = image::load_from_memory(content)?;
        self.compress_png_to_size(&self.encode_png(&img)?, max_size).await
    }

    /// Resize image and compress to JPEG
    async fn resize_and_compress_jpeg(&self, img: &DynamicImage, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        let (width, height) = img.dimensions();
        let mut scale_factor = 0.9;
        
        for iteration in 0..self.compression_settings.max_iterations {
            let new_width = std::cmp::max(1, (width as f32 * scale_factor) as u32);
            let new_height = std::cmp::max(1, (height as f32 * scale_factor) as u32);
            
            let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
            let compressed = self.encode_jpeg(&resized, self.compression_settings.quality)?;
            
            if compressed.len() as u64 <= max_size {
                log::info!("JPEG resized and compressed: {}x{}, {} bytes", new_width, new_height, compressed.len());
                return Ok(compressed);
            }
            
            scale_factor *= 0.8;
        }
        
        Err(ConversionError::CompressionFailed {
            message: format!("Could not compress JPEG to {} bytes after {} iterations", max_size, self.compression_settings.max_iterations),
        })
    }

    /// Resize image and compress to PNG
    async fn resize_and_compress_png(&self, img: &DynamicImage, max_size: u64) -> Result<Vec<u8>, ConversionError> {
        let (width, height) = img.dimensions();
        let mut scale_factor = 0.9;
        
        for iteration in 0..self.compression_settings.max_iterations {
            let new_width = std::cmp::max(1, (width as f32 * scale_factor) as u32);
            let new_height = std::cmp::max(1, (height as f32 * scale_factor) as u32);
            
            let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
            let compressed = self.encode_png(&resized)?;
            
            if compressed.len() as u64 <= max_size {
                log::info!("PNG resized: {}x{}, {} bytes", new_width, new_height, compressed.len());
                return Ok(compressed);
            }
            
            scale_factor *= 0.8;
        }
        
        Err(ConversionError::CompressionFailed {
            message: format!("Could not compress PNG to {} bytes after {} iterations", max_size, self.compression_settings.max_iterations),
        })
    }

    /// Encode image as JPEG with specified quality
    fn encode_jpeg(&self, img: &DynamicImage, quality: u8) -> Result<Vec<u8>, ConversionError> {
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        
        img.write_to(&mut cursor, ImageOutputFormat::Jpeg(quality))?;
        Ok(output)
    }

    /// Encode image as PNG
    fn encode_png(&self, img: &DynamicImage) -> Result<Vec<u8>, ConversionError> {
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        
        img.write_to(&mut cursor, ImageOutputFormat::Png)?;
        Ok(output)
    }

    /// Get optimal dimensions for target file size
    pub fn calculate_target_dimensions(&self, width: u32, height: u32, current_size: u64, target_size: u64) -> (u32, u32) {
        if current_size <= target_size {
            return (width, height);
        }

        let scale_factor = (target_size as f64 / current_size as f64).sqrt();
        let new_width = std::cmp::max(1, (width as f64 * scale_factor) as u32);
        let new_height = std::cmp::max(1, (height as f64 * scale_factor) as u32);
        
        (new_width, new_height)
    }
}