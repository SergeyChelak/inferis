use sdl2::{event::Event, keyboard::Keycode, render::WindowCanvas, AudioSubsystem, EventPump, Sdl};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    assets::AssetManager,
    settings::{EngineSettings, WindowSettings},
    EngineError, EngineResult, SizeU32,
};

use super::{Engine, InputEvent, Scene, SceneID};

const TARGET_FPS: u128 = 60;

pub struct GameWorld {
    is_running: bool,
    scenes: HashMap<SceneID, Rc<RefCell<dyn Scene>>>,
    current_scene: SceneID,
    time: Instant,
    event_pump: EventPump,
    _audio_subsystem: AudioSubsystem,
    canvas: WindowCanvas,
    settings: EngineSettings,
}

impl GameWorld {
    pub fn new(settings: EngineSettings) -> EngineResult<Self> {
        let sdl_context = sdl2::init().map_err(EngineError::Sdl)?;
        let canvas = Self::canvas(&sdl_context, &settings.window)?;
        let audio_subsystem = sdl_context.audio().map_err(EngineError::Sdl)?;
        let event_pump = sdl_context.event_pump().map_err(EngineError::Sdl)?;
        Ok(Self {
            is_running: false,
            scenes: HashMap::default(),
            current_scene: SceneID::default(),
            time: Instant::now(),
            event_pump,
            canvas,
            _audio_subsystem: audio_subsystem,
            settings,
        })
    }

    fn canvas(sdl_context: &Sdl, window_settings: &WindowSettings) -> EngineResult<WindowCanvas> {
        let video_subsystem = sdl_context.video().map_err(EngineError::Sdl)?;
        let size = &window_settings.size;
        let window = video_subsystem
            .window(&window_settings.title, size.width, size.height)
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

    pub fn register_scene<T: Scene + 'static>(&mut self, scene: T) {
        let scene_id = scene.id();
        self.scenes.insert(scene_id, Rc::new(RefCell::new(scene)));
    }

    pub fn run(&mut self) -> EngineResult<()> {
        let texture_creator = self.canvas.texture_creator();
        let asset_manager = AssetManager::new(&self.settings.asset_path, &texture_creator)?;
        self.is_running = true;
        let target_duration = 1000 / TARGET_FPS;
        self.time = Instant::now();
        while self.is_running {
            let frame_start = Instant::now();
            let Some(scene_ref) = self.current_scene_ref() else {
                return Err(EngineError::SceneNotFound);
            };
            let mut scene = scene_ref.borrow_mut();
            let events = self.input_events();
            scene.teak(self, &events, &asset_manager)?;
            self.time = Instant::now();
            // delay the rest of the time if needed
            let suspend_ms = target_duration.saturating_sub(frame_start.elapsed().as_millis());
            if suspend_ms > 0 {
                let duration = Duration::from_millis(suspend_ms as u64);
                std::thread::sleep(duration);
            }
        }
        Ok(())
    }

    fn input_events(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.is_running = false;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    events.push(InputEvent::Keyboard {
                        code: keycode,
                        pressed: true,
                    });
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    events.push(InputEvent::Keyboard {
                        code: keycode,
                        pressed: false,
                    });
                }
                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    events.push(InputEvent::Mouse {
                        x,
                        y,
                        x_rel: xrel,
                        y_rel: yrel,
                    });
                }
                _ => {}
            }
        }
        events
    }

    fn current_scene_ref(&self) -> Option<Rc<RefCell<dyn Scene>>> {
        self.scenes.get(&self.current_scene).cloned()
    }
}

impl Engine for GameWorld {
    fn terminate(&mut self) {
        self.is_running = false;
    }

    fn change_scene(&mut self, scene_id: SceneID) {
        self.current_scene = scene_id;
    }

    fn canvas(&mut self) -> &mut WindowCanvas {
        &mut self.canvas
    }

    fn delta_time(&self) -> f32 {
        self.time.elapsed().as_secs_f32()
    }

    fn window_size(&self) -> SizeU32 {
        self.settings.window.size
    }
}
