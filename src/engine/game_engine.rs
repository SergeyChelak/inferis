use sdl2::{event::Event, keyboard::Keycode, AudioSubsystem, EventPump, VideoSubsystem};

use super::{config::Config, EngineError, EngineResult, GameEngineContext};

pub struct GameEngine {
    frame_counter: u64,
    config: Config,
    event_pump: EventPump,
    video_subsystem: VideoSubsystem,
    audio_subsystem: AudioSubsystem,
    is_running: bool,
}

impl GameEngine {
    pub fn new(config: Config) -> EngineResult<Self> {
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
            frame_counter: 0,
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
            .map_err(|err| EngineError::SDLError(err.to_string()))?;

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .target_texture()
            .present_vsync()
            .build()
            .map_err(|err| EngineError::SDLError(err.to_string()))?;

        self.is_running = true;
        while self.is_running {
            self.handle_events();
            // TODO:
            // 1. get current scene
            // 2. update
            // 3. deliver events/user input
            // 4. run systems
            canvas.clear();
            // 5. render
            canvas.present();
            self.frame_counter += 1;
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
}

impl GameEngineContext for GameEngine {
    fn terminate(&mut self) {
        self.is_running = false;
    }

    fn screen_size(&self) -> crate::common::U32Size {
        self.config.resolution
    }

    fn frame_counter(&self) -> u64 {
        self.frame_counter
    }
}
