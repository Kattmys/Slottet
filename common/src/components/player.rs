use super::{ Transform, MovementState };
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player;

#[derive(hecs::Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub transform: Transform,
    pub movement_state: MovementState,
}
