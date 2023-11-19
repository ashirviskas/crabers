# Craber Evolution Simulation

## Overview
This project is a physics-based simulation developed in Bevy, focused on the evolution of virtual creatures called "crabers". These crabers live in a 2D space and evolve over time through natural selection and genetic mutation.

![image](https://github.com/ashirviskas/crabers/assets/11985242/305bbd40-010a-4609-90fa-cf8abb4da18a)



## Getting Started
1. Have rust and cargo installed.
2. Clone the repo.
3. Run `cargo run --release` in the root directory.

## Features roadmap
- **Evolution:** Crabers evolve over time through natural selection and genetic mutation.
- **Neural Brain:** Crabers have a dynamic neural structure for their brains, evolving connections over generations.
- **Genetic Traits:** Each craber has DNA defining basic characteristics like color, size, and maturity factors.
- **Physics-Based Movement:** Crabers can move forward/backward and strafe left/right in a fluid medium with drag. They can also turn or steer.
- **Sensory Inputs:** Include relative speed, angle to nearest food, angle to nearest organism, genetic closeness, pheromone sense, current energy level, and health.
- **Reproduction:** Crabers reproduce asexually upon reaching maturity and having sufficient energy. Offspring may have mutations.
- **Food Sources:** Random blobs of "food" spawn in the environment.
- **Pheromone System:** An experimental feature for inter-craber communication and interaction.
- **Horizontal Gene Transfer:** Crabers can transfer genetic information to other crabers through a "gene transfer" action that might happen if both parties are willing and bump into each other [TBD].

## PoC TODO
- [x] Basic movement and collision physics for crabers.
- [ ] Simple genetic system for craber traits.
- [ ] Initial implementation of the neural brain with predefined inputs and outputs.
- [x] Basic environment setup with food source spawning.
- [ ] Simple reproduction mechanics without advanced features.

## Contributing
Contributions are welcome! Please read [CONTRIBUTING.md](./CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us. TBD

## License
This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments
- Bevy Engine: For providing the game engine.
- Contributors: Everyone who has contributed to the project.
