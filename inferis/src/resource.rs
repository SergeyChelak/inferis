use engine::SceneID;

// files
pub const FILE_ASSET_REGISTRY: &str = "assets/asset_registry.txt";
pub const FILE_ASSET_BUNDLE: &str = "inferis.bin";

// scenes
pub const SCENE_GAME_PLAY: SceneID = 1;
pub const SCENE_MAIN_MENU: SceneID = 2;

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
pub const WORLD_GAME_OVER: &str = "game_over";

// player
pub const PLAYER_SHOTGUN_SHOT_ANIM: &str = "anim_shotgun_shot";
pub const PLAYER_SHOTGUN_IDLE_ANIM: &str = "anim_shotgun_idle";
pub const PLAYER_PLAYER_DAMAGE_COLOR: &str = "player_damage_clr";

// npc soldier
pub const NPC_SOLDIER_IDLE: &str = "anim_soldier_idle";
pub const NPC_SOLDIER_ATTACK: &str = "anim_soldier_attack";
pub const NPC_SOLDIER_DEATH: &str = "anim_soldier_death";
pub const NPC_SOLDIER_DAMAGE: &str = "anim_soldier_pain";
pub const NPC_SOLDIER_WALK: &str = "anim_soldier_walk";

// sound
pub const SOUND_PLAYER_ATTACK: &str = "sound_player_attack";
pub const SOUND_PLAYER_PAIN: &str = "sound_player_pain";
pub const SOUND_NPC_ATTACK: &str = "sound_npc_attack";
pub const SOUND_NPC_DEATH: &str = "sound_npc_death";
pub const SOUND_NPC_PAIN: &str = "sound_npc_pain";

pub const MENU_BACKGROUND: &str = "menu_background";
pub const MENU_LABEL_CONTINUE: &str = "menu_lbl_continue";
pub const MENU_LABEL_EXIT: &str = "menu_lbl_exit";
pub const MENU_LABEL_GAME_OVER: &str = "menu_lbl_game_over";
pub const MENU_LABEL_NEW_GAME: &str = "menu_lbl_new_game";
pub const MENU_LABEL_PRESS_FIRE: &str = "menu_lbl_press_fire";
pub const MENU_LABEL_WIN: &str = "menu_lbl_win";
