use super::{ Transform, MovementState, Inventory };
use serde::{ Deserialize, Serialize };

pub const ENTITY_SPRITE_SIZE: f32 = 0.6;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player;

#[derive(hecs::Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub transform: Transform,
    pub movement_state: MovementState,
    pub inventory: Inventory,
}
