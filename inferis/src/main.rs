use engine::*;
mod gameplay;
use gameplay::main_scene::*;
mod pbm;
mod resource;

const WINDOW_TITLE: &str = "INFERIS";

fn main() -> EngineResult<()> {
    let settings = EngineSettings {
        asset_path: "asset_registry.txt".to_string(),
        window: WindowSettings {
            title: WINDOW_TITLE.to_owned(),
            size: SizeU32 {
                width: 1600,
                height: 900,
            },
        },
    };
    let mut world = GameWorld::new(settings)?;

    let game_scene = GameScene::new()?;
    let id = game_scene.id();
    world.register_scene(game_scene);
    world.change_scene(id);
    world.run()
}
