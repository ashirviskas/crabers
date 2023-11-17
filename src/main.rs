use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::{Timer, TimerMode},
};

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
const QUAD_TREE_CAPACITY: usize = 8;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(SelectedEntity::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, entity_selection)
        .add_systems(Update, update_selected_entity_info)
        .add_systems(Update, update_ui)
        .add_systems(Update, food_spawner)
        .add_systems(Update, craber_spawner)
        .add_systems(Update, craber_movement)
        // SAP
        // .add_systems(Update, update_sap)
        // .add_systems(Update, handle_collisions_sap)
        // Quadtree
        .add_systems(Update, handle_collisions_quadtree)
        .add_systems(Update, quad_tree_update)
        // other
        .add_systems(Update, energy_consumption)
        .add_systems(Update, despawn_dead_crabers)
        .add_systems(Update, update_craber_color)
        .add_systems(Update, print_current_entity_count)
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

    commands.insert_resource(Quadtree::new(boundary, QUAD_TREE_CAPACITY));
    // commands.insert_resource(Sap::new());
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

fn handle_collisions_quadtree(
    mut commands: Commands,
    mut quadtree: ResMut<Quadtree>,
    mut craber_query: Query<(Entity, &EntityType, &mut Craber, &Transform)>,
    food_query: Query<(Entity, &EntityType, &Food, &Transform)>,
) {
    for (entity, entity_type, mut craber, transform) in craber_query.iter_mut() {
        let search_area = Rectangle {
            x: transform.translation.x,
            y: transform.translation.y,
            width: SOME_COLLISION_THRESHOLD,
            height: SOME_COLLISION_THRESHOLD,
        };
        let found = quadtree.query(&search_area);
        // quadtree.query(&search_area);
        for point in found.into_iter() {
            if point.entity != entity {
                if let Ok(food) = food_query.get(point.entity) {
                    craber.energy += food.2.energy_value;
                    // println!("Destroying {:?}", point.entity);
                    // println!("Collision between {:?} and {:?}", entity_type, point.entity);
                    commands.entity(point.entity).despawn();
                }
            }
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

// SAP

fn update_sap(mut sap: ResMut<Sap>, query: Query<(Entity, &Transform, &CollidableEntity)>) {
    sap.update(query);
}

fn handle_collisions_sap(
    sap: Res<Sap>,
    mut commands: Commands,
    mut craber_query: Query<(Entity, &mut Craber, &Transform)>, // Mutable reference to Craber
    food_query: Query<(Entity, &Food, &Transform)>,
) {
    let potential_collisions = sap.sweep_and_prune();
    if potential_collisions.is_empty() {
        return;
    }
    // println!("Potential collisions: {:?}", potential_collisions);
    for (entity_a, entity_b) in potential_collisions {
        // Check if entity_a is a craber
        if let Ok((craber_entity, mut craber, craber_transform)) = craber_query.get_mut(entity_a) {
            // Check if entity_b is food
            if let Ok((food_entity, food, food_transform)) = food_query.get(entity_b) {
                // Perform collision check between craber and food
                if collides(craber_transform, food_transform, SOME_COLLISION_THRESHOLD) {
                    // Collision logic (e.g., increase energy of Craber, despawn Food entity)
                    craber.energy += food.energy_value;
                    commands.entity(food_entity).despawn();
                }
            }
        }
        // Check if entity_b is a craber
        else if let Ok((craber_entity, mut craber, craber_transform)) =
            craber_query.get_mut(entity_b)
        {
            // Check if entity_a is food
            if let Ok((food_entity, food, food_transform)) = food_query.get(entity_a) {
                // Perform collision check between craber and food
                if collides(craber_transform, food_transform, SOME_COLLISION_THRESHOLD) {
                    // Collision logic
                    craber.energy += food.energy_value;
                    commands.entity(food_entity).despawn();
                }
            }
        }
    }
}
