mod client;

use std::{
    collections::HashMap,
    env,
    sync::Arc,
};

use tokio::{
    net::{
        TcpListener, TcpStream,
    },
    sync::Mutex,
    };
use uuid::Uuid;
use crate::ws::client::WsClient;

pub struct WsManager {
    connections: Arc<Mutex<HashMap<Uuid, Mutex<WsClient>>>>,
}

impl WsManager {
    pub async fn new() -> Self {
        let _ = env_logger::try_init();
        let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");

        let connections = Arc::new(Mutex::new(HashMap::new()));
        let connections2 = connections.clone();

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let connections3 = connections2.clone();
                Self::handle_stream(connections3, stream);
            }
        });

        Self {
            connections
        }
    }

    fn handle_stream(connections: Arc<Mutex<HashMap<Uuid, Mutex<WsClient>>>>, stream: TcpStream) {
        tokio::spawn(async move {
            connections.lock().await.insert(
                Uuid::new_v4(),
                Mutex::new(WsClient::new(stream).await));
        });
    }
}