use common::terrain::Tile;

use serde::{ Serialize, Deserialize };
use macroquad::math::Vec2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Terrain {
    pub origin_pos: Vec2,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl Terrain {
    #[allow(unused)]
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.width && y < self.height {
            self.tiles.get(y * self.width + x)
        } else {
            None
        }
    }
}
