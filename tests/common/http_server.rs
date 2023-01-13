use actix_files::Files;
use actix_web::{middleware, App, HttpServer};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn err(s: &str) -> std::io::Error {
    Error::new(ErrorKind::Other, s)
}

#[actix_web::main]
pub async fn run(dir: &str, port: u16) -> std::io::Result<()> {
    let serve_from = String::from(dir);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .service(Files::new("/", &serve_from).prefer_utf8(true))
    })
    .bind(("127.0.0.1", port))?;

    server.run().await
}

pub fn wait_server_ready(port: u16) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    thread::sleep(Duration::from_millis(100));
    for _ in 0..10 {
        match TcpStream::connect(&addr) {
            Ok(_) => return Ok(()),
            Err(_) => {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    Err(err("Server not ready after 10 seconds"))
}
