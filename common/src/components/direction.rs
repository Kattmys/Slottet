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

pub type MoveDirection = Direction; // component
