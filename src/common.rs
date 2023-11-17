use bevy::prelude::*;

#[derive(Component)]
pub enum SelectableEntity {
    Craber,
    Food,
}

#[derive(Default, Resource)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
    pub health: f32,
    pub energy: f32,
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

pub fn collides(a: &Transform, b: &Transform, collision_threshold: f32) -> bool {
    // Simple AABB collision check
    // Adjust the logic based on your entity's size and collision requirements
    let distance = a.translation.truncate() - b.translation.truncate();
    distance.length() < collision_threshold
}

pub fn wrap_around(coord: f32, boundary: f32) -> f32 {
    if coord > boundary {
        -boundary
    } else if coord < -boundary {
        boundary
    } else {
        coord
    }
}
