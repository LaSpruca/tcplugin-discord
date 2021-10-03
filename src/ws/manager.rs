use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use log::info;
use tokio::net::{TcpListener};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

use super::*;

type Data<T> = Arc<Mutex<T>>;

#[macro_export]
macro_rules! data {
    ($a:expr) => {Arc::new(Mutex::new($a))};
}

pub struct WsManager {
    message_stream: Sender<(WsMessage, WsTarget)>,
    active_servers: Data<HashMap<Uuid, String>>,
}

impl WsManager {
    pub async fn new() -> Self {
        let _ = env_logger::try_init();
        let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        info!("Listening on: {}", addr);

        let (outgoing, _): (Sender<(WsMessage, WsTarget)>, Receiver<(WsMessage, WsTarget)>) = tokio::sync::broadcast::channel(128);
        let outgoing2 = outgoing.clone();

        let (incoming_sender, incoming) = tokio::sync::mpsc::channel(12);

        let active_servers = data!(HashMap::new());
        let active_servers2 = active_servers.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listner.accept() => {
                        if let Some((stream, _)) = result {
                            let uuid = Uuid::new_v4();
                            tokio::spawn(
                                WsClient::new(outgoing2.subscribe(), uuid.clone()).await.accept(stream)
                            );
                        }
                    }
                }
            }
        });

        Self {
            message_stream: outgoing,
            active_servers
        }
    }
}