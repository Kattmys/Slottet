pub mod network;
pub mod components;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>; // temporary

use hecs;
use macroquad::math::{ Vec2 };
use serde::{ Serialize, Deserialize };

// from server
#[derive(Serialize, Deserialize)]
pub enum ServerPacket {
    Pong,
    ClientVelocity {
        entity: hecs::Entity,
        velocity: Vec2,
    },
    PlayerPosition {
        entity: hecs::Entity,
        position: Vec2,
    },
    InitialData {
        entity: hecs::Entity,
    },
}

// from client
#[derive(Serialize, Deserialize)]
pub enum ClientPacket {
    Ping,
    Velocity(Vec2),
}
