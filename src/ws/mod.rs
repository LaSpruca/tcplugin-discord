mod client;
mod packets;

use crate::ws::client::WsClient;
use log:: info;
use regex::Regex;
use std::time::Duration;
use std::{collections::HashMap, env, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use twilight_http::client::Client as HttpClient;
use uuid::Uuid;

pub struct WsManager {
    connections: Am<HashMap<Uuid, Am<WsClient>>>,
    get_http: Am<Option<Arc<HttpClient>>>,
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
        let get_discord: Arc<Mutex<Option<Arc<HttpClient>>>> = am!(None);
        let get_discord2 = get_discord.clone();

        tokio::spawn(async move {
            loop {
                let k = get_discord2.clone();
                let lock = k.lock().await;
                if lock.is_some() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
            }
            while let Ok((stream, addr)) = listener.accept().await {
                let connections3 = connections2.clone();
                let get_http = get_discord2.clone();
                let locked = get_http.lock().await;
                Self::handle_stream(connections3, (*locked).as_ref().unwrap().clone(), stream);
                info!("New connection from {}", addr);
            }
        });

        let connections2 = connections.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                let mut connections = connections2.lock().await;
                for (id, _) in connections.clone().iter().filter(|(_, client)| {
                    futures::executor::block_on(async { !client.lock().await.alive })
                }) {
                    connections.remove(id);
                    info!("Removing dead client {}", id);
                }
                drop(connections);
            }
        });

        Self {
            connections,
            get_http: get_discord,
        }
    }

    fn handle_stream(
        connections: Am<HashMap<Uuid, Am<WsClient>>>,
        http: Arc<HttpClient>,
        stream: TcpStream,
    ) {
        tokio::spawn(async move {
            let new_uuid = Uuid::new_v4();
            connections.lock().await.insert(
                new_uuid.clone(),
                WsClient::new(new_uuid, http, stream).await,
            );
        });
    }

    pub async fn get_connected_by_ctrl_channel_id(&self, ctrl_channel_id: String) -> Vec<(Uuid, Am<WsClient>)> {
        self.connections
            .lock()
            .await
            .iter()
            .filter(|(_, connection)| {
                futures::executor::block_on(async {
                    &connection.lock().await.ctrl_channel_id == &ctrl_channel_id
                })
            })
            .map(|(e, connection)| (e.to_owned(), connection.to_owned()))
            .collect::<Vec<(Uuid, Am<WsClient>)>>()
    }
    pub async fn get_connections_by_regex(
        &self,
        regex: Regex,
        ctrl_channel_id: String,
    ) -> Vec<(Uuid, Am<WsClient>)> {
        self
            .connections
            .lock()
            .await
            .iter()
            .filter(|(_, connection)| {
                futures::executor::block_on(async {
                    let lock = connection.lock().await;
                    let a = &lock.ctrl_channel_id == &ctrl_channel_id;
                    let b = regex.is_match(&lock.name);
                    a && b
                })
            })
            .map(|(a, b)| (a.to_owned(), b.to_owned()))
            .collect::<Vec<(Uuid, Am<WsClient>)>>()
    }

    pub async fn get_connection_by_name(
        &self,
        name: String,
        ctrl_channel_id: String,
    ) -> Option<(Uuid, Am<WsClient>)> {
        match self
            .connections
            .lock()
            .await
            .iter()
            .filter(|(_, connection)| {
                futures::executor::block_on(async {
                    let lock = connection.lock().await;
                    &lock.ctrl_channel_id == &ctrl_channel_id && &lock.name == &name
                })
            })
            .next()
        {
            Some((a, b)) => Some((a.to_owned(), b.to_owned())),
            None => None,
        }
    }

    pub async fn set_get_http(&mut self, fun: Arc<HttpClient>) {
        *self.get_http.lock().await = Some(fun);
    }
}
