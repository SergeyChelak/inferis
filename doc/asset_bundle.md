# Asset Bundle Format

Assets stored in the compiled bundle. The data format is pretty simple and it could be described as following:

| Name        | Type              | Description                                          |
| ----------- | ----------------- | ---------------------------------------------------- |
| Asset type  | u8                | texture, animation, color, gradient, binary          |
| Id length   | u64 (LE)          | length of asset name (used to identify) in the app   |
| Id          | [u8; id length]   | asset id (name)                                      |
| Raw Type    | u8                | 0 for binary, 1 for string                           |
| Data length | u64 (LE)          | length of asset payload                              |
| Data        | [u8; data length] | asset payload (binary or string)                     |

This structure is repeated for each asset in bundle.

All multi-byte integers are stored as fixed-width little-endian values,
so a bundle built on one platform is readable on any other.

## Asset types

| Type | Payload | Format | Example |
| --- | --- | --- | --- |
| texture | binary | PNG | |
| animation | text | `texture_id frames duration` | `soldier_death 9 7` |
| color | text | `r,g,b` or `r,g,b,a` | `136,8,8,60` |
| vertical gradient | text | `from-to height` | `0,0,0-35,35,35 450` |
| binary | binary | opaque bytes | |
| sound chunk | binary | anything SDL_mixer loads | |

Text fields are whitespace separated. A colour's alpha defaults to opaque.
A gradient's two colours are separated by `-` and the texture it generates
is one pixel wide by `height` tall, stretched across the screen when drawn.

An animation names the texture holding its frames, laid out in a single row:
frame width is the texture width divided by `frames`, and `duration` is how
many simulation steps each frame is held. Steps are a fixed 1/60 s
(see [Architecture](architecture.md#timing)), so `soldier_death 9 7` is nine
frames at 7/60 s each — just over a second.

## Loading

`AssetManager` reads either a bundle or the loose registry and keeps
textures in a vector behind a `TextureId`. Renderers resolve the names they
need once, at setup, and draw by handle — see
[Rendering](rendering.md#textures).

A texture larger than the renderer can hold is scaled down on load rather
than refused; the aspect ratio is preserved so a sprite sheet still divides
evenly into frames. Keep source art at or under 4096 pixels to avoid it.

## See also

- [Prerequisites and running](prerequisites.md#rebuilding-the-asset-bundle)
- [Rendering](rendering.md)
