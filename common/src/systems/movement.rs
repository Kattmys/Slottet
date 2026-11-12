use crate::components::Transform;
use crate::components::MovementState;
use crate::utils::epoch_time;
use macroquad::math::Vec2;
use std::time::Duration;

pub async fn update_movement(world: &mut hecs::World, dt: Duration) {
    for (transform, move_state) in &mut world.query::<(&mut Transform, &MovementState)>() {
        if let Some(dir) = Vec2::from(&move_state.dir).try_normalize() {
            transform.pos += dir * 1000. * dt.as_secs_f32();
        }
    }
}

// pub async fn handle_movement_update()
