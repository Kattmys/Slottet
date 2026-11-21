use macroquad::prelude::*;
use serde::{ Deserialize, Serialize };

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

impl From<&Direction> for Vec2 {
    fn from(dir: &Direction) -> Self {
        match dir {
            Direction::North     => vec2(0., -1.),
            Direction::NorthEast => vec2(1., -1.),
            Direction::East      => vec2(1., 0.),
            Direction::SouthEast => vec2(1., 1.),
            Direction::South     => vec2(0., 1.),
            Direction::SouthWest => vec2(-1., 1.),
            Direction::West      => vec2(-1., 0.),
            Direction::NorthWest => vec2(-1., -1.),
            Direction::None      => vec2(0., 0.),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovementState {
    pub dir: Direction,
}
