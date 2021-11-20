//! All of the serialization/deserialization code for packets sent and received over websockets
mod incoming;
mod outgoing;

pub use incoming::IncomingPacket;
pub use outgoing::*;
