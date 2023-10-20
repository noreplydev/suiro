use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::net::{TcpListener, TcpStream};
use std::thread;
use unique_id::string::StringGenerator;
use unique_id::Generator;

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
    ctrlc::set_handler(move || {
        println!("[SUIRO] Stopping service");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let http_port = Port::new(8080);
    let tcp_port = Port::new(3040);

    let http = thread::spawn(move || {
        let runtime = actix_rt::System::new();
        runtime.block_on(async move {
            let _ = http_server(http_port).await;
        });
    });

    // Start the TCP server in a new thread
    let tcp = thread::spawn(move || {
        let _ = tcp_server(tcp_port);
    });

    // Wait for the threads to finish
    http.join().unwrap();
    tcp.join().unwrap();
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
    let gen = StringGenerator::default();
    println!("[TCP] New connection: {}", gen.next_id());
}

async fn http_server(port: Port) -> Result<(), std::io::Error> {
    // a thread pool is created here
    HttpServer::new(|| {
        println!("[HTTP] Server started on port 3000");
        App::new().route(
            "/",
            web::get().to(|| async {
                println!("[HTTP] GET /");
                HttpResponse::Ok()
            }),
        )
    })
    .bind(("127.0.0.1", port.num))
    .unwrap()
    .workers(1) // only one thread for the http server
    .run()
    .await
    .expect("Error starting HTTP server");

    Ok(())
}
