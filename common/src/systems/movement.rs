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
    for (transform, move_state) in
        &mut world.query::<(&mut Transform, &MovementState)>()
    {
        if let Some(dir) = Vec2::from(&move_state.dir).try_normalize() {
            let vel = dir * MOVEMENT_SPEED;
            let mut next_pos = transform.pos + vel * dt.as_secs_f32();

            let mut player_rect = Rect::new(
                next_pos.x,
                next_pos.y,
                ENTITY_SPRITE_SIZE,
                ENTITY_SPRITE_SIZE,
            );

            let nearby_tiles = map.get_3x3_around(next_pos);
            let base_x = next_pos.x.floor() as i32 - 1;
            let base_y = next_pos.y.floor() as i32 - 1;

            for y in 0..3 {
                for x in 0..3 {
                    let tile = nearby_tiles[x + y * 3];
                    if !tile.has_collision() {
                        continue;
                    }

                    let tile_rect = Rect::new(
                        (base_x + x as i32) as f32,
                        (base_y + y as i32) as f32,
                        1.0,
                        1.0,
                    );

                    if let Some(overlap) = player_rect.intersect(tile_rect) {
                        let player_center = player_rect.center();
                        let tile_center = tile_rect.center();

                        if overlap.w < overlap.h {
                            if player_center.x < tile_center.x {
                                next_pos.x -= overlap.w;
                            } else {
                                next_pos.x += overlap.w;
                            }
                            player_rect.x = next_pos.x;
                        } else {
                            if player_center.y < tile_center.y {
                                next_pos.y -= overlap.h;
                            } else {
                                next_pos.y += overlap.h;
                            }
                            player_rect.y = next_pos.y;
                        }
                    }
                }
            }

            transform.pos = next_pos;
        }
    }
}
