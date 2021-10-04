use std::sync::Arc;

use anyhow::anyhow;
use futures::{prelude::*};
use tokio::{net::TcpStream, sync::mpsc::{Receiver, Sender}};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::ws::packets::{ErrorType, IncomingPacket, OutgoingPacket};

#[derive(Clone)]
pub struct WsClient {
    outgoing_stream: Sender<OutgoingPacket>,
    pub(super) name: String,
    uuid: Uuid
}

impl WsClient {
    pub(super) async fn new(uuid: Uuid, stream: TcpStream) -> Arc<Mutex<WsClient>> {
        let (outgoing_stream, mut incoming_stream) = tokio::sync::mpsc::channel::<OutgoingPacket>(16);

        let mut gamer = Arc::new(Mutex::new(Self {
            outgoing_stream,
            name: Default::default(),
            uuid
        }));

        tokio::spawn(Self::main_loop(gamer.clone(), stream, incoming_stream));

        gamer
    }

    async fn main_loop(this: Arc<Mutex<Self>>, stream: TcpStream, mut incoming_stream: Receiver<OutgoingPacket>) {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let (mut sender, mut receiver) = ws_stream.split();

        loop {
            tokio::select! {
                    packet = receiver.next() => {
                        if let Some(Ok(Message::Text(message))) = packet {
                            let mut lock = this.lock().await;
                            lock.handle_packet(message).await;
                        } else {
                            println!("Fuk");
                        }
                    },
                    agree = incoming_stream.recv() => {
                        let serialized = serde_json::to_string(&agree.unwrap()).unwrap();
                        sender.send(Message::Text(serialized)).await.unwrap();
                    }
                }
        }
    }

    async fn handle_packet(&mut self, packet: String) {
        let parsed = IncomingPacket::from(packet);

        match parsed {
            IncomingPacket::Name(new_name) => {
                println!("Set name to: {}", &new_name);
                self.name = new_name;
            }
            IncomingPacket::InvalidID => {
                self.outgoing_stream.send(OutgoingPacket::Error(ErrorType::PacketInvalidID, format!("Invalid packet ID")));
            }
            IncomingPacket::Invalid(err) => {
                eprintln!("Received Invalid packet for {}", self.name);
                self.outgoing_stream.send(OutgoingPacket::Error(ErrorType::PacketDeserializationError, format!("{}", err)));
            }
        }
    }
}
