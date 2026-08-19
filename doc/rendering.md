# Rendering

The world is drawn by ray casting: one ray per screen column, each returning
the wall it hit and how far away it is, which becomes the height of a
vertical strip of wall texture. Sprites are drawn on top, sorted by depth.

The renderer system never calls SDL. It fills three layers with
`RendererEffect` values and the run loop draws them:

| Layer | Contents | Order |
| --- | --- | --- |
| `background` | sky, floor gradient | first, unsorted |
| `depth` | wall columns and sprites | sorted far to near |
| `hud` | weapon, minimap, damage flash | last, unsorted |

The depth layer is a painter's algorithm — there is no depth buffer, so
everything in it is sorted by distance and drawn back to front.

## The wall cast

At a window width of 1600 the renderer casts `width / 2` = 800 rays across a
60° field of view, each drawn two pixels wide.

```rust
const FIELD_OF_VIEW: Float = PI / 3.0;
self.rays_count = window_size.width >> 1;
self.ray_angle_step = FIELD_OF_VIEW / self.rays_count as Float;
self.screen_distance = (window_size.width >> 1) as Float / HALF_FIELD_OF_VIEW.tan();
```

Every ray in the fan is the view direction turned by a fixed offset, and
those offsets never change while the window exists. So they are computed
once as sine/cosine pairs, and each frame costs one `sin_cos` for the view
direction plus a rotation per ray:

```rust
let (view_sin, view_cos) = self.angle.sin_cos();
for (ray, &(offset_sin, offset_cos)) in self.ray_offsets.iter().enumerate() {
    let sin = view_sin * offset_cos + view_cos * offset_sin;
    let cos = view_cos * offset_cos - view_sin * offset_sin;
    ...
}
```

The offset's cosine is also exactly the fisheye correction — the angle
between a ray and the view direction *is* its offset — so the same table
serves both and no per-ray transcendental is needed at all.

### Inside `ray_cast`

The caster walks two sweeps, one crossing the grid's horizontal lines and
one its vertical lines, and keeps whichever hits nearer. The second sweep
stops as soon as it is further away than the first sweep's hit, since
nothing beyond that can win.

A ray parallel to one axis never crosses that axis's lines, and its sweep is
skipped rather than walked: it would divide by zero and step along a line of
infinities, which a `check` written as a plain bounds test reports as a hit
— at infinite depth, which would then cut the other sweep short.

The step limit comes from the maze, not from a constant:

```rust
let max_steps = maze.ray_cast_steps();   // max(rows, cols)
```

A ray advances one row or one column per step, so the larger dimension
always reaches the far side. Fixing this limit independently of the map is a
trap: too small and distant walls silently vanish and shots pass through
them, with no error anywhere. On an open 120×120 map, a bound of 50 finds
nothing at all — every one of the 800 rays reports no hit.

## Textures

A wall hit reports an *index* into `WALL_TEXTURES`, not a name, and the
renderer resolves names to handles once at setup:

```rust
pub struct TextureId(usize);   // engine::assets
```

`RendererEffect::Texture` carries the handle, so drawing indexes a slice.
Carrying names instead meant hashing a string for every one of the ~812
effects in a frame.

### Texture size limit

GPUs cap texture dimensions — 4096 on a Raspberry Pi's V3D. A larger image
cannot be uploaded, and one oversized asset would otherwise take the whole
game down at startup with nothing but a failed-to-load message.

Images are therefore decoded to a surface first, and anything over the
renderer's reported maximum is scaled down to fit, keeping its aspect ratio
so sprite sheets still divide evenly into frames. It warns when it does:

```
texture 'shotgun_shot' is 5820x1050, larger than the renderer's 4096x4096;
scaling it to 4096x738. Re-export it smaller to keep full detail.
```

A renderer reporting `0x0`, as the software one does, means no limit.

## Sprites

Sprites are billboards. Their screen position comes from the angle between
the sprite and the view direction, converted to a column:

```rust
let delta_rays = delta / self.ray_angle_step;
let x = ((self.rays_count >> 1) as Float + delta_rays) * self.scale;
let norm_distance = vector.length() * delta.cos();
```

They are culled when off screen or nearer than half a tile, but **not**
occlusion-culled — a sprite behind a wall is still drawn, then painted over
by the wall columns in front of it.

`ScaleRatio` and `HeightShift` per entity set how large a sprite is relative
to its projection and how far down the screen it sits, so a soldier stands
on the floor rather than floating at eye level.

## Where the time goes

Measured on a Raspberry Pi 5, per frame, with 20 soldiers:

| Phase | Cost |
| --- | --- |
| all game systems, one step | ~100 µs |
| building the effect lists | ~225 µs, of which the 800-ray fan is ~75 µs |
| submitting ~812 draw calls | ~45 µs |
| filling the pixels | milliseconds |

The ray fan is a small part of a frame; filling pixels dominates. Those
numbers come from the software renderer used for headless runs — the
accelerated path could not be profiled on that machine, so treat the fill
cost as an upper bound and the CPU costs above it as representative.

Worth knowing before optimising the caster again. What is left on the CPU
side is draw-call submission, roughly 55 ns per `copy`, and cutting that
means fewer and wider quads. Adjacent wall columns cannot simply be merged:
each has its own projected height and texture offset, and merging them
distorts the perspective that makes them look right.

## See also

- [Architecture](architecture.md) — where the renderer sits in the loop
- [Asset bundle format](asset_bundle.md) — where textures come from
