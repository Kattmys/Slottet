use crate::Result;
use serde::{ Serialize, Deserialize };
pub const TILE_SIZE: f32 = 60.;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum BackTile {
    Grass,
    Stone,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum FrontTile {
    Tree,
    Wall,
    Stone,
    Flowers,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Tile {
    pub front: FrontTile,
    pub back: BackTile,
}

impl BackTile {
    pub fn from_str(string: &str) -> Result<Self> {
        match string {
            "grass" => Ok(Self::Grass),
            "stone" => Ok(Self::Stone),
            "none" => Ok(Self::None),
            _ => Err(format!("could not create BackTile from {string}").into())
        }
    }
}

impl FrontTile {
    pub fn from_str(string: &str) -> Result<Self> {
        match string {
            "tree" => Ok(Self::Tree),
            "wall" => Ok(Self::Wall),
            "stone" => Ok(Self::Stone),
            "flowers" => Ok(Self::Flowers),
            "none" => Ok(Self::None),
            _ => Err(format!("could not create FrontTile from {string}").into())
        }
    }
}

impl Tile {
    pub const fn empty() -> Self {
        Tile {
            front: FrontTile::None,
            back: BackTile::None,
        }
    }
}
