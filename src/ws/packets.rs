use std::convert::TryFrom;

use anyhow::anyhow;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::{Error, SerializeStruct};

macro_rules! parse_packet {
    ($a:ident) => {match serde_json::from_str(&$a) { Err(e) => {return IncomingPacket::Invalid(anyhow!(e))} Ok(a) => a }};
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PacketBase {
    id: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NameIncoming {
    name: String,
}

#[derive(Debug)]
pub enum IncomingPacket {
    Name(String),
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
                let NameIncoming { name } = parse_packet!(source);
                IncomingPacket::Name(name)
            }
            _ => {
                IncomingPacket::InvalidID
            }
        }
    }
}

#[derive(Serialize)]
pub enum ErrorType {
    PacketInvalidID,
    PacketDeserializationError,
}

pub enum OutgoingPacket {
    Error(ErrorType, String)
}

impl Serialize for OutgoingPacket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match self {
            OutgoingPacket::Error(error_type, msg) => {
                let mut state = serializer.serialize_struct("ErrorPacket", 3)?;
                state.serialize_field("id", &-1);
                state.serialize_field("message", msg);
                state.serialize_field("error", error_type);
                state.end()
            }
        }
    }
}
