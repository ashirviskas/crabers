use bevy::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::PI;

use avian2d::prelude::*;

// Define the collision layers
#[derive(PhysicsLayer, Default)]
pub enum Layer {
    #[default]
    Craber,
    Food,
    Vision,
    Wall,
}

// Constants for debug build
#[cfg(not(target_arch = "wasm32"))]
pub const WORLD_SIZE: f32 = 10000.0;

#[cfg(target_arch = "wasm32")]
pub const WORLD_SIZE: f32 = 10000.0;

#[derive(Message)]
pub struct DespawnEvent {
    pub entity: Entity,
}

#[derive(Message)]
pub struct FoodSpawnEvent {
    pub transform: Transform,
    pub food_energy: f32,
}

#[derive(Message)]
pub struct CraberDespawnEvent {
    pub entity: Entity,
}

#[derive(Resource)]
pub struct CraberSpawnTimer(pub Timer);

// Vision update timer
#[derive(Resource)]
pub struct FoodSpawnTimer(pub Timer);

#[derive(Component)]
pub enum SelectableEntity {
    Craber,
    Food,
}

#[derive(Component)]
pub struct Weight {
    pub weight: f32,
}

#[derive(Default, Resource)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
    pub health: f32,
    pub energy: f32,
    pub generation: u32,
    pub age: f32,
    pub children_count: u32,
    pub rotation: Quat,
    pub vision_rotation: Quat,
    pub nearest_food_anlge: f32,
    pub brain_info: String,
}

#[derive(Resource, Default)]
pub struct DebugInfo {
    pub fps: f64,
    pub entity_count: usize,
    pub craber_count: usize,
    pub food_count: usize,
}

#[derive(Resource)]
pub struct SimulationStats {
    pub craber_history: VecDeque<[f64; 2]>,
    pub sample_timer: Timer,
    capacity: usize,
}

impl SimulationStats {
    pub fn new(capacity: usize) -> Self {
        Self {
            craber_history: VecDeque::with_capacity(capacity),
            sample_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            capacity,
        }
    }

    pub fn record(&mut self, time: f64, count: f64) {
        if self.craber_history.len() >= self.capacity {
            self.craber_history.pop_front();
        }
        self.craber_history.push_back([time, count]);
    }
}

pub fn collides(a: &Transform, b: &Transform, collision_threshold: f32) -> bool {
    let distance = a.translation.truncate() - b.translation.truncate();
    distance.length() < collision_threshold
}

// Movement constants
pub const MAX_IMPULSE: f32 = 200.0;
pub const KICK_THRESHOLD: f32 = 0.01;
pub const KICK_ENERGY_MODIFIER: f32 = 2.0;

// Quadratic water drag constants
pub const LINEAR_DRAG_COEFFICIENT: f32 = 0.01;
pub const ANGULAR_DRAG_COEFFICIENT: f32 = 1.0;
pub const KICK_STEEPNESS: f32 = 0.5;
pub const KICK_RATE_STEEPNESS: f32 = 0.5;

// Rotation constants
pub const ROTATION_THRESHOLD: f32 = 0.01;
pub const ROTATION_RATE_STEEPNESS: f32 = 0.5;
pub const MAX_ANGULAR_IMPULSE: f32 = 0.1;

// Brain tick constants
pub const BRAIN_TICK_MIN_RATE: f32 = 1.0; // min ticks per second (Hz)
pub const BRAIN_TICK_MAX_RATE: f32 = 30.0; // max ticks per second (Hz)
pub const BRAIN_TICK_ENERGY_COST: f32 = 0.05; // energy per tick

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    Craber,
    Food,
    Vision,
    Wall,
}

/// To be used only for getting directions for food/other crabers
pub fn angle_direction_between_vectors(v1: Vec3, v2: Vec3) -> f32 {
    let v1_2d = Vec2::new(v1.x, v1.y);
    let v2_2d = Vec2::new(v2.x, v2.y);

    // Guard against non-finite or zero-length vectors
    if !v1_2d.x.is_finite() || !v1_2d.y.is_finite()
        || !v2_2d.x.is_finite() || !v2_2d.y.is_finite()
        || v1_2d.length_squared() == 0.0
        || v2_2d.length_squared() == 0.0
    {
        return 0.0;
    }

    // Calculate the angle between vectors using atan2
    let mut angle_radians = v2_2d.y.atan2(v2_2d.x) - v1_2d.y.atan2(v1_2d.x);

    // Adjust angle to range [0, 2PI]
    angle_radians = angle_radians.rem_euclid(2.0 * PI);

    angle_radians
}
