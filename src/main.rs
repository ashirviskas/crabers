use avian2d::prelude::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::{Timer, TimerMode},
};

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
use common::*;

const SOME_COLLISION_THRESHOLD: f32 = 20.0;
const FOOD_SPAWN_RATE: f32 = 0.0004;
const CRABER_SPAWN_RATE: f32 = 0.1;

pub const MAX_FOOD_COUNT: usize = 10000;

const WALL_THICKNESS: f32 = 60.0;
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
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanCamPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(SelectedEntity::default())
        .insert_resource(DebugInfo::default())
        .insert_resource(NeuralNetworkLayout::default())
        .add_message::<DespawnEvent>()
        .add_message::<SpawnEvent>()
        .add_message::<ReproduceEvent>()
        .add_message::<VisionEvent>()
        .add_message::<LoseEnergyEvent>()
        .add_message::<LoseHealthEvent>()
        .add_message::<CraberCollisionEvent>()
        .add_message::<CraberAttackEvent>()
        .add_message::<CraberDespawnEvent>()
        .add_message::<FoodSpawnEvent>()
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
        .run();
}

fn setup(mut commands: Commands) {
    // setup
    let grab_buttons = if ENABLE_LEFT_MOUSE_BUTTON_DRAG {
        vec![MouseButton::Left, MouseButton::Right]
    } else {
        vec![MouseButton::Right]
    };
    commands.spawn(Camera2d).insert(PanCam {
        grab_buttons: grab_buttons,
        enabled: true,
        zoom_to_cursor: true,
        min_scale: 0.1,
        max_scale: 40.,
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
    commands.insert_resource(Gravity(Vec2::NEG_Y * GRAVITY));

    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WORLD_SIZE * 2.0, WALL_THICKNESS)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, WORLD_SIZE, 0.0)),
        ))
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WORLD_SIZE * 2.0, WALL_THICKNESS / 2.0));

    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WORLD_SIZE * 2.0, WALL_THICKNESS)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, -WORLD_SIZE, 0.0)),
        ))
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WORLD_SIZE * 2.0, WALL_THICKNESS / 2.0));
    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WALL_THICKNESS, WORLD_SIZE * 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(WORLD_SIZE, 0.0, 0.0)),
        ))
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WALL_THICKNESS / 2.0, WORLD_SIZE * 2.0));
    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(WALL_THICKNESS, WORLD_SIZE * 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(-WORLD_SIZE, 0.0, 0.0)),
        ))
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber]))
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WALL_THICKNESS / 2.0, WORLD_SIZE * 2.0));
}

/// Marker component for the main status/debug text UI
#[derive(Component)]
struct StatusText;

/// Marker for the debug info text span (child of StatusText)
#[derive(Component)]
struct DebugTextSpan;

fn setup_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Text::new("No craber selected"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
            StatusText,
        ))
        .with_child((
            Text::new("\nEntities: 0\nFps: 0.0"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
            DebugTextSpan,
        ));
}

fn update_ui(selected: Res<SelectedEntity>, mut query: Query<&mut Text, With<StatusText>>) {
    for mut text in query.iter_mut() {
        if let Some(_) = selected.entity {
            text.0 = format!(
                "Health: {:.2}, Energy: {:.2}\nGeneration: {}\nNearest food angle: {}",
                selected.health, selected.energy, selected.generation, selected.nearest_food_anlge
            );
        } else {
            text.0 = "No craber selected".to_string();
        }
    }
}

fn update_debug_info(
    mut debug_info: ResMut<DebugInfo>,
    craber_query: Query<&Craber>,
    food_query: Query<&Food>,
    diagnostics: Res<DiagnosticsStore>,
    _time: Res<Time>,
    mut query: Query<&mut Text, With<DebugTextSpan>>,
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
        text.0 = format!(
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
    camera_query: Query<(&Camera, &Transform, &Projection)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(position) = window.cursor_position() {
            let camera = camera_query.iter().next().unwrap().1;
            let projection = camera_query.iter().next().unwrap().2;
            let new_position = window_to_world(position, &window, camera, projection);
            for (entity, transform, selectable_type) in query.iter() {
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
    projection: &Projection,
) -> Vec3 {
    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    let centered_click_location = Vec2::new(
        position.x - window.width() / 2.,
        (position.y - window.height() / 2.) * -1.,
    );
    let scaled_click_location = centered_click_location * scale;
    let moved_click_location = scaled_click_location + camera.translation.truncate();
    let world_position = moved_click_location.extend(0.);

    world_position
}

fn update_selected_entity_info(
    mut selected: ResMut<SelectedEntity>,
    craber_query: Query<(&Transform, &Children, &Generation, &Brain, &Health, &Energy)>,
    vision_query: Query<(&Vision, &Transform, Entity, &ChildOf)>,
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
                if let Ok((vision, vision_transform, _, _child_of)) = vision_query.get(child) {
                    selected.vision_rotation = vision_transform.rotation;
                    selected.nearest_food_anlge = vision.nearest_food_direction;
                }
            }
        }
        // Check if the selected entity is Food
        else if let Ok(food) = food_query.get(entity) {
            selected.health = 0.0;
            selected.energy = food.energy_value;
        }
    }
}

fn do_collision(
    _commands: Commands,
    collisions: Collisions,
    query: Query<(Entity, &Transform, &EntityType)>,
    mut craber_query: Query<(Entity, &mut Energy)>,
    food_query: Query<(Entity, &mut Food, &Transform)>,
    mut despawn_events: MessageWriter<DespawnEvent>,
    mut vision_events: MessageWriter<VisionEvent>,
    mut craber_collision_events: MessageWriter<CraberCollisionEvent>,
) {
    for contacts in collisions.iter() {
        let entity1 = contacts.collider1;
        let entity2 = contacts.collider2;
        let manifolds = &contacts.manifolds;
        if let Ok((entity1, _, entity1_type)) = query.get(entity1) {
            if let Ok((entity2, _, entity2_type)) = query.get(entity2) {
                match (entity1_type, entity2_type) {
                    (EntityType::Craber, EntityType::Craber) => {
                        craber_collision_events.write(CraberCollisionEvent {
                            entity_a: entity1,
                            entity_b: entity2,
                        });
                    }
                    (EntityType::Craber, EntityType::Food) => {
                        if let Ok(mut craber) = craber_query.get_mut(entity1) {
                            if let Ok(food) = food_query.get(entity2) {
                                craber.1.energy += food.1.energy_value;
                                despawn_events.write(DespawnEvent { entity: entity2 });
                            }
                        }
                    }
                    (EntityType::Food, EntityType::Craber) => {
                        if let Ok(mut craber) = craber_query.get_mut(entity2) {
                            if let Ok(food) = food_query.get(entity1) {
                                craber.1.energy += food.1.energy_value;
                                despawn_events.write(DespawnEvent { entity: entity1 });
                            }
                        }
                    }
                    (EntityType::Food, EntityType::Vision) => {
                        vision_events.write(VisionEvent {
                            vision_entity: entity2,
                            entity: entity1,
                            event_type: VisionEventType::Food,
                            entity_id: 2,
                            manifolds: manifolds.clone(),
                        });
                    }
                    (EntityType::Vision, EntityType::Food) => {
                        vision_events.write(VisionEvent {
                            vision_entity: entity1,
                            entity: entity2,
                            event_type: VisionEventType::Food,
                            entity_id: 1,
                            manifolds: manifolds.clone(),
                        });
                    }
                    (EntityType::Craber, EntityType::Vision) => {
                        vision_events.write(VisionEvent {
                            vision_entity: entity2,
                            entity: entity1,
                            event_type: VisionEventType::Craber,
                            entity_id: 2,
                            manifolds: manifolds.clone(),
                        });
                    }
                    (EntityType::Vision, EntityType::Craber) => {
                        vision_events.write(VisionEvent {
                            vision_entity: entity1,
                            entity: entity2,
                            event_type: VisionEventType::Craber,
                            entity_id: 1,
                            manifolds: manifolds.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn do_craber_collision(
    mut craber_collision_events: MessageReader<CraberCollisionEvent>,
    query: Query<(Entity, &Brain, &LinearVelocity, &AngularVelocity)>,
    mut craber_attack_events: MessageWriter<CraberAttackEvent>,
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
                        brain_b.get_want_to_attack() * 5. * velocity_b.length() - angular_b.0.abs();
                    let b_damaged =
                        brain_a.get_want_to_attack() * 5. * velocity_a.length() - angular_a.0.abs();
                    if a_damaged > 0. {
                        craber_attack_events.write(CraberAttackEvent {
                            attacking_craber_entity: entity_b,
                            attacked_craber_entity: entity_a,
                            attack_damage: a_damaged,
                            energy_to_gain: a_damaged * 0.3,
                        });
                    }
                    if b_damaged > 0. {
                        craber_attack_events.write(CraberAttackEvent {
                            attacking_craber_entity: entity_a,
                            attacked_craber_entity: entity_b,
                            attack_damage: b_damaged,
                            energy_to_gain: b_damaged * 0.3,
                        });
                    }
                }
            }
        }
    }
}

fn do_despawning(
    mut commands: Commands,
    mut despawn_events: MessageReader<DespawnEvent>,
    mut craber_query: Query<Entity>,
) {
    for despawn_event in despawn_events.read() {
        if let Ok(_entity) = craber_query.get_mut(despawn_event.entity) {
            commands.entity(despawn_event.entity).despawn();
        }
    }
}

/// System 1: Continuous rotation via torque
fn apply_rotation(mut query: Query<(Forces, &Brain), With<Craber>>) {
    for (mut forces, brain) in query.iter_mut() {
        let rotation = brain.get_rotation();
        let torque = rotation * TORQUE_SCALE;
        forces.apply_torque(torque);
    }
}

/// System 2: Continuous perpendicular velocity damping (keel effect)
fn apply_alignment() {}

/// System 3: Accumulator-gated kick impulse
fn apply_kick(
    mut query: Query<(Entity, Forces, &mut KickAccumulator, &Transform, &Brain), With<Craber>>,
    time: Res<Time>,
    mut lose_energy_events: MessageWriter<LoseEnergyEvent>,
) {
    let dt = time.delta_secs();
    for (entity, mut forces, mut accumulator, transform, brain) in query.iter_mut() {
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
        forces.apply_linear_impulse(thrust);

        let energy_cost = effective_strength.powf(1.5) * KICK_ENERGY_MODIFIER;
        lose_energy_events.write(LoseEnergyEvent {
            entity,
            energy_lost: energy_cost,
        });
    }
}

pub fn vision_update(
    mut query: Query<(&mut Vision, &Transform, &Collider, &ChildOf)>,
    mut vision_events: MessageReader<VisionEvent>,
) {
    for vision_event in vision_events.read() {
        match vision_event.event_type {
            VisionEventType::Food => {
                if let Ok((mut vision, transform, _collider, _parent)) =
                    query.get_mut(vision_event.vision_entity)
                {
                    let manifolds = &vision_event.manifolds;
                    let mut min_distance = f32::MAX;
                    let mut closest_point = Vec2::new(0.0, 0.0);

                    for manifold in manifolds {
                        for contact in &manifold.points {
                            if vision_event.entity_id == 1 {
                                let distance = contact.anchor1.length() - contact.penetration;
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.anchor1;
                                }
                            } else {
                                let distance = contact.anchor2.length() - contact.penetration;
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.anchor2;
                                }
                            }
                        }
                    }
                    closest_point = -closest_point / min_distance;
                    vision.entities_in_vision.push(vision_event.entity);
                    let vision_direction = transform.rotation.mul_vec3(Vec3::Y);
                    let craber_direction = vision_direction;
                    vision.nearest_food_distance = min_distance;
                    vision.nearest_food_direction = angle_direction_between_vectors(
                        craber_direction,
                        Vec3::new(closest_point.x, closest_point.y, 0.),
                    );
                    vision.see_food = true;
                }
            }
            VisionEventType::Craber => {
                if let Ok((mut vision, transform, _collider, _parent)) =
                    query.get_mut(vision_event.vision_entity)
                {
                    let manifolds = &vision_event.manifolds;
                    let mut min_distance = f32::MAX;
                    let mut closest_point = Vec2::new(0.0, 0.0);

                    for manifold in manifolds {
                        for contact in &manifold.points {
                            if vision_event.entity_id == 1 {
                                let distance = contact.anchor1.length() - contact.penetration;
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.anchor1;
                                }
                            } else {
                                let distance = contact.anchor2.length() - contact.penetration;
                                if distance < min_distance {
                                    min_distance = distance;
                                    closest_point = contact.anchor2;
                                }
                            }
                        }
                    }
                    closest_point = -closest_point / min_distance;
                    vision.entities_in_vision.push(vision_event.entity);
                    let vision_direction = transform.rotation.mul_vec3(Vec3::Y);
                    let craber_direction = vision_direction;
                    vision.nearest_craber_distance = min_distance;
                    vision.nearest_craber_direction = angle_direction_between_vectors(
                        craber_direction,
                        Vec3::new(closest_point.x, closest_point.y, 0.),
                    );
                    vision.see_craber = true;
                }
            }
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
    mut lose_energy_events: MessageWriter<LoseEnergyEvent>,
) {
    let dt = time.delta_secs();
    for (entity, mut brain, _craber, mut accumulator, children) in query.iter_mut() {
        let modify_output = brain.get_modify_brain_interval().clamp(0.0, 1.0);
        let effective_rate =
            BRAIN_TICK_MIN_RATE + modify_output * (BRAIN_TICK_MAX_RATE - BRAIN_TICK_MIN_RATE);

        accumulator.0 += effective_rate * dt;
        if accumulator.0 < 1.0 {
            continue;
        }
        accumulator.0 -= 1.0;

        let interval_normalized = (BRAIN_TICK_MIN_RATE / effective_rate).clamp(0.0, 1.0);
        brain.update_input(NeuronType::BrainInterval, interval_normalized);

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

        lose_energy_events.write(LoseEnergyEvent {
            entity,
            energy_lost: BRAIN_TICK_ENERGY_COST,
        });
    }
}

pub fn craber_lose_energy(
    mut lose_energy_events: MessageReader<LoseEnergyEvent>,
    mut query: Query<&mut Energy>,
) {
    for lose_energy_event in lose_energy_events.read() {
        if let Ok(mut energy) = query.get_mut(lose_energy_event.entity) {
            energy.energy -= lose_energy_event.energy_lost;
        }
    }
}

pub fn craber_lose_health(
    mut lose_health_events: MessageReader<LoseHealthEvent>,
    mut query: Query<&mut Health>,
) {
    for lose_health_event in lose_health_events.read() {
        if let Ok(mut health) = query.get_mut(lose_health_event.entity) {
            health.health -= lose_health_event.health_lost;
        }
    }
}

pub fn craber_attack_lose_health_add_energy(
    mut craber_attack_events: MessageReader<CraberAttackEvent>,
    mut attacked_query: Query<&mut Health>,
    mut attacker_query: Query<&mut Energy>,
) {
    for craber_attack_event in craber_attack_events.read() {
        if let Ok(mut health) = attacked_query.get_mut(craber_attack_event.attacked_craber_entity) {
            if health.health <= 0. {
                continue;
            }
            if health.health - craber_attack_event.attack_damage < 0. {
                let actual_damage = health.health;
                let energy_modifier = actual_damage / craber_attack_event.attack_damage;
                if let Ok(mut energy) =
                    attacker_query.get_mut(craber_attack_event.attacking_craber_entity)
                {
                    energy.energy += craber_attack_event.energy_to_gain * energy_modifier;
                }
                health.health = 0.0;
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
