use std::convert::TryFrom;
use tokio::sync::broadcast::Receiver;
use tokio_tungstenite::tungstenite::Message;
use convert_case::{Case, Casing};
use uuid::Uuid;

use super::*;

pub struct WsClient {
    incoming: Receiver<(WsMessage, WsTarget)>,
    name: String,
    uuid: Uuid
}

impl WsClient {
    pub async fn new(incoming: Receiver<(WsMessage, WsTarget)>, uuid: Uuid) -> Self {
        Self {
            incoming,
            uuid,
            name: Default::default(),
        }
    }

    pub async fn accept(mut self, stream: TcpStream) {
        let addr = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", addr);

        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        info!("New WebSocket connection: {}", addr);

        let (write, mut read) = ws_stream.split();

        loop {
            tokio::select! {
                ws_incoming = read.next() => {
                    if let Message::Text(message) = ws_incoming.unwrap().unwrap() {
                        self.handle_packet(message);
                    }
                },
                queue_incomming = self.incoming.recv() => {
                    println!("Received message: {:?}", queue_incomming);
                }
            }
        }
    }

    fn handle_packet(&mut self, packet: String) {
        let packet = IncomingPacket::try_from(packet).unwrap();
        match packet {
            IncomingPacket::Name(name) => {
                self.name = name.trim().to_case(Case::Camel).to_string();
                println!("Name {}", name);
            }
        }
    }
}