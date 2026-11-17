use serde::{ Serialize, Deserialize };
pub const TILE_SIZE: f32 = 60.;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum BackTile {
    Grass,
    Stone,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum FrontTile {
    Tree,
    Wall,
    Stone,
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
