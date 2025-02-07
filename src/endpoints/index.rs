use actix_web::{get, http::header::ContentType, HttpResponse};
use askama::Template;
use tracing::{info, instrument};

use super::templates::IndexTemplate;

#[instrument(
    name = "Serving main page",
    level = "debug",
    target = "web_app_bloodhound",
    fields(samples = 25, title = "Home")
)]
#[get("/")]
pub async fn index() -> HttpResponse {
    info!("Serving main page");
    let version: &str = env!("CARGO_PKG_VERSION");

    let var_name = IndexTemplate {
        // EUBands: vec!["1", "8", "40", "20", "28A", "28B"]
        title: "Home",
        content: vec!["friendly", "messages"],
        version,
        linkedin: "https://www.linkedin.com/in/christerpher",
        github: "https://github.com/djhunter67",
        source_url: "https://christerpher.com",
    };

    let rendered = var_name.render().expect("Failed to render template");

    // qs.await;

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered)
}
