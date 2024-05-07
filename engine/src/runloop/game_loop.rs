use sdl2::{
    event::Event,
    keyboard::Keycode,
    mixer::{Chunk, InitFlag},
    pixels::Color,
    render::WindowCanvas,
    EventPump, Sdl,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    assets::AssetManager,
    settings::{EngineSettings, WindowSettings},
    AudioSettings, EngineError, EngineResult, SizeU32,
};

use super::{Engine, InputEvent, Scene, SceneID};

const TARGET_FPS: u128 = 60;

pub struct GameLoop {
    is_running: bool,
    scenes: HashMap<SceneID, Rc<RefCell<dyn Scene>>>,
    current_scene: SceneID,
    time: Instant,
    event_pump: EventPump,
    canvas: WindowCanvas,
    settings: EngineSettings,
    audio_enabled: bool,
}

impl GameLoop {
    pub fn new(settings: EngineSettings) -> EngineResult<Self> {
        let sdl_context = sdl2::init().map_err(EngineError::Sdl)?;
        let canvas = Self::canvas(&sdl_context, &settings.window)?;
        let event_pump = sdl_context.event_pump().map_err(EngineError::Sdl)?;
        let audio_enabled = Self::setup_audio(&sdl_context, &settings.audio_setting)
            .map_err(EngineError::Sdl)
            .is_ok();
        Ok(Self {
            is_running: false,
            scenes: HashMap::default(),
            current_scene: SceneID::default(),
            time: Instant::now(),
            event_pump,
            canvas,
            settings,
            audio_enabled,
        })
    }

    fn setup_audio(sdl: &Sdl, settings: &AudioSettings) -> Result<(), String> {
        _ = sdl.audio()?;
        sdl2::mixer::open_audio(
            settings.frequency,
            settings.format,
            settings.channels,
            settings.chunk_size,
        )?;
        _ = sdl2::mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG)?;
        sdl2::mixer::allocate_channels(settings.mixing_channels);
        Ok(())
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
        let mut asset_manager = AssetManager::default();
        asset_manager.setup(&self.settings.asset_source, &texture_creator)?;
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
            scene.process_events(&events)?;
            scene.run_systems(self, &asset_manager)?;
            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();
            scene.render_scene(self, &asset_manager)?;
            self.canvas.present();
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

impl Engine for GameLoop {
    fn terminate(&mut self) {
        self.is_running = false;
    }

    fn change_scene(&mut self, scene_id: SceneID) {
        self.current_scene = scene_id;
    }

    fn canvas(&mut self) -> &mut WindowCanvas {
        &mut self.canvas
    }

    fn play_sound(&self, sound_chunk: &Chunk, loops: i32) -> EngineResult<()> {
        if self.audio_enabled {
            sdl2::mixer::Channel::all()
                .play(sound_chunk, loops)
                .map_err(EngineError::sdl)?;
        }
        Ok(())
    }

    fn delta_time(&self) -> f32 {
        self.time.elapsed().as_secs_f32()
    }

    fn window_size(&self) -> SizeU32 {
        self.settings.window.size
    }
}
