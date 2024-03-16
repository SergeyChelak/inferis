use engine::prelude::{
    settings::{WindowSettings, WindowSize},
    *,
};

const SCENE_MAIN: &str = "game_scene";

fn main() -> EngineResult<()> {
    let settings = WindowSettings {
        title: "INFERIS".to_string(),
        size: WindowSize {
            width: 800,
            height: 600,
        },
    };
    let mut world = GameWorld::new(settings)?;

    let game_scene = GameScene::new();
    world.register_scene(SCENE_MAIN.to_string(), game_scene);

    world.change_scene(SCENE_MAIN.to_string());
    world.run();
    Ok(())
}
