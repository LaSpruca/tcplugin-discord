use std::convert::TryFrom;
use serde::Deserialize;
use anyhow::anyhow;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct PacketBase {
    id: i32
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct NameIncoming {
    name: String
}

pub enum IncomingPacket {
    Name(String),
}

impl TryFrom<String> for IncomingPacket {
    type Error = anyhow::Error;

    fn try_from(source: String) -> Result<Self, Self::Error> {
        let PacketBase { id} =  serde_json::from_str::<PacketBase>(&source)?;

        match id {
            0 => {
                let NameIncoming { name } = serde_json::from_str(&source)?;
                Ok(IncomingPacket::Name(name))
            }
            _ => {
                Err(anyhow!("Invalid packet id, {}", id))
            }
        }
    }
}