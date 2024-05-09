use std::f32::consts::PI;

use engine::{
    frame_counter::AggregatedFrameCounter,
    game_scene::{GameRendererSystem, RendererEffect},
    EngineError, EntityID, Float, SizeU32,
};

use crate::game_scene::fetch_player_id;

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

#[derive(Default)]
pub struct RendererSystem {
    player_id: EntityID,
    window_size: SizeU32,
    rays_count: u32,
    ray_angle_step: Float,
    scale: Float,
    screen_distance: Float,
}

impl RendererSystem {
    pub fn new() -> Self {
        Self::default()
    }
}

impl GameRendererSystem for RendererSystem {
    fn setup(
        &mut self,
        storage: &engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
        window_size: engine::SizeU32,
    ) -> engine::EngineResult<()> {
        println!("[v2.renderer] setup");
        let Some(player_id) = fetch_player_id(storage) else {
            return Err(EngineError::unexpected_state(
                "[v2.renderer] player entity not found",
            ));
        };
        self.player_id = player_id;
        // precalculated values
        self.rays_count = window_size.width >> 1;
        self.ray_angle_step = FIELD_OF_VIEW / self.rays_count as Float;
        self.scale = window_size.width as Float / self.rays_count as Float;
        self.screen_distance = (window_size.width >> 1) as Float / HALF_FIELD_OF_VIEW.tan();
        Ok(())
    }

    fn render(
        &mut self,
        frame_counter: &mut AggregatedFrameCounter,
        storage: &engine::ComponentStorage,
        assets: &engine::AssetManager,
    ) -> engine::EngineResult<Vec<RendererEffect>> {
        Ok(vec![])
    }
}
