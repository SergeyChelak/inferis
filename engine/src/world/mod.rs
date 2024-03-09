pub mod game_world;
pub mod scene;

type SceneID = String;
pub trait Engine {
    fn change_scene(&mut self, scene_id: SceneID);
    fn terminate(&mut self);
}

pub trait Scene {
    fn update(&mut self, engine: &mut dyn Engine);
    fn render(&self, engine: &dyn Engine);
}
