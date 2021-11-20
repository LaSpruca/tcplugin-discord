use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use crate::discord::server_command::ServerCommand;

#[derive(Serialize)]
pub enum ErrorType {
    PacketInvalidID,
    PacketDeserializationError,
}

pub enum OutgoingPacket {
    Error(ErrorType, String),
    ServerRun(ServerCommand),
}

impl Serialize for OutgoingPacket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OutgoingPacket::Error(error_type, msg) => {
                let mut state = serializer.serialize_struct("ErrorPacket", 3)?;
                state.serialize_field("id", &-1)?;
                state.serialize_field("message", msg)?;
                state.serialize_field("error", error_type)?;
                state.end()
            }
            OutgoingPacket::ServerRun(packet) => {
                let mut state = serializer.serialize_struct("ServerRun", 2)?;
                state.serialize_field("id", &0)?;
                state.serialize_field("exec", packet)?;
                state.end()
            }
        }
    }
}
