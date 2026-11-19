use crate::map::Map;
use crate::components::ENTITY_SPRITE_SIZE;
use crate::components::Transform;
use crate::components::{ MovementState, DirectionComponentX, DirectionComponentY };

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
            let mut vel = dir * MOVEMENT_SPEED;
            let next_pos = transform.pos + vel * dt.as_secs_f32();

            let player_rect = Rect {
                x: next_pos.x,
                y: next_pos.y,
                w: ENTITY_SPRITE_SIZE,
                h: ENTITY_SPRITE_SIZE,
            };

            let rounded_pos = next_pos.floor();
            let (y_dir, x_dir) = move_state.dir.components();

            fn check_collision
                (map: &Map, player_rect: &Rect, tile_x: usize, tile_y: usize) -> bool
            {
                if let Some(tile) = map.get(tile_x, tile_y) {
                    if !tile.has_collision() {
                        return false
                    }
                } else {
                    return false
                };

                let tile_rect = Rect {
                    x: tile_x as f32,
                    y: tile_y as f32,
                    w: 1.,
                    h: 1.,
                };

                player_rect.overlaps(&tile_rect)
            }

            'x_collision: {
                let x = rounded_pos.x as isize + match x_dir {
                    DirectionComponentX::East => 1,
                    DirectionComponentX::West => -1,
                    DirectionComponentX::None => {
                        break 'x_collision
                    }
                };

                if check_collision(
                    &map, &player_rect, x as usize, rounded_pos.y as usize
                ) {
                    vel.x = 0.0;
                }
            }

            'y_collision: {
                let y = rounded_pos.y as isize + match y_dir {
                    DirectionComponentY::South => 1,
                    DirectionComponentY::North => -1,
                    DirectionComponentY::None => {
                        break 'y_collision
                    }
                };

                if check_collision(
                    &map, &player_rect, rounded_pos.x as usize, y as usize
                ) {
                    vel.y = 0.0;
                }
            }

            transform.pos += vel * dt.as_secs_f32();
        }
    }
}
