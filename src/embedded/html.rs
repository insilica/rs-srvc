use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Mutex;

use actix_web::http::header::ContentType;
use actix_web::web::Data;
use actix_web::{get, middleware, App, HttpResponse, HttpServer};
use reqwest::blocking::Client;
use url::Url;

use lib_sr::errors::*;

use crate::embedded;
use crate::embedded::MapContext;

#[derive(Clone, Debug)]
struct AppContext {
    html: String,
}

#[get("/config")]
async fn get_config(map_ctx_mutex: Data<Mutex<MapContext>>) -> HttpResponse {
    let config = &map_ctx_mutex.lock().unwrap().config;
    HttpResponse::Ok().json(config)
}

#[get("/")]
async fn get_index(app_ctx_mutex: Data<Mutex<AppContext>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(app_ctx_mutex.lock().unwrap().to_owned().html)
}

#[actix_web::main]
async fn serve(map_ctx: MapContext, html: String) -> std::io::Result<()> {
    let app_ctx = AppContext { html: html };
    let app_ctx_mutex = Data::new(Mutex::new(app_ctx));
    let map_ctx_mutex = Data::new(Mutex::new(map_ctx));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(app_ctx_mutex.to_owned())
            .app_data(map_ctx_mutex.to_owned())
            .service(get_config)
            .service(get_index)
    })
    .workers(4)
    .bind(("127.0.0.1", 0))?;

    println!("Listening on http://{:?}", server.addrs().first().unwrap());

    server.run().await
}

fn get_file_or_url(file_or_url: &str) -> Result<String> {
    match Url::parse(file_or_url) {
        Ok(url) => {
            let client = Client::new();
            let response = client
                .get(url.clone())
                .send()
                .chain_err(|| format!("Failed to complete HTTP request to {}", url))?;
            let status = response.status().as_u16();
            if status == 200 {
                response.text().chain_err(|| "Failed to read response text")
            } else {
                Err(format!("Unexpected {} status for {}", status, url).into())
            }
        }
        Err(_) => {
            let path = PathBuf::from(file_or_url);
            let file =
                File::open(path).chain_err(|| format!("Failed to open file {}", file_or_url))?;
            let mut reader = BufReader::new(file);
            let mut s = String::new();
            reader
                .read_to_string(&mut s)
                .chain_err(|| format!("Buffer read failed for file {}", file_or_url))?;
            Ok(s)
        }
    }
}

pub fn run(file_or_url: &str) -> Result<()> {
    let map_ctx = embedded::get_map_context()?;
    let html = get_file_or_url(file_or_url)?;

    serve(map_ctx, html).chain_err(|| "Error starting server")
}
