use engine::{prelude::*, world::Scene};
use game_scene::GameScene;
mod game_scene;

const WINDOW_TITLE: &str = "INFERIS";

fn main() -> EngineResult<()> {
    let settings = EngineSettings {
        asset_path: "assets/asset_registry.txt".to_string(),
        window: WindowSettings {
            title: WINDOW_TITLE.to_owned(),
            size: WindowSize {
                width: 1024,
                height: 768,
            },
        },
    };
    let mut world = GameWorld::new(settings)?;

    let game_scene = GameScene::new();
    let id = game_scene.id();
    world.register_scene(game_scene);
    world.change_scene(id);
    world.run()
}
