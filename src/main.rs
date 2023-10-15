use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Copy, Clone)]
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

fn main() {
    println!("[SUIRO] Starting service");

    let http_port = Port::new(8080);
    let tcp_port = Port::new(3040);

    // Start the HTTP server
    // a thread pool is created here
    /*     let http = HttpServer::new(|| {
        println!("[HTTP] SERVER STARTED ON PORT: 3000");
        App::new()
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .route("/hi", web::get().to(|| async { "Hello world!" }))
    })
    .bind(("127.0.0.1", 3050))
    .unwrap()
    .workers(1) // only one thread for the http server
    .run()
    .await; */

    // Start the TCP server in a new thread
    let tcp = thread::spawn(move || {
        let tcp_server = tcp_server(tcp_port);
    });

    let http = thread::spawn(move || {
        let http_server = tcp_server(http_port);
    });

    tcp.join().unwrap();
    http.join().unwrap();

    println!("[SUIRO] Stopping service");
}

fn tcp_server(port: Port) -> Result<std::net::TcpListener, std::io::Error> {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).unwrap();
    let requests = listener.incoming();

    println!("[TCP] listening on {}", &port.num);
    println!("[TCP] waiting for connections");

    for stream in requests {
        let stream = stream.unwrap();

        handle_connection(stream);
    }

    Ok(listener)
}

fn handle_connection(stream: TcpStream) {
    println!("[TCP] New connection: {}", stream.peer_addr().unwrap());
}
