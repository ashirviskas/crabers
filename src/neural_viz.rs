use bevy::prelude::*;
use crate::brain::{Brain, NeuronType};
use crate::SelectedEntity;

/// Marker component for the neural network panel
#[derive(Component)]
pub struct NeuralNetworkPanel;

/// Marker for the columns container
#[derive(Component)]
pub struct NeuralColumnsContainer;

/// Marker for input column
#[derive(Component)]
pub struct InputColumn;

/// Marker for hidden column
#[derive(Component)]
pub struct HiddenColumn;

/// Marker for output column
#[derive(Component)]
pub struct OutputColumn;

/// Marker for connections list container
#[derive(Component)]
pub struct ConnectionsContainer;

/// Component to identify a neuron node in the UI
#[derive(Component)]
pub struct NeuronNode {
    pub neuron_id: usize,
    pub layer: NeuronLayer,
}

/// Component for neuron value text
#[derive(Component)]
pub struct NeuronValueText {
    pub neuron_id: usize,
}

/// Component for neuron indicator (the colored box)
#[derive(Component)]
pub struct NeuronIndicator {
    pub neuron_id: usize,
    pub layer: NeuronLayer,
}

/// Marker for connection text entries
#[derive(Component)]
pub struct ConnectionText;

/// Enum to identify neuron layer for coloring
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NeuronLayer {
    Input,
    Hidden,
    Output,
}

/// Resource to track if we need to rebuild the panel
#[derive(Resource, Default)]
pub struct NeuralNetworkLayout {
    pub visible: bool,
    pub last_entity: Option<Entity>,
}

/// Panel configuration constants
const PANEL_WIDTH: f32 = 340.0;
const PANEL_HEIGHT: f32 = 500.0;
const PANEL_MARGIN: f32 = 10.0;
const NEURON_BOX_SIZE: f32 = 16.0;

/// Setup the neural network panel UI container
pub fn setup_neural_panel(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(PANEL_MARGIN),
                top: Val::Px(PANEL_MARGIN),
                width: Val::Px(PANEL_WIDTH),
                max_height: Val::Px(PANEL_HEIGHT),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(8.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.92)),
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(NeuralNetworkPanel)
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Neural Network",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            });

            // Columns container (inputs, hidden, outputs)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: Val::Px(8.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(NeuralColumnsContainer)
                .with_children(|columns| {
                    // Input column
                    spawn_column(columns, "INPUTS", InputColumn);
                    // Hidden column
                    spawn_column(columns, "HIDDEN", HiddenColumn);
                    // Output column
                    spawn_column(columns, "OUTPUTS", OutputColumn);
                });

            // Connections section title
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Connections:",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::srgba(0.8, 0.8, 0.8, 1.0),
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                ..default()
            });

            // Connections container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        max_height: Val::Px(150.0),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    ..default()
                })
                .insert(ConnectionsContainer);
        });
}

fn spawn_column<T: Component>(parent: &mut ChildBuilder, title: &str, marker: T) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                min_width: Val::Px(90.0),
                flex_grow: 1.0,
                ..default()
            },
            ..default()
        })
        .insert(marker)
        .with_children(|col| {
            col.spawn(TextBundle {
                text: Text::from_section(
                    title,
                    TextStyle {
                        font_size: 12.0,
                        color: Color::srgba(0.6, 0.6, 0.6, 1.0),
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            });
        });
}

/// Update the neural network panel based on the selected craber's brain
pub fn update_nn_layout(
    selected: Res<SelectedEntity>,
    brain_query: Query<&Brain>,
    mut layout: ResMut<NeuralNetworkLayout>,
    mut panel_query: Query<&mut Visibility, With<NeuralNetworkPanel>>,
) {
    // Check if we have a selected entity with a brain
    let has_brain = if let Some(entity) = selected.entity {
        brain_query.get(entity).is_ok()
    } else {
        false
    };

    // Update panel visibility
    for mut visibility in panel_query.iter_mut() {
        *visibility = if has_brain {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    layout.visible = has_brain;

    // Track entity changes
    if selected.entity != layout.last_entity {
        layout.last_entity = selected.entity;
    }
}

/// Spawn neuron nodes when a craber is selected (or rebuild when selection changes)
pub fn spawn_neuron_nodes(
    mut commands: Commands,
    selected: Res<SelectedEntity>,
    brain_query: Query<&Brain>,
    layout: Res<NeuralNetworkLayout>,
    input_col: Query<Entity, With<InputColumn>>,
    hidden_col: Query<Entity, With<HiddenColumn>>,
    output_col: Query<Entity, With<OutputColumn>>,
    connections_container: Query<Entity, With<ConnectionsContainer>>,
    existing_neurons: Query<Entity, With<NeuronNode>>,
    existing_connections: Query<Entity, With<ConnectionText>>,
) {
    // Only rebuild when entity changes
    if !selected.is_changed() {
        return;
    }

    // Clear existing neurons
    for entity in existing_neurons.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in existing_connections.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let Some(selected_entity) = selected.entity else {
        return;
    };

    let Ok(brain) = brain_query.get(selected_entity) else {
        return;
    };

    let Ok(input_col_entity) = input_col.get_single() else {
        return;
    };
    let Ok(hidden_col_entity) = hidden_col.get_single() else {
        return;
    };
    let Ok(output_col_entity) = output_col.get_single() else {
        return;
    };
    let Ok(connections_entity) = connections_container.get_single() else {
        return;
    };

    // Spawn input neurons
    for (i, neuron) in brain.inputs.iter().enumerate() {
        spawn_neuron_card(
            &mut commands,
            input_col_entity,
            i,
            NeuronLayer::Input,
            &format_neuron_type(neuron.neuron_type),
            neuron.value,
        );
    }

    // Spawn hidden neurons
    for (i, neuron) in brain.hidden_layers.iter().enumerate() {
        let neuron_id = 100 + i;
        spawn_neuron_card(
            &mut commands,
            hidden_col_entity,
            neuron_id,
            NeuronLayer::Hidden,
            &format!("H{}", i),
            neuron.value,
        );
    }

    // Spawn output neurons
    for (i, neuron) in brain.outputs.iter().enumerate() {
        let neuron_id = 200 + i;
        spawn_neuron_card(
            &mut commands,
            output_col_entity,
            neuron_id,
            NeuronLayer::Output,
            &format_neuron_type(neuron.neuron_type),
            neuron.value,
        );
    }

    // Spawn connection entries
    for connection in brain.connections.iter() {
        let status = if connection.enabled { "✓" } else { "✗" };
        let weight_color = if connection.weight >= 0.0 {
            Color::srgba(0.4, 0.9, 0.4, 1.0)
        } else {
            Color::srgba(0.9, 0.4, 0.4, 1.0)
        };

        commands.entity(connections_entity).with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text::from_sections([
                        TextSection {
                            value: format!("{}→{} ", connection.from_id, connection.to_id),
                            style: TextStyle {
                                font_size: 11.0,
                                color: Color::srgba(0.7, 0.7, 0.7, 1.0),
                                ..default()
                            },
                        },
                        TextSection {
                            value: format!("w:{:.2} ", connection.weight),
                            style: TextStyle {
                                font_size: 11.0,
                                color: weight_color,
                                ..default()
                            },
                        },
                        TextSection {
                            value: status.to_string(),
                            style: TextStyle {
                                font_size: 11.0,
                                color: if connection.enabled {
                                    Color::srgba(0.4, 0.9, 0.4, 1.0)
                                } else {
                                    Color::srgba(0.5, 0.5, 0.5, 1.0)
                                },
                                ..default()
                            },
                        },
                    ]),
                    ..default()
                })
                .insert(ConnectionText);
        });
    }
}

fn spawn_neuron_card(
    commands: &mut Commands,
    parent_entity: Entity,
    neuron_id: usize,
    layer: NeuronLayer,
    label: &str,
    value: f32,
) {
    let base_color = get_layer_color(layer);

    commands.entity(parent_entity).with_children(|parent| {
        parent
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(4.0)),
                    row_gap: Val::Px(2.0),
                    min_width: Val::Px(70.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.8)),
                ..default()
            })
            .insert(NeuronNode { neuron_id, layer })
            .with_children(|card| {
                // Colored indicator box
                card.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(NEURON_BOX_SIZE),
                        height: Val::Px(NEURON_BOX_SIZE),
                        ..default()
                    },
                    background_color: BackgroundColor(base_color),
                    ..default()
                })
                .insert(NeuronIndicator { neuron_id, layer });

                // Label
                card.spawn(TextBundle {
                    text: Text::from_section(
                        label,
                        TextStyle {
                            font_size: 10.0,
                            color: Color::srgba(0.8, 0.8, 0.8, 1.0),
                            ..default()
                        },
                    ),
                    ..default()
                });

                // Value
                card.spawn(TextBundle {
                    text: Text::from_section(
                        format!("{:.2}", value),
                        TextStyle {
                            font_size: 11.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                })
                .insert(NeuronValueText { neuron_id });
            });
    });
}

/// Update neuron displays each frame based on brain state
pub fn update_neuron_display(
    selected: Res<SelectedEntity>,
    brain_query: Query<&Brain>,
    mut value_texts: Query<(&NeuronValueText, &mut Text)>,
    mut indicators: Query<(&NeuronIndicator, &mut BackgroundColor)>,
) {
    let Some(selected_entity) = selected.entity else {
        return;
    };

    let Ok(brain) = brain_query.get(selected_entity) else {
        return;
    };

    // Update value texts
    for (value_text, mut text) in value_texts.iter_mut() {
        let value = get_neuron_value(brain, value_text.neuron_id);
        text.sections[0].value = format!("{:.2}", value);
    }

    // Update indicator colors based on activation
    for (indicator, mut bg_color) in indicators.iter_mut() {
        let value = get_neuron_value(brain, indicator.neuron_id);
        let activation = value.abs().clamp(0.0, 1.0);
        let intensity = 0.3 + activation * 0.7;

        let (base_r, base_g, base_b) = match indicator.layer {
            NeuronLayer::Input => (0.3, 0.5, 1.0),
            NeuronLayer::Hidden => (0.5, 0.5, 0.5),
            NeuronLayer::Output => (1.0, 0.6, 0.2),
        };

        *bg_color = BackgroundColor(Color::srgba(
            base_r * intensity,
            base_g * intensity,
            base_b * intensity,
            1.0,
        ));
    }
}

fn get_neuron_value(brain: &Brain, neuron_id: usize) -> f32 {
    if neuron_id < 100 {
        brain.inputs.get(neuron_id).map(|n| n.value).unwrap_or(0.0)
    } else if neuron_id < 200 {
        brain
            .hidden_layers
            .get(neuron_id - 100)
            .map(|n| n.value)
            .unwrap_or(0.0)
    } else {
        brain
            .outputs
            .get(neuron_id - 200)
            .map(|n| n.value)
            .unwrap_or(0.0)
    }
}

fn get_layer_color(layer: NeuronLayer) -> Color {
    match layer {
        NeuronLayer::Input => Color::srgba(0.3, 0.5, 1.0, 1.0),
        NeuronLayer::Hidden => Color::srgba(0.5, 0.5, 0.5, 1.0),
        NeuronLayer::Output => Color::srgba(1.0, 0.6, 0.2, 1.0),
    }
}

fn format_neuron_type(neuron_type: NeuronType) -> String {
    match neuron_type {
        NeuronType::AlwaysOn => "AlwOn".to_string(),
        NeuronType::CraberHealth => "Health".to_string(),
        NeuronType::CraberSpeed => "Speed".to_string(),
        NeuronType::NearestFoodAngle => "FdAngl".to_string(),
        NeuronType::NearestFoodDistance => "FdDist".to_string(),
        NeuronType::NearestCraberAngle => "CrAngl".to_string(),
        NeuronType::NearestCraberDistance => "CrDist".to_string(),
        NeuronType::NearestWallAngle => "WlAngl".to_string(),
        NeuronType::NearestWallDistance => "WlDist".to_string(),
        NeuronType::BrainInterval => "BrInt".to_string(),
        NeuronType::Hidden => "Hidden".to_string(),
        NeuronType::MoveForward => "MovFwd".to_string(),
        NeuronType::Rotate => "Rotate".to_string(),
        NeuronType::ModifyBrainInterval => "ModInt".to_string(),
        NeuronType::WantToMate => "Mate".to_string(),
        NeuronType::WantToAttack => "Attack".to_string(),
        NeuronType::WantToDefend => "Defend".to_string(),
    }
}
