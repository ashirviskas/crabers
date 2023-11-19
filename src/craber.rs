use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rand::prelude::SliceRandom;
use rand::Rng;

use crate::common::*;

const ENERGY_CONSUMPTION_RATE: f32 = 0.1;
pub const SPEED_FACTOR: f32 = 100.0;
pub const CRABER_SIZE: f32 = 10.0;

pub enum CraberTexture {
    A,
    B,
    C,
}

impl CraberTexture {
    pub fn path(&self) -> &str {
        match self {
            CraberTexture::A => "textures/crabers/Craber_a.png",
            CraberTexture::B => "textures/crabers/Craber_b.png",
            CraberTexture::C => "textures/crabers/Craber_c.png",
        }
    }
}

#[derive(Component)]
pub struct Craber {
    pub max_energy: f32,
    pub max_health: f32,
    pub energy: f32,
    pub health: f32,
}

#[derive(Resource)]
pub struct CraberSpawnTimer(pub Timer);

#[derive(Resource)]
pub struct RaversTimer(pub Timer);

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
    asset_server: Res<AssetServer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let position = Vec2::new(
            rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
            rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
        );
        // let velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * SPEED_FACTOR;
        let velocity: Velocity = Velocity::linear(
            Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * SPEED_FACTOR,
        );
        // Choose a random texture
        let craber_texture = [CraberTexture::A, CraberTexture::B, CraberTexture::C]
            .choose(&mut rng)
            .unwrap();
        commands
            // .spawn(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::rgb(1.0, 1.0, 1.0),
            //         custom_size: Some(Vec2::new(CRABER_SIZE, CRABER_SIZE)),
            //         ..Default::default()
            //     },
            //     texture: asset_server.load(craber_texture.path()),
            //     transform: Transform::from_translation(position.extend(0.0)),
            //     ..Default::default()
            // })
            .spawn(RigidBody::Dynamic)
            .insert(Collider::ball(CRABER_SIZE / 2.0))
            .insert(Restitution::coefficient(0.8))
            .insert(Name::new("Craber"))
            .insert(TransformBundle::from(Transform::from_translation(
                position.extend(0.0),
            )))
            .insert(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(CRABER_SIZE, CRABER_SIZE)),
                    ..Default::default()
                },
                texture: asset_server.load(craber_texture.path()),
                ..Default::default()
            })
            .insert(Craber {
                max_energy: 100.0,
                max_health: 100.0,
                energy: 100.0,
                health: 100.0,
            })
            .insert(SelectableEntity::Craber)
            .insert(velocity)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(EntityType::Craber);
    }
}

// Make crabers lose energy over time
pub fn energy_consumption(mut query: Query<(&mut Craber, &mut Velocity)>, time: Res<Time>) {
    for (mut craber, mut velocity) in query.iter_mut() {
        craber.energy -= ENERGY_CONSUMPTION_RATE * time.delta_seconds();

        // Handle low energy situations
        if craber.energy <= 0.0 {
            // velocity.0 = Vec2::ZERO; // Stop movement
            craber.health -= 1.0; // Reduce health if needed
        }
    }
}

// Make crabers switch directions every X seconds and change color to random
pub fn ravers(
    mut query: Query<(&mut Craber, &mut Velocity, &mut Sprite)>,
    time: Res<Time>,
    mut timer: ResMut<RaversTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut craber, mut velocity, mut sprite) in query.iter_mut() {
            // velocity.0 = velocity.0 * -1.0;

            sprite.color = Color::rgb(
                rand::thread_rng().gen_range(0.0..1.0),
                rand::thread_rng().gen_range(0.0..1.0),
                rand::thread_rng().gen_range(0.0..1.0),
            );
        }
    }
}
