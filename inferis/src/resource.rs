use crate::gameplay::HealthType;

// scenes
pub const SCENE_GAME_PLAY: &str = "game_scene";

// world
pub const WORLD_WALL1: &str = "wall1";
pub const WORLD_WALL2: &str = "wall2";
pub const WORLD_WALL3: &str = "wall3";
pub const WORLD_WALL4: &str = "wall4";
pub const WORLD_WALL5: &str = "wall5";
pub const WORLD_SKY: &str = "sky";
pub const WORLD_FLOOR_GRADIENT: &str = "floor_grad";
pub const WORLD_TORCH_RED_ANIM: &str = "anim_torch_red";
pub const WORLD_TORCH_GREEN_ANIM: &str = "anim_torch_green";
pub const WORLD_CANDELABRA: &str = "candelabra";
pub const WORLD_GAME_OVER: &str = "game_over";

// player
pub const PLAYER_SHOTGUN_SHOT_ANIM: &str = "anim_shotgun_shot";
pub const PLAYER_SHOTGUN_IDLE_ANIM: &str = "anim_shotgun_idle";
pub const PLAYER_SHOTGUN_DAMAGE: HealthType = 27;
pub const PLAYER_PLAYER_DAMAGE_COLOR: &str = "player_damage_clr";

// npc soldier
pub const NPC_SOLDIER_IDLE: &str = "anim_soldier_idle";
pub const NPC_SOLDIER_ATTACK: &str = "anim_soldier_attack";
pub const NPC_SOLDIER_DEATH: &str = "anim_soldier_death";
pub const NPC_SOLDIER_DAMAGE: &str = "anim_soldier_pain";
pub const NPC_SOLDIER_WALK: &str = "anim_soldier_walk";
