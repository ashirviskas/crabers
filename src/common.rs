use bevy::prelude::*;
use std::f32::consts::PI;

use avian2d::prelude::*;

// Define the collision layers
#[derive(PhysicsLayer)]
pub enum Layer {
    Craber,
    Food,
    Vision,
    Blue,
    Wall,
}

// Constants for debug build
#[cfg(not(target_arch = "wasm32"))]
pub const WORLD_SIZE: f32 = 10000.0;

#[cfg(target_arch = "wasm32")]
pub const WORLD_SIZE: f32 = 10000.0;

#[derive(Event)]
pub struct DespawnEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct FoodSpawnEvent {
    pub transform: Transform,
    pub food_energy: f32,
}

#[derive(Event)]
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

#[derive(Component)]
pub struct DebugRectangle;

#[derive(Default, Resource)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
    pub health: f32,
    pub energy: f32,
    pub generation: u32,
    pub rotation: Quat,
    pub vision_rotation: Quat,
    pub nearest_food_anlge: f32,
    pub brain_info: String,
}

#[derive(Resource)]
pub struct DebugInfo {
    pub fps: f64,
    pub entity_count: usize,
    pub timer: Timer,
}

impl Default for DebugInfo {
    fn default() -> Self {
        DebugInfo {
            fps: 0.0,
            entity_count: 0,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

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

#[derive(Resource)]
pub struct InformationTimer(pub Timer);

pub fn print_current_entity_count(
    time: Res<Time>,
    query: Query<&Transform>,
    mut timer: ResMut<InformationTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        println!("Current entity count: {}", query.iter().count());
    }
}

#[derive(Resource)]
pub struct ForceApplicationTimer(pub Timer);

#[derive(Resource)]
pub struct SyncVisionPositionTimer(pub Timer);


#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    Craber,
    Food,
    Vision,
}

#[derive(Component, Debug)]
pub struct QuadtreeEntity {
    pub position: Vec2,
    pub entity: Entity,
    pub entity_type: EntityType,
}

impl QuadtreeEntity {
    pub fn new(position: Vec2, entity: Entity, entity_type: EntityType) -> Self {
        QuadtreeEntity {
            position,
            entity,
            entity_type,
        }
    }
}

pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn contains(&self, quadtreeentity: &QuadtreeEntity) -> bool {
        let point = quadtreeentity.position;
        point.x >= self.x - self.width / 2.0
            && point.x <= self.x + self.width / 2.0
            && point.y >= self.y - self.height / 2.0
            && point.y <= self.y + self.height / 2.0
    }

    pub fn intersects(&self, range: &Rectangle) -> bool {
        !(range.x - range.width / 2.0 > self.x + self.width / 2.0
            || range.x + range.width / 2.0 < self.x - self.width / 2.0
            || range.y - range.height / 2.0 > self.y + self.height / 2.0
            || range.y + range.height / 2.0 < self.y - self.height / 2.0)
    }
}

#[derive(Resource)]
pub struct Quadtree {
    pub boundary: Rectangle,
    pub capacity: usize,
    pub points: Vec<QuadtreeEntity>,
    pub divided: bool,
    pub northeast: Option<Box<Quadtree>>,
    pub northwest: Option<Box<Quadtree>>,
    pub southeast: Option<Box<Quadtree>>,
    pub southwest: Option<Box<Quadtree>>,
}

impl Quadtree {
    pub fn new(boundary: Rectangle, capacity: usize) -> Self {
        Quadtree {
            boundary,
            capacity,
            points: Vec::new(),
            divided: false,
            northeast: None,
            northwest: None,
            southeast: None,
            southwest: None,
        }
    }
    pub fn try_insert(&mut self, point: &QuadtreeEntity) -> bool {
        if !self.boundary.contains(point) {
            return false;
        }

        if self.points.len() < self.capacity {
            return true;
        }

        if !self.divided {
            self.subdivide();
        }

        if !self.northeast.as_mut().unwrap().try_insert(point) {
            if !self.northwest.as_mut().unwrap().try_insert(point) {
                if !self.southeast.as_mut().unwrap().try_insert(point) {
                    if !self.southwest.as_mut().unwrap().try_insert(point) {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn insert(&mut self, point: QuadtreeEntity) -> bool {
        if !self.boundary.contains(&point) {
            return false;
        }

        if self.points.len() < self.capacity {
            self.points.push(point);
            return true;
        }

        if !self.divided {
            self.subdivide();
        }

        if self.northeast.as_mut().unwrap().try_insert(&point) {
            self.northeast.as_mut().unwrap().insert(point);
            return true;
        }
        if self.northwest.as_mut().unwrap().try_insert(&point) {
            self.northwest.as_mut().unwrap().insert(point);
            return true;
        }
        if self.southeast.as_mut().unwrap().try_insert(&point) {
            self.southeast.as_mut().unwrap().insert(point);
            return true;
        }
        if self.southwest.as_mut().unwrap().try_insert(&point) {
            self.southwest.as_mut().unwrap().insert(point);
            return true;
        }

        false
    }

    fn subdivide(&mut self) {
        let x = self.boundary.x;
        let y = self.boundary.y;
        let w = self.boundary.width / 2.0;
        let h = self.boundary.height / 2.0;

        let ne = Rectangle {
            x: x + w / 2.0,
            y: y - h / 2.0,
            width: w,
            height: h,
        };
        self.northeast = Some(Box::new(Quadtree::new(ne, self.capacity)));

        let nw = Rectangle {
            x: x - w / 2.0,
            y: y - h / 2.0,
            width: w,
            height: h,
        };
        self.northwest = Some(Box::new(Quadtree::new(nw, self.capacity)));

        let se = Rectangle {
            x: x + w / 2.0,
            y: y + h / 2.0,
            width: w,
            height: h,
        };
        self.southeast = Some(Box::new(Quadtree::new(se, self.capacity)));

        let sw = Rectangle {
            x: x - w / 2.0,
            y: y + h / 2.0,
            width: w,
            height: h,
        };
        self.southwest = Some(Box::new(Quadtree::new(sw, self.capacity)));

        self.divided = true;
    }

    pub fn query(&self, range: &Rectangle) -> Vec<&QuadtreeEntity> {
        let mut found_new: Vec<&QuadtreeEntity> = Vec::new();
        if !self.boundary.intersects(range) {
            return found_new;
        }

        for point in &self.points {
            if range.contains(point) {
                found_new.push(point);
            }
        }

        if self.divided {
            found_new.append(&mut self.northeast.as_ref().unwrap().query(range));
            found_new.append(&mut self.northwest.as_ref().unwrap().query(range));
            found_new.append(&mut self.southeast.as_ref().unwrap().query(range));
            found_new.append(&mut self.southwest.as_ref().unwrap().query(range));
        }
        for found in &found_new {
            if found.entity_type != EntityType::Craber {
                // println!("found: {:?}", found);
            }
        }
        found_new
    }
    pub fn clear(&mut self) {
        self.points.clear();
        self.divided = false;
        self.northeast = None;
        self.northwest = None;
        self.southeast = None;
        self.southwest = None;
    }
    pub fn draw(&self, commands: &mut Commands) {
        let x = self.boundary.x;
        let y = self.boundary.y;
        let w = self.boundary.width;
        let h = self.boundary.height;

        // commands.spawn(SpriteBundle {
        //     sprite: Sprite {
        //         color: Color::WHITE,
        //         custom_size: Some(Vec2::new(w, h)),

        //         ..Default::default()
        //     },
        //     transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
        //     ..Default::default()
        // })
        // .insert(DebugRectangle);
        // Draw each wall separately so we can see the lines
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(w, 5.0)),

                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x, y + h / 2.0, 0.0)),
                ..Default::default()
            })
            .insert(DebugRectangle);

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(w, 5.0)),

                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x, y - h / 2.0, 0.0)),
                ..Default::default()
            })
            .insert(DebugRectangle);

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(5.0, h)),

                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x - w / 2.0, y, 0.0)),
                ..Default::default()
            })
            .insert(DebugRectangle);

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(5.0, h)),

                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x + w / 2.0, y, 0.0)),
                ..Default::default()
            })
            .insert(DebugRectangle);

        if self.divided {
            self.northeast.as_ref().unwrap().draw(commands);
            self.northwest.as_ref().unwrap().draw(commands);
            self.southeast.as_ref().unwrap().draw(commands);
            self.southwest.as_ref().unwrap().draw(commands);
        }
    }
}

/// To be used only for getting directions for food/other crabers
pub fn angle_direction_between_vectors(v1: Vec3, v2: Vec3) -> f32 {

    let v1_2d = Vec2::new(v1.x, v1.y);
    let v2_2d = Vec2::new(v2.x, v2.y);

    // Calculate the angle between vectors using atan2
    let mut angle_radians = v2_2d.y.atan2(v2_2d.x) - v1_2d.y.atan2(v1_2d.x);

    // let's turn our vision by 90 degrees
    // angle_radians += PI * 0.5;
    // Adjust angle to range [0, 2PI]
    angle_radians = angle_radians.rem_euclid(2.0 * PI);

    // Normalize the angle to [-1, 1]
    let normalized_value = if angle_radians <= PI {
        // [0, PI] maps to [0, +1]
        angle_radians / PI
    } else {
        // [PI, 2PI] maps to [-1, 0]
        (1. +((((angle_radians - PI) / PI) * -1.))) * -1.
    };
    // println!("V1: {} V2: {} normalized_value: {}, angle_radians: {}", v1, v2, normalized_value, angle_radians);
    normalized_value
}
