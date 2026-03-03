use avian2d::{math::*, parry::shape::Cuboid, prelude::*};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::{Timer, TimerMode},
};
// use avian2d::collision::collider::parry::*;

mod craber;
use craber::*;

mod brain;
use brain::*;

mod food;
use food::*;

mod neural_viz;
use neural_viz::*;

mod common;
use bevy_pancam::{PanCam, PanCamPlugin};
use common::{Rectangle, *};

const SOME_COLLISION_THRESHOLD: f32 = 20.0;
const FOOD_SPAWN_RATE: f32 = 0.0004;
const CRABER_SPAWN_RATE: f32 = 0.1;

pub const MAX_FOOD_COUNT: usize = 10000;

const WALL_THICKNESS: f32 = 60.0;
// const QUAD_TREE_CAPACITY: usize = 16;
const RAVERS_TIMER: f32 = 0.2;
const GRAVITY: f32 = 0.0;

const VISION_UPDATE_RATE: f32 = 0.1;

#[cfg(target_arch = "wasm32")]
const ENABLE_LEFT_MOUSE_BUTTON_DRAG: bool = true;

#[cfg(not(target_arch = "wasm32"))]
const ENABLE_LEFT_MOUSE_BUTTON_DRAG: bool = false;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanCamPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(WindowPlugin{
        //     primary_window: Some(Window {
        //         resolution: (1920., 1080.).into(),
        //         title: "Crabers".to_string(),
        //         ..default()
        //     }),
        //     ..default()
        // })
        // .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(InspectableRapierPlugin)
        // .add_plugins(WorldInspectorPlugin::default())
        .insert_resource(SelectedEntity::default())
        .insert_resource(DebugInfo::default())
        .insert_resource(NeuralNetworkLayout::default())
        .add_event::<DespawnEvent>()
        .add_event::<SpawnEvent>()
        .add_event::<ReproduceEvent>()
        .add_event::<VisionEvent>()
        .add_event::<LoseEnergyEvent>()
        .add_event::<LoseHealthEvent>()
        .add_event::<CraberCollisionEvent>()
        .add_event::<CraberAttackEvent>()
        .add_event::<CraberDespawnEvent>()
        .add_event::<FoodSpawnEvent>()
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(Startup, setup_neural_panel)
        .add_systems(Update, entity_selection)
        .add_systems(Update, update_selected_entity_info)
        .add_systems(Update, update_ui)
        .add_systems(Update, update_nn_layout)
        .add_systems(Update, spawn_neuron_nodes)
        .add_systems(Update, update_neuron_display)
        .add_systems(Update, update_debug_info)
        .add_systems(Update, food_spawner)
        .add_systems(Update, craber_spawner)
        .add_systems(Update, do_collision)
        .add_systems(Update, do_craber_collision)
        // .add_systems(Update, update_craber_color)
        // .add_systems(Update, print_current_entity_count)
        // .add_systems(Update, do_decollisions)
        .add_systems(Update, vision_update)
        .add_systems(Update, apply_rotation)
        .add_systems(Update, apply_alignment)
        .add_systems(Update, apply_kick)
        .add_systems(Update, brain_update)
        .add_systems(Update, craber_lose_energy)
        .add_systems(Update, craber_lose_health)
        .add_systems(Update, craber_attack_lose_health_add_energy)
        .add_systems(Update, do_despawning)
        // Ordered chains: reproduction pipeline and death pipeline
        .add_systems(Update, energy_consumption.before(craber_reproduce))
        .add_systems(Update, craber_reproduce.before(spawn_craber))
        .add_systems(Update, spawn_craber)
        .add_systems(Update, despawn_dead_crabers.before(craber_despawner))
        .add_systems(Update, craber_despawner)
        .add_systems(Update, spawn_food)
        // .add_systems(Update, vision_inherit_craber_transform)
        // Fun and debug stuff
        // .add_systems(Update, ravers)
        .run();
}

fn setup(mut commands: Commands) {
    let _boundary = Rectangle {
        x: 0.0,
        y: 0.0,
        width: WORLD_SIZE * 2.,
        height: WORLD_SIZE * 2.,
    };
    // setup
    let grab_buttons = if ENABLE_LEFT_MOUSE_BUTTON_DRAG {
        vec![MouseButton::Left, MouseButton::Right]
    } else {
        vec![MouseButton::Right]
    };
    commands.spawn(Camera2dBundle::default()).insert(PanCam {
        grab_buttons: grab_buttons, // which buttons should drag the camera
        enabled: true,              // when false, controls are disabled. See toggle example.
        zoom_to_cursor: true,       // whether to zoom towards the mouse or the center of the screen
        min_scale: 0.1,             // prevent the camera from zooming too far in
        max_scale: 40.,             // prevent the camera from zooming too far out
        ..Default::default()
    });
    commands.insert_resource(FoodSpawnTimer(Timer::from_seconds(
        FOOD_SPAWN_RATE,
        TimerMode::Repeating,
    )));
    commands.insert_resource(CraberSpawnTimer(Timer::from_seconds(
        CRABER_SPAWN_RATE,
        TimerMode::Repeating,
    )));
    commands.insert_resource(InformationTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
    commands.insert_resource(RaversTimer(Timer::from_seconds(
        RAVERS_TIMER,
        TimerMode::Repeating,
    )));
    commands.insert_resource(VisionUpdateTimer(Timer::from_seconds(
        VISION_UPDATE_RATE,
        TimerMode::Repeating,
    )));
    commands.insert_resource(SyncVisionPositionTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));
    commands.insert_resource(InformationTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));

    commands.insert_resource(Gravity(Vec2::NEG_Y * GRAVITY));

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WORLD_SIZE * 2.0, WALL_THICKNESS)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, WORLD_SIZE, 0.0)),
            ..Default::default()
        })
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WORLD_SIZE * 2.0, WALL_THICKNESS / 2.0));
    // .insert(Collider::from(Cuboid::new(WORLD_SIZE * 2.0, WALL_THICKNESS / 2.0)));

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WORLD_SIZE * 2.0, WALL_THICKNESS)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -WORLD_SIZE, 0.0)),
            ..Default::default()
        })
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WORLD_SIZE * 2.0, WALL_THICKNESS / 2.0));
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WALL_THICKNESS, WORLD_SIZE * 2.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(WORLD_SIZE, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WALL_THICKNESS / 2.0, WORLD_SIZE * 2.0));
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WALL_THICKNESS, WORLD_SIZE * 2.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(-WORLD_SIZE, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WALL_THICKNESS / 2.0, WORLD_SIZE * 2.0));
}

/// Marker component for the main status/debug text UI
#[derive(Component)]
struct StatusText;

fn setup_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands
        .spawn(TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "No craber selected".to_string(),
                    style: TextStyle {
                        font: Default::default(),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: "\nEntities: 0\nFps: 0.0".to_string(),
                    style: TextStyle {
                        font: Default::default(),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                },
            ]),
            // ... other properties ...
            ..Default::default()
        })
        .insert(StatusText);
}

fn update_ui(selected: Res<SelectedEntity>, mut query: Query<&mut Text, With<StatusText>>) {
    for mut text in query.iter_mut() {
        if let Some(_) = selected.entity {
            text.sections[0].value = format!(
                "Health: {:.2}, Energy: {:.2}\nGeneration: {}\nNearest food angle: {}",
                selected.health, selected.energy, selected.generation, selected.nearest_food_anlge
            );
        } else {
            text.sections[0].value = "No craber selected".to_string();
        }
    }
}

fn update_debug_info(
    mut debug_info: ResMut<DebugInfo>,
    craber_query: Query<&Craber>,
    food_query: Query<&Food>,
    diagnostics: Res<DiagnosticsStore>,
    _time: Res<Time>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    debug_info.entity_count = craber_query.iter().count() + food_query.iter().count();
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|x| x.average());

    if let Some(fps) = fps {
        debug_info.fps = fps;
    } else {
        debug_info.fps = 0.0;
        return;
    }

    for mut text in query.iter_mut() {
        text.sections[1].value = format!(
            // "\nEntity count: {}, \nFPS: {:.2}",
            "\n Craber count: {}, \n Food count: {}, Total: {}, \n FPS: {:.2}",
            craber_query.iter().count(),
            food_query.iter().count(),
            debug_info.entity_count,
            debug_info.fps
        );
    }
}

fn entity_selection(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    query: Query<(Entity, &Transform, &SelectableEntity), With<SelectableEntity>>,
    mut selected: ResMut<SelectedEntity>,
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection)>,
) {
    let window = windows.single();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(position) = window.cursor_position() {
            let camera = camera_query.iter().next().unwrap().1;
            let camera_projection = camera_query.iter().next().unwrap().2;
            let new_position = window_to_world(position, &window, camera, camera_projection);
            for (entity, transform, selectable_type) in query.iter() {
                // println!("Entity {:?} at {:?}", entity, transform);
                if collides(
                    transform,
                    &Transform::from_translation(new_position),
                    SOME_COLLISION_THRESHOLD,
                ) {
                    selected.entity = Some(entity);
                    match selectable_type {
                        SelectableEntity::Craber => {
                            selected.health = 0.0;
                            selected.energy = 0.0;
                        }
                        SelectableEntity::Food => {
                            selected.health = 0.0;
                            selected.energy = 0.0;
                        }
                    }
                }
            }
        }
    }
}

fn window_to_world(
    position: Vec2,
    window: &Window,
    camera: &Transform,
    camera_projection: &OrthographicProjection,
) -> Vec3 {
    // Center in screen space
    let centered_click_location = Vec2::new(
        position.x - window.width() / 2.,
        (position.y - window.height() / 2.) * -1.,
    );
    let scaled_click_location = centered_click_location * camera_projection.scale;
    let moved_click_location = scaled_click_location + camera.translation.truncate();
    let world_position = moved_click_location.extend(0.);

    world_position
}

fn update_selected_entity_info(
    mut selected: ResMut<SelectedEntity>,
    craber_query: Query<(&Transform, &Children, &Generation, &Brain, &Health, &Energy)>,
    vision_query: Query<(&Vision, &Transform, Entity, &Parent)>,
    food_query: Query<&Food>,
) {
    if let Some(entity) = selected.entity {
        // Check if the selected entity is a Craber
        if let Ok((craber_transform, craber_children, craber_generation, brain, health, energy)) =
            craber_query.get(entity)
        {
            selected.health = health.health;
            selected.energy = energy.energy;
            selected.generation = craber_generation.generation_id;
            selected.rotation = craber_transform.rotation;
            selected.brain_info = brain.get_brain_info();
            for child in craber_children.iter() {
                if let Ok((vision, vision_transform, _, entity2_type)) = vision_query.get(*child) {
                    selected.vision_rotation = vision_transform.rotation;
                    selected.nearest_food_anlge = vision.nearest_food_direction;
                }
            }
            // selected.generation = craber
        }
        // Check if the selected entity is Food
        else if let Ok(food) = food_query.get(entity) {
            selected.health = 0.0; // Food doesn't have health, so set to 0
            selected.energy = food.energy_value; // Set energy to the value of the food
        }
    }
}

fn do_collision(
    _commands: Commands,
    mut collide_event_reader: EventReader<Collision>,
    query: Query<(Entity, &Transform, &EntityType)>,
    mut craber_query: Query<(Entity, &mut Energy)>,
    food_query: Query<(Entity, &mut Food, &Transform)>,
    mut despawn_events: EventWriter<DespawnEvent>,
    mut vision_events: EventWriter<VisionEvent>,
    mut craber_collision_events: EventWriter<CraberCollisionEvent>,
) {
    for Collision(contact) in collide_event_reader.read() {
        let entity1 = contact.entity1;
        let entity2 = contact.entity2;
        let manifolds = &contact.manifolds;
        if let Ok((entity1, _, entity1_type)) = query.get(entity1) {
            if let Ok((entity2, _, entity2_type)) = query.get(entity2) {
                match (entity1_type, entity2_type) {
                    (EntityType::Craber, EntityType::Craber) => {
                        craber_collision_events.send(CraberCollisionEvent {
                            entity_a: entity1,
                            entity_b: entity2,
                        });
                    }
                    (EntityType::Craber, EntityType::Food) => {
                        if let Ok(mut craber) = craber_query.get_mut(entity1) {
                            if let Ok(food) = food_query.get(entity2) {
                                craber.1.energy += food.1.energy_value;
                                // commands.entity(*entity2).despawn();
                                despawn_events.send(DespawnEvent { entity: entity2 });
                            }
                        }
                    }
                    (EntityType::Food, EntityType::Craber) => {
                        if let Ok(mut craber) = craber_query.get_mut(entity2) {
                            if let Ok(food) = food_query.get(entity1) {
                                craber.1.energy += food.1.energy_value;
                                // commands.entity(*entity1).despawn();
                                despawn_events.send(DespawnEvent { entity: entity1 });
                            }
                        }
                    }
                    (EntityType::Food, EntityType::Vision) => {
                        // Do entered vision instead and start tracking
                        vision_events.send(VisionEvent {
                            vision_entity: entity2,
                            entity: entity1,
                            event_type: VisionEventType::Food,
                            entity_id: 2,
                            manifolds: manifolds.clone(),
                        });
                        // println!("A Food with entity {:?} entered vision", entity1);
                        // println!("Food entered vision A");
                    }
                    (EntityType::Vision, EntityType::Food) => {
                        vision_events.send(VisionEvent {
                            vision_entity: entity1,
                            entity: entity2,
                            event_type: VisionEventType::Food,
                            entity_id: 1,
                            manifolds: manifolds.clone(),
                        });
                        // println!("B Food with entity {:?} entered vision", entity2);
                        // println!("Food entered vision B");
                    }
                    (EntityType::Craber, EntityType::Vision) => {
                        // Do entered vision instead and start tracking
                        vision_events.send(VisionEvent {
                            vision_entity: entity2,
                            entity: entity1,
                            event_type: VisionEventType::Craber,
                            entity_id: 2,
                            manifolds: manifolds.clone(),
                        });
                        // println!("A CRABER WITH entity {:?} entered vision", entity1);
                        // println!("Food entered vision A");
                    }
                    (EntityType::Vision, EntityType::Craber) => {
                        vision_events.send(VisionEvent {
                            vision_entity: entity1,
                            entity: entity2,
                            event_type: VisionEventType::Craber,
                            entity_id: 1,
                            manifolds: manifolds.clone(),
                        });
                        // println!("B Food with entity {:?} entered vision", entity2);
                        // println!("Food entered vision B");
                    }
                    _ => {
                        // println!("Entities did not match the test: {} VS {}", entity1, entity2)
                    }
                }
            }
        }
    }
}

pub fn do_craber_collision(
    mut craber_collision_events: EventReader<CraberCollisionEvent>,
    query: Query<(Entity, &Brain, &LinearVelocity, &AngularVelocity)>,
    // mut lose_energy_events: EventWriter<LoseEnergyEvent>,
    // mut lose_health_events: EventWriter<LoseHealthEvent>
    mut craber_attack_events: EventWriter<CraberAttackEvent>,
) {
    for craber_collision_event in craber_collision_events.read() {
        if let Ok((entity_a, brain_a, velocity_a, angular_a)) =
            query.get(craber_collision_event.entity_a)
        {
            if let Ok((entity_b, brain_b, velocity_b, angular_b)) =
                query.get(craber_collision_event.entity_b)
            {
                if brain_a.get_want_to_attack() > 0. || brain_b.get_want_to_attack() > 0. {
                    let a_damaged =
                        brain_b.get_want_to_attack() * 5. * velocity_b.length() - angular_b.0.abs(); // Velocity increases attack, spinning decreases
                    let b_damaged =
                        brain_a.get_want_to_attack() * 5. * velocity_a.length() - angular_a.0.abs();
                    if a_damaged > 0. {
                        craber_attack_events.send(CraberAttackEvent {
                            attacking_craber_entity: entity_b,
                            attacked_craber_entity: entity_a,
                            attack_damage: a_damaged,
                            energy_to_gain: a_damaged * 0.3,
                        });
                    }
                    if b_damaged > 0. {
                        craber_attack_events.send(CraberAttackEvent {
                            attacking_craber_entity: entity_a,
                            attacked_craber_entity: entity_b,
                            attack_damage: b_damaged,
                            energy_to_gain: b_damaged * 0.3,
                        });
                    }
                    // lose_health_events.send(LoseHealthEvent{entity: entity_a, health_lost: energy_lost_a});
                    // lose_health_events.send(LoseHealthEvent{entity: entity_b, health_lost: energy_lost_b});
                    // // // hack to actually get energy
                    // lose_energy_events.send(LoseEnergyEvent{entity: entity_a, energy_lost: -energy_lost_b});
                    // lose_energy_events.send(LoseEnergyEvent{entity: entity_b, energy_lost: -energy_lost_a});
                }
            }
        }
    }
}

fn do_despawning(
    mut commands: Commands,
    mut despawn_events: EventReader<DespawnEvent>,
    mut craber_query: Query<Entity>,
) {
    for despawn_event in despawn_events.read() {
        if let Ok(_entity) = craber_query.get_mut(despawn_event.entity) {
            commands.entity(despawn_event.entity).despawn();
        }
    }
}

/// System 1: Continuous rotation via torque
fn apply_rotation(mut query: Query<(&mut ExternalTorque, &Brain), With<Craber>>) {
    for (mut external_torque, brain) in query.iter_mut() {
        let rotation = brain.get_rotation();
        let torque = rotation * TORQUE_SCALE;
        external_torque.set_torque(torque);
    }
}

/// System 2: Continuous perpendicular velocity damping (keel effect)
fn apply_alignment(
    mut query: Query<(&mut ExternalForce, &LinearVelocity, &Transform, &Brain), With<Craber>>,
) {
    // for (mut external_force, linear_velocity, transform, brain) in query.iter_mut() {
    //     let alignment = brain.get_align_velocity();
    //     let facing_dir = (transform.rotation * Vec3::NEG_Y).truncate();
    //     let current_vel = linear_velocity.0;
    //     let parallel = facing_dir * current_vel.dot(facing_dir);
    //     let perpendicular = current_vel - parallel;
    //     let damping_force = -perpendicular * alignment * ALIGN_DAMPING_COEFF;
    //     external_force.apply_force(damping_force);
    // }
}

/// System 3: Accumulator-gated kick impulse
fn apply_kick(
    mut query: Query<
        (
            Entity,
            &mut ExternalImpulse,
            &mut KickAccumulator,
            &Transform,
            &Brain,
        ),
        With<Craber>,
    >,
    time: Res<Time>,
    mut lose_energy_events: EventWriter<LoseEnergyEvent>,
) {
    let dt = time.delta_seconds();
    for (entity, mut external_impulse, mut accumulator, transform, brain) in query.iter_mut() {
        let kick_rate = brain.get_kick_rate().max(0.0);
        let effective_rate = 1.0 - (-kick_rate * KICK_RATE_STEEPNESS).exp();
        let kick_strength = brain.get_kick_strength().max(0.0);
        let effective_strength = 1.0 - (-kick_strength * KICK_STEEPNESS).exp();

        accumulator.0 += effective_rate * dt;
        if accumulator.0 < KICK_THRESHOLD {
            continue;
        }
        accumulator.0 = 0.0;

        let facing_dir = (transform.rotation * Vec3::NEG_Y).truncate();
        let thrust = facing_dir * effective_strength * MAX_IMPULSE;
        external_impulse.apply_impulse(thrust);

        let energy_cost = effective_strength.powf(1.5) * KICK_ENERGY_MODIFIER;
        lose_energy_events.send(LoseEnergyEvent {
            entity,
            energy_lost: energy_cost,
        });
    }
}

pub fn vision_update(
    mut query: Query<(&mut Vision, &Transform, &Collider, &Parent)>,
    // mut food_query: Query<(&mut Food, &Transform)>,
    // craber_query: Query<(&Craber, &Transform)>,
    mut vision_events: EventReader<VisionEvent>,
) {
    // add or remove food from vision
    for vision_event in vision_events.read() {
        match vision_event.event_type {
            VisionEventType::Food => {
                // println!("Enterantation happened!");
                if let Ok((mut vision, transform, collider, parent)) =
                    query.get_mut(vision_event.vision_entity)
                {
                    // println!("See food true");
                    let manifolds = &vision_event.manifolds;
                    // let mut closest_manifold: Option<&ContactManifold> = None;
                    let mut min_distance = f32::MAX;
                    let mut closest_point = Vec2::new(0.0, 0.0);

                    for manifold in manifolds {
                        for contact in &manifold.contacts {
                            if vision_event.entity_id == 1 {
                                // Compute the distance to the circle's center
                                let distance = contact.point1.length() - contact.penetration;

                                // Update the closest manifold if this distance is smaller
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.point1;
                                }
                            } else {
                                let distance = contact.point2.length() - contact.penetration;

                                // Update the closest manifold if this distance is smaller
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.point2;
                                }
                            }
                        }
                        // println!("Doing some Manifolds {} {}", min_distance, closest_point)
                    }
                    // Normalizing (negated: Avian contact points face inward)
                    closest_point = -closest_point / min_distance;
                    vision.entities_in_vision.push(vision_event.entity);
                    // let craber_transform = craber_query.get(parent.get()).unwrap().1;
                    // let craber_direction = craber_transform.rotation.mul_vec3(Vec3::Y);
                    let vision_direction = transform.rotation.mul_vec3(Vec3::Y);
                    let craber_direction = vision_direction;
                    vision.nearest_food_distance = min_distance;
                    vision.nearest_food_direction = angle_direction_between_vectors(
                        craber_direction,
                        Vec3::new(closest_point.x, closest_point.y, 0.),
                    );
                    vision.see_food = true;
                    let rot_angle = transform.rotation.to_euler(EulerRot::ZYX).0;
                    // println!(
                    //     "FOOD raw_pt:{} dist:{:.2} norm_pt:{} facing:{} rot:{:.2}rad angle:{:.3}",
                    //     closest_point * min_distance,
                    //     min_distance,
                    //     closest_point,
                    //     craber_direction,
                    //     rot_angle,
                    //     vision.nearest_food_direction
                    // );
                }
            }
            VisionEventType::Craber => {
                // println!("Enterantation happened!");
                if let Ok((mut vision, transform, collider, parent)) =
                    query.get_mut(vision_event.vision_entity)
                {
                    // println!("See food true");
                    let manifolds = &vision_event.manifolds;
                    // let mut closest_manifold: Option<&ContactManifold> = None;
                    let mut min_distance = f32::MAX;
                    let mut closest_point = Vec2::new(0.0, 0.0);

                    for manifold in manifolds {
                        for contact in &manifold.contacts {
                            if vision_event.entity_id == 1 {
                                // Compute the distance to the circle's center
                                let distance = contact.point1.length() - contact.penetration;

                                // Update the closest manifold if this distance is smaller
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.point1;
                                }
                            } else {
                                let distance = contact.point2.length() - contact.penetration;

                                // Update the closest manifold if this distance is smaller
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.point2;
                                }
                            }
                        }
                        // println!("Doing some Manifolds {} {}", min_distance, closest_point)
                    }
                    // Normalizing (negated: Avian contact points face inward)
                    closest_point = -closest_point / min_distance;
                    vision.entities_in_vision.push(vision_event.entity);
                    // let craber_transform = craber_query.get(parent.get()).unwrap().1;
                    // let craber_direction = craber_transform.rotation.mul_vec3(Vec3::Y);
                    let vision_direction = transform.rotation.mul_vec3(Vec3::Y);
                    let craber_direction = vision_direction;
                    vision.nearest_craber_distance = min_distance;
                    vision.nearest_craber_direction = angle_direction_between_vectors(
                        craber_direction,
                        Vec3::new(closest_point.x, closest_point.y, 0.),
                    );
                    vision.see_craber = true;
                    // println!("STUFF craber transform: {:?}, D {} Closest P {}  Radians {}", craber_transform, craber_direction, closest_point, vision.nearest_food_direction)
                }
            }
            _ => {}
        }
    }
}

pub fn brain_update(
    mut query: Query<(
        Entity,
        &mut Brain,
        &mut Craber,
        &mut BrainTickAccumulator,
        &Children,
    )>,
    mut vision_query: Query<(&mut Vision, &Transform)>,
    time: Res<Time>,
    mut lose_energy_events: EventWriter<LoseEnergyEvent>,
) {
    let dt = time.delta_seconds();
    for (entity, mut brain, mut craber, mut accumulator, children) in query.iter_mut() {
        // Compute effective tick rate from ModifyBrainInterval output (0-1 sigmoid)
        let modify_output = brain.get_modify_brain_interval().clamp(0.0, 1.0);
        let effective_rate =
            BRAIN_TICK_MIN_RATE + modify_output * (BRAIN_TICK_MAX_RATE - BRAIN_TICK_MIN_RATE);

        // Accumulate time toward next tick
        accumulator.0 += effective_rate * dt;
        if accumulator.0 < 1.0 {
            continue;
        }
        accumulator.0 -= 1.0;

        // Feed the current interval (1/rate, normalized to 0-1 range) into BrainInterval input
        let interval_normalized = (BRAIN_TICK_MIN_RATE / effective_rate).clamp(0.0, 1.0);
        brain.update_input(NeuronType::BrainInterval, interval_normalized);

        // Update vision inputs
        let mut vision = vision_query.get_mut(children[0]).unwrap().0;
        if vision.see_food {
            brain.update_input(NeuronType::NearestFoodAngle, vision.nearest_food_direction);
            brain.update_input(
                NeuronType::NearestFoodDistance,
                vision.nearest_food_distance,
            );
            vision.food_seen_timer = VISION_UPDATE_RATE;
            vision.no_see_food();
        } else {
            vision.food_seen_timer -= dt;
            if vision.food_seen_timer <= 0.0 {
                brain.update_input(NeuronType::NearestFoodAngle, 0.0);
                brain.update_input(NeuronType::NearestFoodDistance, 0.0);
            }
        }
        if vision.see_craber {
            brain.update_input(
                NeuronType::NearestCraberAngle,
                vision.nearest_craber_direction,
            );
            brain.update_input(
                NeuronType::NearestCraberDistance,
                vision.nearest_craber_distance,
            );
            vision.craber_seen_timer = VISION_UPDATE_RATE;
            vision.no_see_craber();
        } else {
            vision.craber_seen_timer -= dt;
            if vision.craber_seen_timer <= 0.0 {
                brain.update_input(NeuronType::NearestCraberAngle, 0.0);
                brain.update_input(NeuronType::NearestCraberDistance, 0.0);
            }
        }
        brain.feed_forward();

        // Flat energy cost per brain tick
        lose_energy_events.send(LoseEnergyEvent {
            entity,
            energy_lost: BRAIN_TICK_ENERGY_COST,
        });
    }
}

pub fn craber_lose_energy(
    mut lose_energy_events: EventReader<LoseEnergyEvent>,
    mut query: Query<&mut Energy>,
) {
    for lose_energy_event in lose_energy_events.read() {
        if let Ok(mut energy) = query.get_mut(lose_energy_event.entity) {
            energy.energy -= lose_energy_event.energy_lost;
        }
    }
}

pub fn craber_lose_health(
    mut lose_health_events: EventReader<LoseHealthEvent>,
    mut query: Query<&mut Health>,
) {
    for lose_health_event in lose_health_events.read() {
        if let Ok(mut health) = query.get_mut(lose_health_event.entity) {
            health.health -= lose_health_event.health_lost;
        }
    }
}

pub fn craber_attack_lose_health_add_energy(
    mut craber_attack_events: EventReader<CraberAttackEvent>,
    mut attacked_query: Query<&mut Health>,
    mut attacker_query: Query<&mut Energy>,
) {
    for craber_attack_event in craber_attack_events.read() {
        if let Ok(mut health) = attacked_query.get_mut(craber_attack_event.attacked_craber_entity) {
            if health.health <= 0. {
                continue;
            }
            if health.health - craber_attack_event.attack_damage < 0. {
                // Overkill: only deal remaining health as actual damage, scale energy gain down
                let actual_damage = health.health;
                let energy_modifier = actual_damage / craber_attack_event.attack_damage;
                if let Ok(mut energy) =
                    attacker_query.get_mut(craber_attack_event.attacking_craber_entity)
                {
                    energy.energy += craber_attack_event.energy_to_gain * energy_modifier;
                }
                health.health = 0.0; // ensure death
                continue;
            }
            if let Ok(mut energy) =
                attacker_query.get_mut(craber_attack_event.attacking_craber_entity)
            {
                health.health -= craber_attack_event.attack_damage;
                energy.energy += craber_attack_event.energy_to_gain;
            }
        }
    }
}
