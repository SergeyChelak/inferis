use crate::gameplay::HealthType;

// files
pub const FILE_ASSET_REGISTRY: &str = "assets/asset_registry.txt";
pub const FILE_ASSET_BUNDLE: &str = "inferis.bin";

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
pub const WORLD_LEVEL_BASIC: &str = "level_basic";

// player
pub const PLAYER_SHOTGUN_SHOT_ANIM: &str = "anim_shotgun_shot";
pub const PLAYER_SHOTGUN_IDLE_ANIM: &str = "anim_shotgun_idle";
pub const PLAYER_SHOTGUN_DAMAGE: HealthType = 27;
pub const PLAYER_PLAYER_DAMAGE_COLOR: &str = "player_damage_clr";
pub const PLAYER_SHOTGUN_RECHARGE_FRAMES: usize = 45;
pub const PLAYER_SHOT_DEADLINE: usize = 3;
pub const PLAYER_DAMAGE_DEADLINE: usize = 5;

// npc soldier
pub const NPC_SOLDIER_IDLE: &str = "anim_soldier_idle";
pub const NPC_SOLDIER_ATTACK: &str = "anim_soldier_attack";
pub const NPC_SOLDIER_DEATH: &str = "anim_soldier_death";
pub const NPC_SOLDIER_DAMAGE: &str = "anim_soldier_pain";
pub const NPC_SOLDIER_WALK: &str = "anim_soldier_walk";
pub const NPC_SOLDIER_DAMAGE_RECOVER: usize = 15;

// sound
pub const SOUND_PLAYER_ATTACK: &str = "sound_player_attack";
pub const SOUND_PLAYER_PAIN: &str = "sound_player_pain";
pub const SOUND_NPC_ATTACK: &str = "sound_npc_attack";
pub const SOUND_NPC_DEATH: &str = "sound_npc_death";
pub const SOUND_NPC_PAIN: &str = "sound_npc_pain";
