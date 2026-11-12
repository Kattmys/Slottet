use super::*;
use macroquad::math::{ Vec2 };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform {
    pub pos: Vec2,
    pub rot: Vec2,
    pub vel: Vec2,
    pub dir: Direction,
}

impl Transform {
    pub fn new(pos: Vec2) -> Self {
        Self { pos, rot: Vec2::Y, vel: Vec2::ZERO }
    }
}
