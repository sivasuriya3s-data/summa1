use actix_web::{web, App, HttpServer, Result, HttpResponse, middleware::Logger};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod converter;
mod types;

use converter::DocumentConverter;
use types::*;

#[derive(Deserialize)]
struct ConvertRequest {
    files: Vec<FileData>,
    exam_type: String,
    target_formats: Vec<String>,
    max_sizes: HashMap<String, u64>,
}

#[derive(Deserialize)]
struct FileData {
    name: String,
    content: String, // base64 encoded
    mime_type: String,
}

#[derive(Serialize)]
struct ConvertResponse {
    success: bool,
    files: Vec<ConvertedFile>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ConvertedFile {
    original_name: String,
    converted_name: String,
    download_url: String,
    format: String,
    size: u64,
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "rust-converter"
    })))
}

async fn convert_documents(req: web::Json<ConvertRequest>) -> Result<HttpResponse> {
    let converter = DocumentConverter::new();
    
    match converter.convert_documents(&req).await {
        Ok(converted_files) => {
            Ok(HttpResponse::Ok().json(ConvertResponse {
                success: true,
                files: converted_files,
                error: None,
            }))
        }
        Err(e) => {
            log::error!("Conversion error: {}", e);
            Ok(HttpResponse::InternalServerError().json(ConvertResponse {
                success: false,
                files: vec![],
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn get_exam_config(path: web::Path<String>) -> Result<HttpResponse> {
    let exam_type = path.into_inner();
    
    let config = match exam_type.as_str() {
        "neet" => ExamConfig {
            name: "NEET".to_string(),
            formats: vec!["PDF".to_string(), "JPEG".to_string()],
            max_sizes: {
                let mut map = HashMap::new();
                map.insert("PDF".to_string(), 2 * 1024 * 1024); // 2MB
                map.insert("JPEG".to_string(), 500 * 1024); // 500KB
                map
            },
        },
        "jee" => ExamConfig {
            name: "JEE".to_string(),
            formats: vec!["PDF".to_string(), "JPEG".to_string(), "PNG".to_string()],
            max_sizes: {
                let mut map = HashMap::new();
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
                let mut map = HashMap::new();
                map.insert("PDF".to_string(), 3 * 1024 * 1024); // 3MB
                map.insert("JPEG".to_string(), 1 * 1024 * 1024); // 1MB
                map.insert("PNG".to_string(), 1 * 1024 * 1024); // 1MB
                map
            },
        },
        _ => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Exam configuration not found"
        })))
    };
    
    Ok(HttpResponse::Ok().json(config))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Starting Rust Document Converter Service on port 8002");
    
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
            .route("/exam-config/{exam_type}", web::get().to(get_exam_config))
    })
    .bind("0.0.0.0:8002")?
    .run()
    .await
}