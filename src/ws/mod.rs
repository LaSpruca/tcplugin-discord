use std::{
    collections::HashMap,
    env,
    sync::Arc,
};

use anyhow::Error;
use tokio::{
    net::{
        TcpListener, TcpStream,
    },
    sync::Mutex,
};
use uuid::Uuid;

use crate::ws::client::WsClient;

mod client;
mod packets;

pub struct WsManager {
    connections: Am<HashMap<Uuid, Am<WsClient>>>,
}

pub type Am<T> = Arc<Mutex<T>>;

#[macro_export]
macro_rules! am {
    ($a:expr) => {Arc::new(Mutex::new($a))};
}

impl WsManager {
    pub async fn new() -> Self {
        let _ = env_logger::try_init();
        let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");

        let connections = am!(HashMap::new());
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

    fn handle_stream(connections: Am<HashMap<Uuid, Am<WsClient>>>, stream: TcpStream) {
        tokio::spawn(async move {
            let new_uuid = Uuid::new_v4();
            connections.lock().await.insert(
                new_uuid.clone(),
                WsClient::new(new_uuid, stream).await);
        });
    }

    async fn get_server_uuid(&self, uuid: Uuid) -> Option<Am<WsClient>> {
        if let Some(connection) = self.connections.lock().await.get(&uuid) {
            return Some(connection.to_owned());
        } else {
            return None;
        }
    }

    async fn get_server_by_name(&self, name: &str) -> Option<Am<WsClient>> {
        self.connections.lock().await.iter().filter(|(_, value)| { value.name == name });
        None
    }
}