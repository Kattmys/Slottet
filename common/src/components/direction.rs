use serde::{ Deserialize, Serialize };
use macroquad::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

impl From<&MoveDirection> for Vec2 {
    fn from(dir: &MoveDirection) -> Self {
        match dir {
            MoveDirection::North     => vec2(0., 1.),
            MoveDirection::NorthEast => vec2(1., 1.),
            MoveDirection::East      => vec2(1., 0.),
            MoveDirection::SouthEast => vec2(1., -1.),
            MoveDirection::South     => vec2(0., -1.),
            MoveDirection::SouthWest => vec2(-1., -1.),
            MoveDirection::West      => vec2(-1., 0.),
            MoveDirection::NorthWest => vec2(-1., 1.),
            MoveDirection::None      => vec2(0., 0.),
        }
    }
}

pub type MoveDirection = Direction; // component
