use sdl2::{event::Event, keyboard::Keycode, AudioSubsystem, EventPump, VideoSubsystem};
use std::collections::HashMap;

use crate::assets::AssetManager;

use super::{Engine, Scene, SceneID};

pub struct GameWorld {
    assets: AssetManager,
    is_running: bool,
    scenes: HashMap<SceneID, Box<dyn Scene>>,
    current_scene: SceneID,

    frame_counter: u64,
    event_pump: EventPump,
    video_subsystem: VideoSubsystem,
    audio_subsystem: AudioSubsystem,
}

impl GameWorld {
    pub fn new() -> Self {
        let sdl_context = sdl2::init()
            //.map_err(|err| EngineError::SDLError(err))?;
            .expect("sdl init error");
        let video_subsystem = sdl_context.video().expect("video_subsystem error");
        // .map_err(|err| EngineError::SDLError(err))?;
        let audio_subsystem = sdl_context
            .audio()
            // .map_err(|err| EngineError::SDLError(err))?;
            .expect("audio_subsystem error");
        let event_pump = sdl_context
            .event_pump()
            // .map_err(|err| EngineError::SDLError(err))?;
            .expect("event_pump error");
        Self {
            assets: AssetManager::new(),
            is_running: false,
            scenes: HashMap::default(),
            current_scene: SceneID::default(),
            frame_counter: 0,
            event_pump,
            video_subsystem,
            audio_subsystem,
        }
    }
    /// main game loop
    pub fn run(&mut self) {
        self.is_running = true;
        while self.is_running {
            todo!()
        }
    }
}

impl Engine for GameWorld {
    fn change_scene(&mut self, scene_id: SceneID) {
        self.current_scene = scene_id;
    }

    fn terminate(&mut self) {
        self.is_running = false;
    }
}
