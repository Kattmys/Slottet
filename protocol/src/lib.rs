pub mod network;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use serde::{ Serialize, Deserialize };

// from server
#[derive(Serialize, Deserialize)]
pub enum ServerPacket {
    Pong,
}

// from client
#[derive(Serialize, Deserialize)]
pub enum ClientPacket {
    Ping,
}
