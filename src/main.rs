use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use std::sync::{Arc, Mutex};
use std::{thread, vec};
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

#[derive(Debug)]
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

    let mutex: Mutex<Vec<Session>> = Mutex::new(vec![]);
    let sessions = Arc::new(mutex);

    let sessions_ref_tcp = Arc::clone(&sessions);
    // Spawn threads for each server
    let tcp = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let sessions_ref = Arc::clone(&sessions_ref_tcp);
        runtime.block_on(async { tcp_server(tcp_port, sessions_ref).await });
    });

    let sessions_ref_http = Arc::clone(&sessions);
    let http = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let sessions_ref = Arc::clone(&sessions_ref_http);
        runtime.block_on(async { http_server(http_port, sessions_ref).await });
    });

    // Wait for the threads to finish
    let _ = tcp.join();
    let _ = http.join();

    println!("{:?}", sessions);
}

async fn tcp_server(port: Port, sessions_ref: Arc<Mutex<Vec<Session>>>) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).await.unwrap();
    println!("[TCP] Waiting connections on {}", port.num);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let sessions_ref_clone = sessions_ref.clone(); // avoid sessions_ref being moved

        tokio::spawn(async move {
            // spawn a task for each inbound socket
            tcp_connection_handler(socket, sessions_ref_clone).await;
        });
    }
}

async fn tcp_connection_handler(mut stream: TcpStream, sessions_ref: Arc<Mutex<Vec<Session>>>) {
    let gen = StringGenerator::default();
    let session_id = gen.next_id();
    let session_endpoint = gen.next_id();

    println!("[TCP] New connection {}: /{}", session_id, session_endpoint);
    stream
        .write(format!("connection\n{}", session_endpoint).as_bytes())
        .await
        .unwrap();

    let session = Session::new(session_id, session_endpoint, stream);
    sessions_ref.lock().unwrap().push(session);
}

async fn http_server(port: Port, sessions_ref: Arc<Mutex<Vec<Session>>>) {
    // The address we'll bind to.
    let addr = ([127, 0, 0, 1], port.num).into();

    // This is our service handler. It receives a Request, processes it, and returns a Response.
    let make_service = make_service_fn(|_conn| {
        let sessions_ref = Arc::clone(&sessions_ref);
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let sessions_clone = sessions_ref.clone();
                http_connection_handler(req, sessions_clone)
            }))
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
    sessions_ref: Arc<Mutex<Vec<Session>>>,
) -> Result<Response<Body>, hyper::Error> {
    println!("[HTTP] New connection {:?}", _req.uri());
    println!("wiiii --------- {:?}", sessions_ref.lock().unwrap());
    Ok(Response::new(Body::from("Hello, World!")))
}
