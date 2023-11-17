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

const ENERGY_CONSUMPTION_RATE: f32 = 1.0;
const SOME_COLLISION_THRESHOLD: f32 = 10.0;
const FOOD_SPAWN_RATE: f32 = 2.0;
const CRABER_SPAWN_RATE: f32 = 3.0;

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
        .add_systems(Update, handle_collisions)
        .add_systems(Update, energy_consumption)
        .add_systems(Update, despawn_dead_crabers)
        .add_systems(Update, update_craber_color)
        .run();
}

fn setup(mut commands: Commands) {
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

fn handle_collisions(
    mut commands: Commands,
    mut craber_query: Query<(Entity, &mut Craber, &Transform)>,
    food_query: Query<(Entity, &Food, &Transform)>,
) {
    for (craber_entity, mut craber, craber_transform) in craber_query.iter_mut() {
        for (food_entity, food, food_transform) in food_query.iter() {
            if collides(craber_transform, food_transform, SOME_COLLISION_THRESHOLD) {
                craber.energy += food.energy_value;
                commands.entity(food_entity).despawn(); // Remove food on collision
            }
        }
    }
}

fn energy_consumption(mut query: Query<(&mut Craber, &mut Velocity)>, time: Res<Time>) {
    for (mut craber, mut velocity) in query.iter_mut() {
        craber.energy -= ENERGY_CONSUMPTION_RATE * time.delta_seconds();

        // Handle low energy situations
        if craber.energy <= 0.0 {
            velocity.0 = Vec2::ZERO; // Stop movement
            craber.health -= 1.0; // Reduce health if needed
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
                println!("Entity {:?} at {:?}", entity, transform);
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
