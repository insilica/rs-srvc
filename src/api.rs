use std::collections::HashSet;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll};
use std::thread;

use actix_web::web::Data;
use actix_web::{middleware, routes, App, HttpResponse, HttpServer};
use bytes::Bytes;
use futures::Stream;
use lib_sr::event::Event;
use log::{info, trace};
use serde_json::json;

use lib_sr::{errors::*, Opts};
use lib_sr::{generate, Config};
use tokio::sync::{ Notify};
use tokio::task;

use crate::sr_yaml;

struct AppContext {
    config: Config,
    read_tokens: HashSet<String>,
    tokens: HashSet<String>,
}

struct EventStream {
    pair: Arc<(Mutex<Option<Event>>, Condvar)>,
    waker: Option<std::task::Waker>,
}

impl EventStream {
    fn new(pair: Arc<(Mutex<Option<Event>>, Condvar)>) -> Self {
        Self { pair, waker: None }
    }

    fn notify_waker(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

impl Stream for EventStream {
    type Item = std::result::Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let new_waker = None;
        {
            trace!("EventStream poll_next acquiring lock");
            let (lock, cvar) = &*self.pair;
            let mut opt_event = lock.lock().unwrap();

            if opt_event.is_some() {
                return Poll::Pending;
            }

            if opt_event.take().is_none() {
                return Poll::Pending;
            }
            cvar.notify_one();
        }

        trace!("EventStream poll_next notifying waker");
        self.waker = new_waker;
        self.notify_waker();
        let (lock, _) = &*self.pair;
        let mut opt_event = lock.lock().unwrap();
        let event = opt_event.take().expect("opt_event");
        let event_json = match serde_json::to_string(&event) {
            Ok(json) => json,
            Err(_) => return Poll::Ready(Some(Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to serialize event")))),
        };
        let event_bytes = Bytes::from(event_json);
        Poll::Ready(Some(Ok(event_bytes)))
    }
}

#[routes]
#[get("/api/v1/events")]
async fn get_events(app_ctx_mutex: Data<std::sync::Mutex<AppContext>>) -> HttpResponse {
    let app_ctx = app_ctx_mutex.lock().unwrap();
    let config = app_ctx.config.to_owned();

    let pair = Arc::new((Mutex::new(None), Condvar::new()));
    let p = pair.clone();

    let mut f = move |event: Event| -> Result<()> {
        trace!("/api/v1/events received event: {} {}", event.r#type, event.hash.clone().unwrap_or(String::from("No hash")));
        let (lock, cvar) = &*pair;
        let mut opt_event = lock.lock().unwrap();

        while opt_event.is_some() {
            trace!("/api/v1/events waiting on guard to be None");
            opt_event = cvar.wait(opt_event).unwrap();
        }

        trace!("/api/v1/events sending event");
        *opt_event = Some(event);
        cvar.notify_one();
        Ok(())
    };

    let file_or_url = "docs.jsonl";

    let thread_f = task::spawn_blocking(move || generate::run_f(file_or_url, &config, &mut f));

    HttpResponse::Ok().streaming(EventStream::new(p))
}

#[actix_web::main]
async fn serve(
    address_file: String,
    config: Config,
    hosts: Vec<String>,
    port: u16,
    read_tokens: HashSet<String>,
    tokens: HashSet<String>,
) -> std::io::Result<()> {
    let app_ctx = AppContext {
        config,
        read_tokens,
        tokens,
    };
    let app_ctx_mutex = Data::new(std::sync::Mutex::new(app_ctx));

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
        address_file,
        config,
        hosts,
        port,
        read_tokens.into_iter().collect(),
        tokens.into_iter().collect(),
    )
    .chain_err(|| "Error starting server")
}
