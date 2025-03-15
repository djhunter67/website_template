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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use actix_web::{test, web::Bytes, App};

    use super::index;

    #[actix_web::test]
    async fn test_get_index() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_index_is_html() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_and_read_body(&app, req).await;
        assert!(!resp.is_empty());
        let first_letters: Bytes = resp.slice(0..15).iter().copied().collect();
        let conv_str = std::str::from_utf8(&first_letters).unwrap();
        assert!(conv_str == "<!DOCTYPE html>");
    }
}
