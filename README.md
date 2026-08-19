# Inferis

## Overview
Welcome to the FPS Game Research Project! This project is a deep dive into the world of game development, focusing on creating a first-person shooter (FPS) game. The primary goal is not to produce a market-ready game but to explore and understand the various aspects of game development.

## Purpose
The main purposes of this project are:
- To gain hands-on experience with game development.
- To experiment with the Entity-Component-System (ECS) architecture.
- To implement a basic game engine from scratch.

## What is in here
The workspace is two crates. `engine` is a small ECS and run loop that knows
nothing about this game; `inferis` is the game built on it — a ray-cast
first-person shooter with a generated maze, wandering soldiers, and a menu.

```
cargo run --release
```

See [Prerequisites and running](doc/prerequisites.md) for dependencies,
controls and logging.

## Documentation

### The engine
- [Architecture](doc/architecture.md) — entities and components, systems,
  scenes, the fixed-timestep loop, errors and logging
- [Rendering](doc/rendering.md) — the ray caster, wall and sprite drawing,
  texture handles and size limits
- [Asset bundle format](doc/asset_bundle.md) — how assets are packed,
  described and loaded

### The game
- [NPC behaviour](doc/npc.md) — how a soldier decides what to do:
  wandering, chasing, dodging, taking cover
- [Maze generator](https://github.com/SergeyChelak/cellular_automata) —
  the cellular automaton the levels come from

### Getting started
- [Prerequisites and running](doc/prerequisites.md)
- [ECS tutorial & resources](doc/references.md) — what this was learned from

## Disclaimer
This project is purely for research and educational purposes. It is not intended to result in a fully-fledged game, and it is not expected that anyone will use it seriously as a game. The focus is on learning and experimentation rather than producing a finished product.

## Contributing
Contributions are welcome! Whether it's suggesting new features, reporting bugs, or improving the documentation, your input is valuable.
