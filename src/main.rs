use actix_web::dev::Server;
use core::panic;
use std::thread;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

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

    let http_port = Port::new(3030);
    let tcp_port = Port::new(3040);

    // Start the HTTP server
    // a thread pool is created here
    let _ = HttpServer::new(move || {
        println!("[HTTP] SERVER STARTED ON PORT: 3000");
        App::new()
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .route("/hi", web::get().to(|| async { "Hello world!" }))
    })
    .bind(("127.0.0.1", 3050))
    .unwrap()
    .workers(1) // only one thread for the http server
    .run()
    .await;

    // Start the TCP server in a new thread
    thread::spawn(|| {
        println!("[TCP] Starting TCP server");
    });

    Ok(())
}

async fn tcp_server(port: Port) -> Result<tokio::net::TcpListener, std::io::Error> {
    println!("[TCP] SERVER STARTED ON PORT: {}", &port.num);
    let tcp = TcpListener::bind(("127.0.0.1", port.num)).await?;

    loop {
        let (stream, _) = tcp.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(_stream: TcpStream) {
    println!("New TCP connection");
    // Handle TCP connection here
}
