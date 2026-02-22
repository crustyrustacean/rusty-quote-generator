// src/routes/health_check.rs

// dependencies
use actix_web::{HttpResponse, Responder};

/// health check endpoint
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
