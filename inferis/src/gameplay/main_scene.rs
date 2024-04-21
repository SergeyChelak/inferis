use engine::*;

use crate::{pbm::PBMImage, resource::*};

use self::npc::npc_update;

use super::{collider::run_collider, controller::ControllerState, player::*, renderer::*, *};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
    player_id: EntityID,
    maze_id: EntityID,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        let mut storage = game_play_component_storage()?;
        let player_id = storage.add_from_bundle(&bundle_player(Vec2f::new(5.0, 10.0)));
        let maze_id = storage.add_from_bundle(&bundle_maze()?);
        // decorations
        storage.add_from_bundle(&bundle_torch(TorchStyle::Green, Vec2f::new(1.2, 12.9)));
        storage.add_from_bundle(&bundle_torch(TorchStyle::Green, Vec2f::new(1.2, 4.1)));
        storage.add_from_bundle(&bundle_torch(TorchStyle::Red, Vec2f::new(1.2, 9.0)));
        storage.add_from_bundle(&bundle_sprite(WORLD_CANDELABRA, Vec2f::new(8.8, 2.8)));
        // npc
        storage.add_from_bundle(&bundle_npc_soldier(Vec2f::new(8.0, 10.0)));
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
        player_update(
            &mut self.storage,
            &self.controller,
            delta_time,
            self.player_id,
            self.maze_id,
        )?;
        npc_update(&mut self.storage)?;
        run_collider(&mut self.storage, self.player_id, self.maze_id)?;
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
        .put(Health(100))
        .put(Velocity(7.0))
        .put(RotationSpeed(2.5))
        .put(Position(position))
        .put(PrevPosition(position))
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
        .put(AnimationData {
            frame_counter: 0,
            target_frames: usize::MAX,
            animation_id,
        })
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
        .put(Position(position))
        .put(NpcTag)
        .put(NpcState::Idle)
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(BoundingBox(SizeFloat::new(1.0, 1.0)))
}
