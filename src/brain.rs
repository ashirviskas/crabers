use bevy::prelude::*;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum NeuronType {
    // Input
    AlwaysOn,
    NearestFoodAngle,
    NearestFoodDistance,

    NearestCraberAngle,
    NearestCraberDistance,

    NearestWallAngle,
    NearestWallDistance,
    // Interval between each update. TODO: Add cost for higher intervals.
    BrainInterval,
    // Hidden
    Hidden,
    // Output
    MoveForward,
    Rotate,
    ModifyBrainInterval,
    MoodOutput,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ActivationFunction {
    None,
    Sigmoid,
    Tanh,
    ReLU,
    LeakyReLU,
    Softmax,
    AngleToNormalizedValue,
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
            ActivationFunction::AngleToNormalizedValue => Brain::angle_to_normalized_value(value),
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

pub struct Connection {
    pub from_id: usize, // Neuron id. < 100 is input < 200 is hidden < 300 is output.
    pub to_id: usize,   // Neuron id. < 100 is input < 200 is hidden < 300 is output.
    pub weight: f32,    // -1.0 to 1.0
    pub bias: f32,      // -1.0 to 1.0
    pub enabled: bool,
}

#[derive(Component)]
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
        ];
        let hidden_layers = vec![Neuron {
            neuron_type: NeuronType::Hidden,
            activation_function: ActivationFunction::AngleToNormalizedValue,
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
            // Angle to hidden
            Connection {
                from_id: 1,
                to_id: 100,
                weight: 1.0,
                bias: 0.0,
                enabled: true,
            },
            // Hidden to rotate
            Connection {
                from_id: 100,
                to_id: 201,
                weight: 4.0, // To make it rotate harder
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

    // Makes radians negative if the angle is above PI.
    pub fn angle_to_normalized_value(angle_radians: f32) -> f32 {
        angle_radians - std::f32::consts::PI
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

    pub fn feed_forward(&mut self) {
        // Input to hidden
        for connection_id in 0..self.connections.len() {
            let connection = &self.connections[connection_id];
            if !connection.enabled {
                continue;
            }
            if connection.from_id > 100 {
                continue;
            }
            let from_neuron = self.get_neuron(connection.from_id).unwrap();
            let to_neuron = self.get_neuron(connection.to_id).unwrap().clone();
            // to_neuron.value += from_neuron.value * connection.weight + connection.bias;
            let new_to_neuron_value = from_neuron.value * connection.weight + connection.bias;
            let new_to_neuron = Neuron {
                neuron_type: to_neuron.neuron_type,
                activation_function: to_neuron.activation_function,
                value: new_to_neuron_value,
            };
            self.set_neuron(connection.to_id, new_to_neuron);
            // self.update_neuron(connection.to_id, new_to_neuron_value);
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
            if connection.from_id < 100 || connection.from_id > 200 {
                continue;
            }
            let from_neuron = self.get_neuron(connection.from_id).unwrap();
            let to_neuron = self.get_neuron(connection.to_id).unwrap().clone();
            // to_neuron.value += from_neuron.value * connection.weight + connection.bias;
            let new_to_neuron_value = from_neuron.value * connection.weight + connection.bias;
            let new_to_neuron = Neuron {
                neuron_type: to_neuron.neuron_type,
                activation_function: to_neuron.activation_function,
                value: new_to_neuron_value,
            };
            self.set_neuron(connection.to_id, new_to_neuron);
            // self.update_neuron(connection.to_id, new_to_neuron_value);
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
}

// Vision update timer
#[derive(Resource)]
pub struct VisionUpdateTimer(pub Timer);

#[derive(Component)]
pub struct Vision {
    pub radius: f32,
    pub nearest_food_angle_radians: f32,
    pub nearest_food_distance: f32,
    pub see_food: bool,
    pub entities_in_vision: Vec<Entity>,
}
