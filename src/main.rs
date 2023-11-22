mod core;
mod entities;
use crate::entities::{Port, Session};
use core::http::http_server;
use core::tcp::tcp_server;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

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

    let mutex: Mutex<HashMap<String, Arc<Mutex<Session>>>> = Mutex::new(HashMap::new());
    let sessions = Arc::new(mutex);

    let tcp = async {
        tcp_server(tcp_port, sessions.clone()).await;
    };

    let http = async {
        http_server(http_port, sessions.clone()).await;
    };

    futures::join!(tcp, http);
}
