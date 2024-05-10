use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use sdl2::{event::Event, mixer::InitFlag, pixels::Color, render::WindowCanvas, EventPump, Sdl};

use crate::{
    game_scene::GameScene,
    systems::{GameSystemCommand, RendererEffect},
    world, AssetManager, AudioSettings, EngineError, EngineResult, EngineSettings, InputEvent,
    WindowSettings,
};

const TARGET_FPS: u128 = 60;

pub struct GameWorld {
    scenes: HashMap<String, GameScene>,
    current_scene: &'static str,
    is_running: bool,
}

pub struct GameWorldBuilder {
    scenes: HashMap<String, GameScene>,
}

impl GameWorldBuilder {
    pub fn new() -> Self {
        Self {
            scenes: Default::default(),
        }
    }

    pub fn with_scene(mut self, scene: GameScene) -> Self {
        let id = scene.id();
        self.scenes.insert(id.to_string(), scene);
        self
    }

    pub fn build(self, initial_scene_id: &'static str) -> GameWorld {
        GameWorld {
            scenes: self.scenes,
            current_scene: initial_scene_id,
            is_running: false,
        }
    }
}

pub fn start(mut world: GameWorld, settings: EngineSettings) -> EngineResult<()> {
    // setup media layer components
    let sdl = sdl2::init().map_err(EngineError::Sdl)?;
    let mut canvas = setup_canvas(&sdl, &settings.window)?;

    // TODO: show loading splash texture

    // audio should be initialized before asset manager will unpack its items
    setup_audio(&sdl, &settings.audio_setting)?;
    let texture_creator = canvas.texture_creator();
    let mut asset_manager = AssetManager::default();
    asset_manager.setup(&settings.asset_source, &texture_creator)?;

    let mut event_pump = sdl.event_pump().map_err(EngineError::Sdl)?;

    // setup all scenes
    for (_, scene) in world.scenes.iter_mut() {
        scene.setup_systems(&asset_manager, settings.window.size)?;
    }

    let target_duration = 1000 / TARGET_FPS;
    let mut time = Instant::now();
    world.is_running = true;
    while world.is_running {
        let frame_start = Instant::now();
        let Some(scene) = world.scenes.get_mut(world.current_scene) else {
            return Err(EngineError::SceneNotFound);
        };
        let events = get_events(&mut event_pump);
        scene.push_events(&events)?;
        let delta_time = time.elapsed().as_secs_f32();
        let commands = scene.update(delta_time, &asset_manager)?;
        commands.iter().for_each(|cmd| {
            use GameSystemCommand::*;
            match cmd {
                Terminate => world.is_running = false,
                SwitchScene(id) => world.current_scene = id,
                _ => {}
            }
        });
        let effects = scene.render(&asset_manager)?;
        render_effects(&mut canvas, &asset_manager, &effects);
        time = Instant::now();
        // delay the rest of the time if needed
        let suspend_ms = target_duration.saturating_sub(frame_start.elapsed().as_millis());
        if suspend_ms > 0 {
            let duration = Duration::from_millis(suspend_ms as u64);
            std::thread::sleep(duration);
        }
    }
    Ok(())
}

fn setup_canvas(sdl: &Sdl, window_settings: &WindowSettings) -> EngineResult<WindowCanvas> {
    let video_subsystem = sdl.video().map_err(EngineError::Sdl)?;
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

fn setup_audio(sdl: &Sdl, settings: &AudioSettings) -> EngineResult<()> {
    _ = sdl.audio().map_err(EngineError::Sdl)?;
    sdl2::mixer::open_audio(
        settings.frequency,
        settings.format,
        settings.channels,
        settings.chunk_size,
    )
    .map_err(EngineError::Sdl)?;
    sdl2::mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG)
        .map_err(EngineError::Sdl)?;
    sdl2::mixer::allocate_channels(settings.mixing_channels);
    Ok(())
}

fn get_events(event_pump: &mut EventPump) -> Vec<InputEvent> {
    let mut events = Vec::new();
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => events.push(InputEvent::Quit),
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

/*
   // setup general purpose effect handler
   let effect_handler = |effects: &[SoundEffect]| -> EngineResult<()> {
       for effect in effects {
           use SoundEffect::*;
           match effect {
               PlaySound { asset_id, loops } => {
                   if let Some(chunk) = asset_manager.sound_chunk(asset_id) {
                       sdl2::mixer::Channel::all()
                           .play(chunk, *loops)
                           .map_err(EngineError::sdl)?;
                   }
               }
           }
       }
       Ok(())
   };
*/

fn render_effects(
    canvas: &mut WindowCanvas,
    asset_manager: &AssetManager,
    effects: &[RendererEffect],
) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    for effect in effects {
        //
    }
    canvas.present();
}
