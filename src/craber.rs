use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use avian2d::prelude::*;

use rand::prelude::SliceRandom;
use rand::Rng;

use crate::common::*;

use crate::brain::*;

const ENERGY_CONSUMPTION_RATE: f32 = 0.15;
const CRABER_MASS: f32 = 0.5;
const CRABER_INERTIA: f32 = 0.05;
const CRABER_ANGULAR_DAMPING: f32 = 0.9;
pub const SPEED_FACTOR: f32 = 100.0;
pub const CRABER_SIZE: f32 = 10.0;
pub const CRABER_REQUIRED_REPRODUCE_ENERGY: f32 = 100.0;
pub const CRABER_REPRODUCE_ENERGY: f32 = 40.0;
pub const MAX_CRABERS: usize = 10000;
pub const MAX_CRABERS_SPAWNER: usize = 10;
pub const CRABER_SPAWN_MULTIPLIER: usize = 1;
pub const CRABER_MUTATION_CHANCE: f32 = 0.05;
pub const CRABER_MUTATION_AMOUNT: f32 = 0.5;
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

#[derive(Component, Copy, Clone, Debug)]
pub struct Craber {
    pub max_energy: f32,
    pub max_health: f32,
    pub energy: f32,
    pub health: f32,
}

#[derive(Component)]
pub struct Generation{
    pub generation_id: u32,
}

#[derive(Component)]
pub struct Acceleration(pub Vec2);

#[derive(Event)]
pub struct ReproduceEvent {
    pub entity: Entity,
    pub generation: Generation,
    // pub brain: &Brain,
}

pub enum VisionEventType {
    Entered,
    Exited,
}

#[derive(Event)]
pub struct VisionEvent {
    pub vision_entity: Entity,
    pub entity: Entity,
    pub event_type: VisionEventType,
    pub entity_id: u8,
    pub manifolds: Vec<ContactManifold>
}

// Spawn event
#[derive(Event)]
pub struct SpawnEvent {
    pub position: Vec3,
    // pub velocity: Velocity,
    pub roation: Quat,
    pub craber: Craber,
    pub generation: u32,
    pub new_brain: Brain,
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
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn spawn_craber(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: EventReader<SpawnEvent>,
    crabers_query: Query<&Craber>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if crabers_query.iter().len() >= MAX_CRABERS {
        return;
    }
    let mut rng = rand::thread_rng();
    for event in spawn_events.read() {
        let craber = event.craber;
        let position = event.position;
        let generation = event.generation;
        // let velocity = event.velocity;
        let rotation = event.roation;
        let craber_texture = [
            CraberTexture::A,
            CraberTexture::B,
            CraberTexture::C,
            CraberTexture::D,
            CraberTexture::E,
        ]
        .choose(&mut rng)
        .unwrap();
        // println!("Position: {:?}", position);

        let new_craber = commands
            .spawn(RigidBody::Dynamic)
            .insert(Collider::from(Circle::new(CRABER_SIZE / 2.0)))
            .insert(ColliderDensity(2.5))
            .insert(Mass(CRABER_MASS))
            .insert(Inertia(CRABER_INERTIA))
            .insert(Restitution::new(0.8))
            .insert(AngularDamping(CRABER_ANGULAR_DAMPING))
            .insert(Name::new("Craber"))
            .insert(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(CRABER_SIZE, CRABER_SIZE)),
                    ..Default::default()
                },
                texture: asset_server.load(craber_texture.path()),
                transform: Transform {
                    translation: position,
                    rotation,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(craber)
            .insert(SelectableEntity::Craber)
            // .insert(velocity)
            .insert(Weight { weight: 1.0 })
            .insert(Acceleration(Vec2::new(0.0, -1.0)))
            .insert(Generation{ generation_id: generation})
            .insert(CollisionLayers::new(
                [Layer::Craber],
                [Layer::Food, Layer::Craber, Layer::Wall],
            ))
            // .insert(ActiveEvents::COLLISION_EVENTS)
            // .insert(ExternalForce::new(Vec2::Y).with_persistence(true),)
            .insert(Friction::new(0.3))
            .insert(Brain::default())
            .insert(EntityType::Craber)
            .id();
        let vision = Vision {
            radius: 100.0,
            nearest_food_direction: 0.0,
            nearest_food_distance: 0.0,
            see_food: false,
            entities_in_vision: Vec::new(),
        };
        let rand_pretty_color = Color::srgba(
            rand::thread_rng().gen_range(0.0..1.0),
            rand::thread_rng().gen_range(0.0..1.0),
            rand::thread_rng().gen_range(0.0..1.0),
            0.2
        );
        let craber_vision = commands
            .spawn((RigidBody::Dynamic, Collider::circle(vision.radius), Sensor, Inertia(0.001), Mass(0.001)))
            // .insert(Collider::circle(vision.radius))
            .insert(Name::new("CraberVision"))
            .insert(MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(vision.radius)).into(),
                material: materials.add(
                    rand_pretty_color
                ),
                transform: Transform {
                    translation: Vec3::new(0., 0., 0.1),
                    rotation: rotation,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(CollisionLayers::new([Layer::Vision], [Layer::Food]))
            // .insert(CollisionLayers::new([Layer::Food], [Layer::Vision]))
            .insert(vision)
            .insert(Weight { weight: 1.0 })
            // .insert()

            .insert(EntityType::Vision)
            .id();

        // commands.entity(new_craber).push_children(&[craber_vision]);
        commands.entity(new_craber).add_child(craber_vision);
        commands.spawn(FixedJoint::new(new_craber, craber_vision));
    }
}

pub fn craber_spawner(
    time: Res<Time>,
    mut timer: ResMut<CraberSpawnTimer>,
    mut spawn_events: EventWriter<SpawnEvent>,
    crabers_query: Query<&Craber>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for _ in 0..CRABER_SPAWN_MULTIPLIER {
            if crabers_query.iter().len() >= MAX_CRABERS_SPAWNER {
                continue;
            }
            let mut rng = rand::thread_rng();
            let position = Vec3::new(
                rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
                rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
                0.0,
            );
            let rotation = Quat::from_rotation_z(rng.gen_range(0.0..std::f32::consts::PI * 2.0));
            spawn_events.send(SpawnEvent {
                position,
                roation: rotation,
                generation: 0,
                craber: Craber {
                    max_energy: 100.0,
                    max_health: 100.0,
                    energy: 100.0,
                    health: 100.0,
                },
                new_brain: Brain::default(),
            });
        }
    }
}

// Make crabers lose energy over time
pub fn energy_consumption(
    mut query: Query<(Entity, &mut Craber, &mut LinearVelocity, &Generation, &Brain)>,
    time: Res<Time>,
    mut reproduce_events: EventWriter<ReproduceEvent>,
) {
    for (entity, mut craber, _velocity, generation, brain) in query.iter_mut() {
        craber.energy -= ENERGY_CONSUMPTION_RATE * time.delta_seconds();
        if craber.energy >= CRABER_REQUIRED_REPRODUCE_ENERGY {
            let new_generation = Generation{generation_id: generation.generation_id + 1};
            reproduce_events.send(ReproduceEvent { entity, generation: new_generation});
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
    mut craber_query: Query<(&mut Craber, &Transform, &Brain)>,
    mut reproduce_events: EventReader<ReproduceEvent>,
    mut spawn_events: EventWriter<SpawnEvent>,
) {
    let mut rng = rand::thread_rng();
    for event in reproduce_events.read() {
        let (mut craber, transform, brain) = craber_query.get_mut(event.entity).unwrap();
        // let velocity: Velocity = Velocity::linear(Vec2::new(0., 0.) * SPEED_FACTOR);

        // Position offset from parent to the back, first find the angle of the parent
        let parent_angle = transform.rotation.to_axis_angle().1;
        let position_offset = Vec2::new(parent_angle.cos(), parent_angle.sin()) * CRABER_SIZE * 2.0;
        let position = transform.translation + position_offset.extend(0.0);
        // println!("Parent position: {:?}", craber.1.translation);
        craber.energy -= CRABER_REPRODUCE_ENERGY;

        // Rotation 180 degrees from parent
        let rotation = Quat::from_rotation_z(parent_angle + std::f32::consts::PI);
        spawn_events.send(SpawnEvent {
            position,
            // velocity,
            new_brain: brain.new_mutated_brain(CRABER_MUTATION_CHANCE, CRABER_MUTATION_AMOUNT, CRABER_MUTATION_CHANCE, CRABER_MUTATION_CHANCE),
            generation: event.generation.generation_id,
            roation: rotation,
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
    mut query: Query<(&mut Craber, &mut LinearVelocity, &mut Sprite)>,
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
