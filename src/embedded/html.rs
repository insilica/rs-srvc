use std::sync::Mutex;

use actix_web::web::Data;
use actix_web::{get, App, HttpResponse, HttpServer};

use lib_sr::errors::*;

use crate::embedded;
use crate::embedded::MapContext;

#[get("/config")]
async fn get_config(map_ctx_mutex: Data<Mutex<MapContext>>) -> HttpResponse {
    let config = &map_ctx_mutex.lock().unwrap().config;
    HttpResponse::Ok().json(config)
}

#[actix_web::main]
async fn serve(map_ctx: MapContext) -> std::io::Result<()> {
    let map_ctx_mutex = Data::new(Mutex::new(map_ctx));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(map_ctx_mutex.clone())
            .service(get_config)
    })
    .bind(("127.0.0.1", 0))?;

    println!("Listening on http://{:?}", server.addrs().first().unwrap());

    server.run().await
}

pub fn run(_file_or_url: &str) -> Result<()> {
    let map_ctx = embedded::get_map_context()?;
    serve(map_ctx).chain_err(|| "Error starting server")
}
