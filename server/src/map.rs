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

    for y in 0..H {
        for x in 0..W {
            if matches!(
                (map[y][x].front, map[y][x].back),
                (FrontTile::None, BackTile::Grass)
            ) {
                if rand::random_bool(1.0 / 30.0) {
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
