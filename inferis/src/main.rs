use engine::prelude::*;

const SCENE_MAIN: &str = "game_scene";

fn main() -> EcsResult<()> {
    let mut world = GameWorld::new();

    let game_scene = GameScene::new();
    world.register_scene(SCENE_MAIN.to_string(), game_scene);

    world.change_scene(SCENE_MAIN.to_string());
    world.run();
    Ok(())
}
