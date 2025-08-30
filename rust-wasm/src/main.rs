use actix_web::{web, App, HttpServer, Result, HttpResponse, middleware::Logger};
use actix_cors::Cors;
use std::sync::Mutex;

mod converter;
mod types;
mod image_processor;
mod pdf_processor;

use converter::DocumentConverter;
use types::*;

// Global converter instance with thread-safe access
type ConverterState = web::Data<Mutex<DocumentConverter>>;

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "rust-converter",
        "version": "1.0.0",
        "capabilities": {
            "image_formats": ["JPEG", "PNG", "WebP"],
            "document_formats": ["PDF", "DOCX", "TXT"],
            "operations": ["compression", "format_conversion", "optimization"]
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn convert_documents(
    req: web::Json<ConvertRequest>,
    converter_state: ConverterState,
) -> Result<HttpResponse> {
    log::info!("🚀 Conversion request received:");
    log::info!("  - Files: {}", req.files.len());
    log::info!("  - Exam type: {}", req.exam_type);
    log::info!("  - Target formats: {:?}", req.target_formats);
    log::info!("  - Size limits: {:?}", req.max_sizes);
    
    let mut converter = match converter_state.lock() {
        Ok(conv) => conv,
        Err(e) => {
            log::error!("Failed to acquire converter lock: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ConvertResponse {
                success: false,
                files: vec![],
                error: Some("Service temporarily unavailable".to_string()),
            }));
        }
    };
    
    match converter.convert_documents(&req).await {
        Ok(converted_files) => {
            let successful_conversions = converted_files.iter()
                .filter(|f| !f.download_url.is_empty())
                .count();
            
            log::info!("✅ Conversion completed: {}/{} files successful", 
                successful_conversions, converted_files.len());
            
            Ok(HttpResponse::Ok().json(ConvertResponse {
                success: true,
                files: converted_files,
                error: None,
            }))
        }
        Err(e) => {
            log::error!("❌ Conversion failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(ConvertResponse {
                success: false,
                files: vec![],
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn download_file(
    path: web::Path<String>,
    converter_state: ConverterState,
) -> Result<HttpResponse> {
    let file_id = path.into_inner();
    log::info!("📥 Download requested for file ID: {}", file_id);
    
    let converter = converter_state.lock().unwrap();
    
    match converter.get_stored_file(&file_id) {
        Some(file_content) => {
            log::info!("✅ File found, serving {} bytes", file_content.len());
            Ok(HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header(("Content-Disposition", "attachment"))
                .append_header(("Cache-Control", "no-cache"))
                .body(file_content.clone()))
        }
        None => {
            log::warn!("❌ File not found: {}", file_id);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "File not found",
                "file_id": file_id
            })))
        }
    }
}

async fn get_exam_config(path: web::Path<String>) -> Result<HttpResponse> {
    let exam_type = path.into_inner();
    log::info!("📋 Exam config requested for: {}", exam_type);
    
    let config = match exam_type.as_str() {
        "neet" => ExamConfig {
            name: "NEET".to_string(),
            formats: vec!["PDF".to_string(), "JPEG".to_string()],
            max_sizes: {
                let mut map = std::collections::HashMap::new();
                map.insert("PDF".to_string(), 2 * 1024 * 1024); // 2MB
                map.insert("JPEG".to_string(), 500 * 1024); // 500KB
                map
            },
        },
        "jee" => ExamConfig {
            name: "JEE".to_string(),
            formats: vec!["PDF".to_string(), "JPEG".to_string(), "PNG".to_string()],
            max_sizes: {
                let mut map = std::collections::HashMap::new();
                map.insert("PDF".to_string(), 1 * 1024 * 1024); // 1MB
                map.insert("JPEG".to_string(), 300 * 1024); // 300KB
                map.insert("PNG".to_string(), 300 * 1024); // 300KB
                map
            },
        },
        "upsc" => ExamConfig {
            name: "UPSC".to_string(),
            formats: vec!["PDF".to_string(), "JPEG".to_string(), "PNG".to_string()],
            max_sizes: {
                let mut map = std::collections::HashMap::new();
                map.insert("PDF".to_string(), 3 * 1024 * 1024); // 3MB
                map.insert("JPEG".to_string(), 1 * 1024 * 1024); // 1MB
                map.insert("PNG".to_string(), 1 * 1024 * 1024); // 1MB
                map
            },
        },
        "cat" => ExamConfig {
            name: "CAT".to_string(),
            formats: vec!["PDF".to_string(), "JPEG".to_string()],
            max_sizes: {
                let mut map = std::collections::HashMap::new();
                map.insert("PDF".to_string(), 1536 * 1024); // 1.5MB
                map.insert("JPEG".to_string(), 400 * 1024); // 400KB
                map
            },
        },
        "gate" => ExamConfig {
            name: "GATE".to_string(),
            formats: vec!["PDF".to_string(), "JPEG".to_string(), "PNG".to_string()],
            max_sizes: {
                let mut map = std::collections::HashMap::new();
                map.insert("PDF".to_string(), 2 * 1024 * 1024); // 2MB
                map.insert("JPEG".to_string(), 500 * 1024); // 500KB
                map.insert("PNG".to_string(), 500 * 1024); // 500KB
                map
            },
        },
        _ => {
            log::warn!("Unknown exam type requested: {}", exam_type);
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Exam configuration not found",
                "available_exams": ["neet", "jee", "upsc", "cat", "gate"],
                "requested": exam_type
            })));
        }
    };
    
    log::info!("✅ Returning config for {}: {} formats", exam_type, config.formats.len());
    Ok(HttpResponse::Ok().json(config))
}

async fn get_conversion_stats(converter_state: ConverterState) -> Result<HttpResponse> {
    let converter = converter_state.lock().unwrap();
    let (file_count, total_size) = converter.get_storage_stats();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "temp_files_count": file_count,
        "temp_storage_size": total_size,
        "service_status": "running",
        "supported_formats": {
            "input": ["PDF", "JPEG", "JPG", "PNG", "WEBP", "DOCX", "DOC", "TXT"],
            "output": ["PDF", "JPEG", "PNG", "DOCX"]
        },
        "max_file_size": "10MB",
        "compression_capabilities": {
            "jpeg_quality_range": "10-100%",
            "png_compression_levels": "0-9",
            "pdf_optimization": true
        }
    })))
}

async fn cleanup_temp_files(converter_state: ConverterState) -> Result<HttpResponse> {
    let mut converter = converter_state.lock().unwrap();
    let (count, size) = converter.get_storage_stats();
    converter.cleanup_temp_files();
    
    log::info!("🧹 Cleaned up {} files ({} bytes)", count, size);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Cleaned up {} temporary files ({} bytes)", count, size),
        "status": "success"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    log::info!("🦀 Starting Rust Document Converter Service");
    log::info!("📍 Port: 8002");
    log::info!("🔧 Features: Image compression, PDF optimization, Format conversion");
    log::info!("📊 Supported input formats: PDF, JPEG, PNG, WEBP, DOCX, DOC, TXT");
    log::info!("📤 Supported output formats: PDF, JPEG, PNG, DOCX");
    
    // Initialize converter state
    let converter_state = web::Data::new(Mutex::new(DocumentConverter::new()));
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            .app_data(converter_state.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .route("/health", web::get().to(health))
            .route("/convert", web::post().to(convert_documents))
            .route("/download/{file_id}", web::get().to(download_file))
            .route("/exam-config/{exam_type}", web::get().to(get_exam_config))
            .route("/stats", web::get().to(get_conversion_stats))
            .route("/cleanup", web::post().to(cleanup_temp_files))
    })
    .bind("0.0.0.0:8002")?
    .run()
    .await
}