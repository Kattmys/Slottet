use crate::components::Transform;
use crate::components::MoveDirection;

use hecs;
use macroquad::math::Vec2;

pub async fn update_movement(world: &mut hecs::World) {
    // TODO move
    const PLAYER_MOVEMENT_SPEED: f32 = 20.;

    for (transform, dir) in &mut world.query::<(&mut Transform, &MoveDirection)>() {
        let dir_vec = Vec2::from(dir);
        transform.pos += dir_vec * PLAYER_MOVEMENT_SPEED;
    }
}
