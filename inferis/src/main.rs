use engine::prelude::*;
use scene::GameScene;
mod scene;

const WINDOW_TITLE: &str = "INFERIS";
const SCENE_MAIN: &str = "game_scene";

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
    world.register_scene(SCENE_MAIN.to_string(), game_scene);

    world.change_scene(SCENE_MAIN.to_string());
    world.run()
}
