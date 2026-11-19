use crate::Result;
use serde::{ Serialize, Deserialize };
use macroquad::math::Vec2;

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Tile {
    pub front: FrontTile,
    pub back: BackTile,
}


impl Tile {
    pub const fn empty() -> Self {
        Tile {
            front: FrontTile::None,
            back: BackTile::None,
        }
    }

    pub fn has_collision(&self) -> bool {
        let back = match self.back {
            BackTile::Grass | BackTile::Stone => false,
            BackTile::None => true,
        };
        let front = match self.front {
            FrontTile::Tree | FrontTile::Wall | FrontTile::Stone => true,
            FrontTile::None | FrontTile::Flowers => false,
        };

        back | front
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map {
    pub origin_pos: Vec2,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl Map {
    #[allow(unused)]
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.width && y < self.height {
            self.tiles.get(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get_3x3_around(&self, pos: Vec2) -> [Tile; 9] {
        let rel_pos = (pos - self.origin_pos);

        let mut tiles = [Tile::empty(); 9];

        for y in 0..3 {
            for x in 0..3 {
                let local_idx = x + y * 3;

                let (Some(idx_x), Some(idx_y)) = (
                    (rel_pos.x as usize + x).checked_sub(1),
                    (rel_pos.y as usize + y).checked_sub(1),
                ) else {
                    tiles[local_idx] = Tile::empty();
                    continue;
                };
                if let Some(tile) = self.tiles.get(idx_x + idx_y * self.width) {
                    tiles[local_idx] = *tile;
                }
            }
        }

        return tiles;
    }
}
