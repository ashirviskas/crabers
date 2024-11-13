# Craber Evolution Simulation

## Overview
This project is a physics-based simulation developed in Bevy, focused on the evolution of virtual creatures called "crabers". These crabers live in a 2D space and evolve over time through natural selection and genetic mutation.

Run it now in your browser: [Here](https://ashirviskas.github.io/)

![image](https://github.com/ashirviskas/crabers/assets/11985242/305bbd40-010a-4609-90fa-cf8abb4da18a)



## Getting Started
### Prerequisites

1. Have rust and cargo installed.
2. Clone the repo.


### Building and running the game from source

Running locally:

1. Run `cargo run --release` in the root directory.

Running in a browser:

1. Run `rustup target install wasm32-unknown-unknown` to install the wasm target.
2. Run `cargo install wasm-server-runner` to install the wasm server runner.
3. Run `cargo run --target wasm32-unknown-unknown --release` to build the wasm build.

Building a WASM build:
1. Run `cargo build --release --target wasm32-unknown-unknown` to build the wasm build.
2. Run `wasm-bindgen --out-dir ./out/ --target web ./target/` to generate the wasm bindings.
3. Copy the `index.html` to the `out/` directory.
4. Run some kind of web server in the `out/` directory. I use `python3 -m http.server`.



### Running the wasm build



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

## TODO
- [x] Basic movement and collision physics for crabers.
- [ ] Simple genetic system for craber traits. WIP
  - [ ] Dominant/recessive genes code for brain
  - [ ] Genes code for and other traits
    - [ ] Size
    - [ ] Aging (no max age? But allow for evolution to implement some dying mechanism, either via brain outputs to kill itself or something else)
    - [ ] Max health (costs for max health?)
    - [ ] Max energy (costs for max energy?)
    - [ ] Power (for speed)
- [x] Initial implementation of the neural brain with predefined inputs and outputs
  - [ ] Inputs:
    - [ ] Speed
    - [x] Angle to nearest food
    - [ ] Angle to nearest organism
    - [ ] Genetic closeness
    - [ ] Current energy level
    - [ ] Health
  - [ ] Outputs:
    - [x] Forward/backward acceleration
    - [ ] Left/right acceleration / strafing
    - [x] Rotational acceleration (turning/steering)
- [x] Basic environment setup with food source spawning.
- [ ] Basic growing and maturity system for crabers.
- [x] Simple reproduction mechanics without advanced features.
- [ ] Sexual reproduction
  - [ ] Brain modifications
  - [ ] Reproduction modifications
- [x] Simple evolution/mutation system
  - [x] Brain mutations
  - [ ] Other craber qualities mutations
- [ ] Damage system
  - [ ] Bite/Attack
  - [ ] Defend
- [ ] Drop food when die
- [ ] More complex brain
  - [ ] Ability to enable/disable connections
  - [ ] Brain costs energy (neurons, inputs, connections cost passively, each feed forward - actively)
  - [ ] Control brain *clock speed* (need a better name)
- [ ] Improve speed for higher number of crabers (simulation should support at least 5k on a modern system, with up to 10k (as per tests, should be viable))
- [ ] Cleanup and do all code `TODO`s.

Nice to haves:
- [x] WASM Build

## License
This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments
- Bevy Engine: For providing the game engine.
- Contributors: Everyone who has contributed to the project.
