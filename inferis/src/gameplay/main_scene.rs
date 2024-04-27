use engine::*;

use crate::{pbm::PBMImage, resource::*};

use self::{
    attack::attack_system, npc::npc_update, state::state_system, transform::transform_entities,
};

use super::{controller::ControllerState, input::*, renderer::*, *};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
    player_id: EntityID,
    maze_id: EntityID,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        let mut storage = game_play_component_storage()?;
        let player_id = storage.append(&bundle_player(Vec2f::new(5.0, 10.0)));
        let maze_id = storage.append(&bundle_maze()?);
        // decorations
        storage.append(&bundle_torch(TorchStyle::Green, Vec2f::new(1.2, 12.9)));
        storage.append(&bundle_torch(TorchStyle::Green, Vec2f::new(1.2, 4.1)));
        storage.append(&bundle_torch(TorchStyle::Red, Vec2f::new(1.2, 9.0)));
        storage.append(&bundle_sprite(WORLD_CANDELABRA, Vec2f::new(8.8, 2.8)));
        // npc
        storage.append(&bundle_npc_soldier(Vec2f::new(8.0, 10.0)));
        storage.append(&bundle_npc_soldier(Vec2f::new(27.0, 13.8)));
        storage.append(&bundle_npc_soldier(Vec2f::new(40.0, 8.0)));
        Ok(Self {
            storage,
            controller: ControllerState::default(),
            player_id,
            maze_id,
        })
    }
}

impl Scene for GameScene {
    fn id(&self) -> SceneID {
        SCENE_GAME_PLAY
    }

    fn process_events(&mut self, events: &[InputEvent]) -> EngineResult<()> {
        self.controller.update(events);
        Ok(())
    }

    fn run_systems(&mut self, engine: &mut dyn Engine) -> EngineResult<()> {
        let delta_time = engine.delta_time();
        user_input_system(
            &mut self.storage,
            &self.controller,
            delta_time,
            self.player_id,
        )?;
        npc_update(&mut self.storage, delta_time, self.player_id, self.maze_id)?;
        transform_entities(&mut self.storage)?;
        attack_system(&mut self.storage)?;
        state_system(&mut self.storage)?;
        Ok(())
    }

    fn render_scene(&mut self, engine: &mut dyn Engine, assets: &AssetManager) -> EngineResult<()> {
        let mut renderer = Renderer::new(
            &mut self.storage,
            engine,
            assets,
            self.player_id,
            self.maze_id,
        );
        renderer.render()
    }
}

// temporary producer functions
fn bundle_player(position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(PlayerTag)
        .put(weapon(27, 20, 100))
        .put(CharacterState::Idle(FrameDuration::infinite()))
        .put(Health(100))
        .put(Velocity(7.0))
        .put(RotationSpeed(2.5))
        .put(Position(position))
        .put(Angle(0.0))
        .put(TextureID(PLAYER_SHOTGUN.to_string()))
}

fn bundle_maze() -> EngineResult<EntityBundle> {
    let image = PBMImage::with_file("assets/level.pbm")
        .map_err(|err| EngineError::MazeGenerationFailed(err.to_string()))?;
    let array = image.transform_to_array(|x| x as i32);
    Ok(EntityBundle::new().put(Maze(array)))
}

enum TorchStyle {
    Green,
    Red,
}

fn bundle_torch(style: TorchStyle, position: Vec2f) -> EntityBundle {
    let animation_id = match style {
        TorchStyle::Green => WORLD_TORCH_GREEN_ANIM,
        TorchStyle::Red => WORLD_TORCH_RED_ANIM,
    }
    .to_string();
    EntityBundle::new()
        .put(AnimationData::new(animation_id))
        .put(Position(position))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(SpriteTag)
}

fn bundle_sprite(texture_id: &str, position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(TextureID(texture_id.to_string()))
        .put(Position(position))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(SpriteTag)
}

fn bundle_npc_soldier(position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(SpriteTag)
        .put(weapon(10, 20, usize::MAX))
        .put(Position(position))
        .put(NpcTag)
        .put(CharacterState::Idle(FrameDuration::infinite()))
        .put(Health(100))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(BoundingBox(SizeFloat::new(0.7, 0.7)))
}

fn weapon(damage: HealthType, recharge_time: usize, ammo_count: usize) -> Weapon {
    Weapon {
        damage,
        recharge_time,
        state: WeaponState::Ready,
        ammo_count,
    }
}
