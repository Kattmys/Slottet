use common::terrain::{ Tile, FrontTile };
use common::utils::epoch_time_millis;

pub fn generate_terrain<const W: usize, const H: usize>() -> [[Tile; W]; H] {
    let mut map = [[const { Tile::grass() }; W]; H];

    for y in 0..H {
        for x in 0..W {
            if x * y * (epoch_time_millis() % (W * H) as u128) as usize + 2 % 1 + x == 0 {
                map[y][x].front = FrontTile::Tree;
            }
        }
    }

    map
}
