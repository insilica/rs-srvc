use std::io::{BufReader, Read};
use std::sync::Mutex;
use std::{fs::File, path::PathBuf};

use actix_files::Files;
use actix_web::dev::PeerAddr;
use actix_web::http::header::ContentType;
use actix_web::{get, middleware, web, App, HttpRequest, HttpServer};
use actix_web::{http::Method, routes, web::Data, HttpResponse};
use anyhow::Result;
use futures_util::StreamExt;
use lib_sr::{common, sr_yaml, Config, Opts};
use log::{debug, info};
use reqwest::blocking::Client;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;

#[derive(Clone, Debug)]
struct AppContext {
    config: Config,
    host: String,
    html: String,
    html_file_path: Option<PathBuf>,
    html_url: Option<Url>,
    port: u16,
    yaml_config: sr_yaml::Config,
    yaml_config_path: PathBuf,
}

#[get("/")]
async fn get_index(app_ctx_mutex: Data<Mutex<AppContext>>) -> std::io::Result<HttpResponse> {
    let app_ctx = app_ctx_mutex.lock().unwrap();
    let html_file_path = &app_ctx.html_file_path;
    match html_file_path {
        Some(path) => {
            let file = File::open(path)?;
            let mut reader = BufReader::new(file);
            let mut s = String::new();
            reader.read_to_string(&mut s)?;
            Ok(HttpResponse::Ok().content_type(ContentType::html()).body(s))
        }
        None => Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(app_ctx.html.to_owned())),
    }
}

#[derive(Serialize)]
struct Configs {
    config: Config,
    #[serde(rename = "yaml-config")]
    yaml_config: sr_yaml::Config,
}

#[routes]
#[get("/srvc/configs")]
async fn get_configs(app_ctx_mutex: Data<Mutex<AppContext>>) -> HttpResponse {
    let guard = &mut app_ctx_mutex.lock().unwrap();
    let yaml_config_path = &guard.yaml_config_path;
    let yaml_config = sr_yaml::add_defaults(
        sr_yaml::get_config(yaml_config_path.clone()).unwrap_or_else(|_| guard.yaml_config.clone()),
    );
    let yc = yaml_config.clone();
    let fut = web::block(move || sr_yaml::parse_config(yaml_config));
    match fut.await {
        Ok(Ok(config)) => {
            let configs = Configs {
                config: config.clone(),
                yaml_config: yc.clone(),
            };
            guard.config = config;
            guard.yaml_config = yc;
            HttpResponse::Ok().json(configs)
        }
        Ok(Err(_)) | Err(_) => {
            let configs = Configs {
                config: guard.config.clone(),
                yaml_config: guard.yaml_config.clone(),
            };
            HttpResponse::Ok().json(configs)
        }
    }
}

// Adapted from https://github.com/actix/examples/blob/2df944c5e55951021e6c1da0feffef8c24c19506/http-proxy/src/main.rs#L57
#[get("/{url:.*}")]
async fn forward_reqwest(
    req: HttpRequest,
    mut payload: web::Payload,
    method: Method,
    peer_addr: Option<PeerAddr>,
    url: web::Data<Url>,
    client: web::Data<reqwest::Client>,
) -> actix_web::error::Result<HttpResponse, actix_web::error::Error> {
    let path = req.uri().path();
    let mut new_url = url
        .join(path.trim_start_matches('/'))
        .map_err(actix_web::error::ErrorInternalServerError)?;
    new_url.set_query(req.uri().query());
    debug! {"Forwarding request to {}", new_url};

    let (tx, rx) = mpsc::unbounded_channel();

    actix_web::rt::spawn(async move {
        while let Some(chunk) = payload.next().await {
            tx.send(chunk).unwrap();
        }
    });

    let forwarded_req = client
        .request(method, new_url)
        .body(reqwest::Body::wrap_stream(UnboundedReceiverStream::new(rx)));

    // TODO: This forwarded implementation is incomplete as it only handles the unofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => forwarded_req.header("x-forwarded-for", addr.ip().to_string()),
        None => forwarded_req,
    };

    let res = forwarded_req
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.streaming(res.bytes_stream()))
}

#[actix_web::main]
async fn serve(app_ctx: AppContext) -> std::io::Result<()> {
    let app_ctx_mutex: Data<Mutex<AppContext>> = Data::new(Mutex::new(app_ctx.clone()));
    let num_workers = match app_ctx.html_url {
        Some(_) => 16,
        None => 4,
    };

    let server = HttpServer::new(move || {
        let mut app = App::new()
            .wrap(middleware::Compress::default())
            .app_data(app_ctx_mutex.to_owned())
            .service(get_configs)
            .service(get_index);
        match &app_ctx.html_file_path {
            Some(path) => {
                let cpath = path.canonicalize().expect("canonicalize");
                let serve_from = cpath.parent().unwrap_or(path);
                app = app.service(Files::new("/", serve_from).prefer_utf8(true))
            }
            None => {}
        };
        match app_ctx.html_url.clone() {
            Some(url) => {
                app = app
                    .app_data(Data::new(reqwest::Client::new()))
                    .app_data(Data::new(url.to_owned()))
                    .service(forward_reqwest)
            }
            None => {}
        }
        app
    })
    .workers(num_workers)
    .bind((app_ctx.host, app_ctx.port))?;

    let addr = server.addrs().first().unwrap().to_owned();
    println!("Listening on http://{:?}", addr);

    server.run().await
}

pub fn run(opts: &mut Opts, editor: Option<String>, host: String, port: u16) -> Result<()> {
    let yaml_config_path = PathBuf::from(&opts.config);
    let yaml_config = sr_yaml::add_defaults(sr_yaml::get_config(yaml_config_path.clone())?);
    let config = sr_yaml::parse_config(yaml_config.clone())?;

    let (html, path, url) = match editor.clone() {
        Some(s) => {
            info! {"Serving edit-config step from {}", s};
            common::get_file_or_url_string(&Client::default(), &s)?
        }
        None => {
            info! {"Serving embedded edit-config step"};
            (String::from(include_str!("edit_config.html")), None, None)
        }
    };
    if editor.is_some() {
        debug! {"Read {} bytes", html.len()};
    }

    let app_ctx = AppContext {
        config,
        host,
        html,
        html_file_path: path,
        html_url: url,
        port,
        yaml_config,
        yaml_config_path,
    };

    Ok(serve(app_ctx)?)
}
