use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rand::prelude::SliceRandom;
use rand::Rng;

use crate::common::*;

const ENERGY_CONSUMPTION_RATE: f32 = 0.5;
pub const SPEED_FACTOR: f32 = 100.0;
pub const CRABER_SIZE: f32 = 10.0;
pub const CRABER_REQUIRED_REPRODUCE_ENERGY: f32 = 100.0;
pub const CRABER_REPRODUCE_ENERGY: f32 = 50.0;
pub enum CraberTexture {
    A,
    B,
    C,
    D,
    E,
}

impl CraberTexture {
    pub fn path(&self) -> &str {
        match self {
            CraberTexture::A => "textures/crabers/Craber_a.png",
            CraberTexture::B => "textures/crabers/Craber_b.png",
            CraberTexture::C => "textures/crabers/Craber_c.png",
            CraberTexture::D => "textures/crabers/Craber_d.png",
            CraberTexture::E => "textures/crabers/Craber_e.png",
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct Craber {
    pub max_energy: f32,
    pub max_health: f32,
    pub energy: f32,
    pub health: f32,
}

#[derive(Component)]
pub struct Acceleration(pub Vec2);

#[derive(Event)]
pub struct ReproduceEvent {
    pub entity: Entity,
}

// Spawn event
#[derive(Event)]
pub struct SpawnEvent {
    pub position: Vec3,
    pub velocity: Velocity,
    pub craber: Craber,
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

pub fn spawn_craber(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: EventReader<SpawnEvent>,
) {
    let mut rng = rand::thread_rng();
    for event in spawn_events.read() {
        let craber = event.craber;
        let position = event.position;
        let velocity = event.velocity;
        let craber_texture = [
            CraberTexture::A,
            CraberTexture::B,
            CraberTexture::C,
            CraberTexture::D,
            CraberTexture::E,
        ]
        .choose(&mut rng)
        .unwrap();

        commands
            .spawn(RigidBody::Dynamic)
            .insert(Collider::ball(CRABER_SIZE / 2.0))
            .insert(Restitution::coefficient(0.8))
            .insert(Name::new("Craber"))
            .insert(TransformBundle::from(Transform::from_translation(position)))
            .insert(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(CRABER_SIZE, CRABER_SIZE)),
                    ..Default::default()
                },
                texture: asset_server.load(craber_texture.path()),
                ..Default::default()
            })
            .insert(craber)
            .insert(SelectableEntity::Craber)
            .insert(velocity)
            .insert(Weight { weight: 1.0 })
            .insert(Acceleration(Vec2::new(0.0, -1.0)))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(EntityType::Craber);
    }
}

pub fn craber_spawner(
    time: Res<Time>,
    mut timer: ResMut<CraberSpawnTimer>,
    mut spawn_events: EventWriter<SpawnEvent>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
            rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
            0.0,
        );
        // let velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * SPEED_FACTOR;
        let velocity: Velocity = Velocity::linear(
            Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * SPEED_FACTOR,
        );
        spawn_events.send(SpawnEvent {
            position,
            velocity,
            craber: Craber {
                max_energy: 100.0,
                max_health: 100.0,
                energy: 100.0,
                health: 100.0,
            },
        });
    }
}

// Make crabers lose energy over time
pub fn energy_consumption(
    mut query: Query<(Entity, &mut Craber, &mut Velocity)>,
    time: Res<Time>,
    mut reproduce_events: EventWriter<ReproduceEvent>,
) {
    for (entity, mut craber, _velocity) in query.iter_mut() {
        craber.energy -= ENERGY_CONSUMPTION_RATE * time.delta_seconds();
        if craber.energy >= CRABER_REQUIRED_REPRODUCE_ENERGY {
            reproduce_events.send(ReproduceEvent { entity });
        }
        // Handle low energy situations
        if craber.energy <= 0.0 {
            // velocity.0 = Vec2::ZERO; // Stop movement
            craber.health -= 1.0; // Reduce health if needed
        }
    }
}

// TODO: Make reproduction for plants/food? Would need a separate health/energy component
pub fn craber_reproduce(
    mut craber_query: Query<(&mut Craber, &Transform)>,
    mut reproduce_events: EventReader<ReproduceEvent>,
    mut spawn_events: EventWriter<SpawnEvent>,
) {
    let mut rng = rand::thread_rng();
    for event in reproduce_events.read() {
        let mut craber = craber_query.get_mut(event.entity).unwrap();
        let velocity: Velocity = Velocity::linear(Vec2::new(0., 0.) * SPEED_FACTOR);

        // Position offset from parent to the back, first find the angle of the parent
        let parent_angle = craber.1.rotation.to_axis_angle().1;
        let position_offset = Vec2::new(parent_angle.cos(), parent_angle.sin()) * CRABER_SIZE * 2.0;
        let position = craber.1.translation + position_offset.extend(0.0);
        println!("Parent position: {:?}", craber.1.translation);
        craber.0.energy -= CRABER_REPRODUCE_ENERGY;
        spawn_events.send(SpawnEvent {
            position,
            velocity,
            craber: Craber {
                max_energy: 100.0,
                max_health: 100.0,
                energy: CRABER_REPRODUCE_ENERGY,
                health: 50.0,
            },
        });
    }
}

// Make crabers switch directions every X seconds and change color to random
pub fn ravers(
    mut query: Query<(&mut Craber, &mut Velocity, &mut Sprite)>,
    time: Res<Time>,
    mut timer: ResMut<RaversTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (_craber, _velocity, mut sprite) in query.iter_mut() {
            // velocity.0 = velocity.0 * -1.0;

            sprite.color = Color::rgb(
                rand::thread_rng().gen_range(0.0..1.0),
                rand::thread_rng().gen_range(0.0..1.0),
                rand::thread_rng().gen_range(0.0..1.0),
            );
        }
    }
}
