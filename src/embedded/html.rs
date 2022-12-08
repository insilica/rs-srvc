use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use actix_web::http::header::ContentType;
use actix_web::web::{block, Data, Json};
use actix_web::{get, middleware, post, App, HttpResponse, HttpServer};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;

use lib_sr::errors::*;
use lib_sr::event::Event;
use lib_sr::{event, Config};

use crate::embedded;
use crate::embedded::MapContext;

struct AppContext {
    config: Config,
    current_doc_events: Option<Vec<Event>>,
    doc_events: DocEventsIterator,
    html: String,
    html_file_path: Option<PathBuf>,
    timestamp_override: Option<u64>,
    writer: Box<dyn Write + Send + Sync>,
}

struct DocEventsIterator {
    in_events: Box<dyn Iterator<Item = Result<Event>> + Send + Sync>,
    next_doc: Option<Event>,
}

#[derive(Deserialize)]
struct SubmitLabelAnswersRequest {
    answers: Option<Vec<Event>>,
}

impl Iterator for DocEventsIterator {
    type Item = std::io::Result<Vec<Event>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut v = Vec::new();
        match self.next_doc.to_owned() {
            Some(doc) => v.push(doc),
            None => {}
        }
        self.next_doc = None;
        loop {
            match self.in_events.next() {
                Some(Ok(event)) => {
                    if event.r#type == "document" {
                        self.next_doc = Some(event);
                        return Some(Ok(v));
                    } else {
                        v.push(event)
                    }
                }
                Some(Err(e)) => return Some(Err(err(&e.to_string()))),
                None => {
                    if v.len() == 0 {
                        return None;
                    } else {
                        return Some(Ok(v));
                    }
                }
            };
        }
    }
}

#[get("/config")]
async fn get_config(app_ctx_mutex: Data<Mutex<AppContext>>) -> HttpResponse {
    let config = &app_ctx_mutex.lock().unwrap().config;
    HttpResponse::Ok().json(config)
}

#[get("/current-doc-events")]
async fn get_current_doc_events(app_ctx_mutex: Data<Mutex<AppContext>>) -> HttpResponse {
    let current_doc_events = &app_ctx_mutex.lock().unwrap().current_doc_events;
    let events = match current_doc_events {
        Some(events) => events.to_owned(),
        None => Vec::new(),
    };
    HttpResponse::Ok().json(json!(events))
}

#[get("/")]
async fn get_index(app_ctx_mutex: Data<Mutex<AppContext>>) -> std::io::Result<HttpResponse> {
    let html_file_path = &app_ctx_mutex.lock().unwrap().html_file_path;
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
            .body(app_ctx_mutex.lock().unwrap().html.to_owned())),
    }
}

#[post("/submit-label-answers")]
async fn post_submit_label_answers(
    app_ctx_mutex: Data<Mutex<AppContext>>,
    request: Json<SubmitLabelAnswersRequest>,
) -> std::io::Result<HttpResponse> {
    let app_ctx = &mut app_ctx_mutex.lock().unwrap();
    match app_ctx.current_doc_events.to_owned() {
        Some(events) => {
            for event in events {
                serde_json::to_writer(&mut app_ctx.writer, &event)?;
                app_ctx.writer.write(b"\n")?;
            }
        }
        None => {}
    };
    match request.answers.to_owned() {
        Some(events) => {
            for event in events {
                serde_json::to_writer(&mut app_ctx.writer, &event)?;
                app_ctx.writer.write(b"\n")?;
            }
        }
        None => {}
    };
    app_ctx.current_doc_events = app_ctx.doc_events.next().transpose()?;
    Ok(HttpResponse::Ok().json(hashmap! {"success" => true}))
}

fn err(s: &str) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, s)
}

fn write_leading_non_docs(app_ctx: &mut AppContext) -> std::io::Result<()> {
    match app_ctx.current_doc_events.to_owned() {
        Some(events) => match events.first() {
            Some(event) => {
                if event.r#type != "document" {
                    for event in events {
                        serde_json::to_writer(&mut app_ctx.writer, &event)?;
                        app_ctx.writer.write(b"\n")?;
                    }
                    app_ctx.current_doc_events = app_ctx.doc_events.next().transpose()?
                }
            }
            None => {}
        },
        None => {}
    };
    Ok(())
}

fn write_port_event(app_ctx_mutex: Data<Mutex<AppContext>>, port: u16) -> std::io::Result<()> {
    let mut app_ctx = app_ctx_mutex.lock().unwrap();
    let mut data = hashmap! {String::from("http-port") => json!(port)};
    match embedded::insert_timestamp(&mut data, app_ctx.timestamp_override) {
        Ok(_) => Ok(()),
        Err(_) => Err(err("Failed to calculate timestamp")),
    }?;
    let mut port_event = Event {
        data: Some(json!(data)),
        extra: HashMap::new(),
        hash: None,
        r#type: String::from("control"),
        uri: None,
    };
    port_event.hash = Some(match event::event_hash(port_event.clone()) {
        Ok(hash) => Ok(hash),
        Err(_) => Err(err("Failed to calculate event hash")),
    }?);

    serde_json::to_writer(&mut app_ctx.writer, &port_event)?;
    app_ctx.writer.write(b"\n")?;
    Ok(())
}

#[actix_web::main]
async fn serve(
    port: u16,
    map_ctx: MapContext,
    html: String,
    html_file_path: Option<PathBuf>,
) -> std::io::Result<()> {
    let mut doc_events = DocEventsIterator {
        in_events: map_ctx.in_events,
        next_doc: None,
    };
    let mut app_ctx = AppContext {
        config: map_ctx.config,
        current_doc_events: doc_events.next().transpose()?,
        doc_events,
        html,
        html_file_path,
        timestamp_override: map_ctx.timestamp_override,
        writer: map_ctx.writer,
    };
    write_leading_non_docs(&mut app_ctx)?;
    let app_ctx_mutex = Data::new(Mutex::new(app_ctx));
    let acm = app_ctx_mutex.to_owned();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(app_ctx_mutex.to_owned())
            .service(get_config)
            .service(get_current_doc_events)
            .service(get_index)
            .service(post_submit_label_answers)
    })
    .workers(4)
    .bind(("127.0.0.1", port))?;

    let addr = server.addrs().first().unwrap().to_owned();
    println!("Listening on http://{:?}", addr);
    match block(move || write_port_event(acm, addr.port())).await {
        Ok(result) => result,
        Err(e) => Err(err(&e.to_string())),
    }?;

    server.run().await
}

pub fn run(file_or_url: &str) -> Result<()> {
    let map_ctx = embedded::get_map_context()?;
    let (html, path) = embedded::get_file_or_url(Client::new(), file_or_url)?;
    let port = map_ctx
        .config
        .to_owned()
        .current_step
        .map(|step| step.extra.get("port").map(|x| x.as_u64()))
        .flatten()
        .flatten()
        .unwrap_or(0) as u16;

    serve(port, map_ctx, html, path).chain_err(|| "Error starting server")
}
