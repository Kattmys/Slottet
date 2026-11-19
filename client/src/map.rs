// use common::map::{ Tile, TILE_SIZE };
//
// use serde::{ Serialize, Deserialize };
// use macroquad::math::Vec2;
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Map {
//     pub origin_pos: Vec2,
//     pub width: usize,
//     pub height: usize,
//     pub tiles: Vec<Tile>,
// }
//
// impl Map {
//     #[allow(unused)]
//     #[must_use]
//     pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
//         if x < self.width && y < self.height {
//             self.tiles.get(y * self.width + x)
//         } else {
//             None
//         }
//     }
//
//     pub fn get_3x3_around(&self, pos: Vec2) -> [[Tile; 3]; 3] {
//         let rel_pos = (pos - self.origin_pos) / TILE_SIZE;
//
//         let tiles = [[Tile; 3]; 3];
//
//         for y: usize in 0..3 {
//             for x: usize in 0..3 {
//                 let (Some(idx_x), Some(idx_y)) = (
//                     (rel_pos.x as usize + x).checked_sub(1),
//                     (rel_pos.y as usize + y).checked_sub(1),
//                 ) else {
//                     tiles[x][y] = Tile::empty();
//                     continue;
//                 }
//
//                 tiles[x][y] = self.tiles.get(idx_x + idx_y * self.width);
//             }
//         }
//
//         return tiles;
//     }
// }
