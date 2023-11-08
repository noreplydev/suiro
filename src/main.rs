use futures::lock::Mutex;
use futures::FutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use unique_id::{string::StringGenerator, Generator};

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
    socket_tx: mpsc::Sender<String>,
    responses_rx: mpsc::Receiver<(String, String)>,
}

impl Session {
    fn new(
        id: String,
        endpoint: String,
        socket_tx: mpsc::Sender<String>,
        responses_rx: mpsc::Receiver<(String, String)>,
    ) -> Self {
        Session {
            session_id: id,
            session_endpoint: endpoint,
            socket_tx,
            responses_rx,
        }
    }
}

type Sessions = Arc<Mutex<HashMap<String, Session>>>;

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

    let mutex: Mutex<HashMap<String, Session>> = Mutex::new(HashMap::new());
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

async fn tcp_connection_handler(mut socket: TcpStream, sessions: Sessions) {
    let gen = StringGenerator::default();
    let session_id = gen.next_id();
    let session_endpoint = gen.next_id();

    println!("[TCP] New connection {}: /{}", session_id, session_endpoint);
    socket
        .write(format!("connection\n{}", session_endpoint).as_bytes())
        .await
        .unwrap();
    // --------------- HANDLE UNWRAP ----------------

    // Add session to hashmap
    let hashmap_key = session_endpoint.clone();
    let (socket_tx, mut rx) = mpsc::channel(100); // 100 message queue
    let (tx, responses_rx) = mpsc::channel(100); // 100 message queue
    let session = Session::new(
        session_id.clone(),
        session_endpoint.clone(),
        socket_tx,
        responses_rx,
    );
    sessions.lock().await.insert(hashmap_key, session);

    // Handle incoming data
    let mut packet_request_id = "".to_string();
    let mut packet_acc_data = "".to_string();
    let mut packet_total_size = 0;
    let mut packet_acc_size = 0;

    let mut buffer = [0; 1024];
    loop {
        // Write data to socket on request
        if let Some(request) = rx.recv().now_or_never() {
            match request {
                Some(request) => {
                    socket.write(request.as_bytes()).await.unwrap();
                }
                None => {}
            }
        }

        match socket.read(&mut buffer).now_or_never() {
            Some(sock) => {
                match sock {
                    Ok(0) => {
                        // connection closed
                        println!("[TCP] Connection closed: {}", session_id);
                        break;
                    }
                    Ok(n) => {
                        // data received
                        let data = &buffer[..n];
                        println!("----------HOLA----------");

                        // Packet fragmentation?
                        if packet_request_id != "".to_string() {
                            println!("hola bloqeuado -1");

                            let cur_packet_data = String::from_utf8(data.to_vec()).unwrap();
                            packet_acc_data = format!("{}{}", packet_acc_data, cur_packet_data);
                            packet_acc_size += data.len();

                            if packet_acc_size == packet_total_size {
                                println!("[TCP] Data on: {}", session_id);
                                println!(
                                    "request id---, {} {}",
                                    packet_request_id,
                                    packet_acc_data.to_string()
                                );

                                // Add data to responses hashmap
                                let _ = tx
                                    .send((packet_request_id, packet_acc_data.to_string()))
                                    .await;

                                packet_acc_size = 0;
                                packet_total_size = 0;
                                packet_acc_data = "".to_string();
                                packet_request_id = "".to_string();
                            }
                            continue;
                        }

                        let packet_string = String::from_utf8(data.to_vec()).unwrap();
                        let mut packet_split = packet_string.split("\n\n\n");
                        let packet_header = packet_split.next().unwrap();
                        let packet_data = packet_split.next().unwrap();

                        let mut packet_header_split = packet_header.split(":::");
                        let request_id = packet_header_split.next().unwrap();
                        let packet_size = packet_header_split.next().unwrap();
                        let packet_size = packet_size.parse::<usize>().unwrap();

                        println!("hola bloqeuado");

                        // First packet appear, is complete?
                        if packet_size == packet_data.as_bytes().len() {
                            println!("[TCP] Data on: {}", session_id);

                            println!("hola bloqeuado 2");

                            // Add data to responses hashmap
                            let _ = tx
                                .send((request_id.to_string(), packet_data.to_string()))
                                .await;
                        } else {
                            // Packet is not complete
                            println!("hola bloqeuado 3");
                            packet_request_id = request_id.to_string();
                            packet_acc_data = packet_data.to_string();
                            packet_acc_size = packet_data.as_bytes().len();
                            packet_total_size = packet_size;
                        }
                    }
                    Err(e) => {
                        // error
                        eprintln!(
                            "[TCP] Error on socket connection: {} \n\n {}",
                            session_id, e
                        );
                        break;
                    }
                }
            }
            _ => {}
        }
    }
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
        eprintln!("[HTTP] server error: {}", e);
    }
}

async fn http_connection_handler(
    _req: hyper::Request<Body>,
    sessions: Sessions,
) -> Result<Response<Body>, hyper::Error> {
    let (session_endpoint, agent_request_path) = get_request_url(&_req);
    let request_path = _req.uri().path();
    println!("[HTTP] New connection {}", request_path);

    if request_path == "/" {
        let response = Response::builder()
            .status(200)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>Home</h1>"))
            .unwrap();
        return Ok(response);
    }

    let mut sessions = sessions.lock().await; // get access to hashmap
    if !sessions.contains_key(session_endpoint.as_str()) {
        let response = Response::builder()
            .status(404)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>404 Not found</h1>"))
            .unwrap();
        return Ok(response);
    }
    let session = sessions.get_mut(session_endpoint.as_str());

    // Create raw http from request
    // ----------------------------
    let request_id = StringGenerator::default().next_id();
    // headers
    let http_request_info = format!(
        "{} {} {:?}\n",
        _req.method().as_str(),
        agent_request_path,
        _req.version()
    );
    let mut request = request_id.clone() + "\n" + http_request_info.as_str();

    for (key, value) in _req.headers() {
        match value.to_str() {
            Ok(value) => request += &format!("{}: {}\n", capitilize(key.as_str()), value),
            Err(_) => request += &format!("{}: {:?}\n", capitilize(key.as_str()), value.as_bytes()),
        }
    }

    // body
    let body = hyper::body::to_bytes(_req.into_body()).await;
    if body.is_ok() {
        let body = body.unwrap();
        if body.len() > 0 {
            request += &format!("\n{}", String::from_utf8(body.to_vec()).unwrap());
        }
    }

    let session = match session {
        Some(session) => session,
        None => {
            let response = Response::builder()
                .status(500)
                .header("Content-type", "text/html")
                .body(Body::from("<h1>500 Internal server error</h1>"))
                .unwrap();
            return Ok(response);
        }
    };

    // Send raw http to tcp socket
    session.socket_tx.send(request).await.unwrap();

    // Wait for response
    let max_time = 5000; // 100 seconds
    let mut time = 0;
    let mut http_raw_response = String::from("");
    loop {
        // Check if response is ready
        if let Some(agent_response) = session.responses_rx.recv().now_or_never() {
            let agent_response = agent_response.unwrap();
            if request_id == agent_response.0 {
                http_raw_response = agent_response.1;
                break;
            }
        }

        // Check if timeout
        if time >= max_time {
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        time += 100;
    }

    println!("-------------------");
    println!("response {:?}", http_raw_response);

    if time >= max_time {
        let response = Response::builder()
            .status(500)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>524 A timeout error ocurred</h1>"))
            .unwrap();
        return Ok(response);
    }

    /*
    // Get response from hashmap and remove it
    let response = session.responses.remove(&request_id);
    let response = match response {
        Some(response) => response,
        None => {
            let response = Response::builder()
                .status(500)
                .header("Content-type", "text/html")
                .body(Body::from("<h1>500 Internal server error</h1>"))
                .unwrap();
            return Ok(response);
        }
    };

    println!("Response: {}", response); */
    Ok(Response::new(Body::from("Hello, World!")))
}

fn get_request_url(_req: &hyper::Request<Body>) -> (String, String) {
    let referer = _req.headers().get("referer");

    // [no referer]
    if !referer.is_some() {
        let mut segments = _req.uri().path().split("/").collect::<Vec<&str>>(); // /abc/paco -> ["", "abc", "paco"]
        let session_endpoint = segments[1].to_string();
        segments.drain(0..2); // request path
        return (session_endpoint, "/".to_string() + &segments.join("/"));
    }

    let referer = referer.unwrap().to_str().unwrap(); // https://localhost:8080/abc/paco
    let mut referer = referer
        .split("/")
        .map(|r| r.to_string())
        .collect::<Vec<String>>();
    referer.drain(0..3); // drop -> "https:" + "" + "localhost:8080"

    let mut url = _req
        .uri()
        .path()
        .split("/")
        .map(|r| r.to_string())
        .filter(|r| r != "")
        .collect::<Vec<String>>();

    // [different session-endpoint]
    if referer[0] != url[0] {
        return (referer[0].clone(), "/".to_string() + &url.join("/"));
    }

    // [same session-endpoint]
    url.drain(0..1);
    return (referer[0].clone(), "/".to_string() + &url.join("/"));
}

fn capitilize(string: &str) -> String {
    let segments = string.split("-");
    let mut result: Vec<String> = Vec::new();

    for segment in segments {
        let mut chars = segment.chars();
        let mut capitalized = String::new();
        if let Some(first_char) = chars.next() {
            capitalized.push(first_char.to_ascii_uppercase());
        }
        for c in chars {
            capitalized.push(c);
        }

        result.push(capitalized);
    }

    result.join("-")
}
