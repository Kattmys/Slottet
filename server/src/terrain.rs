use common::terrain::{ Tile, FrontTile };

pub fn generate_terrain<const W: usize, const H: usize>() -> [[Tile; W]; H] {
    let mut map = [[const { Tile::grass() }; W]; H];

    for y in 0..H {
        for x in 0..W {
            if rand::random_bool(1.0 / 30.0) {
                map[y][x].front = FrontTile::Stone;
            }
        }
    }

    map
}

pub fn print_terrain<const W: usize, const H: usize>(terrain: &[[Tile; W]; H]) {
    for _ in 0..terrain[0].len() {
        print!("=");
    }
    println!("====");
    for row in terrain {
        print!("||");
        for tile in row {
            if matches!(tile.front, FrontTile::Stone) {
                print!("x");
            } else {
                print!(" ");
            }
        }
        println!("||");
    }
    for _ in 0..terrain[0].len() {
        print!("=");
    }
    println!("====");
}
