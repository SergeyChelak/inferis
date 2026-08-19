# Prerequisites and running

## Cargo/Rust language
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## SDL2

macOS
```
brew install sdl2 sdl2_image sdl2_mixer
```
Arch
```
sudo pacman -S sdl2 sdl2_image sdl2_mixer
```
Debian / Raspberry Pi OS
```
sudo apt install libsdl2-dev libsdl2-image-dev libsdl2-mixer-dev
```

A [Nix shell](../shell.nix) is also provided.

## Building and running

```
cargo run --release
```

Debug builds spawn 5 soldiers and release builds 20, which is the difference
between debugging the AI and playing against it.

The game looks for its assets in this order, relative to the working
directory:

1. `inferis.bin`, the compiled bundle — this is what is committed
2. `assets/asset_registry.txt`, the loose source assets

so it runs from the repository root with no extra setup.

## Controls

| | |
| --- | --- |
| `W` `S` or `↑` `↓` | forward, back |
| `A` `D` | strafe |
| `←` `→` | turn |
| `X` | shoot |
| `Esc` | pause, back to the menu |

In the menu: `↑` `↓` to move, `Return` to select. Mouse movement is
captured but not yet used to look around.

## Logging

Diagnostics go through the `log` crate, with module paths as targets.
Default level is `info`; `RUST_LOG` overrides it.

```
RUST_LOG=warn cargo run --release              only problems
RUST_LOG=inferis::game_scene=debug cargo run   one subsystem
```

## Rebuilding the asset bundle

The loose assets under `assets/` are not committed — `inferis.bin` is. To
rebuild it after changing them:

```
./assets.sh
```

which runs the bundler directly:

```
cargo run --bin asset_bundler assets/asset_registry.txt inferis.bin
```

Keep source images at or under 4096 pixels in both dimensions. Larger ones
are scaled down at load time so the game still starts, but they lose detail
and warn every run — see [Rendering](rendering.md#texture-size-limit).

## See also

- [Architecture](architecture.md)
- [Asset bundle format](asset_bundle.md)
