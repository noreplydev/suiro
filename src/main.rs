use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use unique_id::string::StringGenerator;
use unique_id::Generator;

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

    // Start the TCP server in a new thread
    let http = thread::spawn(move || {
        http_server(http_port);
    });

    // Start the TCP server in a new thread
    let tcp = thread::spawn(move || {
        tcp_server(tcp_port);
    });

    // Wait for the threads to finish
    let _ = http.join();
    let _ = tcp.join();
}

fn tcp_server(port: Port) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).unwrap();
    let requests = listener.incoming();

    println!("[TCP] Waiting connections on {}", port.num);

    for stream in requests {
        let stream = stream.unwrap();

        tcp_connection_handler(stream);
    }
}

fn tcp_connection_handler(stream: TcpStream) {
    let gen = StringGenerator::default();
    println!("[TCP] New connection: {}", gen.next_id());
}

fn http_server(port: Port) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).unwrap();
    println!("[HTTP] Waiting connections on {}", port.num);

    for incoming in listener.incoming() {
        let stream = incoming.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut http_request: Vec<String> = Vec::new();

    // Read headers.
    for line in buf_reader.by_ref().lines() {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        http_request.push(line);
    }

    // Extract Content-Length from headers if present.
    let mut content_length = 0;
    for line in &http_request {
        if line.starts_with("Content-Length:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                content_length = parts[1].parse::<usize>().unwrap_or(0);
            }
        }
    }

    // Read body if Content-Length is present and greater than 0.
    if content_length > 0 {
        let mut body = vec![0u8; content_length];
        buf_reader.read_exact(&mut body).unwrap();
        // Here you can process the body if needed.
        println!("Received body: {}", String::from_utf8_lossy(&body));
    }
}
