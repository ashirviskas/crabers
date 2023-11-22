use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::{Timer, TimerMode},
};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_inspector_egui_rapier::InspectableRapierPlugin;
// use bevy_rapier2d::prelude::*;
use bevy_xpbd_2d::prelude::*;

mod craber;
use craber::*;

mod food;
use food::*;

mod common;
use bevy_pancam::{PanCam, PanCamPlugin};
use common::*;

const SOME_COLLISION_THRESHOLD: f32 = 20.0;
const FOOD_SPAWN_RATE: f32 = 0.05;
const CRABER_SPAWN_RATE: f32 = 0.1;

const WALL_THICKNESS: f32 = 60.0;
// const QUAD_TREE_CAPACITY: usize = 16;
const RAVERS_TIMER: f32 = 0.2;
const GRAVITY: f32 = 0.0;
const DRAG: f32 = 0.01;

const FORCE_APPLICATION_RATE: f32 = 0.5;

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
        .add_systems(Update, do_despawning)
        .add_systems(Update, spawn_craber)
        .add_systems(Update, (apply_acceleration))
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
        max_scale: Some(40.),       // prevent the camera from zooming too far out
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

    // commands.insert_resource(RapierConfiguration {
    //     gravity: Vect::new(0.0, GRAVITY),
    //     ..Default::default()
    // });
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
        .insert(CollisionLayers::new([Layer::Blue], [Layer::Blue]))
        .insert(RigidBody::Static)
        .insert(Collider::cuboid(WORLD_SIZE * 2.0, WALL_THICKNESS / 2.0));
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
        .insert(CollisionLayers::new([Layer::Blue], [Layer::Blue]))
        .insert(RigidBody::Static)
        .insert(Collider::cuboid(WORLD_SIZE * 2.0, WALL_THICKNESS / 2.0));
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
        .insert(CollisionLayers::new([Layer::Blue], [Layer::Blue]))
        .insert(RigidBody::Static)
        .insert(Collider::cuboid(WALL_THICKNESS / 2.0, WORLD_SIZE * 2.0));
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
        .insert(CollisionLayers::new([Layer::Blue], [Layer::Blue]))
        .insert(RigidBody::Static)
        .insert(Collider::cuboid(WALL_THICKNESS / 2.0, WORLD_SIZE * 2.0));
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
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|x| x.average());

    if let Some(fps) = fps {
        debug_info.fps = fps;
    } else {
        debug_info.fps = 0.0;
        return;
    }

    for mut text in query.iter_mut() {
        text.sections[1].value = format!(
            "\nEntity count: {}, \nFPS: {:.2}",
            debug_info.entity_count, debug_info.fps
        );
    }
}

fn entity_selection(
    mouse_button_input: Res<Input<MouseButton>>,
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

fn quad_tree_update(
    _commands: Commands,
    mut quadtree: ResMut<Quadtree>,
    mut craber_query: Query<(Entity, &EntityType, &mut Craber, &Transform)>,
    food_query: Query<(Entity, &EntityType, &Food, &Transform)>,
) {
    quadtree.clear();
    for (entity, _, _, transform) in craber_query.iter_mut() {
        let quad_tree_entity =
            QuadtreeEntity::new(transform.translation.truncate(), entity, EntityType::Craber);
        quadtree.insert(quad_tree_entity);
    }
    for (entity, _, _, transform) in food_query.iter() {
        let quad_tree_entity =
            QuadtreeEntity::new(transform.translation.truncate(), entity, EntityType::Food);
        quadtree.insert(quad_tree_entity);
    }
}

fn draw_quadtree_debug(
    mut commands: Commands,
    quadtree: Res<Quadtree>,
    debug_rectangles: Query<Entity, With<DebugRectangle>>,
) {
    for entity in debug_rectangles.iter() {
        commands.entity(entity).despawn();
    }
    quadtree.draw(&mut commands);
}

// TODO: Make a separate despawn system for each entity type

fn do_collision(
    _commands: Commands,
    mut collide_event_reader: EventReader<CollisionStarted>,
    query: Query<(Entity, &Transform, &EntityType)>,
    mut craber_query: Query<(Entity, &mut Craber)>,
    food_query: Query<(Entity, &mut Food)>,
    mut despawn_events: EventWriter<DespawnEvent>,
) {
    for collide_event in collide_event_reader.read() {
        let entity1 = collide_event.0;
        let entity2 = collide_event.1;

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
                    _ => {}
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

// fn apply_drag(_commands: Commands, mut query: Query<(Entity, &mut LinearVelocity, &Weight)>) {
//     for (_entity, mut velocity, weight) in query.iter_mut() {
//         velocity.0 *= 1.0 - DRAG * weight.weight;
//         velocity.0 *= 1.0 - DRAG * weight.weight;
//     }
// }

fn apply_acceleration(
    _commands: Commands,
    mut query: Query<(
        Entity,
        &mut LinearVelocity,
        &Acceleration,
        &Transform,
        &mut ExternalForce,
    )>,
    time: Res<Time>,
    mut timer: ResMut<ForceApplicationTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    for (_, mut velocity, acceleration, transform, mut external_force) in query.iter_mut() {
        // velocity.linvel += acceleration.0;
        // let forward = transform.rotation.mul_vec3(acceleration.0.extend(0.0));
        // // Convert the rotated vector back to 2D
        // let rotated_forward_2d = Vec2::new(forward.x, forward.y);
        // let acceleration_vector = rotated_forward_2d;
        // // println!("Acceleration vector: {:?}", acceleration_vector);
        // // println!("Velocity: {:?}", velocity.linvel);
        // velocity.0 += acceleration_vector;
        // println!("Velocity: {:?}", velocity.linvel);
        // let transform_rotation = Transform::from_rotation(transform.rotation);
        // println!("Force before: {:?}", external_force);
        let force = transform
            .rotation
            .mul_vec3(acceleration.0.extend(0.0) * 100.);
        let force_2d = Vec2::new(force.x, force.y);
        velocity.0 = force_2d;
        // external_force.set_force(force_2d);
        // external_force.with_persistence(true);
        // println!("Force after: {:?}", external_force);
    }
}
