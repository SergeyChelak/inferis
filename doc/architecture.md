# Architecture

The workspace is two crates. `engine` knows nothing about this game: it
holds the component storage, the run loop, asset loading, geometry and the
SDL boundary. `inferis` is the game: components, systems and scenes built on
top of it.

```
engine/
  entities/     component storage, entity ids, queries
  runloop/      the loop, scenes, system traits
  assets/       bundle and registry loading, textures, sounds
  geometry/     vectors, rectangles, the ray caster
  prelude.rs    the SDL types that cross into game code
inferis/
  game_scene/   the level: player, soldiers, damage, movement, rendering
  menu_scene/   the menu
  resource.rs   asset ids and scene ids
```

## Entities and components

`ComponentStorage` maps an entity to its components. A component is any
`'static` type; there is no trait to implement.

```rust
storage.register_component::<Position>()?;   // once, at scene setup
let id = storage.append(&bundle);            // spawn with components
storage.set(id, Some(Position(point)))?;     // attach or replace
storage.set::<Position>(id, None)?;          // detach
let pos = storage.get::<Position>(id);       // Ref<Position>
```

Every type must be registered before use, and `set` fails loudly if it is
not — the type is usually inferred from the value, so an unregistered
component is easy to pass by accident.

`EntityID` carries a generation alongside its index, so an id kept after the
entity died is rejected rather than silently addressing whatever entity
recycled the slot.

### Queries

Each entity holds a 128-bit footprint of the components it has, so a query
is a bitmask test rather than a series of map lookups.

```rust
let query = Query::new().with_component::<NpcTag>();
for id in storage.fetch_entities(&query) { ... }
```

A query naming an unregistered type matches nothing, rather than matching
everything.

### Caching an entity id

Systems that work on one distinguished entity — the player, the maze, the
menu cursor — keep its id between steps instead of scanning:

```rust
refresh_cached_entity::<PlayerTag>(storage, &mut self.player_id, "[npc] player")?;
```

It re-runs the lookup only once the cached entity is gone.

## Systems

A scene owns four kinds of system, each a trait in `engine::systems`:

| Trait | Runs | Purpose |
| --- | --- | --- |
| `GameSystem` | every step, in registration order | gameplay |
| `GameControlSystem` | when input arrives | translate keys into component state |
| `GameRendererSystem` | every rendered frame | build a list of draw effects |
| `GameSoundSystem` | every rendered frame | collect sounds to play |

Renderer and sound systems never touch SDL. They return `RendererEffect` and
`SoundEffect` values and the run loop draws and plays them, which is what
keeps the game crate free of the backend.

The game scene's systems run in this order, and the order matters:

```
GeneratorSystem   builds the level; rebuilds it on a new game
PlayerSystem      turns controller state into movement and shots
NpcSystem         decides what each soldier does
DamageSystem      resolves shots that have reached their deadline
MovementSystem    applies movement, with collision against walls and entities
```

Movement is last because every other system only ever *requests* movement by
attaching a `Movement` component; `MovementSystem` is the only place a
`Position` changes, and the only place collision is decided.

A system returns a `GameSystemCommand` — `Nothing`, `SwitchScene`, or
`Terminate`.

## Scenes

A scene is an id, its own `ComponentStorage`, and its systems. Scenes are
registered on `GameWorld` and switched by command:

| Scene | Id | |
| --- | --- | --- |
| `SCENE_GAME_PLAY` | 1 | the level |
| `SCENE_MAIN_MENU` | 2 | menu, also the pause and win screen |

A switch carries string parameters, which is how the menu knows why it was
entered: `invalidate` (start a new level), `pause`, `win`.

Each scene keeps its own entities, so switching does not disturb the level —
pausing and resuming returns to the same maze.

## Timing

Gameplay advances in fixed steps of 1/60 s, decoupled from rendering:

```rust
accumulator += elapsed.min(MAX_CATCH_UP);
while accumulator >= FIXED_STEP {
    accumulator -= FIXED_STEP;
    scene.update(FIXED_STEP_SECS, &assets)?;   // frames += 1 here
}
scene.render(&assets)?;                        // as often as vsync allows
```

This matters because gameplay durations are counted in frames — weapon
recharge, damage recovery, shot deadlines, animation. If `frames` tracked
rendered images, those durations would stretch or shrink with the display's
refresh rate while movement, which uses `delta_time`, stayed correct. The
game would desynchronise from itself.

Because a step is exactly 1/60 s, a frame count is an honest duration:
`PLAYER_SHOTGUN_RECHARGE_FRAMES = 45` is three quarters of a second, on any
display.

Two guards:

- **Catch-up is capped** at five steps. A long stall — level generation, a
  dragged window, a breakpoint — must not queue a burst of steps and
  fast-forward the game.
- **The clock resets on a scene switch**, since that handler may rebuild the
  level and should not be billed for the time it took.

`delta_time` is therefore always the same value. Movement is frame-rate
independent and deterministic.

## Errors and logging

`EngineError` implements `Display` and `std::error::Error`, so it composes
with `?` chains ending in `Box<dyn Error>`. `main` reports the `Display`
form itself, because a `main` returning `Result` prints `Debug` regardless.

Diagnostics go through the `log` crate. Module paths are the log targets, so
records carry their origin without hand-written prefixes:

```
RUST_LOG=warn                         only problems
RUST_LOG=inferis::game_scene=debug    one subsystem
```

## See also

- [Rendering](rendering.md) — the ray caster and how a frame is drawn
- [NPC behaviour](npc.md) — what the soldiers do
- [Asset bundle format](asset_bundle.md) — how assets are packed and loaded
