use anyhow::anyhow;
use serde::{Deserialize};

macro_rules! parse_packet {
    ($a:ident) => {
        match serde_json::from_str(&$a) {
            Err(e) => {
                return IncomingPacket::Invalid(anyhow!(e))
            }
            Ok(a) => a
        }
    };
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PacketBase {
    id: i32,
}

/// Packet for setting the name of the server
/// # Packet Structure
/// ```
/// id: 0
/// name String
/// ```
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SetNamePacket {
    name: String,
}

/// Packet for setting the discord server (guild) that can manage a server
/// # Packet Structure
/// ```
/// id: 1
/// guildId: String
/// ```
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SetServerPacket {
    guild_id: String,
}

/// Struct to represent any incoming packet
#[derive(Debug)]
pub enum IncomingPacket {
    SetName(String),
    SetServer(String),
    InvalidID,
    Invalid(anyhow::Error),
}

impl From<String> for IncomingPacket {
    fn from(source: String) -> Self {
        let PacketBase { id } = match serde_json::from_str::<PacketBase>(&source) {
            Ok(x) => { x }
            Err(err) => {
                return IncomingPacket::Invalid(anyhow!(err));
            }
        };

        match id {
            0 => {
                let SetNamePacket { name } = parse_packet!(source);
                IncomingPacket::SetName(name)
            }
            1 => {
                let SetServerPacket { guild_id } = parse_packet!(source);
                IncomingPacket::SetServer(guild_id)
            }
            _ => {
                IncomingPacket::InvalidID
            }
        }
    }
}
