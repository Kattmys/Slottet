use crate::components::Transform;
use crate::components::MoveDirection;
use macroquad::math::Vec2;
use std::time::Duration;

pub async fn update_movement(world: &mut hecs::World, dt: Duration) {
    for (transform, dir) in &mut world.query::<(&mut Transform, &MoveDirection)>() {
        if let Some(dir) = Vec2::from(dir).try_normalize() {
            transform.pos += dir * 1000. * dt.as_secs_f32();
        }
    }
}
