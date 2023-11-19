use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::{Timer, TimerMode},
    utils::tracing::Event,
};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude::*;

use rand::Rng;
mod craber;
use craber::*;

mod food;
use food::*;

mod common;
use bevy_pancam::{PanCam, PanCamPlugin};
use common::*;

const SOME_COLLISION_THRESHOLD: f32 = 20.0;
const FOOD_SPAWN_RATE: f32 = 0.01;
const CRABER_SPAWN_RATE: f32 = 0.001;
const QUAD_TREE_CAPACITY: usize = 16;
const RAVERS_TIMER: f32 = 0.2;
const BUMPINESS_RANDOMNESS_STRENGTH: f32 = 0.01;
const GRAVITY: f32 = 0.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        // .add_plugins(InspectableRapierPlugin)
        // .add_plugins(WorldInspectorPlugin::default())
        .insert_resource(SelectedEntity::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, entity_selection)
        .add_systems(Update, update_selected_entity_info)
        .add_systems(Update, update_ui)
        .add_systems(Update, food_spawner)
        .add_systems(Update, craber_spawner)
        // .add_systems(Update, craber_movement)
        // SAP
        // .add_systems(Update, update_sap)
        // .add_systems(Update, handle_collisions_sap)
        // Quadtree
        // .add_systems(Update, handle_collisions_quadtree)
        // .add_systems(Update, quad_tree_update)
        // other
        .add_systems(Update, energy_consumption)
        .add_systems(Update, despawn_dead_crabers)
        // .add_systems(Update, update_craber_color)
        .add_systems(Update, print_current_entity_count)
        .add_event::<DespawnEvent>()
        .add_systems(Update, do_collision)
        .add_systems(Update, do_despawning)
        // Fun and debug stuff
        // .add_systems(Update, ravers)
        // .add_systems(Update, draw_quadtree_debug)
        .run();
}

fn setup(mut commands: Commands) {
    let boundary = Rectangle {
        x: 0.0,
        y: 0.0,
        width: WORLD_SIZE * 2.,
        height: WORLD_SIZE * 2.,
    };
    // setup

    commands.spawn(Camera2dBundle::default()).insert(PanCam {
        grab_buttons: vec![MouseButton::Right], // which buttons should drag the camera
        enabled: true,        // when false, controls are disabled. See toggle example.
        zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
        min_scale: 0.1,       // prevent the camera from zooming too far in
        max_scale: Some(40.), // prevent the camera from zooming too far out
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

    commands.insert_resource(RapierConfiguration {
        gravity: Vect::new(0.0, GRAVITY),
        ..Default::default()
    });

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WORLD_SIZE * 2.0, 10.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, WORLD_SIZE, 0.0)),
            ..Default::default()
        })
        .insert(Collider::cuboid(WORLD_SIZE * 2.0, 5.0));
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WORLD_SIZE * 2.0, 10.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -WORLD_SIZE, 0.0)),
            ..Default::default()
        })
        .insert(Collider::cuboid(WORLD_SIZE * 2.0, 5.0));
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(10.0, WORLD_SIZE * 2.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(WORLD_SIZE, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Collider::cuboid(5.0, WORLD_SIZE * 2.0));
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(10.0, WORLD_SIZE * 2.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(-WORLD_SIZE, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Collider::cuboid(5.0, WORLD_SIZE * 2.0));
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TextBundle {
        text: Text::from_section(
            "No craber selected",
            TextStyle {
                font: Default::default(),
                font_size: 30.0,
                color: Color::WHITE,
            },
        ),
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
    mut commands: Commands,
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
    mut commands: Commands,
    mut collide_event_reader: EventReader<CollisionEvent>,
    mut query: Query<(Entity, &Transform, &EntityType)>,
    mut craber_query: Query<(Entity, &mut Craber)>,
    mut food_query: Query<(Entity, &mut Food)>,
    mut despawn_events: EventWriter<DespawnEvent>,
) {
    for collide_event in collide_event_reader.read() {
        // Check if event Started or Stopped
        if let CollisionEvent::Started(_, _, _) = collide_event {
            // println!("Collision started");
        } else {
            // println!("Collision stopped");
            continue;
        }
        if let CollisionEvent::Started(entity1, entity2, _) = collide_event {
            let (entity1_type, entity2_type) = match query.get(*entity1) {
                Ok(entity1) => match query.get(*entity2) {
                    Ok(entity2) => (entity1.2, entity2.2),
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            match (entity1_type, entity2_type) {
                (EntityType::Craber, EntityType::Craber) => {
                    continue;
                }
                (EntityType::Craber, EntityType::Food) => {
                    if let Ok(mut craber) = craber_query.get_mut(*entity1) {
                        if let Ok(food) = food_query.get(*entity2) {
                            craber.1.energy += food.1.energy_value;
                            // commands.entity(*entity2).despawn();
                            despawn_events.send(DespawnEvent { entity: *entity2 });
                        }
                    }
                }
                (EntityType::Food, EntityType::Craber) => {
                    if let Ok(mut craber) = craber_query.get_mut(*entity2) {
                        if let Ok(food) = food_query.get(*entity1) {
                            craber.1.energy += food.1.energy_value;
                            // commands.entity(*entity1).despawn();
                            despawn_events.send(DespawnEvent { entity: *entity1 });
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn do_despawning(
    mut commands: Commands,
    mut despawn_events: EventReader<DespawnEvent>,
    mut craber_query: Query<(Entity)>,
) {
    for despawn_event in despawn_events.read() {
        if let Ok(entity) = craber_query.get_mut(despawn_event.entity) {
            commands.entity(despawn_event.entity).despawn();
        }
    }
}
