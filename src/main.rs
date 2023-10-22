use std::io::{BufRead, BufReader};
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
    let requests = listener.incoming();

    println!("[HTTP] Waiting connections on {}", port.num);

    for stream in requests {
        let stream = stream.unwrap();

        http_connection_handler(stream);
    }
}

fn http_connection_handler(stream: TcpStream) {
    let raw = get_raw_request(&stream).unwrap();
    println!("[HTTP] New connection: {}", raw);
}

fn get_raw_request(stream: &TcpStream) -> Result<String, std::io::Error> {
    let buf_reader = BufReader::new(stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let raw = http_request.join("\r\n");

    Ok(raw)
}
