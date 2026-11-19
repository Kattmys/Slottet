use crate::map::Map;
use crate::components::ENTITY_SPRITE_SIZE;
use crate::components::Transform;
use crate::components::MovementState;

use macroquad::math::{ Vec2, Rect };
use std::time::Duration;

const MOVEMENT_SPEED: f32 = 10.0;

pub fn update_movement(
    world: &mut hecs::World,
    map: &Map,
    dt: Duration,
) {
    for (transform,move_state) in &mut world.query::<(&mut Transform, &MovementState)>(){
        if let Some(dir) = Vec2::from(&move_state.dir).try_normalize() {
            let next_pos = transform.pos + dir * MOVEMENT_SPEED * dt.as_secs_f32();
            let mut vel = dir * MOVEMENT_SPEED;

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
                        x: next_pos.x.round() + x as f32 - 1.0,
                        y: next_pos.y.round() + y as f32 - 1.0,
                        w: 1.,
                        h: 1.,
                    };

                    let Some(intersect) = player_rect.intersect(tile_rect) else { 
                        continue
                    };

                    if intersect.w > intersect.h {
                        vel.y = 0.0;
                    } else {
                        vel.x = 0.0;
                    }
                    
                }
            }

            transform.pos += vel * dt.as_secs_f32();
        }
    }
}
