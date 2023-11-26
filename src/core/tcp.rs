use futures::FutureExt;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use unique_id::{string::StringGenerator, Generator};

use crate::entities::{Port, Session, Sessions};

pub async fn tcp_server(port: Port, sessions: Sessions) {
    let listener = TcpListener::bind(("0.0.0.0", port.num)).await.unwrap();
    println!("○ [TCP] Waiting connections on {}", port.num);

    loop {
        let Ok((socket, _addr)) = listener.accept().await else {
            continue;
        };

        let sessions_clone = sessions.clone();
        tokio::spawn(async {
            // spawn a task for each inbound socket
            tcp_connection_handler(socket, sessions_clone).await;
        });
    }
}

async fn tcp_connection_handler(mut socket: TcpStream, sessions: Sessions) {
    let session_id = StringGenerator::default().next_id();
    let session_endpoint = StringGenerator::default().next_id();

    println!("○ [TCP] New connection {session_id}: /{session_endpoint}");
    socket // write request to the agent socket
        .write_all(format!("connection\n{session_endpoint}").as_bytes())
        .await
        .unwrap(); // handle unwrap...

    // Add session to hashmap
    let hashmap_key = session_endpoint.clone();
    let (socket_tx, mut rx) = mpsc::channel(100); // 100 message queue
    let (tx, responses_rx) = mpsc::channel(100); // 100 message queue
    let session = Session::new(socket_tx, responses_rx);
    {
        sessions
            .lock()
            .await
            .insert(hashmap_key, Arc::new(Mutex::new(session))); // create a block to avoid infinite lock
    }

    // Handle incoming data
    let mut packet_request_id = "".to_string();
    let mut packet_acc_data = "".to_string();
    let mut packet_total_size = 0;
    let mut packet_acc_size = 0;

    let mut buffer = [0; 31250]; // 32 Kb
    loop {
        // Write data to socket on request
        if let Some(Some(request)) = rx.recv().now_or_never() {
            let _ = socket.write_all(request.as_bytes()).await;
        }

        if let Some(sock) = socket.read(&mut buffer).now_or_never() {
            match sock {
                Ok(0) => {
                    // connection closed
                    println!("○ [TCP] Connection closed: {}", session_id);
                    break;
                }
                Ok(n) => {
                    // data received
                    let data = &buffer[..n];

                    // check packet integrity
                    let cur_packet_data = String::from_utf8(data.to_vec());
                    let cur_packet_data = match cur_packet_data {
                        Ok(cur_packet_data) => cur_packet_data,
                        Err(_) => {
                            eprintln!("○ [TCP] EPACKGRAG: Not valid utf8");
                            // Add data to responses hashmap
                            let _ = tx.send((packet_request_id, "EPACKFRAG".to_string())).await;

                            packet_acc_size = 0;
                            packet_total_size = 0;
                            packet_acc_data = "".to_string();
                            packet_request_id = "".to_string();

                            continue;
                        }
                    };

                    // Packet fragmentation?
                    if packet_request_id != "" {
                        packet_acc_data = format!("{packet_acc_data}{cur_packet_data}");
                        packet_acc_size = packet_acc_size + cur_packet_data.as_bytes().len();

                        if packet_acc_size == packet_total_size {
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

                    let mut packet_split = cur_packet_data.split("\n\n\n");
                    let packet_header = packet_split.next().unwrap();
                    let packet_data = packet_split.next().unwrap();

                    let mut packet_header_split = packet_header.split(":::");
                    let request_id = packet_header_split.next().unwrap();
                    let packet_size = packet_header_split.next().unwrap();
                    let packet_size = packet_size.parse::<usize>().unwrap();

                    // First packet appear, is complete?
                    if packet_size == packet_data.as_bytes().len() {
                        // Add data to responses hashmap
                        let _ = tx
                            .send((request_id.to_string(), packet_data.to_string()))
                            .await;
                    } else {
                        // Packet is not complete
                        packet_request_id = request_id.to_string();
                        packet_acc_data = packet_data.to_string();
                        packet_acc_size = packet_data.as_bytes().len();
                        packet_total_size = packet_size;
                    }
                }
                Err(_) => {
                    // error
                    eprintln!("○ [TCP] Error on socket connection: {session_id}");
                    break;
                }
            }
        }
    }
}
