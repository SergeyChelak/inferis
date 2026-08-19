# NPC behaviour

A soldier walks the level, chases the player when it can see them, goes to
look where they were when it loses sight, steps aside when hit, and breaks
off for cover when badly wounded.

## Intent and animation are separate

`ActorState` selects the sprite animation — idle, walk, attack, damage,
death. `NpcPlan` holds what the soldier is actually doing:

```rust
pub struct NpcPlan {
    pub intent: NpcIntent,          // Wander | Investigate | Reposition | Hide
    pub route: VecDeque<Vec2f>,     // remaining waypoints
    pub hold_until: usize,          // stand still until this step
    pub last_seen: Option<Vec2f>,   // where the player was
    pub player_visible: bool,       // standing verdict between casts
    ...
}
```

These have to be separate. A soldier walking a patrol and one closing for a
kill look identical and are not doing the same thing, so `ActorState` cannot
decide behaviour. `NpcSystem` chooses an action, performs it, and *derives*
the animation from it.

## The ladder

Each step, in priority order:

1. **Hit** — step aside, so as not to stand where the last shot landed.
2. **Wounded**, at or below a quarter health — route to the nearest tile the
   player cannot see, lie low four seconds, then come back.
3. **Player visible** — close the distance, or shoot if within
   `NPC_SOLDIER_ATTACK_DISTANCE`.
4. **Lost sight** — walk to where the player was last seen, look around.
5. **Otherwise** — wander to a random reachable tile, then pause.

Being hit or dying overrides all of it, and a soldier flinching from a hit
holds still until the flinch expires.

## Seeing the player

A cast from the soldier toward the player hits the player only if no wall
intervenes. The soldier turns to face the player **only when that cast
confirms line of sight** — turning first makes soldiers visibly track the
player through walls, which shows up on the minimap direction lines.

The cast is the dominant per-step AI cost, so it is throttled by distance,
because a stale verdict goes unnoticed for longer the further away the
player is:

| Distance | Steps between casts |
| --- | --- |
| within twice attack range | 1 |
| under 15 tiles | 4 |
| beyond | 12 |

The entity index offsets the phase, so soldiers sharing an interval spread
their casts across it instead of all firing on the same step. Measured over
20 soldiers this takes the fan from 20 casts per step to about 5, and halves
the time `NpcSystem::update` takes.

## Navigation

`navigation.rs` floods the walkable tiles outward from a soldier, breadth
first, and returns routes through them. One flood answers all three
questions the AI asks — somewhere to wander, the way to a remembered spot,
the nearest tile out of sight — so a soldier that re-plans pays for it once.

Steps are orthogonal only. A diagonal between two wall corners fits on the
grid but not around a 0.7-tile bounding box, and the soldier would catch on
the corner.

### Getting stuck

Soldiers are obstacles to each other, so a route planned around the walls
can still be blocked by a comrade standing in it. A soldier that covers no
ground for three quarters of a second abandons its route and plans another.

The timer resets whenever a route is issued. It has to: without that, a
soldier that has just finished a pause is compared against the last time it
moved — before the pause — and is judged stuck immediately, throwing away
every fresh route. That bug is invisible except as a statistic; it showed up
as soldiers spending 80% of their steps standing still.

## Dodging

A dodge is a sidestep, not a sprint: 1 to 2.5 tiles at 60% speed, with the
soldier still facing the player while it moves. Movement direction and
facing are separate components and only looked joined.

This matters more than it sounds. An earlier version picked from the
sixteen nearest tiles past 1.5 away — four or more tiles in open ground —
and ran there at full speed. A soldier standing 2.2 tiles away ended up 4.9
tiles away after one hit, and at longer range that carries it behind cover
and out of the fight.

## Friendly fire

A soldier's shot can hit another soldier standing in the line of fire: the
cast excludes only the shooter. This is deliberate, and it matters more now
that soldiers move and cross each other's lines. Positioning yourself so
they thin each other out is a legitimate tactic.

## Death

A dead soldier loses its `NpcTag` and its `BoundingBox` — it stops being
updated, stops blocking movement, and can no longer be shot — and keeps its
sprite showing the death animation.

The animation is applied directly rather than through `replace_actor_state`,
because `state_if_damaged` has already written the new `ActorState` by then.
Asking `replace_actor_state` to change it reports "no change" and silently
skips the sprite and the sound. When that happened, a killed soldier went on
standing in its walk pose: solid-looking, unshootable, walk-through. It
reads exactly like an invulnerability bug and is not one.

## Tuning

All in `npc.rs`:

| Constant | | |
| --- | --- | --- |
| `NPC_SOLDIER_ATTACK_DISTANCE` | 5.0 | closes to here, then shoots |
| `NPC_SOLDIER_CRITICAL_HEALTH` | 25 | quarter health; breaks off below it |
| `NPC_SOLDIER_HIDE_FRAMES` | 240 | four seconds out of sight |
| `NPC_SOLDIER_WANDER_PAUSE` | 30..150 | half a second to two and a half |
| `NPC_SOLDIER_DODGE_RANGE` | 1.0..2.5 | tiles per sidestep |
| `NPC_SOLDIER_DODGE_SPEED` | 0.6 | fraction of running speed |
| `NPC_NAV_FLOOD_CELLS` | 400 | tiles one re-plan considers |

Note that the shotgun does 27 and a soldier has 100, so four hits kill and
the third leaves it on 19 — under critical. Every soldier therefore breaks
off exactly once, just before the hit that would finish it. That is the
intended drama, but it means the two numbers are coupled: change the
shotgun's damage and the retreat moves to a different hit.

## See also

- [Architecture](architecture.md) — systems, storage and the step loop
- [Maze generator](https://github.com/SergeyChelak/cellular_automata)
