use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
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

type Sessions = Arc<RwLock<HashMap<String, Session>>>;

#[tokio::main]
async fn main() {
    println!("[SUIRO] Starting service");
    ctrlc::set_handler(move || {
        println!("[SUIRO] Stopping service");
        std::process::exit(0);
    })
    .expect("[SUIRO](ERROR) setting Ctrl-C handler");

    let http_port = Port::new(8080);
    let tcp_port = Port::new(3040);

    let mutex: RwLock<HashMap<String, Session>> = RwLock::new(HashMap::new());
    let sessions = Arc::new(mutex);

    let sessions_tcp = Arc::clone(&sessions);
    let tcp = async move {
        tcp_server(tcp_port, sessions_tcp).await;
    };

    let http = async move {
        http_server(http_port, sessions).await;
    };

    futures::join!(tcp, http);
}

async fn tcp_server(port: Port, sessions: Sessions) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).await.unwrap();
    println!("[TCP] Waiting connections on {}", port.num);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let sessions_clone = sessions.clone(); // avoid sessions_ref being moved

        tokio::spawn(async move {
            // spawn a task for each inbound socket
            tcp_connection_handler(socket, sessions_clone).await;
        });
    }
}

async fn tcp_connection_handler(mut stream: TcpStream, sessions: Sessions) {
    let gen = StringGenerator::default();
    let session_id = gen.next_id();
    let session_endpoint = gen.next_id();

    println!("[TCP] New connection {}: /{}", session_id, session_endpoint);
    stream
        .write(format!("connection\n{}", session_endpoint).as_bytes())
        .await
        .unwrap();

    // --------------- HANDLE UNWRAP ----------------

    // Add session to hashmap
    let hashmap_key = session_endpoint.clone();
    let session = Session::new(session_id, session_endpoint, stream);
    sessions.write().await.insert(hashmap_key, session);
}

async fn http_server(port: Port, sessions: Sessions) {
    // The address we'll bind to.
    let addr = ([127, 0, 0, 1], port.num).into();

    // This is our service handler. It receives a Request, processes it, and returns a Response.
    let make_service = make_service_fn(|_conn| {
        let sessions_ref = Arc::clone(&sessions);
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
    sessions: Sessions,
) -> Result<Response<Body>, hyper::Error> {
    println!("[HTTP] New connection {:?}", _req.uri().path());

    let whole_endpoint = _req.uri().path().to_string();
    let session_endpoint = whole_endpoint.split("/").collect::<Vec<&str>>()[1];

    let sessions = sessions.read().await;
    println!("Sessionsss {:?}", sessions);

    if !sessions.contains_key(session_endpoint) {
        return Ok(Response::new(Body::from("Session not found")));
    }

    let session = sessions.get(session_endpoint);

    // _req create request raw
    //

    Ok(Response::new(Body::from("Hello, World!")))
}
