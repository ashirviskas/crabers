use avian2d::prelude::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::{Timer, TimerMode},
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use egui_plot::{Line, Plot, PlotPoints};
use std::collections::VecDeque;

mod craber;
use craber::*;

mod brain;
use brain::*;

mod food;
use food::*;

mod neural_viz;

mod common;
use bevy_pancam::{PanCam, PanCamPlugin};
use common::*;

const SOME_COLLISION_THRESHOLD: f32 = 20.0;
const FOOD_SPAWN_RATE: f32 = 0.0004;
const CRABER_SPAWN_RATE: f32 = 0.1;

pub const MAX_FOOD_COUNT: usize = 10000;

const WALL_THICKNESS: f32 = 60.0;
const GRAVITY: f32 = 0.0;

const VISION_UPDATE_RATE: f32 = 0.01;

#[derive(Resource, Default)]
struct DebugVisionEnabled(bool);

#[derive(Component)]
struct SelectionRing;

#[cfg(target_arch = "wasm32")]
const ENABLE_LEFT_MOUSE_BUTTON_DRAG: bool = true;

#[cfg(not(target_arch = "wasm32"))]
const ENABLE_LEFT_MOUSE_BUTTON_DRAG: bool = false;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#game-canvas".to_string()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanCamPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(SelectedEntity::default())
        .insert_resource(DebugInfo::default())
        .insert_resource(SimulationStats::new(600))
        .insert_resource(DebugVisionEnabled::default())
        .add_message::<DespawnEvent>()
        .add_message::<SpawnEvent>()
        .add_message::<ReproduceEvent>()
        .add_message::<SexualReproduceRequestEvent>()
        .add_message::<SexualReproduceEvent>()
        .add_message::<VisionEvent>()
        .add_message::<LoseEnergyEvent>()
        .add_message::<LoseHealthEvent>()
        .add_message::<CraberCollisionEvent>()
        .add_message::<CraberAttackEvent>()
        .add_message::<CraberDespawnEvent>()
        .add_message::<FoodSpawnEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, entity_selection)
        .add_systems(Update, highlight_selected_entity.after(entity_selection))
        .add_systems(Update, update_selected_entity_info)
        .add_systems(Update, update_debug_info)
        .add_systems(EguiPrimaryContextPass, egui_ui)
        .add_systems(Update, record_simulation_stats.after(update_debug_info))
        .add_systems(EguiPrimaryContextPass, egui_charts.after(egui_ui))
        .add_systems(Update, food_spawner)
        .add_systems(Update, craber_spawner)
        .add_systems(Update, do_collision)
        .add_systems(Update, do_craber_collision)
        .add_systems(Update, vision_update)
        .add_systems(Update, apply_rotation)
        .add_systems(Update, apply_water_drag)
        .add_systems(Update, apply_kick)
        .add_systems(Update, brain_update)
        .add_systems(Update, craber_lose_energy)
        .add_systems(Update, craber_lose_health)
        .add_systems(Update, craber_attack_lose_health_add_energy)
        .add_systems(Update, do_despawning)
        // Ordered chains: reproduction pipeline and death pipeline
        .add_systems(Update, energy_consumption.before(match_sexual_partners))
        .add_systems(Update, match_sexual_partners.before(craber_sexual_reproduce))
        .add_systems(Update, craber_sexual_reproduce.before(craber_reproduce))
        .add_systems(Update, craber_reproduce.before(spawn_craber))
        .add_systems(Update, spawn_craber)
        .add_systems(Update, despawn_dead_crabers.before(craber_despawner))
        .add_systems(Update, craber_despawner)
        .add_systems(Update, spawn_food)
        .add_systems(Update, toggle_debug_vision)
        .add_systems(Update, draw_vision_debug)
        .add_systems(Update, debug_check_finite)
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
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber, Layer::Vision]))
        .insert(EntityType::Wall)
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
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber, Layer::Vision]))
        .insert(EntityType::Wall)
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
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber, Layer::Vision]))
        .insert(EntityType::Wall)
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
        .insert(CollisionLayers::new([Layer::Wall], [Layer::Craber, Layer::Vision]))
        .insert(EntityType::Wall)
        .insert(RigidBody::Static)
        .insert(Collider::rectangle(WALL_THICKNESS / 2.0, WORLD_SIZE * 2.0));
}

fn update_debug_info(
    mut debug_info: ResMut<DebugInfo>,
    craber_query: Query<&Craber>,
    food_query: Query<&Food>,
    diagnostics: Res<DiagnosticsStore>,
) {
    debug_info.craber_count = craber_query.iter().count();
    debug_info.food_count = food_query.iter().count();
    debug_info.entity_count = debug_info.craber_count + debug_info.food_count;
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|x| x.average())
    {
        debug_info.fps = fps;
    }
}

fn record_simulation_stats(
    time: Res<Time>,
    debug_info: Res<DebugInfo>,
    mut stats: ResMut<SimulationStats>,
    craber_query: Query<(&CraberAge, &Generation, &Energy, &Health), With<Craber>>,
    brain_query: Query<&Brain, With<Craber>>,
) {
    stats.sample_timer.tick(time.delta());
    if stats.sample_timer.just_finished() {
        let elapsed = time.elapsed_secs_f64();
        let cap = stats.capacity;

        push_sample(&mut stats.craber_history, cap, elapsed, debug_info.craber_count as f64);
        push_sample(&mut stats.food_history, cap, elapsed, debug_info.food_count as f64);

        // Compute craber metrics
        let mut total_age = 0.0_f64;
        let mut max_age = 0.0_f64;
        let mut total_gen = 0.0_f64;
        let mut max_gen = 0.0_f64;
        let mut total_energy = 0.0_f64;
        let mut total_health = 0.0_f64;
        let mut count = 0_u32;

        for (age, generation, energy, health) in craber_query.iter() {
            let a = age.0 as f64;
            let g = generation.generation_id as f64;
            total_age += a;
            if a > max_age { max_age = a; }
            total_gen += g;
            if g > max_gen { max_gen = g; }
            total_energy += energy.energy as f64;
            total_health += health.health as f64;
            count += 1;
        }

        let divisor = count.max(1) as f64;
        push_sample(&mut stats.avg_age_history, cap, elapsed, total_age / divisor);
        push_sample(&mut stats.max_age_history, cap, elapsed, max_age);
        push_sample(&mut stats.avg_generation_history, cap, elapsed, total_gen / divisor);
        push_sample(&mut stats.max_generation_history, cap, elapsed, max_gen);
        push_sample(&mut stats.avg_energy_history, cap, elapsed, total_energy / divisor);
        push_sample(&mut stats.avg_health_history, cap, elapsed, total_health / divisor);

        // Brain complexity
        let mut total_hidden = 0_u64;
        let mut total_connections = 0_u64;
        let mut brain_count = 0_u32;
        for brain in brain_query.iter() {
            total_hidden += brain.hidden_layers.len() as u64;
            total_connections += brain.connections.len() as u64;
            brain_count += 1;
        }
        let brain_divisor = brain_count.max(1) as f64;
        push_sample(&mut stats.avg_hidden_neurons_history, cap, elapsed, total_hidden as f64 / brain_divisor);
        push_sample(&mut stats.avg_connections_history, cap, elapsed, total_connections as f64 / brain_divisor);

        // Birth/death rates (counts since last sample)
        let births = stats.birth_counter as f64;
        let deaths = stats.death_counter as f64;
        push_sample(&mut stats.birth_rate_history, cap, elapsed, births);
        push_sample(&mut stats.death_rate_history, cap, elapsed, deaths);
        let sexual_births = stats.sexual_birth_counter as f64;
        let asexual_births = stats.asexual_birth_counter as f64;
        push_sample(&mut stats.sexual_birth_rate_history, cap, elapsed, sexual_births);
        push_sample(&mut stats.asexual_birth_rate_history, cap, elapsed, asexual_births);
        stats.birth_counter = 0;
        stats.death_counter = 0;
        stats.sexual_birth_counter = 0;
        stats.asexual_birth_counter = 0;
    }
}

fn plot_lines(ui: &mut egui::Ui, plot_id: &str, lines: &[(&str, &VecDeque<[f64; 2]>)]) {
    Plot::new(plot_id)
        .view_aspect(2.0)
        .label_formatter(|name, value| {
            if name.is_empty() {
                format!("t = {:.1}s\n{:.1}", value.x, value.y)
            } else {
                format!("{name}\nt = {:.1}s\n{:.1}", value.x, value.y)
            }
        })
        .show(ui, |plot_ui| {
            for (name, data) in lines {
                let points: PlotPoints = PlotPoints::new(data.iter().copied().collect());
                plot_ui.line(Line::new(*name, points));
            }
        });
}

fn egui_charts(
    mut contexts: EguiContexts,
    stats: Res<SimulationStats>,
    mut initialized: Local<bool>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if !*initialized {
        *initialized = true;
        return;
    }

    let transparent_frame = egui::Frame::new()
        .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 25, 200))
        .corner_radius(6.0)
        .inner_margin(10.0);

    // Population window
    egui::Window::new("Population")
        .default_pos([10.0, 350.0])
        .default_size([300.0, 200.0])
        .resizable(true)
        .collapsible(true)
        .default_open(true)
        .frame(transparent_frame)
        .show(ctx, |ui| {
            ui.label("Crabers");
            plot_lines(ui, "craber_population", &[("Crabers", &stats.craber_history)]);
            ui.separator();
            ui.label("Food");
            plot_lines(ui, "food_population", &[("Food", &stats.food_history)]);
        });

    // Vitals window
    egui::Window::new("Vitals")
        .default_pos([10.0, 560.0])
        .default_size([300.0, 200.0])
        .resizable(true)
        .collapsible(true)
        .default_open(false)
        .frame(transparent_frame)
        .show(ctx, |ui| {
            plot_lines(ui, "vitals", &[
                ("Avg Energy", &stats.avg_energy_history),
                ("Avg Health", &stats.avg_health_history),
            ]);
        });

    // Demographics window
    egui::Window::new("Demographics")
        .default_pos([10.0, 585.0])
        .default_size([300.0, 200.0])
        .resizable(true)
        .collapsible(true)
        .default_open(false)
        .frame(transparent_frame)
        .show(ctx, |ui| {
            ui.label("Age");
            plot_lines(ui, "age", &[
                ("Avg Age", &stats.avg_age_history),
                ("Max Age", &stats.max_age_history),
            ]);
            ui.separator();
            ui.label("Generation");
            plot_lines(ui, "generation", &[
                ("Avg Generation", &stats.avg_generation_history),
                ("Max Generation", &stats.max_generation_history),
            ]);
        });

    // Complexity & Rates window
    egui::Window::new("Complexity & Rates")
        .default_pos([10.0, 610.0])
        .default_size([300.0, 200.0])
        .resizable(true)
        .collapsible(true)
        .default_open(false)
        .frame(transparent_frame)
        .show(ctx, |ui| {
            ui.label("Brain Complexity");
            plot_lines(ui, "brain_complexity", &[
                ("Avg Hidden Neurons", &stats.avg_hidden_neurons_history),
                ("Avg Connections", &stats.avg_connections_history),
            ]);
            ui.separator();
            ui.label("Birth / Death Rate");
            plot_lines(ui, "birth_death_rate", &[
                ("Births", &stats.birth_rate_history),
                ("Deaths", &stats.death_rate_history),
            ]);
            ui.separator();
            ui.label("Sexual vs Asexual Births");
            plot_lines(ui, "birth_type_rate", &[
                ("Sexual", &stats.sexual_birth_rate_history),
                ("Asexual", &stats.asexual_birth_rate_history),
            ]);
        });
}

fn egui_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedEntity>,
    debug_info: Res<DebugInfo>,
    brain_query: Query<&Brain>,
    mut initialized: Local<bool>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Skip until egui has run its first pass and initialized fonts
    if !*initialized {
        *initialized = true;
        return;
    }

    let transparent_frame = egui::Frame::new()
        .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 25, 200))
        .corner_radius(6.0)
        .inner_margin(10.0);

    // Left window: inspector + debug info
    egui::Window::new("Inspector")
        .default_pos([10.0, 10.0])
        .resizable(false)
        .collapsible(true)
        .frame(transparent_frame)
        .show(ctx, |ui| {
            if let Some(_entity) = selected.entity {
                ui.label(format!("Health: {:.2}", selected.health));
                ui.label(format!("Energy: {:.2}", selected.energy));
                ui.label(format!("Generation: {}", selected.generation));
                ui.label(format!("Age: {:.1}s", selected.age));
                ui.label(format!("Children: {}", selected.children_count));
                ui.label(format!(
                    "Nearest food angle: {:.2}",
                    selected.nearest_food_anlge
                ));
            } else {
                ui.label("No craber selected");
            }

            ui.separator();
            ui.label(format!("Crabers: {}", debug_info.craber_count));
            ui.label(format!("Food: {}", debug_info.food_count));
            ui.label(format!("Total: {}", debug_info.entity_count));
            ui.label(format!("FPS: {:.1}", debug_info.fps));
            ui.separator();
            ui.label("Press P for vision debug");
        });

    // Right panel: neural network (only when a craber is selected)
    if let Some(entity) = selected.entity {
        if let Ok(brain) = brain_query.get(entity) {
            egui::SidePanel::right("neural_network")
                .default_width(440.0)
                .resizable(false)
                .frame(transparent_frame)
                .show(ctx, |ui| {
                    ui.heading("Neural Network");
                    ui.separator();
                    neural_viz::draw_neural_network(ui, brain);
                });
        }
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

fn highlight_selected_entity(
    mut commands: Commands,
    selected: Res<SelectedEntity>,
    mut prev_selected: Local<Option<Entity>>,
    ring_query: Query<Entity, With<SelectionRing>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if selected.entity == *prev_selected {
        return;
    }
    // Despawn old ring(s)
    for ring in ring_query.iter() {
        commands.entity(ring).despawn();
    }
    // Spawn new ring as child of selected entity
    if let Some(entity) = selected.entity {
        let ring = commands
            .spawn((
                Mesh2d(meshes.add(Circle::new(CRABER_SIZE))),
                MeshMaterial2d(materials.add(ColorMaterial::from(
                    Color::srgba(1.0, 1.0, 0.0, 0.35),
                ))),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
                SelectionRing,
            ))
            .id();
        commands.entity(entity).add_child(ring);
    }
    *prev_selected = selected.entity;
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
    craber_query: Query<(&Transform, &Children, &Generation, &Brain, &Health, &Energy, &CraberAge, &ChildrenCount)>,
    vision_query: Query<(&Vision, &Transform, Entity, &ChildOf)>,
    food_query: Query<&Food>,
) {
    if let Some(entity) = selected.entity {
        // Check if the selected entity is a Craber
        if let Ok((craber_transform, craber_children, craber_generation, brain, health, energy, age, children_count)) =
            craber_query.get(entity)
        {
            selected.health = health.health;
            selected.energy = energy.energy;
            selected.generation = craber_generation.generation_id;
            selected.age = age.0;
            selected.children_count = children_count.0;
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
                    (EntityType::Wall, EntityType::Vision) => {
                        vision_events.write(VisionEvent {
                            vision_entity: entity2,
                            entity: entity1,
                            event_type: VisionEventType::Wall,
                            entity_id: 2,
                            manifolds: manifolds.clone(),
                        });
                    }
                    (EntityType::Vision, EntityType::Wall) => {
                        vision_events.write(VisionEvent {
                            vision_entity: entity1,
                            entity: entity2,
                            event_type: VisionEventType::Wall,
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

/// System 1: Accumulator-gated angular impulse rotation
fn apply_rotation(
    mut query: Query<(Forces, &mut RotationAccumulator, &Brain), With<Craber>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut forces, mut accumulator, brain) in query.iter_mut() {
        let rotation_direction = brain.get_rotation(); // [-1, 1] direction
        let rotation_rate = brain.get_rotate_rate().max(0.0); // [0, ∞) rate
        let effective_rate = 1.0 - (-rotation_rate * ROTATION_RATE_STEEPNESS).exp();

        accumulator.0 += effective_rate * dt;
        if accumulator.0 < ROTATION_THRESHOLD {
            continue;
        }
        accumulator.0 -= ROTATION_THRESHOLD;

        let angular_impulse = rotation_direction * effective_rate * MAX_ANGULAR_IMPULSE;
        if angular_impulse.is_finite() {
            forces.apply_angular_impulse(angular_impulse);
        } else {
            warn!("apply_rotation: NaN angular_impulse! rot_dir={} eff_rate={} brain_rotation={} brain_rotate_rate={}",
                rotation_direction, effective_rate, brain.get_rotation(), brain.get_rotate_rate());
        }
    }
}

/// System 2: Water drag via direct velocity damping — guarantees convergence, no overflow
fn apply_water_drag(
    mut query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<Craber>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut lin_vel, mut ang_vel) in query.iter_mut() {
        // Linear drag: damp velocity directly each frame
        let speed = lin_vel.0.length();
        if speed > 0.0 && speed.is_finite() {
            // Quadratic feel: faster speeds get damped more aggressively
            let damp_factor = (-LINEAR_DRAG_COEFFICIENT * speed * dt).exp();
            lin_vel.0 *= damp_factor;
        }

        // Angular drag: damp angular velocity directly each frame
        let w = ang_vel.0;
        if w.abs() > 0.0 && w.is_finite() {
            let damp_factor = (-ANGULAR_DRAG_COEFFICIENT * w.abs() * dt).exp();
            ang_vel.0 *= damp_factor;
        }
    }
}

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
        accumulator.0 -= KICK_THRESHOLD;

        let facing_dir = (transform.rotation * Vec3::NEG_Y).truncate();
        let thrust = facing_dir * effective_strength * MAX_IMPULSE;
        if !thrust.x.is_finite() || !thrust.y.is_finite() {
            warn!("apply_kick: NaN thrust! entity={:?} facing_dir={:?} eff_strength={} rot={:?} kick_strength={} kick_rate={}",
                entity, facing_dir, effective_strength, transform.rotation,
                brain.get_kick_strength(), brain.get_kick_rate());
            continue;
        }
        forces.apply_linear_impulse(thrust);

        let energy_cost = effective_strength.powf(1.5) * KICK_ENERGY_MODIFIER;
        lose_energy_events.write(LoseEnergyEvent {
            entity,
            energy_lost: energy_cost,
        });
    }
}

pub fn vision_update(
    mut query: Query<(&mut Vision, &GlobalTransform, &Collider, &ChildOf)>,
    mut vision_events: MessageReader<VisionEvent>,
    brain_query: Query<&Brain>,
) {
    for vision_event in vision_events.read() {
        match vision_event.event_type {
            VisionEventType::Food => {
                if let Ok((mut vision, global_transform, _collider, _parent)) =
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
                    if !closest_point.x.is_finite() || !closest_point.y.is_finite() || !min_distance.is_finite() {
                        continue;
                    }
                    if min_distance > 0.0 {
                        closest_point = closest_point / min_distance;
                    }
                    vision.entities_in_vision.push(vision_event.entity);
                    let vision_direction = global_transform.rotation().mul_vec3(Vec3::Y);
                    let craber_direction = vision_direction;
                    vision.nearest_food_distance = min_distance;
                    vision.nearest_food_direction = -angle_direction_between_vectors(
                        craber_direction,
                        Vec3::new(closest_point.x, closest_point.y, 0.),
                    );
                    vision.see_food = true;
                }
            }
            VisionEventType::Craber => {
                if let Ok((mut vision, global_transform, _collider, _parent)) =
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
                    if !closest_point.x.is_finite() || !closest_point.y.is_finite() || !min_distance.is_finite() {
                        continue;
                    }
                    if min_distance > 0.0 {
                        closest_point = closest_point / min_distance;
                    }
                    vision.entities_in_vision.push(vision_event.entity);
                    let vision_direction = global_transform.rotation().mul_vec3(Vec3::Y);
                    let craber_direction = vision_direction;
                    vision.nearest_craber_distance = min_distance;
                    vision.nearest_craber_direction = -angle_direction_between_vectors(
                        craber_direction,
                        Vec3::new(closest_point.x, closest_point.y, 0.),
                    );
                    vision.see_craber = true;

                    // Compute genetic closeness between parent craber and seen craber
                    let parent_entity = _parent.parent();
                    if let (Ok(parent_brain), Ok(seen_brain)) =
                        (brain_query.get(parent_entity), brain_query.get(vision_event.entity))
                    {
                        vision.nearest_craber_genetic_closeness =
                            parent_brain.genetic_closeness(seen_brain);
                    }
                }
            }
            VisionEventType::Wall => {
                if let Ok((mut vision, global_transform, _collider, _parent)) =
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
                    if !closest_point.x.is_finite() || !closest_point.y.is_finite() || !min_distance.is_finite() {
                        continue;
                    }
                    if min_distance > 0.0 {
                        closest_point = closest_point / min_distance;
                    }
                    let vision_direction = global_transform.rotation().mul_vec3(Vec3::Y);
                    let craber_direction = vision_direction;
                    vision.nearest_wall_distance = min_distance;
                    vision.nearest_wall_direction = -angle_direction_between_vectors(
                        craber_direction,
                        Vec3::new(closest_point.x, closest_point.y, 0.),
                    );
                    vision.see_wall = true;
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
        &mut LastReproducedValue,
        &Health,
        &Energy,
        &mut CraberAge,
    )>,
    mut vision_query: Query<(&mut Vision, &Transform)>,
    time: Res<Time>,
    mut lose_energy_events: MessageWriter<LoseEnergyEvent>,
) {
    let dt = time.delta_secs();
    for (entity, mut brain, _craber, mut accumulator, children, mut last_reproduced, health, energy, mut age) in query.iter_mut() {
        let modify_output = brain.get_modify_brain_interval().clamp(0.0, 1.0);
        let effective_rate =
            BRAIN_TICK_MIN_RATE + modify_output * (BRAIN_TICK_MAX_RATE - BRAIN_TICK_MIN_RATE);

        accumulator.0 += effective_rate * dt;
        age.0 += dt; // Track real elapsed time, independent of brain tick rate
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
            brain.update_input(
                NeuronType::NearestCraberGeneticCloseness,
                vision.nearest_craber_genetic_closeness,
            );
            vision.craber_seen_timer = VISION_UPDATE_RATE;
            vision.no_see_craber();
        } else {
            vision.craber_seen_timer -= dt;
            if vision.craber_seen_timer <= 0.0 {
                brain.update_input(NeuronType::NearestCraberAngle, 0.0);
                brain.update_input(NeuronType::NearestCraberDistance, 0.0);
                brain.update_input(NeuronType::NearestCraberGeneticCloseness, 0.0);
            }
        }
        if vision.see_wall {
            brain.update_input(
                NeuronType::NearestWallAngle,
                vision.nearest_wall_direction,
            );
            brain.update_input(
                NeuronType::NearestWallDistance,
                vision.nearest_wall_distance,
            );
            vision.wall_seen_timer = VISION_UPDATE_RATE;
            vision.no_see_wall();
        } else {
            vision.wall_seen_timer -= dt;
            if vision.wall_seen_timer <= 0.0 {
                brain.update_input(NeuronType::NearestWallAngle, 0.0);
                brain.update_input(NeuronType::NearestWallDistance, 0.0);
            }
        }
        // Feed health/energy/age inputs (normalized 0-1)
        brain.update_input(NeuronType::CraberHealth, health.health / health.max_health);
        brain.update_input(NeuronType::CraberEnergy, energy.energy / energy.max_energy);
        brain.update_input(NeuronType::CraberAge, 1.0 - (-0.01 * age.0).exp());

        // Decay and feed LastReproduced input
        last_reproduced.0 *= 1.0 - 0.5 * dt;
        brain.update_input(NeuronType::LastReproduced, last_reproduced.0);

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
                let energy_modifier = if craber_attack_event.attack_damage > 0.0 {
                    actual_damage / craber_attack_event.attack_damage
                } else {
                    0.0
                };
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

fn toggle_debug_vision(keyboard: Res<ButtonInput<KeyCode>>, mut debug: ResMut<DebugVisionEnabled>) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        debug.0 = !debug.0;
    }
}

fn debug_check_finite(
    craber_query: Query<(Entity, &Transform, &LinearVelocity, &AngularVelocity, &Brain, &Children), With<Craber>>,
    vision_query: Query<&Vision>,
    food_query: Query<(Entity, &Transform), With<Food>>,
) {
    for (entity, transform, lin_vel, ang_vel, brain, children) in craber_query.iter() {
        let pos = transform.translation;
        let rot = transform.rotation;
        let has_bad_pos = !pos.x.is_finite() || !pos.y.is_finite();
        let has_bad_vel = !lin_vel.x.is_finite() || !lin_vel.y.is_finite();
        let has_bad_ang = !ang_vel.0.is_finite();
        let has_bad_rot = !rot.x.is_finite() || !rot.y.is_finite() || !rot.z.is_finite() || !rot.w.is_finite();

        if has_bad_pos || has_bad_vel || has_bad_ang || has_bad_rot {
            // Dump brain state
            let mut nan_inputs = Vec::new();
            for (i, n) in brain.inputs.iter().enumerate() {
                if !n.value.is_finite() {
                    nan_inputs.push(format!("input[{}]({:?})={}", i, n.neuron_type, n.value));
                }
            }
            let mut nan_hidden = Vec::new();
            for (i, n) in brain.hidden_layers.iter().enumerate() {
                if !n.value.is_finite() {
                    nan_hidden.push(format!("hidden[{}]={}", i, n.value));
                }
            }
            let mut nan_outputs = Vec::new();
            for (i, n) in brain.outputs.iter().enumerate() {
                if !n.value.is_finite() {
                    nan_outputs.push(format!("output[{}]({:?})={}", i, n.neuron_type, n.value));
                }
            }
            let mut nan_weights = Vec::new();
            for (i, c) in brain.connections.iter().enumerate() {
                if !c.weight.is_finite() || !c.bias.is_finite() {
                    nan_weights.push(format!("conn[{}] {}->{}  w={} b={}", i, c.from_id, c.to_id, c.weight, c.bias));
                }
            }

            // Dump vision state
            let mut vision_info = String::from("no vision child");
            for child in children.iter() {
                if let Ok(vision) = vision_query.get(child) {
                    vision_info = format!(
                        "food_dir={} food_dist={} craber_dir={} craber_dist={} wall_dir={} wall_dist={} see_food={} see_craber={} see_wall={}",
                        vision.nearest_food_direction, vision.nearest_food_distance,
                        vision.nearest_craber_direction, vision.nearest_craber_distance,
                        vision.nearest_wall_direction, vision.nearest_wall_distance,
                        vision.see_food, vision.see_craber, vision.see_wall
                    );
                }
            }

            panic!(
                "NON-FINITE STATE on Craber {:?}:\n  pos={:?}\n  vel={:?}\n  ang_vel={:?}\n  rot={:?}\n  \
                 nan_inputs={:?}\n  nan_hidden={:?}\n  nan_outputs={:?}\n  nan_weights={:?}\n  vision: {}",
                entity, pos, lin_vel, ang_vel, rot,
                nan_inputs, nan_hidden, nan_outputs, nan_weights, vision_info
            );
        }
    }
    for (entity, transform) in food_query.iter() {
        let pos = transform.translation;
        if !pos.x.is_finite() || !pos.y.is_finite() {
            panic!(
                "NON-FINITE POSITION on Food {:?}: pos={:?}",
                entity, pos
            );
        }
    }
}

fn draw_vision_debug(
    debug: Res<DebugVisionEnabled>,
    mut gizmos: Gizmos,
    craber_query: Query<(&Transform, &Children, &LinearVelocity), With<Craber>>,
    vision_query: Query<&Vision>,
) {
    if !debug.0 {
        return;
    }
    for (transform, children, linear_vel) in craber_query.iter() {
        let pos = transform.translation.truncate();
        let facing = (transform.rotation * Vec3::Y).truncate().normalize();

        // White: facing direction (sprite faces NEG_Y)
        gizmos.line_2d(pos, pos - facing * 50.0, Color::WHITE);

        // Cyan: velocity indicator
        let vel = linear_vel.0;
        if vel.length() > 0.1 {
            gizmos.line_2d(pos, pos + vel * 0.3, Color::srgb(0.0, 1.0, 1.0));
        }

        for child in children.iter() {
            if let Ok(vision) = vision_query.get(child) {
                if vision.see_food {
                    let angle = -vision.nearest_food_direction;
                    let food_dir = Vec2::new(
                        facing.x * angle.cos() - facing.y * angle.sin(),
                        facing.x * angle.sin() + facing.y * angle.cos(),
                    );
                    gizmos.line_2d(pos, pos + food_dir * 40.0, Color::srgb(0.0, 1.0, 0.0));
                }
                if vision.see_craber {
                    let angle = -vision.nearest_craber_direction;
                    let craber_dir = Vec2::new(
                        facing.x * angle.cos() - facing.y * angle.sin(),
                        facing.x * angle.sin() + facing.y * angle.cos(),
                    );
                    gizmos.line_2d(pos, pos + craber_dir * 40.0, Color::srgb(1.0, 0.0, 0.0));
                }
                if vision.see_wall {
                    let angle = -vision.nearest_wall_direction;
                    let wall_dir = Vec2::new(
                        facing.x * angle.cos() - facing.y * angle.sin(),
                        facing.x * angle.sin() + facing.y * angle.cos(),
                    );
                    gizmos.line_2d(pos, pos + wall_dir * 40.0, Color::srgb(1.0, 1.0, 0.0));
                }
            }
        }
    }
}
