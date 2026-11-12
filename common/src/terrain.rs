use serde::{ Serialize, Deserialize };
const TILE_SIZE: usize = 60;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum BackTile {
    Grass,
    Stone,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum FrontTile {
    Tree,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Tile {
    pub front: FrontTile,
    pub back: BackTile,
}

impl Tile {
    pub const fn grass() -> Self {
        Tile {
            front: FrontTile::None,
            back: BackTile::Grass
        }
    }
}
