use bevy_egui::egui;

use crate::brain::{Brain, NeuronType};

// Layout constants
const PANEL_WIDTH: f32 = 420.0;
const NEURON_RADIUS: f32 = 14.0;
const COLUMN_X: [f32; 3] = [60.0, 200.0, 340.0];
const ROW_START_Y: f32 = 60.0;
const ROW_SPACING: f32 = 65.0;

/// Neuron layer enum for position calculation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NeuronLayer {
    Input,
    Hidden,
    Output,
}

/// Classify a neuron ID into its layer
fn id_to_layer(id: usize) -> NeuronLayer {
    if id < 100 {
        NeuronLayer::Input
    } else if id < 200 {
        NeuronLayer::Hidden
    } else {
        NeuronLayer::Output
    }
}

/// Calculate the position of a neuron based on its layer and index
fn neuron_position(layer: NeuronLayer, index: usize) -> egui::Pos2 {
    let col = match layer {
        NeuronLayer::Input => 0,
        NeuronLayer::Hidden => 1,
        NeuronLayer::Output => 2,
    };
    egui::pos2(COLUMN_X[col], ROW_START_Y + index as f32 * ROW_SPACING)
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

/// Get a short label for a neuron type
fn neuron_label(neuron_type: NeuronType) -> &'static str {
    match neuron_type {
        NeuronType::AlwaysOn => "ON",
        NeuronType::CraberHealth => "HP",
        NeuronType::CraberSpeed => "SPD",
        NeuronType::CraberEnergy => "NRG",
        NeuronType::CraberAge => "AGE",
        NeuronType::NearestFoodAngle => "F.ANG",
        NeuronType::NearestFoodDistance => "F.DST",
        NeuronType::NearestCraberAngle => "C.ANG",
        NeuronType::NearestCraberDistance => "C.DST",
        NeuronType::NearestWallAngle => "W.ANG",
        NeuronType::NearestWallDistance => "W.DST",
        NeuronType::NearestCraberGeneticCloseness => "GEN.C",
        NeuronType::BrainInterval => "INT",
        NeuronType::LastReproduced => "REPR.T",
        NeuronType::Hidden => "H",
        NeuronType::KickStrength => "K.STR",
        NeuronType::KickRate => "K.RT",
        NeuronType::AlignVelocity => "ALIGN",
        NeuronType::Rotate => "ROT",
        NeuronType::RotateRate => "R.RT",
        NeuronType::ModifyBrainInterval => "M.INT",
        NeuronType::WantToReproduce => "REPR",
        NeuronType::WantSexualReproduction => "SEX",
        NeuronType::WantToAttack => "ATK",
        NeuronType::WantToDefend => "DEF",
    }
}

/// Convert a weight value to a color (green for positive, red for negative)
fn weight_to_color(weight: f32, enabled: bool) -> egui::Color32 {
    if !enabled {
        return egui::Color32::from_rgba_unmultiplied(77, 77, 77, 77);
    }
    let intensity = weight.abs().min(1.0);
    if weight >= 0.0 {
        let g = (128.0 + intensity * 127.0) as u8;
        let a = (153.0 + intensity * 102.0) as u8;
        egui::Color32::from_rgba_unmultiplied(51, g, 51, a)
    } else {
        let r = (128.0 + intensity * 127.0) as u8;
        let a = (153.0 + intensity * 102.0) as u8;
        egui::Color32::from_rgba_unmultiplied(r, 51, 51, a)
    }
}

/// Convert a neuron value to a display color
fn value_to_color(value: f32) -> egui::Color32 {
    let clamped = value.clamp(-1.0, 1.0);
    if clamped >= 0.0 {
        let g = (102.0 + clamped * 153.0) as u8;
        egui::Color32::from_rgb(51, g, 51)
    } else {
        let r = (102.0 + (-clamped) * 153.0) as u8;
        egui::Color32::from_rgb(r, 51, 51)
    }
}

/// Draw the neural network visualization using egui
pub fn draw_neural_network(ui: &mut egui::Ui, brain: &Brain) {
    let input_count = brain.inputs.len();
    let hidden_count = brain.hidden_layers.len();
    let output_count = brain.outputs.len();
    let max_rows = input_count.max(hidden_count).max(output_count);
    let panel_height = ROW_START_Y + max_rows as f32 * ROW_SPACING + 20.0;

    let (response, painter) =
        ui.allocate_painter(egui::vec2(PANEL_WIDTH, panel_height), egui::Sense::hover());
    let origin = response.rect.min;

    // Column headers
    let header_color = egui::Color32::from_rgb(180, 180, 180);
    for (label, x) in [("INPUTS", COLUMN_X[0]), ("HIDDEN", COLUMN_X[1]), ("OUTPUTS", COLUMN_X[2])] {
        painter.text(
            origin + egui::vec2(x, 20.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(13.0),
            header_color,
        );
    }

    // Helper: get position for a neuron ID (absolute in painter space)
    let get_pos = |id: usize| -> Option<egui::Pos2> {
        if id < 100 {
            if id < input_count { Some(origin + neuron_position(NeuronLayer::Input, id).to_vec2()) } else { None }
        } else if id < 200 {
            let idx = id - 100;
            if idx < hidden_count { Some(origin + neuron_position(NeuronLayer::Hidden, idx).to_vec2()) } else { None }
        } else {
            let idx = id - 200;
            if idx < output_count { Some(origin + neuron_position(NeuronLayer::Output, idx).to_vec2()) } else { None }
        }
    };

    // Draw connections
    for connection in &brain.connections {
        let from_pos = get_pos(connection.from_id);
        let to_pos = get_pos(connection.to_id);

        if let (Some(from), Some(to)) = (from_pos, to_pos) {
            let color = weight_to_color(connection.weight, connection.enabled);
            let thickness = 1.0 + connection.weight.abs().min(1.0) * 2.0;
            let from_layer = id_to_layer(connection.from_id);
            let to_layer = id_to_layer(connection.to_id);

            match (from_layer, to_layer) {
                // Self-loop: draw a small arc to the right
                (NeuronLayer::Hidden, NeuronLayer::Hidden) if connection.from_id == connection.to_id => {
                    let loop_r = 20.0;
                    let center = from + egui::vec2(NEURON_RADIUS + loop_r, 0.0);
                    let points: Vec<egui::Pos2> = (0..=20)
                        .map(|i| {
                            let a = -std::f32::consts::PI * 0.7
                                + (i as f32 / 20.0) * std::f32::consts::PI * 1.4;
                            center + egui::vec2(loop_r * a.cos(), loop_r * a.sin())
                        })
                        .collect();
                    for w in points.windows(2) {
                        painter.line_segment([w[0], w[1]], egui::Stroke::new(thickness, color));
                    }
                }
                // Input→Output: curve above hidden column
                (NeuronLayer::Input, NeuronLayer::Output) => {
                    let control = egui::pos2(
                        (from.x + to.x) / 2.0,
                        (from.y + to.y) / 2.0 - 50.0,
                    );
                    let bezier = egui::epaint::QuadraticBezierShape::from_points_stroke(
                        [from, control, to],
                        false,
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(thickness, color),
                    );
                    painter.add(bezier);
                }
                // Hidden→Hidden different neurons: curve bulging right
                (NeuronLayer::Hidden, NeuronLayer::Hidden) => {
                    let control = egui::pos2(
                        (from.x + to.x) / 2.0 + 50.0,
                        (from.y + to.y) / 2.0,
                    );
                    let bezier = egui::epaint::QuadraticBezierShape::from_points_stroke(
                        [from, control, to],
                        false,
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(thickness, color),
                    );
                    painter.add(bezier);
                }
                // Normal: straight line
                _ => {
                    painter.line_segment([from, to], egui::Stroke::new(thickness, color));
                }
            }
        }
    }

    // Draw neurons
    let draw_neuron = |pos: egui::Pos2, neuron_id: usize, label: &str| {
        let value = get_neuron_value(brain, neuron_id).unwrap_or(0.0);
        // Outer circle
        painter.circle(
            pos,
            NEURON_RADIUS,
            egui::Color32::from_rgb(51, 51, 64),
            egui::Stroke::new(1.5, egui::Color32::from_rgb(102, 102, 128)),
        );
        // Inner indicator
        painter.circle_filled(pos, NEURON_RADIUS - 3.0, value_to_color(value));
        // Label below
        painter.text(
            pos + egui::vec2(0.0, NEURON_RADIUS + 10.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(10.0),
            egui::Color32::from_rgb(204, 204, 204),
        );
        // Value below label
        painter.text(
            pos + egui::vec2(0.0, NEURON_RADIUS + 22.0),
            egui::Align2::CENTER_CENTER,
            format!("{:.2}", value),
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
    };

    for (idx, neuron) in brain.inputs.iter().enumerate() {
        let pos = origin + neuron_position(NeuronLayer::Input, idx).to_vec2();
        draw_neuron(pos, idx, neuron_label(neuron.neuron_type));
    }
    for (idx, neuron) in brain.hidden_layers.iter().enumerate() {
        let pos = origin + neuron_position(NeuronLayer::Hidden, idx).to_vec2();
        draw_neuron(pos, 100 + idx, neuron_label(neuron.neuron_type));
    }
    for (idx, neuron) in brain.outputs.iter().enumerate() {
        let pos = origin + neuron_position(NeuronLayer::Output, idx).to_vec2();
        draw_neuron(pos, 200 + idx, neuron_label(neuron.neuron_type));
    }
}
