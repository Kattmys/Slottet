use super::*;
use macroquad::math::{ Vec2 };

#[derive(Serialize, Deserialize, Debug)]
pub struct Transform {
    pub position: Vec2,
}
