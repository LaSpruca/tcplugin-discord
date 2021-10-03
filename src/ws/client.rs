use tokio::{net::TcpStream, sync::mpsc::Sender};
use futures::{prelude::*};
use tokio_tungstenite:: tungstenite::Message;

#[derive(Clone)]
pub struct WsClient {
    outgoing_stream: Sender<String>,
}

impl WsClient {
    pub(super) async fn new(stream: TcpStream) -> WsClient {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let (outgoing_stream, mut incoming_stream) = tokio::sync::mpsc::channel::<String>(16);

        let (mut sender, mut reciver) = ws_stream.split();

        let s = outgoing_stream.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    packet = reciver.next() => {
                        if let Some(Ok(Message::Text(message))) = packet {
                            println!("{}", &message);
                            s.send(message).await.unwrap();
                        } else {
                            println!("Fuk");
                        }
                    },
                    agree = incoming_stream.recv() => {
                        sender.send(Message::Text(agree.unwrap())).await.unwrap();
                    }
                }
            }
        });

        Self {
            outgoing_stream
        }
    }
}
