use futures_util::{ StreamExt };
use log::info;
use tokio::net::{TcpStream};
use uuid::Uuid;

pub use self::{client::WsClient, manager::WsManager, packets::IncomingPacket};

mod manager;
mod client;
pub mod packets;

#[derive(Clone, Debug)]
pub enum WsMessage {}

#[derive(Clone, Debug)]
pub enum WsTarget {
    All,
    Specific(Uuid),
}
