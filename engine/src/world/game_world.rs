use sdl2::{
    event::Event, keyboard::Keycode, render::WindowCanvas, AudioSubsystem, EventPump, Sdl,
    VideoSubsystem,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{assets::AssetManager, settings::WindowSettings, EngineError, EngineResult};

use super::{scene, Engine, Scene, SceneID};

const TARGET_FPS: u128 = 60;

pub struct GameWorld {
    assets: AssetManager,
    is_running: bool,
    scenes: HashMap<SceneID, Rc<RefCell<dyn Scene>>>,
    current_scene: SceneID,

    frame_counter: u64,
    event_pump: EventPump,
    audio_subsystem: AudioSubsystem,
    canvas: WindowCanvas,
}

impl GameWorld {
    pub fn new(settings: WindowSettings) -> EngineResult<Self> {
        let sdl_context = sdl2::init().map_err(|err| EngineError::Sdl(err))?;
        let size = settings.size;
        let canvas = Self::canvas(&sdl_context, &settings.title, size.width, size.height)?;
        let audio_subsystem = sdl_context.audio().map_err(|err| EngineError::Sdl(err))?;
        let event_pump = sdl_context
            .event_pump()
            .map_err(|err| EngineError::Sdl(err))?;
        Ok(Self {
            assets: AssetManager::new(),
            is_running: false,
            scenes: HashMap::default(),
            current_scene: SceneID::default(),
            frame_counter: 0,
            event_pump,
            canvas,
            audio_subsystem,
        })
    }

    fn canvas(
        sdl_context: &Sdl,
        title: &str,
        width: u32,
        height: u32,
    ) -> EngineResult<WindowCanvas> {
        let video_subsystem = sdl_context.video().map_err(|err| EngineError::Sdl(err))?;
        let window = video_subsystem
            .window(title, width, height)
            .position_centered()
            .build()
            .map_err(|op| EngineError::Sdl(op.to_string()))?;
        window
            .into_canvas()
            .accelerated()
            .target_texture()
            .present_vsync()
            .build()
            .map_err(|op| EngineError::Sdl(op.to_string()))
    }

    pub fn register_scene<T: Scene + 'static>(&mut self, scene_id: SceneID, scene: T) {
        self.scenes.insert(scene_id, Rc::new(RefCell::new(scene)));
    }

    pub fn run(&mut self) {
        self.is_running = true;
        let mut time = Instant::now();
        let target_duration = 1000 / TARGET_FPS;
        while self.is_running {
            let frame_start = Instant::now();
            let Some(scene_ref) = self.current_scene_ref() else {
                println!("[GameWorld] Can't get current scene");
                break;
            };
            let scene = scene_ref.borrow_mut();
            // TODO: process systems
            self.canvas.clear();
            scene.render(self);
            self.canvas.present();

            // delay the rest of the time if needed
            let elapsed = time.elapsed();
            if elapsed.as_millis() > 1000 {
                time = Instant::now();
            }
            let suspend_ms = target_duration.saturating_sub(frame_start.elapsed().as_millis());
            if suspend_ms > 0 {
                let duration = Duration::from_millis(suspend_ms as u64);
                std::thread::sleep(duration);
            }
        }
    }

    fn current_scene_ref(&self) -> Option<Rc<RefCell<dyn Scene>>> {
        self.scenes.get(&self.current_scene).map(|x| x.clone())
    }
}

impl Engine for GameWorld {
    fn terminate(&mut self) {
        self.is_running = false;
    }

    fn change_scene(&mut self, scene_id: SceneID) {
        self.current_scene = scene_id;
    }
}
