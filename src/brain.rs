use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;


const CRABER_MAX_WANT_TO_ATTACK: f32 = 10.;


#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum NeuronType {
    // Input
    AlwaysOn, // Implemented
    CraberHealth, // TODO
    CraberSpeed, // TODO

    NearestFoodAngle, // Implemented, value between -1 (left) and +1 (right) corresponding to the angle
    NearestFoodDistance, // TODO

    NearestCraberAngle, // TODO
    NearestCraberDistance, // TODO

    NearestWallAngle, // TODO
    NearestWallDistance, // TODO
    // Interval between each update. TODO: Add cost for higher intervals.
    BrainInterval, // TODO
    // Hidden
    Hidden,
    // Output
    MoveForward, // ?
    Rotate, // WIP
    ModifyBrainInterval, // TODO
    WantToMate,
    WantToAttack,
    WantToDefend,
}

impl NeuronType {
    pub fn random_input_type() -> Self {
        let input_types = vec![
            NeuronType::AlwaysOn,
            NeuronType::CraberHealth,
            NeuronType::CraberSpeed,
            NeuronType::NearestFoodAngle,
            NeuronType::NearestFoodDistance,
            NeuronType::NearestCraberAngle,
            NeuronType::NearestCraberDistance,
            NeuronType::NearestWallAngle,
            NeuronType::NearestWallDistance,
            NeuronType::BrainInterval,
        ];
        let mut rng = rand::thread_rng();
        *input_types.choose(&mut rng).unwrap()
    }

    pub fn random_hidden_type() -> Self {
        NeuronType::Hidden
    }

    pub fn random_output_type() -> Self {
        let output_types = vec![
            NeuronType::MoveForward,
            NeuronType::Rotate,
            NeuronType::ModifyBrainInterval,
            NeuronType::WantToMate,
        ];
        let mut rng = rand::thread_rng();
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
    // AngleToNormalizedValue,
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
            // ActivationFunction::AngleToNormalizedValue => Brain::angle_to_normalized_value(value),
        }
    }
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..5) {
            0 => ActivationFunction::None,
            1 => ActivationFunction::Sigmoid,
            2 => ActivationFunction::Tanh,
            3 => ActivationFunction::ReLU,
            4 => ActivationFunction::LeakyReLU,
            5 => ActivationFunction::Softmax,
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
            }

        ];
        let outputs = vec![
            Neuron {
                neuron_type: NeuronType::MoveForward,
                activation_function: ActivationFunction::None,
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
                value: 0.0
            }
        ];
        let hidden_layers = vec![Neuron {
            neuron_type: NeuronType::Hidden,
            activation_function: ActivationFunction::None,
            value: 0.0,
        }];
        let connections = vec![
            // Always on to move forward
            Connection {
                from_id: 0,
                to_id: 200,
                weight: 1.0,
                bias: 0.0,
                enabled: true,
            },
            // Food angle to hidden
            Connection {
                from_id: 1,
                to_id: 100,
                weight: 1.0,
                bias: 0.0,
                enabled: true,
            },
            // Craber angle to hidden
            Connection {
                from_id: 3,
                to_id: 100,
                weight: 1.0,
                bias: 0.0,
                enabled: true,
            },
            // Hidden to rotate
            Connection {
                from_id: 100,
                to_id: 201,
                weight: 4.5, // To make it rotate harder
                bias: 0.0,
                enabled: true,
            },
            // Always on to want to attack TODO: Remove after testing
            Connection {
                from_id: 0,
                to_id: 202,
                weight: 0.5,
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
        // print!("Updating craber neuron input");
        for neuron in self.inputs.iter_mut() {
            if neuron.neuron_type == input_neuron_type {
                neuron.value = value;
                // println!("Updated neuron type {:?} to value {}", input_neuron_type, value)
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
    pub fn get_forward_acceleration(&self) -> f32 {
        let mut acceleration = 0.0;
        for neuron in self.outputs.iter() {
            if neuron.neuron_type == NeuronType::MoveForward {
                acceleration = neuron.value;
            }
        }
        acceleration
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

    pub fn feed_forward(&mut self) {
        // Reset outputs and hidden
        for output_id in 0..self.outputs.len() {
            self.outputs[output_id].value = 0.0;
        }
        for hidden_id in 0..self.hidden_layers.len() {
            self.hidden_layers[hidden_id].value = 0.0;
        }
        // Input to hidden
        for connection_id in 0..self.connections.len() {
            let connection = &self.connections[connection_id];
            if !connection.enabled {
                continue;
            }
            if connection.from_id >= 100 {
                continue;
            }
            let from_neuron = self.get_neuron(connection.from_id).unwrap();
            let to_neuron = self.get_neuron(connection.to_id).unwrap().clone();
            let new_value = to_neuron.value + from_neuron.value * connection.weight + connection.bias;
            self.set_neuron_value(connection.to_id, new_value);
        }
        // Hidden functions
        for neuron in self.hidden_layers.iter_mut() {
            neuron.value = neuron.activation_function.calculate(neuron.value);
        }
        // Hidden to output
        for connection_id in 0..self.connections.len() {
            let connection = &self.connections[connection_id];
            if !connection.enabled {
                continue;
            }
            if connection.from_id < 100 || connection.from_id >= 200 {
                continue;
            }
            let from_neuron = self.get_neuron(connection.from_id).unwrap();
            let to_neuron = self.get_neuron(connection.to_id).unwrap().clone();
            let new_value = to_neuron.value + from_neuron.value * connection.weight + connection.bias;
            self.set_neuron_value(connection.to_id, new_value);
        }
        for neuron in self.outputs.iter_mut() {
            neuron.value = neuron.activation_function.calculate(neuron.value);
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
    

    pub fn new_mutated_brain(&self, mutation_chance: f32, mutation_amount: f32, insertion_chance: f32, deletion_chance: f32) -> Self {
        let mut mutated_brain = self.clone();
        let mut rng = rand::thread_rng();

        // Insertion mutations
        if rng.gen_range(0.0..1.) < insertion_chance {
            // rng between input/hidden/output
            // TODO. placeholder for only hidden layers
            match rng.gen_range(0..2) { // TODO Implement outputs insertion
                0 => {
                    let new_neuron = Neuron {
                        neuron_type: NeuronType::random_hidden_type(),
                        activation_function: ActivationFunction::random(),
                        value: 0.0,
                    };
                    mutated_brain.hidden_layers.push(new_neuron);
                },
                1 => {
                    let new_neuron = Neuron {
                        neuron_type: NeuronType::random_input_type(),
                        activation_function: ActivationFunction::random(),
                        value: 0.0,
                    };
                    mutated_brain.inputs.push(new_neuron);
                },
                2 => {
                    let new_neuron = Neuron {
                        neuron_type: NeuronType::random_output_type(),
                        activation_function: ActivationFunction::random(),
                        value: 0.0,
                    };
                    mutated_brain.outputs.push(new_neuron);
                },
                _ => {}
            }
        }
        // insertion of connection
        if rng.gen_range(0.0..1.) < insertion_chance {
            let from_a = rng.gen_range(0..mutated_brain.inputs.len());
            let from_b = rng.gen_range(0..mutated_brain.hidden_layers.len()) + 100;
            let mut new_connection = Connection {
                from_id: 0, // Initial placeholder value
                to_id: 0,   // Initial placeholder value
                weight: 0.0, // Initial placeholder value
                bias: 0.0,   // Initial placeholder value
                enabled: false, // Initial placeholder value
            };
            match rng.gen_range(0..2)
            {
                0 => {
                    let to_a = rng.gen_range(0..mutated_brain.hidden_layers.len()) + 100;
                    let to_b = rng.gen_range(0..mutated_brain.outputs.len()) + 200;
                    match rng.gen_range(0..2) {
                        0 => {
                            new_connection = Connection {
                                from_id: from_a,
                                to_id: to_a,
                                weight: rng.gen_range(-1.0..1.0),
                                bias: rng.gen_range(-1.0..1.0),
                                enabled: true,
                            };
                        },
                        1 | _ => {
                            new_connection = Connection {
                                from_id: from_a,
                                to_id: to_b,
                                weight: rng.gen_range(-1.0..1.0),
                                bias: rng.gen_range(-1.0..1.0),
                                enabled: true,
                            };
                        },
                        
                    }
                },
                1 | _ => {
                    let to_b = rng.gen_range(0..self.outputs.len()) + 200;
                    new_connection = Connection {
                        from_id: from_b,
                        to_id: to_b,
                        weight: rng.gen_range(-1.0..1.0),
                        bias: rng.gen_range(-1.0..1.0),
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
            if rng.gen_range(0.0..1.) < mutation_chance {
                let change = rng.gen_range(-mutation_amount..mutation_amount);
                connection.weight += change;
            }

            // Mutate the bias
            if rng.gen_range(0.0..1.) < mutation_chance {
                let change = rng.gen_range(-mutation_amount..mutation_amount);
                connection.bias += change;
            }

            // Optionally, mutate the 'enabled' status
            if rng.gen_range(0.0..1.) < mutation_chance {
                connection.enabled = !connection.enabled;
            }
        }

        // Optionally, mutate neurons (e.g., activation functions)
        for neuron in mutated_brain.hidden_layers.iter_mut() {
            if rng.gen_range(0.0..1.) < mutation_chance {
                neuron.activation_function = ActivationFunction::random();
            }
        }

        mutated_brain
    }
}

// Vision update timer
#[derive(Resource)]
pub struct VisionUpdateTimer(pub Timer);

#[derive(Component)]
pub struct Vision {
    pub radius: f32,
    pub nearest_food_direction: f32,
    pub nearest_food_distance: f32,
    pub nearest_craber_direction: f32,
    pub nearest_craber_distance: f32,
    pub see_food: bool,
    pub see_craber: bool,
    pub entities_in_vision: Vec<Entity>,
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
        self.entities_in_vision = Vec::new();
    }
}
