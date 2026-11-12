use common::terrain::{ Tile, FrontTile, BackTile };
use common::utils::epoch_time;

pub fn generate<const W: usize, const H: usize>() -> [[Tile; W]; H] {
    let mut map = [[const { Tile::grass() }; W]; H];

    for y in 0..H {
        for x in 0..W {
            if x * y + 2 % 1 + x == 0 {
                map[y][x].front = FrontTile::Tree;
            }
        }
    }

    map
}
