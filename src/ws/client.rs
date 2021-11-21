use std::sync::Arc;

use crate::discord::server_command::ServerCommand;
use futures::prelude::*;
use log::{debug, error, info};
use tokio::sync::Mutex;
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::ws::packets::{ErrorType, IncomingPacket, OutgoingPacket};

#[derive(Clone)]
pub struct WsClient {
    outgoing_stream: Sender<OutgoingPacket>,
    pub(super) name: String,
    pub(super) guild_id: String,
    uuid: Uuid,
    pub(super) alive: bool,
}

impl WsClient {
    /// Scaffold out a new client and start the main event loop
    pub(super) async fn new(uuid: Uuid, stream: TcpStream) -> Arc<Mutex<WsClient>> {
        let (outgoing_stream, incoming_stream) = tokio::sync::mpsc::channel::<OutgoingPacket>(16);

        let gamer = Arc::new(Mutex::new(Self {
            outgoing_stream,
            name: Default::default(),
            guild_id: Default::default(),
            uuid,
            alive: true,
        }));

        tokio::spawn(Self::main_loop(gamer.clone(), stream, incoming_stream));

        gamer
    }

    async fn main_loop(
        this: Arc<Mutex<Self>>,
        stream: TcpStream,
        mut incoming_stream: Receiver<OutgoingPacket>,
    ) {
        debug!("Upgrading client");
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let (mut sender, mut receiver) = ws_stream.split();

        debug!("Upgraded client");

        loop {
            tokio::select! {
                packet = receiver.next() => {
                    if let Some(Ok(Message::Text(message))) = packet {
                        let mut lock = this.lock().await;
                        lock.handle_packet(message).await;
                    } else {
                        let mut lock = this.lock().await;
                        lock.kill();
                        sender.send(Message::Close(None)).await;
                        sender.close().await;
                        break;
                    }
                },
                agree = incoming_stream.recv() => {
                    let serialized = serde_json::to_string(&agree.unwrap()).unwrap();
                    info!("Sending packet to {}", this.lock().await.uuid);
                    sender.send(Message::Text(serialized)).await.unwrap();
                }
            }
        }
    }

    async fn handle_packet(&mut self, packet: String) {
        let parsed = IncomingPacket::from(packet);

        match parsed {
            IncomingPacket::SetName(new_name) => {
                info!("Set name to: {} for {}", &new_name, self.uuid.to_string());
                self.name = new_name;
            }
            IncomingPacket::SetServer(guild_id) => {
                info!("Set server to: {} for {}", &guild_id, self.uuid.to_string());
                self.guild_id = guild_id;
            }
            IncomingPacket::InvalidID => {
                self.outgoing_stream
                    .send(OutgoingPacket::Error(
                        ErrorType::PacketInvalidID,
                        format!("Invalid packet ID"),
                    ))
                    .await
                    .unwrap_or(());
            }
            IncomingPacket::Invalid(err) => {
                debug!("Received Invalid packet for {}", self.name);
                self.outgoing_stream
                    .send(OutgoingPacket::Error(
                        ErrorType::PacketDeserializationError,
                        format!("{}", err),
                    ))
                    .await
                    .unwrap_or(());
            }
        }
    }

    pub async fn send_server_command(&self, exec: ServerCommand) {
        self.outgoing_stream
            .send(OutgoingPacket::ServerRun(exec))
            .await;
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn kill(&mut self) {
        info!("Stopping {}", self.uuid);
        self.alive = false;
    }
}
