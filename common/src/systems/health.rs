use crate::components::Health;

pub fn update_health(
    world: &mut hecs::World,
) {
    for health in &mut world.query::<&mut Health>() {
        health.update();
    }
}
