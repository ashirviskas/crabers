use avian2d::prelude::*;
use bevy::prelude::*;

use rand::RngExt;
use rand::prelude::IndexedRandom;

use crate::common::*;

use crate::brain::*;

const ENERGY_CONSUMPTION_RATE: f32 = 0.03;
const CRABER_HEALING_RATE: f32 = 0.05;
const CRABER_HEALING_COST: f32 = 1.3;
const CRABER_DEATH_ENERGY_FACTOR: f32 = 0.7;
const CRABER_MASS: f32 = 0.5;
const CRABER_INERTIA: f32 = 0.05;
pub const CRABER_SIZE: f32 = 10.0;
pub const CRABER_REQUIRED_REPRODUCE_ENERGY: f32 = 100.0;
pub const CRABER_REPRODUCE_ENERGY: f32 = 60.0;
pub const MAX_CRABERS: usize = 5000;
pub const MAX_CRABERS_SPAWNER: usize = 20;
pub const CRABER_SPAWN_MULTIPLIER: usize = 1;
pub const CRABER_MUTATION_CHANCE: f32 = 0.05;
pub const CRABER_MUTATION_AMOUNT: f32 = 0.5;

/// Decay-based input: 1.0 after reproduction, decays toward 0 over time.
#[derive(Component)]
pub struct LastReproducedValue(pub f32);

/// Tracks craber age in seconds since spawn.
#[derive(Component)]
pub struct CraberAge(pub f32);

/// Tracks how many children this craber has produced.
#[derive(Component)]
pub struct ChildrenCount(pub u32);

/// Accumulator for discrete kick impulses. Each critter has its own.
#[derive(Component)]
pub struct KickAccumulator(pub f32);

/// Accumulator for discrete angular impulses. Same pattern as KickAccumulator.
#[derive(Component)]
pub struct RotationAccumulator(pub f32);

/// Accumulator for brain tick timing. Brain fires when this reaches >= 1.0.
#[derive(Component)]
pub struct BrainTickAccumulator(pub f32);
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
pub struct Craber {}

/// Marker component to prevent double-despawn of dead crabers
#[derive(Component)]
pub struct Dying;

#[derive(Component)]
pub struct Generation {
    pub generation_id: u32,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Health {
    pub max_health: f32,
    pub health: f32,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Energy {
    pub max_energy: f32,
    pub energy: f32,
}

#[derive(Component, Debug)]
pub struct ReproduceCooldown {
    pub timer: Timer,
}

impl Default for ReproduceCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}

#[derive(Message)]
pub struct ReproduceEvent {
    pub entity: Entity,
    pub generation: Generation,
}

#[derive(Message)]
pub struct SexualReproduceRequestEvent {
    pub entity: Entity,
    pub generation: Generation,
}

#[derive(Message)]
pub struct SexualReproduceEvent {
    pub initiator: Entity,
    pub partner: Entity,
    pub generation: Generation,
}

pub enum VisionEventType {
    Craber,
    Food,
    Wall,
}

#[derive(Message)]
pub struct VisionEvent {
    pub vision_entity: Entity,
    pub entity: Entity,
    pub event_type: VisionEventType,
    pub entity_id: u8,
    pub manifolds: Vec<ContactManifold>,
}

#[derive(Message)]
pub struct CraberCollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

// Spawn event
#[derive(Message)]
pub struct SpawnEvent {
    pub position: Vec3,
    pub roation: Quat,
    pub craber: Craber,
    pub generation: u32,
    pub new_brain: Brain,
    pub health: Health,
    pub energy: Energy,
}

#[derive(Message)]
pub struct LoseEnergyEvent {
    pub entity: Entity,
    pub energy_lost: f32,
}

#[derive(Message)]
pub struct LoseHealthEvent {
    pub entity: Entity,
    pub health_lost: f32,
}

#[derive(Message)]
pub struct CraberAttackEvent {
    pub attacking_craber_entity: Entity,
    pub attacked_craber_entity: Entity,
    pub attack_damage: f32,
    pub energy_to_gain: f32,
}

pub fn despawn_dead_crabers(
    mut commands: Commands,
    query: Query<(Entity, &Health), Without<Dying>>,
    mut craber_despawn_events: MessageWriter<CraberDespawnEvent>,
) {
    for (entity, craber) in query.iter() {
        if craber.health <= 0.0 {
            commands.entity(entity).insert(Dying);
            craber_despawn_events.write(CraberDespawnEvent { entity });
        }
    }
}

pub fn craber_despawner(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Energy, &Transform)>,
    mut craber_despawn_events: MessageReader<CraberDespawnEvent>,
    mut food_spawn_events: MessageWriter<FoodSpawnEvent>,
) {
    for event in craber_despawn_events.read() {
        if let Ok((craber_entity, _craber_health, craber_energy, craber_transform)) =
            query.get(event.entity)
        {
            commands.entity(craber_entity).despawn();
            let new_food_energy = craber_energy.energy * CRABER_DEATH_ENERGY_FACTOR;
            food_spawn_events.write(FoodSpawnEvent {
                transform: craber_transform.clone(),
                food_energy: new_food_energy,
            });
        }
    }
}

pub fn spawn_craber(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: MessageReader<SpawnEvent>,
    crabers_query: Query<&Craber>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if crabers_query.iter().len() >= MAX_CRABERS {
        return;
    }
    let mut rng = rand::rng();
    for event in spawn_events.read() {
        let craber = event.craber;
        let position = event.position;
        let generation = event.generation;
        let health = event.health;
        let energy = event.energy;
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

        let new_craber = commands
            .spawn(RigidBody::Dynamic)
            .insert(Collider::circle(CRABER_SIZE / 2.0))
            .insert(ColliderDensity(2.5))
            .insert(Mass(CRABER_MASS))
            .insert(AngularInertia(CRABER_INERTIA))
            .insert(Restitution::new(0.8))
            .insert(AngularDamping(0.0))
            .insert(LinearDamping(0.0))
            .insert(KickAccumulator(0.0))
            .insert(RotationAccumulator(0.0))
            .insert(BrainTickAccumulator(0.0))
            .insert(Name::new("Craber"))
            .insert((
                Sprite {
                    image: asset_server.load(craber_texture.path()),
                    color: Color::srgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(CRABER_SIZE, CRABER_SIZE)),
                    ..default()
                },
                Transform {
                    translation: position,
                    rotation,
                    ..default()
                },
            ))
            .insert(craber)
            .insert(health)
            .insert(energy)
            .insert(SelectableEntity::Craber)
            .insert(Weight { weight: 1.0 })
            .insert(Generation {
                generation_id: generation,
            })
            .insert(CollisionLayers::new(
                [Layer::Craber],
                [Layer::Food, Layer::Craber, Layer::Wall, Layer::Vision],
            ))
            .insert(Friction::new(0.8))
            .insert(event.new_brain.clone())
            .insert(EntityType::Craber)
            .insert(ReproduceCooldown::default())
            .insert(LastReproducedValue(0.0))
            .insert(CraberAge(0.0))
            .insert(ChildrenCount(0))
            .id();
        let vision = Vision {
            radius: 100.0,
            nearest_food_direction: 0.0,
            nearest_food_distance: 0.0,
            nearest_craber_direction: 0.0,
            nearest_craber_distance: 0.0,
            nearest_craber_genetic_closeness: 0.0,
            nearest_wall_direction: 0.0,
            nearest_wall_distance: 0.0,
            see_food: false,
            see_craber: false,
            see_wall: false,
            entities_in_vision: Vec::new(),
            food_seen_timer: 0.0,
            craber_seen_timer: 0.0,
            wall_seen_timer: 0.0,
        };
        let rand_pretty_color = Color::srgba(
            rand::rng().random_range(0.0..1.0),
            rand::rng().random_range(0.0..1.0),
            rand::rng().random_range(0.0..1.0),
            0.2,
        );
        let craber_vision = commands
            .spawn((Collider::circle(vision.radius), Sensor))
            .insert(Name::new("CraberVision"))
            .insert((
                Mesh2d(meshes.add(Circle::new(vision.radius))),
                MeshMaterial2d(materials.add(rand_pretty_color)),
                Transform {
                    translation: Vec3::new(0., 0., 0.1),
                    ..default()
                },
            ))
            .insert(CollisionLayers::new(
                [Layer::Vision],
                [Layer::Food, Layer::Craber, Layer::Wall],
            ))
            .insert(vision)
            .insert(Weight { weight: 0.0 })
            .insert(EntityType::Vision)
            .id();

        commands.entity(new_craber).add_child(craber_vision);
    }
}

pub fn craber_spawner(
    time: Res<Time>,
    mut timer: ResMut<CraberSpawnTimer>,
    mut spawn_events: MessageWriter<SpawnEvent>,
    crabers_query: Query<&Craber>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for _ in 0..CRABER_SPAWN_MULTIPLIER {
            if crabers_query.iter().len() >= MAX_CRABERS_SPAWNER {
                continue;
            }
            let mut rng = rand::rng();
            let position = Vec3::new(
                rng.random_range((WORLD_SIZE * -1.)..WORLD_SIZE),
                rng.random_range((WORLD_SIZE * -1.)..WORLD_SIZE),
                0.0,
            );
            let rotation = Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::PI * 2.0));
            spawn_events.write(SpawnEvent {
                position,
                roation: rotation,
                generation: 0,
                craber: Craber {},
                health: Health {
                    max_health: 100.,
                    health: 100.,
                },
                energy: Energy {
                    max_energy: 100.,
                    energy: 100.,
                },
                new_brain: Brain::default(),
            });
        }
    }
}

// Make crabers lose energy over time
pub fn energy_consumption(
    mut query: Query<(
        Entity,
        &mut Health,
        &mut Energy,
        &mut LinearVelocity,
        &Generation,
        &Brain,
        &mut ReproduceCooldown,
    )>,
    time: Res<Time>,
    mut reproduce_events: MessageWriter<ReproduceEvent>,
    mut sexual_request_events: MessageWriter<SexualReproduceRequestEvent>,
) {
    for (entity, mut health, mut energy, _velocity, generation, brain, mut cooldown) in
        query.iter_mut()
    {
        let delta_seconds = time.delta_secs();
        energy.energy -= ENERGY_CONSUMPTION_RATE * delta_seconds;
        if health.health < 100.0 {
            health.health += CRABER_HEALING_RATE * delta_seconds;
            energy.energy -= CRABER_HEALING_COST * delta_seconds;
        }
        // Tick the reproduction cooldown
        cooldown.timer.tick(time.delta());
        if energy.energy >= CRABER_REQUIRED_REPRODUCE_ENERGY && cooldown.timer.is_finished() {
            // Neural-network gated reproduction: craber must want to reproduce
            if brain.get_want_to_reproduce() < 1.0 {
                // Not ready to reproduce yet
            } else {
                let new_generation = Generation {
                    generation_id: generation.generation_id + 1,
                };
                if brain.get_want_sexual_reproduction() >= 1.0 {
                    // Sexual reproduction: request a partner
                    sexual_request_events.write(SexualReproduceRequestEvent {
                        entity,
                        generation: new_generation,
                    });
                } else {
                    // Asexual reproduction
                    reproduce_events.write(ReproduceEvent {
                        entity,
                        generation: new_generation,
                    });
                }
                cooldown.timer.reset();
            }
        }
        // Handle low energy situations
        if energy.energy <= 0.0 {
            health.health -= 60.0 * delta_seconds;
        }
    }
}

pub fn match_sexual_partners(
    mut sexual_request_events: MessageReader<SexualReproduceRequestEvent>,
    craber_query: Query<(&Children, &Brain)>,
    vision_query: Query<&Vision>,
    brain_query: Query<&Brain>,
    mut sexual_reproduce_events: MessageWriter<SexualReproduceEvent>,
) {
    for event in sexual_request_events.read() {
        let Ok((children, _initiator_brain)) = craber_query.get(event.entity) else {
            continue;
        };
        // Find the vision child to get entities in range
        let mut found_partner = false;
        for child in children.iter() {
            let Ok(vision) = vision_query.get(child) else {
                continue;
            };
            for &visible_entity in &vision.entities_in_vision {
                if visible_entity == event.entity {
                    continue;
                }
                if let Ok(partner_brain) = brain_query.get(visible_entity) {
                    if partner_brain.get_want_sexual_reproduction() >= 1.0 {
                        sexual_reproduce_events.write(SexualReproduceEvent {
                            initiator: event.entity,
                            partner: visible_entity,
                            generation: Generation {
                                generation_id: event.generation.generation_id,
                            },
                        });
                        found_partner = true;
                        break;
                    }
                }
            }
            if found_partner {
                break;
            }
        }
    }
}

pub fn craber_sexual_reproduce(
    mut craber_query: Query<(&Transform, &Brain, &mut Energy, &mut LastReproducedValue, &mut ChildrenCount)>,
    mut sexual_reproduce_events: MessageReader<SexualReproduceEvent>,
    mut spawn_events: MessageWriter<SpawnEvent>,
) {
    for event in sexual_reproduce_events.read() {
        // Get partner brain first (immutable borrow)
        let partner_brain = if let Ok((_, brain, _, _, _)) = craber_query.get(event.partner) {
            brain.clone()
        } else {
            continue;
        };

        // Now get initiator (mutable borrow)
        let Ok((transform, brain, mut energy, mut last_reproduced, mut children_count)) = craber_query.get_mut(event.initiator) else {
            continue;
        };
        if energy.energy < CRABER_REPRODUCE_ENERGY {
            continue;
        }
        energy.energy -= CRABER_REPRODUCE_ENERGY;
        last_reproduced.0 = 1.0;
        children_count.0 += 1;

        let child_brain = brain.crossover_brain(
            &partner_brain,
            CRABER_MUTATION_CHANCE,
            CRABER_MUTATION_AMOUNT,
            CRABER_MUTATION_CHANCE,
        );

        // Spawn offspring between the two parents
        let parent_angle = transform.rotation.to_axis_angle().1;
        let position_offset = Vec2::new(parent_angle.cos(), parent_angle.sin()) * CRABER_SIZE * 5.0;
        let position = transform.translation + position_offset.extend(0.0);
        let rotation = Quat::from_rotation_z(parent_angle + std::f32::consts::PI);

        spawn_events.write(SpawnEvent {
            position,
            new_brain: child_brain,
            generation: event.generation.generation_id,
            roation: rotation,
            craber: Craber {},
            health: Health {
                max_health: 100.0,
                health: 50.0,
            },
            energy: Energy {
                max_energy: 100.0,
                energy: CRABER_REPRODUCE_ENERGY,
            },
        });
    }
}

// TODO: Make reproduction for plants/food? Would need a separate health/energy component
pub fn craber_reproduce(
    mut craber_query: Query<(&Transform, &Brain, &mut Energy, &mut LastReproducedValue, &mut ChildrenCount)>,
    mut reproduce_events: MessageReader<ReproduceEvent>,
    mut spawn_events: MessageWriter<SpawnEvent>,
) {
    for event in reproduce_events.read() {
        if let Ok((transform, brain, mut energy, mut last_reproduced, mut children_count)) = craber_query.get_mut(event.entity) {
            // Guard: ensure parent still has enough energy (may have been spent since event was sent)
            if energy.energy < CRABER_REPRODUCE_ENERGY {
                continue;
            }
            // Deduct energy directly to prevent multi-frame burst
            energy.energy -= CRABER_REPRODUCE_ENERGY;
            last_reproduced.0 = 1.0;
            children_count.0 += 1;

            // Position offset from parent to the back, first find the angle of the parent
            let parent_angle = transform.rotation.to_axis_angle().1;
            let position_offset =
                Vec2::new(parent_angle.cos(), parent_angle.sin()) * CRABER_SIZE * 5.0;
            let position = transform.translation + position_offset.extend(0.0);

            // Rotation 180 degrees from parent
            let rotation = Quat::from_rotation_z(parent_angle + std::f32::consts::PI);
            spawn_events.write(SpawnEvent {
                position,
                new_brain: brain.new_mutated_brain(
                    CRABER_MUTATION_CHANCE,
                    CRABER_MUTATION_AMOUNT,
                    CRABER_MUTATION_CHANCE,
                    CRABER_MUTATION_CHANCE,
                ),
                generation: event.generation.generation_id,
                roation: rotation,
                craber: Craber {},
                health: Health {
                    max_health: 100.0,
                    health: 50.0,
                },
                energy: Energy {
                    max_energy: 100.0,
                    energy: CRABER_REPRODUCE_ENERGY,
                },
            });
        }
    }
}
