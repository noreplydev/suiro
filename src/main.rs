use std::io::{BufRead, BufReader, Error, Read, Write};
use std::thread;
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

fn main() {
    println!("[SUIRO] Starting service");
    ctrlc::set_handler(move || {
        println!("[SUIRO] Stopping service");
        std::process::exit(0);
    })
    .expect("[SUIRO](ERROR) setting Ctrl-C handler");

    let http_port = Port::new(8080);
    let tcp_port = Port::new(3040);

    // Start the TCP server in a new thread
    /*     let http = thread::spawn(move || {
        let runtime =
            tokio::runtime::Builder::build(&mut tokio::runtime::Builder::new_current_thread());
        let runtime = runtime.unwrap();

        runtime.block_on(async { http_server(http_port).await });
    }); */

    // Start the TCP server in a new thread
    let tcp = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async { tcp_server(tcp_port).await });
    });

    // Wait for the threads to finish
    //let _ = http.join();
    let _ = tcp.join();
}

async fn tcp_server(port: Port) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).await.unwrap();
    println!("[TCP] Waiting connections on {}", port.num);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            tcp_connection_handler(socket).await;
        });
    }

    /*     let listener = TcpListener::bind(("127.0.0.1", port.num)).unwrap();
    let requests = listener.incoming();

    println!("[TCP] Waiting connections on {}", port.num);

    for stream in requests {
        let stream = stream.unwrap();

        tcp_connection_handler(stream);
    } */
}

async fn tcp_connection_handler(stream: TcpStream) {
    let gen = StringGenerator::default();
    println!("[TCP] New connection: {}", gen.next_id());
}
/*

async fn http_server(port: Port) {
    let listener = TcpListener::bind(("127.0.0.1", port.num)).unwrap();
    println!("[HTTP] Waiting connections on {}", port.num);

    let total_bytes = 0;
    let readed_bytes = 0;
    let readed = "";

    for incoming in listener.incoming() {
        println!("Spawning async function");
        let algo = tokio::spawn(handle_http_connection(incoming));

        if algo.is_finished() {
            println!("termino {:?}", algo);
        }

        println!("yo que se");
        /*
               // Extract Content-Length from headers if present.
               let mut content_length = 0;
               for line in &headers {
                   if line.to_lowercase().starts_with("content-length:") {
                       let parts: Vec<&str> = line.split_whitespace().collect();
                       if parts.len() > 1 {
                           content_length = parts[1].parse::<usize>().unwrap_or(0);
                       }
                   }
               }

               // Read body if Content-Length is present and greater than 0.
               if content_length > 0 {
                   println!("[HTTP] Body length: {}", content_length);
                   let mut body = vec![0u8; content_length];

                   buf_reader.read_exact(&mut body).unwrap();

                   println!("Received body: {}", String::from_utf8_lossy(&body));
               }
        */
        //handle_connection(stream);
    }
}
async fn handle_http_connection(incoming: Result<TcpStream, Error>) {
    println!("ultima");
    let mut stream = incoming.unwrap();
    let mut buf_reader = BufReader::new(&stream);

    let mut headers: Vec<String> = Vec::new();
    let mut body: Vec<String> = Vec::new();

    // Read headers.
    for (index, line) in buf_reader.by_ref().lines().enumerate() {
        let line = line.unwrap();
        if line.is_empty() {
            let (_headers, body_lines): (Vec<_>, Vec<_>) = buf_reader
                .by_ref()
                .lines()
                .enumerate()
                .partition(|(i, _)| i > &index);

            body = body_lines
                .into_iter()
                .filter_map(|(_, result)| match result {
                    Ok(string) => Some(string),
                    Err(_) => {
                        println!("[HTTP] Error parsing body line");
                        None
                    }
                })
                .collect();
            break;
        }
        headers.push(line);
    }

    println!("[HTTP] Request headers: \n\n\r{}\n", headers.join("\n"));
    println!("[HTTP] Request body: \n\n\r{}\n", body.join("\n"));
    let _ = stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
}
 */
