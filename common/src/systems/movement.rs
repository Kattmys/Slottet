use crate::components::Transform;
use crate::components::MovementState;
use macroquad::math::Vec2;
use std::time::Duration;

const MOVEMENT_SPEED: f32 = 1000.0;

pub async fn update_movement(world: &mut hecs::World, dt: Duration) {
    for (transform, move_state) in &mut world.query::<(&mut Transform, &MovementState)>() {
        if let Some(dir) = Vec2::from(&move_state.dir).try_normalize() {
            transform.pos += dir * MOVEMENT_SPEED * dt.as_secs_f32();
        }
    }
}
