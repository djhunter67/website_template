use actix_web::{get, HttpResponse, Responder};
use tracing::{info, instrument};

#[get("/health_check")]
#[instrument(name = "Health check", target = "demo_web_app", level = "info")]
pub async fn health_check() -> impl Responder {
    info!("Health check endpoint called.");
    HttpResponse::Ok().finish()
}
