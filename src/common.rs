use bevy::prelude::*;

pub const WORLD_SIZE: f32 = 5000.0;

#[derive(Component)]
pub enum SelectableEntity {
    Craber,
    Food,
}

#[derive(Component)]
pub struct DebugRectangle;

#[derive(Component)]
pub struct CollidableEntity {
    pub collision_threshold: f32,
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

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    Craber,
    Food,
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

// SAP - Sweep and Prune

#[derive(Component, Debug, Clone)]
pub struct SapEntity {
    pub entity: Entity,
    pub start: f32, // Start of the interval on the x-axis
    pub end: f32,   // End of the interval on the x-axis
}

#[derive(Resource)]
pub struct Sap {
    pub entities: Vec<SapEntity>, // Sorted by the start of the interval
}

impl Sap {
    pub fn new() -> Self {
        Sap {
            entities: Vec::new(),
        }
    }

    // Update the SAP structure
    pub fn update(&mut self, query: Query<(Entity, &Transform, &CollidableEntity)>) {
        self.entities.clear();
        for (entity, transform, collidable) in query.iter() {
            let start = transform.translation.x - collidable.collision_threshold / 2.0;
            let end = transform.translation.x + collidable.collision_threshold / 2.0;
            self.entities.push(SapEntity { entity, start, end });
        }
        self.entities
            .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    }

    // Function to perform the sweep and prune
    pub fn sweep_and_prune(&self) -> Vec<(Entity, Entity)> {
        let mut active = Vec::new();
        let mut potential_collisions = Vec::new();

        for entity in &self.entities {
            active.retain(|&e: &usize| self.entities[e].end > entity.start);
            for &active_entity in &active {
                potential_collisions.push((entity.entity, self.entities[active_entity].entity));
            }
            active.push(
                self.entities
                    .iter()
                    .position(|e| e.entity == entity.entity)
                    .unwrap(),
            );
        }

        potential_collisions
    }
}
