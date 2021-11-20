use std::{collections::HashMap, env, sync::Arc};

use tokio::{
    net::{TcpListener, TcpStream},
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
    ($a:expr) => {
        Arc::new(Mutex::new($a))
    };
}

impl WsManager {
    pub async fn new() -> Self {
        let addr = env::args()
            .nth(1)
            .unwrap_or_else(|| "127.0.0.1:8080".to_string());

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

        Self { connections }
    }

    fn handle_stream(connections: Am<HashMap<Uuid, Am<WsClient>>>, stream: TcpStream) {
        tokio::spawn(async move {
            let new_uuid = Uuid::new_v4();
            connections
                .lock()
                .await
                .insert(new_uuid.clone(), WsClient::new(new_uuid, stream).await);
        });
    }

    pub async fn get_connected_by_guild(&self, guild_id: String) -> Vec<(Uuid, Am<WsClient>)> {
        self.connections
            .lock()
            .await
            .iter()
            .filter(|(_, connection)|
                futures::executor::block_on(async {
                    &connection.lock().await.guild_id == &guild_id
                }
                )
            )
            .map(|(e, connection)| {
                (e.to_owned(), connection.to_owned())
            })
            .collect::<Vec<(Uuid, Am<WsClient>)>>()
    }

    pub async fn get_connection_by_name(&self, name: String, guild_id: String) -> Am<WsClient> {
        self.connections
            .lock()
            .await
            .iter()
            .filter(|(_, connection)|
                futures::executor::block_on(async {
                    let lock = connection.lock().await;
                    &lock.guild_id == &guild_id && &lock.name == &name
                }
                )
            )
            .next()
            .unwrap()
            .1
            .to_owned()
    }


    pub async fn get_connection_by_uuid(&self, uuid: Uuid) -> Am<WsClient> {
        self.connections
            .lock()
            .await
            .iter()
            .filter(|(e, connection)|
                futures::executor::block_on(async { e.to_owned() == &uuid }))
            .next()
            .unwrap()
            .1
            .to_owned()
    }
}
