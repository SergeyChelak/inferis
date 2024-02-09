use sdl2::{event::Event, keyboard::Keycode, AudioSubsystem, EventPump, VideoSubsystem};

use super::{config::Config, scene::Scene, EngineError, EngineResult};

pub struct World {
    scenes: Vec<Scene>,
    config: Config,
    event_pump: EventPump,
    video_subsystem: VideoSubsystem,
    audio_subsystem: AudioSubsystem,
    is_running: bool,
}

impl World {
    pub fn new(config: Config, scenes: Vec<Scene>) -> EngineResult<Self> {
        let sdl_context = sdl2::init().map_err(|err| EngineError::SDLError(err))?;
        let video_subsystem = sdl_context
            .video()
            .map_err(|err| EngineError::SDLError(err))?;
        let audio_subsystem = sdl_context
            .audio()
            .map_err(|err| EngineError::SDLError(err))?;
        let event_pump = sdl_context
            .event_pump()
            .map_err(|err| EngineError::SDLError(err))?;
        Ok(Self {
            scenes,
            config,
            event_pump,
            video_subsystem,
            audio_subsystem,
            is_running: false,
        })
    }

    pub fn run(&mut self) -> EngineResult<()> {
        let window = self
            .video_subsystem
            .window(
                &self.config.window_title,
                self.config.resolution.width,
                self.config.resolution.height,
            )
            .build()
            .map_err(|err| EngineError::SDLError(err.to_string()));

        self.is_running = true;
        while self.is_running {
            let Some(scene) = self.current_scene() else {
                panic!("No scenes");
            };
            scene.update()?;
            self.handle_events();
        }
        Ok(())
    }

    fn handle_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.is_running = false,
                _ => {}
            }
        }
    }

    fn current_scene(&mut self) -> Option<&mut Scene> {
        self.scenes.get_mut(0)
    }
}
