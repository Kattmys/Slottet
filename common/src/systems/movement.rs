use crate::map::{ Map, Tile, FrontTile, BackTile, TILE_SIZE };
use crate::components::ENTITY_SPRITE_SIZE;
use crate::components::Transform;
use crate::components::MovementState;

use macroquad::math::{ Vec2, Rect };
use std::time::Duration;
use std::sync::{ Arc, RwLock };

const MOVEMENT_SPEED: f32 = 1000.0;


pub fn update_movement(
    world: Arc<RwLock<hecs::World>>,
    map: &Map,
    dt: Duration,
) {
    let world = world.write().unwrap();
    for (transform,move_state) in &mut world.query::<(&mut Transform, &MovementState)>(){
        if let Some(dir) = Vec2::from(&move_state.dir).try_normalize() {
            let mut next_pos = transform.pos + dir * MOVEMENT_SPEED * dt.as_secs_f32();

            let nearby_tiles = map.get_3x3_around(next_pos);
            let player_rect = Rect {
                x: next_pos.x,
                y: next_pos.y,
                w: ENTITY_SPRITE_SIZE,
                h: ENTITY_SPRITE_SIZE,
            };


            for y in 0..3 {
                for x in 0..3 {
                    let tile = nearby_tiles[x + y * 3];
                    if !tile.has_collision() { continue } 
                    let tile_rect = Rect {
                        x: next_pos.x +x as f32,
                        y: next_pos.y +y as f32,
                        w: TILE_SIZE,
                        h: TILE_SIZE,
                    };

                    let Some(intersect) = player_rect.intersect(tile_rect) else { 
                        continue
                    };

                    if intersect.x > intersect.y {
                        next_pos.x -= intersect.w;
                    } else {
                        next_pos.y -= intersect.w;
                    }
                    
                }
            }

            transform.pos = next_pos;
        }
    }
}
