use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{get, HttpResponse, Responder};
use askama::Template;
use tracing::{error, info, instrument};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub title: &'a str,
    pub content: Vec<&'a str>,
    pub version: &'a str,
}

#[derive(Template)]
#[template(path = "errors.html")]
pub struct ErrorPage<'a> {
    pub title: &'a str,
    pub code: u32,
    pub error: &'a str,
    pub message: &'a str,
}

impl<'a> ErrorPage<'a> {
    #[must_use]
    pub const fn new(message: &'a str) -> Self {
        Self {
            title: "Error",
            code: 500,
            error: "Internal Server Error",
            message,
        }
    }
}

#[get("/favicon")]
#[instrument(
    name = "Serving favicon",
    level = "info",
    target = "web_app_bloodhound"
)]
async fn favicon() -> Result<NamedFile, actix_web::Error> {
    info!("Serving favicon");
    let filename = "ca_icon.ico";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/logomain")]
#[instrument(name = "Serving logo", level = "info", target = "web_app_bloodhound")]
async fn logomain() -> Result<NamedFile, actix_web::Error> {
    info!("Serving logo");
    let filename = "logomain.jpeg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/stylesheet")]
#[instrument(
    name = "Serving stylesheet",
    level = "info",
    target = "web_app_bloodhound"
)]
async fn stylesheet() -> impl Responder {
    info!("Serving stylesheet");
    let file = include_str!("../../static/css/style.css");
    HttpResponse::Ok().content_type("text/css").body(file)
}

#[get("/style.css.map")]
#[instrument(
    name = "Serving source map",
    level = "info",
    target = "web_app_bloodhound"
)]
async fn source_map() -> impl Responder {
    info!("Serving source map");
    let file = include_str!("../../static/css/style.css.map");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(file)
}

#[get("/htmx")]
#[instrument(
    name = "Serving htmx.min.js",
    level = "info",
    target = "web_app_bloodhound"
)]
async fn htmx() -> Result<NamedFile, actix_web::Error> {
    info!("Serving htmx.min.js");

    let filename = "htmx.min.js";
    let path: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/response-targets")]
#[instrument(
    name = "Serving response-targets.js",
    level = "info",
    target = "web_app_bloodhound"
)]
async fn response_targets() -> Result<NamedFile, actix_web::Error> {
    info!("Serving response-targets.js");

    let filename = "response-targets.js";
    let pash: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(pash) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/sse")]
#[instrument(name = "Serving sse.js", level = "info", target = "web_app_bloodhound")]
async fn sse() -> Result<NamedFile, actix_web::Error> {
    info!("Serving sse.js");

    let filename = "sse.js";
    let path: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

// #[get("/action_script")]
// #[instrument(
//     name = "Serving action_script.js",
//     level = "info",
//     target = "web_app_bloodhound"
// )]
// async fn action_script() -> Result<NamedFile, actix_web::Error> {
//     info!("Serving action_script.js");

//     let filename = "time-dilation.js";
//     let path: PathBuf = ["static", "js", filename].iter().collect();

//     match NamedFile::open(path) {
//         Ok(file) => Ok(file),
//         Err(err) => {
//             error!("Error opening file -- {filename} -- : {err:#?}");
//             Err(actix_web::error::ErrorInternalServerError(err))
//         }
//     }
// }
