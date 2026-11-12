use super::*;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player;

// #[derive(hecs::Bundle)]
// pub struct PlayerBundle {
//     player: Player,
//     transform: Transform,
// }
