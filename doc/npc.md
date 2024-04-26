# NPC

## Player detection
NPC can detect player by applying a ray caster. We can say that NPC can _see_ that player if there is no obstacles crossing segment that connects NPC and player positions. The ray direction could be calculated this way
```rust
let Some(position) = self.storage.get_mut::<Position>(npc_id).map(|x| x.0) else {
    panic!("Doesn't matter in the snippet")
};
let vector = self.player_pos - position;
let angle = delta.y.atan2(delta.x);
```

## Pursuit & Attack
Let's define these rules for pursuit & attack state for NPC
- Required condition: NPC can pursuit or attack player if he is visible (see the player detection above)
- if the euclidean distance between player and NPC is less that some _delta_ NPC must attack otherwise NPC must pursuit the player