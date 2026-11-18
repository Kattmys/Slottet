use common::Result;
use common::map::{ Tile, FrontTile, BackTile };
use std::collections::HashMap;
use std::fs::read_to_string;
use serde::Deserialize;

#[derive(Deserialize)]
struct MapData {
    front: String,
    back: String,
    tiles: HashMap<char, String>,
}

pub fn load_map<const W: usize, const H: usize>(filename: &str) -> Result<[[Tile; W]; H]> {
    let mut map = [[const { Tile::empty() }; W]; H];
    let text = read_to_string(filename)?;
    let text = text.trim();
    let data: MapData = toml::from_str(&text)?;

    for (row, characters) in data.back.split('\n').enumerate() {
        for (col, ch) in characters.chars().enumerate() {
            if let Some(id) = data.tiles.get(&ch) {
                match BackTile::from_str(id) {
                    Ok(tile) => {
                        map[row][col].back = tile;
                    }
                    Err(e) => {
                        eprintln!("error while loading map {filename}: {e}");
                    }
                }
            } else {
                eprintln!("error while loading map {filename}: invalid character {ch}");
            }
        }
    }

    for (row, characters) in data.front.split('\n').enumerate() {
        for (col, ch) in characters.chars().enumerate() {
            if let Some(id) = data.tiles.get(&ch) {
                match FrontTile::from_str(id) {
                    Ok(tile) => {
                        map[row][col].front = tile;
                    }
                    Err(e) => {
                        eprintln!("error while loading map {filename}: {e}");
                    }
                }
            } else {
                eprintln!("error while loading map {filename}: invalid character {ch}");
            }
        }
    }

    // random scattering is implemented as follows:
    // the map is divided into divisions, each with the size:
    // (SCATTER_DIVISION_SIZE, SCATTER_DIVISION_SIZE),
    // and in each such division, up to SCATTER_AMOUNT flowers is placed.

    const SCATTER_DIVISION_SIZE: usize = 4;
    const SCATTER_AMOUNT: usize = 1;

    for y in 0..(H / SCATTER_DIVISION_SIZE) {
        for x in 0..(W / SCATTER_DIVISION_SIZE) {
            for _ in 0..SCATTER_AMOUNT {
                let y = y * SCATTER_DIVISION_SIZE
                    + rand::random_range(0..SCATTER_DIVISION_SIZE);
                let x = x * SCATTER_DIVISION_SIZE
                    + rand::random_range(0..SCATTER_DIVISION_SIZE);

                if matches!(
                    (map[y][x].front, map[y][x].back),
                    (FrontTile::None, BackTile::Grass)
                ) {
                    map[y][x].front = FrontTile::Flowers;
                }
            }
        }
    }

    Ok(map)
}

#[allow(unused)]
pub(crate) fn print_map<const W: usize, const H: usize>(map: &[[Tile; W]; H]) {
    for _ in 0..map[0].len() {
        print!("=");
    }
    println!("====");
    for row in map {
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
    for _ in 0..map[0].len() {
        print!("=");
    }
    println!("====");
}
