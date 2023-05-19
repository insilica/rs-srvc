use std::collections::HashSet;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use actix_web::web::Data;
use actix_web::{middleware, routes, App, HttpResponse, HttpServer};
use log::info;
use serde_json::json;

use lib_sr::Config;
use lib_sr::{errors::*, Opts};

use crate::sr_yaml;

struct AppContext {
    config: Config,
    read_tokens: HashSet<String>,
    tokens: HashSet<String>,
}

#[routes]
#[get("/api/v1/events")]
async fn get_events(app_ctx_mutex: Data<Mutex<AppContext>>) -> HttpResponse {
    let v: Vec<String> = Vec::new();
    HttpResponse::Ok().json(json!(v))
}

#[actix_web::main]
async fn serve(
    opts: &mut Opts,
    address_file: String,
    config: Config,
    hosts: Vec<String>,
    port: u16,
    read_tokens: HashSet<String>,
    tokens: HashSet<String>,
) -> std::io::Result<()> {
    let mut app_ctx = AppContext {
        config,
        read_tokens,
        tokens,
    };
    let app_ctx_mutex = Data::new(Mutex::new(app_ctx));

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(app_ctx_mutex.to_owned())
            .service(get_events)
    })
    .workers(16);

    for host in hosts {
        server = server.bind((host, port))?;
    }

    let mut address_writer = if !address_file.is_empty() {
        Some(LineWriter::new(File::create(&address_file)?))
    } else {
        None
    };

    for addr in server.addrs() {
        println!("Listening on http://{:?}", addr);
        if let Some(writer) = &mut address_writer {
            writer.write_all(format! {"{:?}\n", addr}.as_bytes())?;
        }
    }

    drop(address_writer);

    server.run().await
}

pub fn run(
    opts: &mut Opts,
    address_file: String,
    db: Option<String>,
    hosts: Vec<String>,
    port: u16,
    read_tokens: Vec<String>,
    tokens: Vec<String>,
) -> Result<()> {
    info! {"Starting SRVC API server"};
    let yaml_config = sr_yaml::get_config(PathBuf::from(&opts.config))?;
    let mut config = sr_yaml::parse_config(yaml_config)?;
    config.db = db.unwrap_or(config.db);

    serve(
        opts,
        address_file,
        config,
        hosts,
        port,
        read_tokens.into_iter().collect(),
        tokens.into_iter().collect(),
    )
    .chain_err(|| "Error starting server")
}
