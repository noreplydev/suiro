use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use std::thread;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use unique_id::string::StringGenerator;
use unique_id::Generator;

struct Port {
    num: u16,
}

impl Port {
    fn new(port: u16) -> Self {
        if port < 1024 {
            println!("Port number must be greater than 1024 or run as root");
        }
        Port { num: port }
    }
}

struct Session {
    session_id: String,
    session_endpoint: String,
    socket: TcpStream,
}

impl Session {
    fn new(id: String, endpoint: String, socket: TcpStream) -> Self {
        Session {
            session_id: id,
            session_endpoint: endpoint,
            socket,
        }
    }
}

fn main() {
    println!("[SUIRO] Starting service");
    ctrlc::set_handler(move || {
        println!("[SUIRO] Stopping service");
        std::process::exit(0);
    })
    .expect("[SUIRO](ERROR) setting Ctrl-C handler");

    let http_port = Port::new(8080);
    let tcp_port = Port::new(3040);

    // Spawn threads for each server
    let tcp = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async { tcp_server(tcp_port).await });
    });

    let http = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async { http_server(http_port).await });
    });

    // Wait for the threads to finish
    let _ = tcp.join();
    let _ = http.join();
}

async fn tcp_server(port: Port) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).await.unwrap();
    println!("[TCP] Waiting connections on {}", port.num);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            // spawn a task for each inbound socket
            tcp_connection_handler(socket).await;
        });
    }
}

async fn tcp_connection_handler(mut stream: TcpStream) {
    let gen = StringGenerator::default();
    let session_id = gen.next_id();
    let session_endpoint = gen.next_id();

    println!("[TCP] New connection {}: /{}", session_id, session_endpoint);
    stream
        .write(format!("connection\n{}", session_endpoint).as_bytes())
        .await
        .unwrap();

    let session = Session::new(session_id, session_endpoint, stream);
}

async fn http_server(port: Port) {
    // The address we'll bind to.
    let addr = ([127, 0, 0, 1], 3000).into();

    // This is our service handler. It receives a Request, processes it, and returns a Response.
    let make_service = make_service_fn(|_conn| {
        async {
            // service_fn converts our function into a `Service`.
            Ok::<_, hyper::Error>(service_fn(http_connection_handler))
        }
    });

    let server = Server::bind(&addr).serve(make_service);
    println!("[HTTP] Waiting connections on {}", port.num);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn http_connection_handler(
    _req: hyper::Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    println!("[HTTP] New connection {:?}", _req.uri());
    Ok(Response::new(Body::from("Hello, World!")))
}

////////////////////
/*
async fn http_server(port: Port) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).await.unwrap();
    println!("[HTTP] Waiting connections on {}", port.num);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async {
            // spawn a task for each inbound socket
            http_connection_handler(socket).await;
        });
    }
}

async fn http_connection_handler(stream: TcpStream) {
    println!("[HTTP] New connection {}", stream.peer_addr().unwrap());
    let mut buf_reader = BufReader::new(stream);

    let mut headers: Vec<String> = Vec::new();
    let mut body: Vec<String> = Vec::new();

    println!("[HTTP] Request headers: \n\n\r{}\n", headers.join("\n"));
} */
////////////////////
