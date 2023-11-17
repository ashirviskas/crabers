use bevy::prelude::*;
use rand::Rng;

use crate::common::*;

const ENERGY_CONSUMPTION_RATE: f32 = 1.0;

#[derive(Component)]
pub struct Craber {
    pub max_energy: f32,
    pub max_health: f32,
    pub energy: f32,
    pub health: f32,
}

#[derive(Resource)]
pub struct CraberSpawnTimer(pub Timer);

pub fn energy_to_color(energy: f32, max_energy: f32) -> Color {
    let energy_ratio = energy / max_energy;
    Color::rgb(1.0 - energy_ratio, energy_ratio, 0.0) // Red to green transition
}

pub fn update_craber_color(mut query: Query<(&Craber, &mut Sprite)>) {
    for (craber, mut sprite) in query.iter_mut() {
        sprite.color = energy_to_color(craber.energy, craber.max_energy);
    }
}

pub fn despawn_dead_crabers(mut commands: Commands, query: Query<(Entity, &Craber)>) {
    for (entity, craber) in query.iter() {
        if craber.health <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn craber_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<CraberSpawnTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let position = Vec2::new(rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE), rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE));
        let velocity = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED, // Different color to distinguish from food
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..Default::default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
                ..Default::default()
            })
            .insert(Craber {
                max_energy: 100.0,
                max_health: 100.0,
                energy: 100.0,
                health: 100.0,
            })
            .insert(SelectableEntity::Craber)
            .insert(Velocity(velocity))
            .insert(EntityType::Craber);
    }
}

pub fn craber_movement(
    mut query: Query<(&mut Transform, &Velocity), With<Craber>>,
    time: Res<Time>,
) {
    let boundary = WORLD_SIZE; // Define the boundary of your 2D space
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_seconds()).extend(0.0);

        // Wrap around logic
        let translation = &mut transform.translation;
        translation.x = wrap_around(translation.x, boundary);
        translation.y = wrap_around(translation.y, boundary);
    }
}

pub fn energy_consumption(mut query: Query<(&mut Craber, &mut Velocity)>, time: Res<Time>) {
    for (mut craber, mut velocity) in query.iter_mut() {
        craber.energy -= ENERGY_CONSUMPTION_RATE * time.delta_seconds();

        // Handle low energy situations
        if craber.energy <= 0.0 {
            velocity.0 = Vec2::ZERO; // Stop movement
            craber.health -= 1.0; // Reduce health if needed
        }
    }
}
