use crate::components::Transform;
use crate::components::MoveDirection;

use hecs;
use macroquad::math::Vec2;

pub async fn update_movement(world: &mut hecs::World) {
    for (transform, dir) in &mut world.query::<(&mut Transform, &MoveDirection)>() {
        if let Some(dir) = Vec2::from(dir).try_normalize() {
            transform.pos += dir * 5.;
        }
    }
}
