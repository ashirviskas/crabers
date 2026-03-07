use bevy::prelude::*;
use rand::RngExt;
use rand::seq::IndexedRandom;

/// Clamp that maps NaN/Inf to 0.0 instead of propagating.
fn finite_clamp(v: f32, min: f32, max: f32) -> f32 {
    if v.is_finite() {
        v.clamp(min, max)
    } else {
        0.0
    }
}

const CRABER_MAX_WANT_TO_ATTACK: f32 = 10.;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum NeuronType {
    // Input
    AlwaysOn,
    CraberHealth,
    CraberSpeed,  // TODO
    CraberEnergy,
    CraberAge,

    NearestFoodAngle, // Implemented, value between -1 (left) and +1 (right) corresponding to the angle
    NearestFoodDistance, // TODO

    NearestCraberAngle,    // TODO
    NearestCraberDistance, // TODO

    NearestWallAngle,    // TODO
    NearestWallDistance, // TODO
    NearestCraberGeneticCloseness, // Genetic closeness to nearest visible craber (0-1)
    // Interval between each update. TODO: Add cost for higher intervals.
    BrainInterval, // TODO
    LastReproduced, // Decay-based: 1.0 after reproduction, decays toward 0
    // Hidden
    Hidden,
    // Output
    KickStrength,        // How hard each kick pushes (sigmoid, 0-1)
    KickRate,            // How often kicks fire (sigmoid, 0-1; 0=disabled, 1=max)
    AlignVelocity,       // How much velocity redirects toward facing (sigmoid, 0-1)
    Rotate,              // Angular impulse direction (tanh, -1 to +1)
    RotateRate,          // How often rotation impulses fire (ReLU, 0+)
    ModifyBrainInterval, // TODO
    WantToReproduce,
    WantSexualReproduction,
    WantToAttack,
    WantToDefend,
}

impl NeuronType {
    pub fn random_input_type() -> Self {
        let input_types = vec![
            NeuronType::AlwaysOn,
            NeuronType::CraberHealth,
            NeuronType::CraberSpeed,
            NeuronType::CraberEnergy,
            NeuronType::CraberAge,
            NeuronType::NearestFoodAngle,
            NeuronType::NearestFoodDistance,
            NeuronType::NearestCraberAngle,
            NeuronType::NearestCraberDistance,
            NeuronType::NearestWallAngle,
            NeuronType::NearestWallDistance,
            NeuronType::NearestCraberGeneticCloseness,
            NeuronType::BrainInterval,
            NeuronType::LastReproduced,
        ];
        let mut rng = rand::rng();
        *input_types.choose(&mut rng).unwrap()
    }

    pub fn random_hidden_type() -> Self {
        NeuronType::Hidden
    }

    pub fn random_output_type() -> Self {
        let output_types = vec![
            NeuronType::KickStrength,
            NeuronType::KickRate,
            NeuronType::AlignVelocity,
            NeuronType::Rotate,
            NeuronType::RotateRate,
            NeuronType::ModifyBrainInterval,
            NeuronType::WantToReproduce,
            NeuronType::WantSexualReproduction,
        ];
        let mut rng = rand::rng();
        *output_types.choose(&mut rng).unwrap()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ActivationFunction {
    None,
    Sigmoid,
    Tanh,
    ReLU,
    LeakyReLU,
    Softmax,
    Sin,
}

impl ActivationFunction {
    pub fn calculate(&self, value: f32) -> f32 {
        match self {
            ActivationFunction::None => value,
            ActivationFunction::Sigmoid => 1.0 / (1.0 + (-value).exp()),
            ActivationFunction::Tanh => value.tanh(),
            ActivationFunction::ReLU => value.max(0.0),
            ActivationFunction::LeakyReLU => value.max(0.01 * value),
            ActivationFunction::Softmax => value.exp() / value.exp(),
            ActivationFunction::Sin => value.sin(),
        }
    }
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..6) {
            0 => ActivationFunction::None,
            1 => ActivationFunction::Sigmoid,
            2 => ActivationFunction::Tanh,
            3 => ActivationFunction::ReLU,
            4 => ActivationFunction::LeakyReLU,
            5 => ActivationFunction::Softmax,
            6 => ActivationFunction::Sin,
            _ => ActivationFunction::None, // Fallback
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Neuron {
    pub neuron_type: NeuronType,
    // Optional activation function. If none is provided, the value is used directly.
    pub activation_function: ActivationFunction,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub from_id: usize, // Neuron id. < 100 is input < 200 is hidden < 300 is output.
    pub to_id: usize,   // Neuron id. < 100 is input < 200 is hidden < 300 is output.
    pub weight: f32,    // -1.0 to 1.0
    pub bias: f32,      // -1.0 to 1.0
    pub enabled: bool,
}
/// Craber brain
/// Neurons are mapped to indexes using this map:
///     inputs          [0..99]
///     hidden_layers   [100..199]
///     outputs         [200..inf]
#[derive(Component, Debug, Clone)]
pub struct Brain {
    pub inputs: Vec<Neuron>,
    pub outputs: Vec<Neuron>,
    pub hidden_layers: Vec<Neuron>,
    pub connections: Vec<Connection>,
}

impl Brain {
    // Default brain
    pub fn default() -> Self {
        let inputs = vec![
            Neuron {
                neuron_type: NeuronType::AlwaysOn,
                activation_function: ActivationFunction::None,
                value: 1.0,
            },
            Neuron {
                neuron_type: NeuronType::CraberHealth,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::CraberEnergy,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::CraberAge,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::NearestFoodAngle,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::NearestFoodDistance,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::NearestCraberAngle,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::NearestCraberDistance,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::NearestWallAngle,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::NearestWallDistance,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::NearestCraberGeneticCloseness,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::BrainInterval,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::LastReproduced,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
        ];
        let outputs = vec![
            Neuron {
                neuron_type: NeuronType::KickStrength,
                activation_function: ActivationFunction::ReLU,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::KickRate,
                activation_function: ActivationFunction::ReLU,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::AlignVelocity,
                activation_function: ActivationFunction::Sigmoid,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::Rotate,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::WantToAttack,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::ModifyBrainInterval,
                activation_function: ActivationFunction::Sigmoid,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::RotateRate,
                activation_function: ActivationFunction::ReLU,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::WantToReproduce,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
            Neuron {
                neuron_type: NeuronType::WantSexualReproduction,
                activation_function: ActivationFunction::None,
                value: 0.0,
            },
        ];
        let hidden_layers = vec![Neuron {
            neuron_type: NeuronType::Hidden,
            activation_function: ActivationFunction::Sin,
            value: 0.0,
        }];
        let connections = vec![
            // AlwaysOn -> KickStrength (ReLU(0.2)=0.2, gentle kicks)
            Connection {
                from_id: 0,
                to_id: 200,
                weight: 0.5,
                bias: 0.0,
                enabled: true,
            },
            // AlwaysOn -> KickRate (ReLU(0.2)=0.2, infrequent)
            Connection {
                from_id: 0,
                to_id: 201,
                weight: 0.05,
                bias: 0.0,
                enabled: true,
            },
            // AlwaysOn -> AlignVelocity (sigmoid(2.0)=0.88, mostly ship-like)
            Connection {
                from_id: 0,
                to_id: 202,
                weight: 2.0,
                bias: 0.0,
                enabled: true,
            },
            // FoodAngle -> Hidden
            Connection {
                from_id: 4,
                to_id: 100,
                weight: 1.0,
                bias: 0.0,
                enabled: true,
            },
            // CraberAngle -> Hidden
            Connection {
                from_id: 6,
                to_id: 100,
                weight: 0.1,
                bias: 0.0,
                enabled: false,
            },
            // Hidden -> Rotate
            Connection {
                from_id: 100,
                to_id: 203,
                weight: 2.5,
                bias: 0.0,
                enabled: true,
            },
            // AlwaysOn -> WantToAttack
            Connection {
                from_id: 0,
                to_id: 204,
                weight: 0.2,
                bias: 0.0,
                enabled: true,
            },
            // AlwaysOn -> RotateRate
            Connection {
                from_id: 0,
                to_id: 206,
                weight: 0.5,
                bias: 0.0,
                enabled: true,
            },
            // AlwaysOn -> WantToReproduce (always on: 1.0 * 1.5 = 1.5 >= 1.0)
            Connection {
                from_id: 0,
                to_id: 207,
                weight: 1.5,
                bias: 0.0,
                enabled: true,
            },
            // AlwaysOn -> WantSexualReproduction (always on: 1.0 * 1.5 = 1.5 >= 1.0)
            Connection {
                from_id: 0,
                to_id: 208,
                weight: 1.5,
                bias: 0.0,
                enabled: true,
            },
        ];
        Self {
            inputs,
            outputs,
            hidden_layers,
            connections,
        }
    }
    pub fn get_neuron(&self, id: usize) -> Option<&Neuron> {
        if id < 100 {
            self.inputs.get(id)
        } else if id < 200 {
            self.hidden_layers.get(id - 100)
        } else if id < 300 {
            self.outputs.get(id - 200)
        } else {
            None
        }
    }

    pub fn set_neuron(&mut self, id: usize, neuron: Neuron) {
        if id < 100 {
            self.inputs[id] = neuron;
        } else if id < 200 {
            self.hidden_layers[id - 100] = neuron;
        } else if id < 300 {
            self.outputs[id - 200] = neuron;
        }
    }

    pub fn set_neuron_value(&mut self, id: usize, new_value: f32) {
        if id < 100 {
            self.inputs[id].value = new_value;
        } else if id < 200 {
            self.hidden_layers[id - 100].value = new_value;
        } else if id < 300 {
            self.outputs[id - 200].value = new_value;
        }
    }

    pub fn update_input(&mut self, input_neuron_type: NeuronType, value: f32) {
        for neuron in self.inputs.iter_mut() {
            if neuron.neuron_type == input_neuron_type {
                neuron.value = value;
            }
        }
    }

    pub fn get_input_types(&self) -> Vec<NeuronType> {
        let mut input_types = Vec::new();
        for neuron in self.inputs.iter() {
            input_types.push(neuron.neuron_type);
        }
        input_types
    }

    pub fn get_rotation(&self) -> f32 {
        let mut rotation = 0.0;
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::Rotate {
                rotation = neuron.value;
            }
        }
        rotation
    }
    pub fn get_kick_strength(&self) -> f32 {
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::KickStrength {
                return neuron.value;
            }
        }
        0.0
    }
    pub fn get_rotate_rate(&self) -> f32 {
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::RotateRate {
                return neuron.value;
            }
        }
        0.0
    }
    pub fn get_kick_rate(&self) -> f32 {
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::KickRate {
                return neuron.value;
            }
        }
        0.0
    }
    pub fn get_align_velocity(&self) -> f32 {
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::AlignVelocity {
                return neuron.value;
            }
        }
        0.0
    }
    pub fn get_want_to_attack(&self) -> f32 {
        let mut want_to_attack = 0.0;
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::WantToAttack {
                want_to_attack = neuron.value;
                if want_to_attack > CRABER_MAX_WANT_TO_ATTACK {
                    want_to_attack = CRABER_MAX_WANT_TO_ATTACK;
                }
            }
        }
        want_to_attack
    }

    pub fn get_want_to_defent(&self) -> f32 {
        let mut want_to_defend = 0.0;
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::WantToDefend {
                want_to_defend = neuron.value;
            }
        }
        want_to_defend
    }

    pub fn get_modify_brain_interval(&self) -> f32 {
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::ModifyBrainInterval {
                return neuron.value;
            }
        }
        0.0
    }

    pub fn get_want_to_reproduce(&self) -> f32 {
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::WantToReproduce {
                return neuron.value;
            }
        }
        0.0
    }

    pub fn get_want_sexual_reproduction(&self) -> f32 {
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::WantSexualReproduction {
                return neuron.value;
            }
        }
        0.0
    }

    pub fn crossover_brain(
        &self,
        other: &Brain,
        mutation_chance: f32,
        mutation_amount: f32,
        insertion_chance: f32,
    ) -> Brain {
        let mut rng = rand::rng();

        // Inputs: union of both parents' input types
        let mut input_types_seen = std::collections::HashSet::new();
        let mut inputs = Vec::new();
        for neuron in self.inputs.iter().chain(other.inputs.iter()) {
            if input_types_seen.insert(neuron.neuron_type) {
                inputs.push(*neuron);
            }
        }

        // Hidden: align by index, 50/50 pick activation
        let max_hidden = self.hidden_layers.len().max(other.hidden_layers.len());
        let mut hidden_layers = Vec::new();
        for i in 0..max_hidden {
            let a = self.hidden_layers.get(i);
            let b = other.hidden_layers.get(i);
            match (a, b) {
                (Some(na), Some(nb)) => {
                    let activation = if rng.random_range(0.0..1.0) < 0.5 {
                        na.activation_function
                    } else {
                        nb.activation_function
                    };
                    hidden_layers.push(Neuron {
                        neuron_type: NeuronType::Hidden,
                        activation_function: activation,
                        value: 0.0,
                    });
                }
                (Some(n), None) | (None, Some(n)) => {
                    if rng.random_range(0.0..1.0) < 0.5 {
                        hidden_layers.push(Neuron {
                            neuron_type: NeuronType::Hidden,
                            activation_function: n.activation_function,
                            value: 0.0,
                        });
                    }
                }
                (None, None) => {}
            }
        }
        if hidden_layers.is_empty() {
            hidden_layers.push(Neuron {
                neuron_type: NeuronType::Hidden,
                activation_function: ActivationFunction::Sin,
                value: 0.0,
            });
        }

        // Outputs: union of both parents' output types; shared types → 50/50 pick activation
        let mut output_types_seen = std::collections::HashSet::new();
        let mut outputs = Vec::new();
        for neuron in self.outputs.iter() {
            output_types_seen.insert(neuron.neuron_type);
            let other_neuron = other
                .outputs
                .iter()
                .find(|n| n.neuron_type == neuron.neuron_type);
            let activation = if let Some(on) = other_neuron {
                if rng.random_range(0.0..1.0) < 0.5 {
                    neuron.activation_function
                } else {
                    on.activation_function
                }
            } else {
                neuron.activation_function
            };
            outputs.push(Neuron {
                neuron_type: neuron.neuron_type,
                activation_function: activation,
                value: 0.0,
            });
        }
        for neuron in other.outputs.iter() {
            if !output_types_seen.contains(&neuron.neuron_type) {
                output_types_seen.insert(neuron.neuron_type);
                outputs.push(Neuron {
                    neuron_type: neuron.neuron_type,
                    activation_function: neuron.activation_function,
                    value: 0.0,
                });
            }
        }

        // Build neuron type→index maps for remapping connections
        let input_type_to_idx: std::collections::HashMap<NeuronType, usize> = inputs
            .iter()
            .enumerate()
            .map(|(i, n)| (n.neuron_type, i))
            .collect();
        let output_type_to_idx: std::collections::HashMap<NeuronType, usize> = outputs
            .iter()
            .enumerate()
            .map(|(i, n)| (n.neuron_type, i + 200))
            .collect();

        // Helper to remap a neuron ID from a parent brain to the child brain
        let remap_id = |id: usize, parent: &Brain| -> Option<usize> {
            if id < 100 {
                // Input neuron — remap by type
                let neuron_type = parent.inputs.get(id)?.neuron_type;
                input_type_to_idx.get(&neuron_type).copied()
            } else if id < 200 {
                // Hidden neuron — keep index if within bounds
                let idx = id - 100;
                if idx < hidden_layers.len() {
                    Some(id)
                } else {
                    None
                }
            } else {
                // Output neuron — remap by type
                let idx = id - 200;
                let neuron_type = parent.outputs.get(idx)?.neuron_type;
                output_type_to_idx.get(&neuron_type).copied()
            }
        };

        // Connections: match by (from_type, to_type); shared → 50/50 pick weight+bias
        type ConnKey = (usize, usize); // (from_neuron_type_hash, to_neuron_type_hash)

        // Collect parent A connections (remapped)
        let mut a_conns: Vec<(ConnKey, Connection)> = Vec::new();
        for conn in &self.connections {
            if let (Some(new_from), Some(new_to)) =
                (remap_id(conn.from_id, self), remap_id(conn.to_id, self))
            {
                let key = (new_from, new_to);
                a_conns.push((
                    key,
                    Connection {
                        from_id: new_from,
                        to_id: new_to,
                        weight: conn.weight,
                        bias: conn.bias,
                        enabled: conn.enabled,
                    },
                ));
            }
        }

        // Collect parent B connections (remapped)
        let mut b_conn_map: std::collections::HashMap<ConnKey, Connection> =
            std::collections::HashMap::new();
        for conn in &other.connections {
            if let (Some(new_from), Some(new_to)) =
                (remap_id(conn.from_id, other), remap_id(conn.to_id, other))
            {
                let key = (new_from, new_to);
                b_conn_map.insert(
                    key,
                    Connection {
                        from_id: new_from,
                        to_id: new_to,
                        weight: conn.weight,
                        bias: conn.bias,
                        enabled: conn.enabled,
                    },
                );
            }
        }

        let mut connections = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();
        for (key, a_conn) in &a_conns {
            seen_keys.insert(*key);
            if let Some(b_conn) = b_conn_map.get(key) {
                // Shared: 50/50 pick
                let conn = if rng.random_range(0.0..1.0) < 0.5 {
                    a_conn
                } else {
                    b_conn
                };
                connections.push(conn.clone());
            } else {
                connections.push(a_conn.clone());
            }
        }
        for (key, b_conn) in &b_conn_map {
            if !seen_keys.contains(key) {
                if rng.random_range(0.0..1.0) < 0.5 {
                    connections.push(b_conn.clone());
                }
            }
        }

        let child = Brain {
            inputs,
            outputs,
            hidden_layers,
            connections,
        };

        // Apply standard mutation on the crossover result
        child.new_mutated_brain(mutation_chance, mutation_amount, insertion_chance, 0.0)
    }

    pub fn genetic_closeness(&self, other: &Brain) -> f32 {
        // Structural similarity: Jaccard index of connection keys
        let self_keys: std::collections::HashSet<(usize, usize)> = self
            .connections
            .iter()
            .map(|c| (c.from_id, c.to_id))
            .collect();
        let other_keys: std::collections::HashSet<(usize, usize)> = other
            .connections
            .iter()
            .map(|c| (c.from_id, c.to_id))
            .collect();
        let intersection = self_keys.intersection(&other_keys).count() as f32;
        let union = self_keys.union(&other_keys).count() as f32;
        let structural = if union > 0.0 {
            intersection / union
        } else {
            1.0
        };

        // Weight similarity: average 1 - |diff| for shared connections
        let other_map: std::collections::HashMap<(usize, usize), &Connection> = other
            .connections
            .iter()
            .map(|c| ((c.from_id, c.to_id), c))
            .collect();
        let mut weight_sim_sum = 0.0;
        let mut weight_count = 0;
        for conn in &self.connections {
            if let Some(oc) = other_map.get(&(conn.from_id, conn.to_id)) {
                weight_sim_sum += 1.0 - (conn.weight - oc.weight).abs().min(2.0) / 2.0;
                weight_count += 1;
            }
        }
        let weight_sim = if weight_count > 0 {
            weight_sim_sum / weight_count as f32
        } else {
            0.0
        };

        // Activation similarity: matching activations in hidden layers
        let max_hidden = self.hidden_layers.len().min(other.hidden_layers.len());
        let mut act_match = 0;
        for i in 0..max_hidden {
            if self.hidden_layers[i].activation_function
                == other.hidden_layers[i].activation_function
            {
                act_match += 1;
            }
        }
        let total_hidden = self.hidden_layers.len().max(other.hidden_layers.len());
        let activation_sim = if total_hidden > 0 {
            act_match as f32 / total_hidden as f32
        } else {
            1.0
        };

        0.5 * structural + 0.3 * weight_sim + 0.2 * activation_sim
    }

    pub fn feed_forward(&mut self) {
        // Snapshot all neuron values into prev (double-buffer)
        let max_id = 200 + self.outputs.len();
        let mut prev = vec![0.0f32; max_id];
        for (i, n) in self.inputs.iter().enumerate() {
            prev[i] = n.value;
        }
        for (i, n) in self.hidden_layers.iter().enumerate() {
            prev[100 + i] = n.value;
        }
        for (i, n) in self.outputs.iter().enumerate() {
            prev[200 + i] = n.value;
        }

        // Pull-compute hidden neurons
        for h_idx in 0..self.hidden_layers.len() {
            let h_id = 100 + h_idx;
            let mut sum = 0.0f32;
            for conn in &self.connections {
                if !conn.enabled || conn.to_id != h_id {
                    continue;
                }
                if conn.from_id < prev.len() {
                    sum += prev[conn.from_id] * conn.weight + conn.bias;
                }
            }
            self.hidden_layers[h_idx].value = finite_clamp(
                self.hidden_layers[h_idx].activation_function.calculate(sum),
                -1e6,
                1e6,
            );
        }

        // Pull-compute output neurons
        for o_idx in 0..self.outputs.len() {
            let o_id = 200 + o_idx;
            let mut sum = 0.0f32;
            for conn in &self.connections {
                if !conn.enabled || conn.to_id != o_id {
                    continue;
                }
                if conn.from_id < prev.len() {
                    sum += prev[conn.from_id] * conn.weight + conn.bias;
                }
            }
            self.outputs[o_idx].value = finite_clamp(
                self.outputs[o_idx].activation_function.calculate(sum),
                -1e6,
                1e6,
            );
        }
    }

    pub fn print_brain(&self) {
        if self.outputs[1].value != 0.0 {
            println!("Angling");
        } else {
            return;
        }
        println!("\n");
        println!("Brain:\n");
        println!("Inputs:");
        for neuron in self.inputs.iter() {
            println!(
                "Neuron type: {:?}, Activation function: {:?}, Value: {}",
                neuron.neuron_type, neuron.activation_function, neuron.value
            );
        }
        println!("Hidden:");
        for neuron in self.hidden_layers.iter() {
            println!(
                "Neuron type: {:?}, Activation function: {:?}, Value: {}",
                neuron.neuron_type, neuron.activation_function, neuron.value
            );
        }
        println!("Outputs:");
        for neuron in self.outputs.iter() {
            println!(
                "Neuron type: {:?}, Activation function: {:?}, Value: {}",
                neuron.neuron_type, neuron.activation_function, neuron.value
            );
        }
        println!("Connections:");
        for connection in self.connections.iter() {
            println!(
                "From: {}, To: {}, Weight: {}, Bias: {}, Enabled: {}",
                connection.from_id,
                connection.to_id,
                connection.weight,
                connection.bias,
                connection.enabled
            );
        }
    }
    pub fn get_brain_info(&self) -> String {
        let mut result = String::new();

        result.push_str("Brain:\n\n");
        result.push_str("Inputs:\n");
        for neuron in self.inputs.iter() {
            result.push_str(&format!(
                "Neuron type: {:?}, Activation function: {:?}, Value: {}\n",
                neuron.neuron_type, neuron.activation_function, neuron.value
            ));
        }

        result.push_str("Hidden:\n");
        for neuron in self.hidden_layers.iter() {
            result.push_str(&format!(
                "Neuron type: {:?}, Activation function: {:?}, Value: {}\n",
                neuron.neuron_type, neuron.activation_function, neuron.value
            ));
        }

        result.push_str("Outputs:\n");
        for neuron in self.outputs.iter() {
            result.push_str(&format!(
                "Neuron type: {:?}, Activation function: {:?}, Value: {}\n",
                neuron.neuron_type, neuron.activation_function, neuron.value
            ));
        }

        result.push_str("Connections:\n");
        for connection in self.connections.iter() {
            result.push_str(&format!(
                "From: {}, To: {}, Weight: {}, Bias: {}, Enabled: {}\n",
                connection.from_id,
                connection.to_id,
                connection.weight,
                connection.bias,
                connection.enabled
            ));
        }
        result
    }

    pub fn new_mutated_brain(
        &self,
        mutation_chance: f32,
        mutation_amount: f32,
        insertion_chance: f32,
        _deletion_chance: f32,
    ) -> Self {
        let mut mutated_brain = self.clone();
        let mut rng = rand::rng();

        // Insertion mutations
        if rand::random_range(0.0..1.) < insertion_chance {
            // rng between input/hidden/output
            // TODO. placeholder for only hidden layers
            match rng.random_range(0..2) {
                // TODO Implement outputs insertion
                0 => {
                    let new_neuron = Neuron {
                        neuron_type: NeuronType::random_hidden_type(),
                        activation_function: ActivationFunction::random(),
                        value: 0.0,
                    };
                    mutated_brain.hidden_layers.push(new_neuron);
                }
                1 => {
                    let new_neuron = Neuron {
                        neuron_type: NeuronType::random_input_type(),
                        activation_function: ActivationFunction::random(),
                        value: 0.0,
                    };
                    mutated_brain.inputs.push(new_neuron);
                }
                2 => {
                    let new_neuron = Neuron {
                        neuron_type: NeuronType::random_output_type(),
                        activation_function: ActivationFunction::random(),
                        value: 0.0,
                    };
                    mutated_brain.outputs.push(new_neuron);
                }
                _ => {}
            }
        }
        // insertion of connection
        if rng.random_range(0.0..1.) < insertion_chance {
            let from_a = rng.random_range(0..mutated_brain.inputs.len());
            let from_b = rng.random_range(0..mutated_brain.hidden_layers.len()) + 100;
            #[allow(unused_assignments)]
            let mut new_connection = Connection {
                from_id: 0,
                to_id: 0,
                weight: 0.0,
                bias: 0.0,
                enabled: false,
            };
            match rng.random_range(0..3) {
                0 => {
                    let to_a = rng.random_range(0..mutated_brain.hidden_layers.len()) + 100;
                    let to_b = rng.random_range(0..mutated_brain.outputs.len()) + 200;
                    match rng.random_range(0..2) {
                        0 => {
                            new_connection = Connection {
                                from_id: from_a,
                                to_id: to_a,
                                weight: rng.random_range(-1.0..1.0),
                                bias: rng.random_range(-1.0..1.0),
                                enabled: true,
                            };
                        }
                        1 | _ => {
                            new_connection = Connection {
                                from_id: from_a,
                                to_id: to_b,
                                weight: rng.random_range(-1.0..1.0),
                                bias: rng.random_range(-1.0..1.0),
                                enabled: true,
                            };
                        }
                    }
                }
                1 => {
                    let to_b = rng.random_range(0..self.outputs.len()) + 200;
                    new_connection = Connection {
                        from_id: from_b,
                        to_id: to_b,
                        weight: rng.random_range(-1.0..1.0),
                        bias: rng.random_range(-1.0..1.0),
                        enabled: true,
                    };
                }
                2 | _ => {
                    // Hidden→hidden connection (allows self-connections)
                    let from_h = rng.random_range(0..mutated_brain.hidden_layers.len()) + 100;
                    let to_h = rng.random_range(0..mutated_brain.hidden_layers.len()) + 100;
                    new_connection = Connection {
                        from_id: from_h,
                        to_id: to_h,
                        weight: rng.random_range(-1.0..1.0),
                        bias: rng.random_range(-1.0..1.0),
                        enabled: true,
                    };
                }
            }
            mutated_brain.connections.push(new_connection);
        }

        // Deletion mutations
        // same as insertion

        for connection in mutated_brain.connections.iter_mut() {
            // Mutate the weight
            if rng.random_range(0.0..1.) < mutation_chance {
                let change = rng.random_range(-mutation_amount..mutation_amount);
                connection.weight += change;
            }

            // Mutate the bias
            if rng.random_range(0.0..1.) < mutation_chance {
                let change = rng.random_range(-mutation_amount..mutation_amount);
                connection.bias += change;
            }

            // Optionally, mutate the 'enabled' status
            if rng.random_range(0.0..1.) < mutation_chance {
                connection.enabled = !connection.enabled;
            }
        }

        // Optionally, mutate neurons (e.g., activation functions)
        for neuron in mutated_brain.hidden_layers.iter_mut() {
            if rng.random_range(0.0..1.) < mutation_chance {
                neuron.activation_function = ActivationFunction::random();
            }
        }

        mutated_brain
    }
}

#[derive(Component)]
pub struct Vision {
    pub radius: f32,
    pub nearest_food_direction: f32,
    pub nearest_food_distance: f32,
    pub nearest_craber_direction: f32,
    pub nearest_craber_distance: f32,
    pub nearest_craber_genetic_closeness: f32,
    pub nearest_wall_direction: f32,
    pub nearest_wall_distance: f32,
    pub see_food: bool,
    pub see_craber: bool,
    pub see_wall: bool,
    pub entities_in_vision: Vec<Entity>,
    pub food_seen_timer: f32,
    pub craber_seen_timer: f32,
    pub wall_seen_timer: f32,
}

impl Vision {
    pub fn no_see_food(&mut self) {
        self.see_food = false;
        self.nearest_food_distance = std::f32::MAX;
        self.nearest_food_direction = 0.;
        self.entities_in_vision = Vec::new();
    }
    pub fn no_see_craber(&mut self) {
        self.see_craber = false;
        self.nearest_craber_distance = std::f32::MAX;
        self.nearest_craber_direction = 0.;
        self.nearest_craber_genetic_closeness = 0.;
        self.entities_in_vision = Vec::new();
    }
    pub fn no_see_wall(&mut self) {
        self.see_wall = false;
        self.nearest_wall_distance = std::f32::MAX;
        self.nearest_wall_direction = 0.;
    }
}
