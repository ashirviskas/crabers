use bevy::prelude::*;

use crate::brain::{Brain, NeuronType};
use crate::common::SelectedEntity;

// UI Scale factor - adjust this to scale the entire panel
const UI_SCALE: f32 = 1.5;

// Panel dimensions (base values, will be scaled)
const PANEL_WIDTH: f32 = 400.0 * UI_SCALE;
const PANEL_HEIGHT: f32 = 450.0 * UI_SCALE;

// Neuron sizing
const NEURON_SIZE: f32 = 24.0 * UI_SCALE;
const NEURON_RADIUS: f32 = NEURON_SIZE / 2.0;

// Layout positioning
const COLUMN_X: [f32; 3] = [50.0 * UI_SCALE, 175.0 * UI_SCALE, 300.0 * UI_SCALE];
const ROW_START_Y: f32 = 60.0 * UI_SCALE;
const ROW_SPACING: f32 = 65.0 * UI_SCALE;

// Connection line settings
const LINE_THICKNESS: f32 = 2.0 * UI_SCALE;

// Text sizes
const LABEL_FONT_SIZE: f32 = 10.0 * UI_SCALE;
const VALUE_FONT_SIZE: f32 = 12.0 * UI_SCALE;
const HEADER_FONT_SIZE: f32 = 14.0 * UI_SCALE;

// Colors
const PANEL_BG_COLOR: Color = Color::srgba(0.1, 0.1, 0.15, 0.95);
const NEURON_BG_COLOR: Color = Color::srgb(0.2, 0.2, 0.25);
const HEADER_TEXT_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 1.0);

/// Neuron layer enum for position calculation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NeuronLayer {
    Input,
    Hidden,
    Output,
}

/// Resource tracking the current neural network layout state
#[derive(Resource, Default)]
pub struct NeuralNetworkLayout {
    pub current_entity: Option<Entity>,
    pub needs_rebuild: bool,
}

/// Marker for the main neural network panel
#[derive(Component)]
pub struct NeuralNetworkPanel;

/// Container that holds all neurons and connection lines
#[derive(Component)]
pub struct NetworkContainer;

/// Marker for a neuron node (the circular element)
#[derive(Component)]
pub struct NeuronNode {
    pub neuron_id: usize,
    pub layer: NeuronLayer,
}

/// The colored indicator circle inside the neuron
#[derive(Component)]
pub struct NeuronIndicator {
    pub neuron_id: usize,
}

/// Text displaying the neuron's current value
#[derive(Component)]
pub struct NeuronValueText {
    pub neuron_id: usize,
}

/// A visual connection line between neurons
#[derive(Component)]
pub struct ConnectionLine;

/// Calculate the position of a neuron based on its layer and index
fn neuron_position(layer: NeuronLayer, index: usize) -> (f32, f32) {
    let col = match layer {
        NeuronLayer::Input => 0,
        NeuronLayer::Hidden => 1,
        NeuronLayer::Output => 2,
    };
    (COLUMN_X[col], ROW_START_Y + index as f32 * ROW_SPACING)
}

/// Convert a weight value to a color (green for positive, red for negative)
fn weight_to_color(weight: f32, enabled: bool) -> Color {
    if !enabled {
        return Color::srgba(0.3, 0.3, 0.3, 0.3);
    }

    let intensity = weight.abs().min(1.0);
    if weight >= 0.0 {
        Color::srgba(0.2, 0.5 + intensity * 0.5, 0.2, 0.6 + intensity * 0.4)
    } else {
        Color::srgba(0.5 + intensity * 0.5, 0.2, 0.2, 0.6 + intensity * 0.4)
    }
}

/// Convert a neuron value to a display color
fn value_to_color(value: f32) -> Color {
    let clamped = value.clamp(-1.0, 1.0);
    if clamped >= 0.0 {
        Color::srgb(0.2, 0.4 + clamped * 0.6, 0.2)
    } else {
        Color::srgb(0.4 + (-clamped) * 0.6, 0.2, 0.2)
    }
}

/// Get a short label for a neuron type
fn neuron_label(neuron_type: NeuronType) -> &'static str {
    match neuron_type {
        NeuronType::AlwaysOn => "ON",
        NeuronType::CraberHealth => "HP",
        NeuronType::CraberSpeed => "SPD",
        NeuronType::NearestFoodAngle => "F.ANG",
        NeuronType::NearestFoodDistance => "F.DST",
        NeuronType::NearestCraberAngle => "C.ANG",
        NeuronType::NearestCraberDistance => "C.DST",
        NeuronType::NearestWallAngle => "W.ANG",
        NeuronType::NearestWallDistance => "W.DST",
        NeuronType::BrainInterval => "INT",
        NeuronType::Hidden => "H",
        NeuronType::KickStrength => "K.STR",
        NeuronType::KickRate => "K.RT",
        NeuronType::AlignVelocity => "ALIGN",
        NeuronType::Rotate => "ROT",
        NeuronType::ModifyBrainInterval => "M.INT",
        NeuronType::WantToMate => "MATE",
        NeuronType::WantToAttack => "ATK",
        NeuronType::WantToDefend => "DEF",
    }
}

/// Startup system: Create the panel skeleton
pub fn setup_neural_panel(mut commands: Commands) {
    // Root panel - positioned in top-right corner
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(PANEL_WIDTH),
                height: Val::Px(PANEL_HEIGHT),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0 * UI_SCALE)),
                display: Display::None, // Hidden by default
                ..default()
            },
            background_color: PANEL_BG_COLOR.into(),
            ..default()
        })
        .insert(NeuralNetworkPanel)
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Neural Network",
                    TextStyle {
                        font_size: 18.0 * UI_SCALE,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::bottom(Val::Px(10.0 * UI_SCALE)),
                    ..default()
                },
                ..default()
            });

            // Network container with relative positioning for absolute children
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(380.0 * UI_SCALE),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    ..default()
                })
                .insert(NetworkContainer)
                .with_children(|container| {
                    // Column headers
                    spawn_column_header(container, "INPUTS", COLUMN_X[0]);
                    spawn_column_header(container, "HIDDEN", COLUMN_X[1]);
                    spawn_column_header(container, "OUTPUTS", COLUMN_X[2]);
                });
        });
}

/// Spawn a column header label
fn spawn_column_header(parent: &mut ChildBuilder, label: &str, x: f32) {
    parent.spawn(TextBundle {
        text: Text::from_section(
            label,
            TextStyle {
                font_size: HEADER_FONT_SIZE,
                color: HEADER_TEXT_COLOR,
                ..default()
            },
        ),
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(x - 20.0 * UI_SCALE),
            top: Val::Px(10.0 * UI_SCALE),
            ..default()
        },
        ..default()
    });
}

/// Update system: Toggle panel visibility based on selection
pub fn update_nn_layout(
    selected: Res<SelectedEntity>,
    mut layout: ResMut<NeuralNetworkLayout>,
    mut panel_query: Query<&mut Style, With<NeuralNetworkPanel>>,
) {
    let new_entity = selected.entity;

    // Check if selection changed
    if layout.current_entity != new_entity {
        layout.current_entity = new_entity;
        layout.needs_rebuild = new_entity.is_some();
    }

    // Update panel visibility
    for mut style in panel_query.iter_mut() {
        style.display = if new_entity.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Update system: Rebuild neuron nodes when selection changes
pub fn spawn_neuron_nodes(
    mut commands: Commands,
    mut layout: ResMut<NeuralNetworkLayout>,
    selected: Res<SelectedEntity>,
    brain_query: Query<&Brain>,
    container_query: Query<Entity, With<NetworkContainer>>,
    existing_nodes: Query<Entity, Or<(With<NeuronNode>, With<ConnectionLine>)>>,
) {
    if !layout.needs_rebuild {
        return;
    }
    layout.needs_rebuild = false;

    let Some(selected_entity) = selected.entity else {
        return;
    };

    let Ok(brain) = brain_query.get(selected_entity) else {
        return;
    };

    let Ok(container_entity) = container_query.get_single() else {
        return;
    };

    // Despawn existing neurons and connection lines
    for entity in existing_nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Clone data needed in closure
    let connections = brain.connections.clone();
    let inputs = brain.inputs.clone();
    let hidden_layers = brain.hidden_layers.clone();
    let outputs = brain.outputs.clone();

    // Pre-compute counts for position calculation
    let input_count = inputs.len();
    let hidden_count = hidden_layers.len();
    let output_count = outputs.len();

    commands.entity(container_entity).with_children(|container| {
        // Helper to get position from neuron ID
        let get_pos = |id: usize| -> Option<(f32, f32)> {
            if id < 100 {
                if id < input_count {
                    Some(neuron_position(NeuronLayer::Input, id))
                } else {
                    None
                }
            } else if id < 200 {
                let idx = id - 100;
                if idx < hidden_count {
                    Some(neuron_position(NeuronLayer::Hidden, idx))
                } else {
                    None
                }
            } else {
                let idx = id - 200;
                if idx < output_count {
                    Some(neuron_position(NeuronLayer::Output, idx))
                } else {
                    None
                }
            }
        };

        // Spawn connection lines first (render behind neurons)
        for connection in &connections {
            let from_pos = get_pos(connection.from_id);
            let to_pos = get_pos(connection.to_id);

            if let (Some(from), Some(to)) = (from_pos, to_pos) {
                spawn_connection_line(
                    container,
                    from,
                    to,
                    connection.weight,
                    connection.enabled,
                );
            }
        }

        // Spawn input neurons
        for (idx, neuron) in inputs.iter().enumerate() {
            let pos = neuron_position(NeuronLayer::Input, idx);
            spawn_neuron(
                container,
                idx,
                NeuronLayer::Input,
                pos,
                neuron_label(neuron.neuron_type),
                neuron.value,
            );
        }

        // Spawn hidden neurons
        for (idx, neuron) in hidden_layers.iter().enumerate() {
            let pos = neuron_position(NeuronLayer::Hidden, idx);
            spawn_neuron(
                container,
                100 + idx,
                NeuronLayer::Hidden,
                pos,
                neuron_label(neuron.neuron_type),
                neuron.value,
            );
        }

        // Spawn output neurons
        for (idx, neuron) in outputs.iter().enumerate() {
            let pos = neuron_position(NeuronLayer::Output, idx);
            spawn_neuron(
                container,
                200 + idx,
                NeuronLayer::Output,
                pos,
                neuron_label(neuron.neuron_type),
                neuron.value,
            );
        }

        // Debug: Show connections with computed positions
        let mut debug_text = String::from("Connections:\n");
        for conn in &connections {
            let from_p = get_pos(conn.from_id);
            let to_p = get_pos(conn.to_id);
            debug_text.push_str(&format!(
                "{}→{}: ({:.0},{:.0})→({:.0},{:.0})\n",
                conn.from_id,
                conn.to_id,
                from_p.map(|p| p.0).unwrap_or(-1.0),
                from_p.map(|p| p.1).unwrap_or(-1.0),
                to_p.map(|p| p.0).unwrap_or(-1.0),
                to_p.map(|p| p.1).unwrap_or(-1.0),
            ));
        }

        container.spawn(TextBundle {
            text: Text::from_section(
                debug_text,
                TextStyle {
                    font_size: 10.0 * UI_SCALE,
                    color: Color::srgba(0.8, 0.8, 0.8, 0.9),
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            ..default()
        });
    });
}

/// Spawn a connection line between two neuron positions
fn spawn_connection_line(
    parent: &mut ChildBuilder,
    from: (f32, f32),
    to: (f32, f32),
    weight: f32,
    enabled: bool,
) {
    // Offset to center of neurons
    let from_x = from.0 + NEURON_RADIUS;
    let from_y = from.1 + NEURON_RADIUS;
    let to_x = to.0 + NEURON_RADIUS;
    let to_y = to.1 + NEURON_RADIUS;

    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let length = (dx * dx + dy * dy).sqrt();
    let angle = dy.atan2(dx);

    // Calculate midpoint - position the line centered so rotation works correctly
    let mid_x = (from_x + to_x) / 2.0;
    let mid_y = (from_y + to_y) / 2.0;

    // Position at midpoint minus half the dimensions (so center is at midpoint)
    let left = mid_x - length / 2.0;
    let top = mid_y - LINE_THICKNESS / 2.0;

    // Create a rotated line using transform
    parent
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(length),
                height: Val::Px(LINE_THICKNESS),
                ..default()
            },
            background_color: weight_to_color(weight, enabled).into(),
            transform: Transform::from_rotation(Quat::from_rotation_z(angle)),
            z_index: ZIndex::Local(-1), // Render behind neurons
            ..default()
        })
        .insert(ConnectionLine);
}

/// Spawn a neuron node with label and value text
fn spawn_neuron(
    parent: &mut ChildBuilder,
    neuron_id: usize,
    layer: NeuronLayer,
    pos: (f32, f32),
    label: &str,
    initial_value: f32,
) {
    // Outer container for neuron + label
    parent
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.0),
                top: Val::Px(pos.1),
                width: Val::Px(NEURON_SIZE),
                height: Val::Px(NEURON_SIZE + 30.0 * UI_SCALE), // Extra space for label + value
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(NeuronNode { neuron_id, layer })
        .with_children(|neuron_parent| {
            // Circular neuron background
            neuron_parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(NEURON_SIZE),
                        height: Val::Px(NEURON_SIZE),
                        border: UiRect::all(Val::Px(2.0 * UI_SCALE)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NEURON_BG_COLOR.into(),
                    border_color: Color::srgb(0.4, 0.4, 0.5).into(),
                    border_radius: BorderRadius::all(Val::Px(NEURON_RADIUS)),
                    ..default()
                })
                .with_children(|circle| {
                    // Inner indicator that shows the value color
                    let indicator_size = NEURON_SIZE - 8.0 * UI_SCALE;
                    circle
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(indicator_size),
                                height: Val::Px(indicator_size),
                                ..default()
                            },
                            background_color: value_to_color(initial_value).into(),
                            border_radius: BorderRadius::all(Val::Px(indicator_size / 2.0)),
                            ..default()
                        })
                        .insert(NeuronIndicator { neuron_id });
                });

            // Label text below neuron
            neuron_parent.spawn(TextBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font_size: LABEL_FONT_SIZE,
                        color: Color::srgb(0.8, 0.8, 0.8),
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::top(Val::Px(2.0 * UI_SCALE)),
                    ..default()
                },
                ..default()
            });

            // Value text below label
            neuron_parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        format!("{:.2}", initial_value),
                        TextStyle {
                            font_size: VALUE_FONT_SIZE,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                })
                .insert(NeuronValueText { neuron_id });
        });
}

/// Update system: Update neuron indicator colors and value texts each frame
pub fn update_neuron_display(
    selected: Res<SelectedEntity>,
    brain_query: Query<&Brain>,
    mut indicator_query: Query<(&NeuronIndicator, &mut BackgroundColor)>,
    mut value_text_query: Query<(&NeuronValueText, &mut Text)>,
) {
    let Some(selected_entity) = selected.entity else {
        return;
    };

    let Ok(brain) = brain_query.get(selected_entity) else {
        return;
    };

    // Update indicators and value texts
    for (indicator, mut bg_color) in indicator_query.iter_mut() {
        if let Some(value) = get_neuron_value(brain, indicator.neuron_id) {
            *bg_color = value_to_color(value).into();
        }
    }

    for (value_text, mut text) in value_text_query.iter_mut() {
        if let Some(value) = get_neuron_value(brain, value_text.neuron_id) {
            if let Some(section) = text.sections.first_mut() {
                section.value = format!("{:.2}", value);
            }
        }
    }
}

/// Get the current value of a neuron by ID
fn get_neuron_value(brain: &Brain, neuron_id: usize) -> Option<f32> {
    if neuron_id < 100 {
        brain.inputs.get(neuron_id).map(|n| n.value)
    } else if neuron_id < 200 {
        brain.hidden_layers.get(neuron_id - 100).map(|n| n.value)
    } else {
        brain.outputs.get(neuron_id - 200).map(|n| n.value)
    }
}
