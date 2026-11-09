pub mod network;
pub mod components;

// temporary
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use hecs;
use macroquad::math::{ Vec2 };
use serde::{ Serialize, Deserialize };

// from server
#[derive(Serialize, Deserialize)]
pub enum ServerPacket {
    Pong,
    Ecs {
        columns: Columns,
    },
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

type Column<T> = Vec<(hecs::Entity, T)>;

#[allow(non_snake_case)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Columns {
    Player: Column<components::Player>,
    Transform: Column<components::Transform>,
}
