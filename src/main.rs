use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::{Timer, TimerMode},
};
use avian2d::{parry::shape::Cuboid, prelude::*, math::*};
// use avian2d::collision::collider::parry::*;

mod craber;
use craber::*;

mod brain;
use brain::*;

mod food;
use food::*;

mod common;
use bevy_pancam::{PanCam, PanCamPlugin};
use common::{
    Rectangle,
    *
};


const SOME_COLLISION_THRESHOLD: f32 = 20.0;
const FOOD_SPAWN_RATE: f32 = 0.01;
const CRABER_SPAWN_RATE: f32 = 0.1;

const MAX_CRABER_COUNT: usize = 10000;
const MAX_FOOD_COUNT: usize = 10000;

const WALL_THICKNESS: f32 = 60.0;
// const QUAD_TREE_CAPACITY: usize = 16;
const RAVERS_TIMER: f32 = 0.2;
const GRAVITY: f32 = 0.0;

const FORCE_APPLICATION_RATE: f32 = 0.5;

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
        .add_event::<DespawnEvent>()
        .add_event::<SpawnEvent>()
        .add_event::<ReproduceEvent>()
        .add_event::<VisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, entity_selection)
        .add_systems(Update, update_selected_entity_info)
        .add_systems(Update, update_ui)
        .add_systems(Update, update_debug_info)
        .add_systems(Update, food_spawner)
        .add_systems(Update, craber_spawner)
        .add_systems(Update, energy_consumption)
        .add_systems(Update, despawn_dead_crabers)
        .add_systems(Update, craber_reproduce)
        // .add_systems(Update, update_craber_color)
        .add_systems(Update, print_current_entity_count)
        .add_systems(Update, do_collision)
        // .add_systems(Update, do_decollisions)
        .add_systems(Update, do_despawning)
        .add_systems(Update, spawn_craber)
        .add_systems(Update, apply_acceleration)
        .add_systems(Update, vision_update)
        .add_systems(Update, brain_update)
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
        max_scale: 40.,       // prevent the camera from zooming too far out
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
    commands.insert_resource(ForceApplicationTimer(Timer::from_seconds(
        FORCE_APPLICATION_RATE,
        TimerMode::Repeating,
    )));

    commands.insert_resource(VisionUpdateTimer(Timer::from_seconds(
        VISION_UPDATE_RATE,
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

fn setup_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn(TextBundle {
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
    });
}

fn update_ui(selected: Res<SelectedEntity>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        if let Some(_) = selected.entity {
            text.sections[0].value = format!(
                "Health: {:.2}, Energy: {:.2}",
                selected.health, selected.energy
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
    mut query: Query<&mut Text>,
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
    craber_query: Query<&Craber>,
    food_query: Query<&Food>,
) {
    if let Some(entity) = selected.entity {
        // Check if the selected entity is a Craber
        if let Ok(craber) = craber_query.get(entity) {
            selected.health = craber.health;
            selected.energy = craber.energy;
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
    mut craber_query: Query<(Entity, &mut Craber)>,
    food_query: Query<(Entity, &mut Food, &Transform)>,
    mut despawn_events: EventWriter<DespawnEvent>,
    mut vision_events: EventWriter<VisionEvent>,
) {
    for collide_event in collide_event_reader.read() {
        let entity1 = collide_event.0.entity1;
        let entity2 = collide_event.0.entity2;

        if let Ok((entity1, _, entity1_type)) = query.get(entity1) {
            if let Ok((entity2, _, entity2_type)) = query.get(entity2) {
                match (entity1_type, entity2_type) {
                    (EntityType::Craber, EntityType::Craber) => {
                        continue;
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
                    (EntityType::Food, EntityType::Vision) => { // Do entered vision instead and start tracking
                        vision_events.send(VisionEvent {
                            vision_entity: entity2,
                            entity: entity1,
                            event_type: VisionEventType::Entered,
                        });
                        println!("Food with entity {:?} entered vision", entity1);
                        // println!("Food entered vision A");
                    }
                    (EntityType::Vision, EntityType::Food) => {
                        vision_events.send(VisionEvent {
                            vision_entity: entity1,
                            entity: entity2,
                            event_type: VisionEventType::Entered,
                        });
                        println!("Food with entity {:?} entered vision", entity2);
                        // println!("Food entered vision B");
                    }
                    _ => {}
                }
            }
        }
    }
}

// fn do_decollisions(
//     _commands: Commands,
//     mut decollide_event_reader: EventReader<CollisionEnded>,
//     query: Query<(Entity, &Transform, &EntityType)>,
//     mut vision_events: EventWriter<VisionEvent>,
// ) {
//     for decollide_event in decollide_event_reader.read() {
//         let entity1 = decollide_event.0;
//         let entity2 = decollide_event.1;

//         if let Ok((entity1, _, entity1_type)) = query.get(entity1) {
//             if let Ok((entity2, _, entity2_type)) = query.get(entity2) {
//                 match (entity1_type, entity2_type) {
//                     (EntityType::Food, EntityType::Vision) => {
//                         vision_events.send(VisionEvent {
//                             vision_entity: entity2,
//                             entity: entity1,
//                             event_type: VisionEventType::Exited,
//                         });
//                         println!("Food with entity {:?} exited vision", entity1);
//                         // println!("Food exited vision A");
//                     }
//                     (EntityType::Vision, EntityType::Food) => {
//                         vision_events.send(VisionEvent {
//                             vision_entity: entity1,
//                             entity: entity2,
//                             event_type: VisionEventType::Exited,
//                         });
//                         println!("Food with entity {:?} exited vision", entity2);
//                         // println!("Food exited vision B");
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }
// }

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

fn apply_acceleration(
    _commands: Commands,
    mut query: Query<(
        Entity,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &Acceleration,
        &Transform,
        &mut ExternalForce,
        &Brain,
    )>,
    time: Res<Time>,
    mut timer: ResMut<ForceApplicationTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    for (
        _,
        mut linear_velocity,
        mut angular_velocity,
        acceleration,
        transform,
        mut external_force,
        brain,
    ) in query.iter_mut()
    {
        let force = transform
            .rotation
            .mul_vec3(acceleration.0.extend(0.0) * 100.);
        let force_2d = Vec2::new(force.x, force.y);
        linear_velocity.0 = force_2d;
        let rotation_vector = brain.get_rotation();
        if rotation_vector != 0.0 {
            println!("Rotation vector: {}", rotation_vector);
        }
        angular_velocity.0 = rotation_vector;
        // Debug, apply rotation as force
        // external_force.apply_force(Vec2::new(rotation_vector * 100., 0.));
        // linear_velocity.0 += rotation_vector * 100.;
        // println!("Rotation vector: {}", rotation_vector);
    }
}

pub fn vision_update(
    spatial_query: SpatialQuery,
    mut query: Query<(&mut Vision, &Transform, &Collider, &Parent)>,
    mut food_query: Query<(&mut Food, &Transform)>,
    craber_query: Query<(&Craber, &Transform)>,
    mut vision_events: EventReader<VisionEvent>,
) {
    // add or remove food from vSision
    for vision_event in vision_events.read() {
        match vision_event.event_type {
            VisionEventType::Entered => {
                    if let Ok((mut vision, _, _, _)) =
                        query.get_mut(vision_event.vision_entity)
                    {
                        vision.see_food = true;
                        vision.entities_in_vision.push(vision_event.entity);
                    }
            }
            // VisionEventType::Exited => {
            //     if let Ok((mut vision, _, _, _)) = query.get_mut(vision_event.vision_entity) {
            //         vision.entities_in_vision.retain(|&x| x != vision_event.entity);
            //         if vision.entities_in_vision.is_empty() {
            //             vision.see_food = false;
            //         }
            //     }
            // }
            _ => {}
        }
    }
    // update food in vision
    let spatial_query_filter = SpatialQueryFilter {
        mask: LayerMask(Layer::Food.to_bits()),
        ..Default::default()
    };
    for (mut vision, _, collider, parent) in query.iter_mut() {
        if vision.entities_in_vision.is_empty() {
            continue;
        }
        let craber_transform = craber_query.get(parent.get()).unwrap().1;
        let intersections_entities = spatial_query.shape_intersections(
            &collider,
            craber_transform.translation.truncate(),
            0.,
            spatial_query_filter.clone(),
        );
        vision.see_food = false;
        if intersections_entities.is_empty() {
            // No intersections, no entities in vision
            vision.entities_in_vision.clear();
            continue;
        }
        let mut nearest_food_distance = 1000.0;
        let mut nearest_food_angle = 0.0;
        for intersection in intersections_entities.iter() {
            let food_transform = match food_query.get_mut(*intersection) {
                Ok(transform) => transform,
                Err(_) => {
                    continue;
                }
            };
            let food_distance =
                (food_transform.1.translation - craber_transform.translation).length();
            if food_distance < nearest_food_distance {
                nearest_food_distance = food_distance;
                // nearest_food_angle = food_transform.translation.angle_between(craber_transform.translation);
                let food_direction = food_transform.1.translation - craber_transform.translation;
                let craber_direction = craber_transform.rotation.mul_vec3(Vec3::Y);
                // nearest_food_angle = craber_direction.angle_between(food_direction);
                nearest_food_angle = full_angle_between_vectors(craber_direction, food_direction);
            }
        }
        if nearest_food_distance == 1000.0 {
            vision.see_food = false;
            continue;
        }
        // println!("Nearest food distance: {}", nearest_food_distance);
        // println!("Nearest food angle: {}", nearest_food_angle);
        vision.nearest_food_distance = nearest_food_distance;
        vision.nearest_food_angle_radians = nearest_food_angle;
        vision.see_food = true;
    }
    // let spatial_query_filter = SpatialQueryFilter {
    //     masks: Layer::Food.to_bits(),
    //     ..Default::default()
    // };
    // for (mut vision, _, collider, parent) in query.iter_mut() {
    //     let craber_transform = entity_transform_query.get(parent.get()).unwrap();
    //     let intersections_entities = spatial_query.shape_intersections(
    //         &collider,
    //         craber_transform.translation.truncate(),
    //         0.,
    //         spatial_query_filter.clone(),
    //     );
    //     vision.see_food = false;
    //     if intersections_entities.is_empty() {
    //         continue;
    //     }
    //     let mut nearest_food_distance = 1000.0;
    //     let mut nearest_food_angle = 0.0;
    //     for intersection in intersections_entities.iter() {
    //         let food_transform = match entity_transform_query.get(*intersection) {
    //             Ok(transform) => transform,
    //             Err(_) => {
    //                 continue;
    //             }
    //         };
    //         let food_distance =
    //             (food_transform.translation - craber_transform.translation).length();
    //         if food_distance < nearest_food_distance {
    //             nearest_food_distance = food_distance;
    //             // nearest_food_angle = food_transform.translation.angle_between(craber_transform.translation);
    //             let food_direction = food_transform.translation - craber_transform.translation;
    //             let craber_direction = craber_transform.rotation.mul_vec3(Vec3::Y);
    //             // nearest_food_angle = craber_direction.angle_between(food_direction);
    //             nearest_food_angle = full_angle_between_vectors(craber_direction, food_direction);
    //         }
    //     }
    //     if nearest_food_distance == 1000.0 {
    //         vision.see_food = false;
    //         continue;
    //     }
    //     // println!("Nearest food distance: {}", nearest_food_distance);
    //     // println!("Nearest food angle: {}", nearest_food_angle);
    //     vision.nearest_food_distance = nearest_food_distance;
    //     vision.nearest_food_angle_radians = nearest_food_angle;
    //     vision.see_food = true;
    // }
}

pub fn brain_update(
    mut query: Query<(&mut Brain, &mut Craber, &Children)>,
    vision_query: Query<(&Vision, &Transform)>,
    time: Res<Time>,
) {
    for (mut brain, mut craber, children) in query.iter_mut() {
        // Update inputs
        if vision_query.get(children[0]).unwrap().0.see_food {
            brain.update_input(
                NeuronType::NearestFoodAngle,
                vision_query
                    .get(children[0])
                    .unwrap()
                    .0
                    .nearest_food_angle_radians,
            );
        } else {
            brain.update_input(NeuronType::NearestFoodAngle, std::f32::consts::PI);
        }
        brain.feed_forward();
        // brain.print_brain();
    }
}
