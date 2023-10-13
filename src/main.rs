use core::panic;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use tokio;

struct Port {
    num: u16,
}

impl Port {
    fn new(port: u16) -> Self {
        if port < 1024 {
            panic!("Port number must be greater than 1024");
        }

        Port { num: port }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("[SUIRO] Starting service");

    let http_port = Port::new(8080);

    let http = http_server(http_port).await; // spawn the http server in a new thread

    Ok(())
}

async fn http_server(port: Port) -> std::io::Result<()> {
    // a thread pool is created here
    HttpServer::new(move || {
        println!("[HTTP] SERVER STARTED ON PORT: {}", &port.num);
        App::new()
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .route("/hi", web::get().to(|| async { "Hello world!" }))
    })
    .bind(("127.0.0.1", port.num))?
    .workers(1) // only one thread for the http server
    .run()
    .await
}
