use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Session {
    pub socket_tx: mpsc::Sender<String>,
    pub responses_rx: mpsc::Receiver<(String, String)>,
}

impl Session {
    pub fn new(
        socket_tx: mpsc::Sender<String>,
        responses_rx: mpsc::Receiver<(String, String)>,
    ) -> Self {
        Session {
            socket_tx,
            responses_rx,
        }
    }
}
