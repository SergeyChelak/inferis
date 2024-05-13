use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

use sdl2::{event::Event, mixer::InitFlag, pixels::Color, render::WindowCanvas, EventPump, Sdl};

use crate::{
    systems::InputEvent, AssetManager, AudioSettings, EngineError, EngineResult, EngineSettings,
    WindowSettings,
};

use super::{
    game_scene::GameScene,
    systems::{GameSystemCommand, RendererEffect, RendererLayersPtr, SoundEffect},
};

const TARGET_FPS: u128 = 60;

pub struct GameWorld {
    scenes: HashMap<String, GameScene>,
    current_scene: &'static str,
    is_running: bool,
}

#[derive(Default)]
pub struct GameWorldBuilder {
    scenes: HashMap<String, GameScene>,
}

impl GameWorldBuilder {
    pub fn new() -> Self {
        Self::default()
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
        render_effects(&mut canvas, &asset_manager, effects)?;
        let sound_effects = scene.sound_effects(&asset_manager)?;
        play_sound_effects(&sound_effects, &asset_manager)?;
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
            Event::Quit { .. } => {
                std::process::exit(0);
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

fn play_sound_effects(effects: &[SoundEffect], asset_manager: &AssetManager) -> EngineResult<()> {
    for effect in effects {
        match effect {
            SoundEffect::PlaySound { asset_id, loops } => {
                if let Some(chunk) = asset_manager.sound_chunk(asset_id) {
                    sdl2::mixer::Channel::all()
                        .play(chunk, *loops)
                        .map_err(EngineError::sdl)?;
                }
            }
        }
    }
    Ok(())
}

fn render_effects(
    canvas: &mut WindowCanvas,
    asset_manager: &AssetManager,
    layers_ptr: RendererLayersPtr,
) -> EngineResult<()> {
    let mut layers = layers_ptr.borrow_mut();
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    for effect in &layers.background {
        render_effect(canvas, asset_manager, effect)?;
    }

    layers
        .depth
        .sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(Ordering::Equal));
    for depth_effect in &layers.depth {
        render_effect(canvas, asset_manager, &depth_effect.effect)?;
    }

    for effect in &layers.hud {
        render_effect(canvas, asset_manager, effect)?;
    }
    canvas.present();
    Ok(())
}

fn render_effect(
    canvas: &mut WindowCanvas,
    asset_manager: &AssetManager,
    effect: &RendererEffect,
) -> EngineResult<()> {
    use RendererEffect::*;
    match effect {
        Texture {
            asset_id,
            source,
            destination,
        } => {
            let Some(texture) = asset_manager.texture(asset_id) else {
                let msg = format!("[run_loop] texture not found {}", asset_id);
                return Err(EngineError::TextureNotFound(msg));
            };
            canvas
                .copy(texture, *source, *destination)
                .map_err(EngineError::sdl)
        }
        Line { color, begin, end } => {
            canvas.set_draw_color(*color);
            canvas.draw_line(*begin, *end).map_err(EngineError::sdl)
        }
        Rectangle {
            color,
            fill,
            blend_mode,
            rect,
        } => {
            canvas.set_blend_mode(*blend_mode);
            canvas.set_draw_color(*color);
            if *fill {
                canvas.fill_rect(*rect)
            } else {
                canvas.draw_rect(*rect)
            }
            .map_err(EngineError::sdl)
        }

        Rectangles {
            color,
            fill,
            blend_mode,
            rects,
        } => {
            canvas.set_blend_mode(*blend_mode);
            canvas.set_draw_color(*color);
            if *fill {
                canvas.fill_rects(rects)
            } else {
                canvas.draw_rects(rects)
            }
            .map_err(EngineError::sdl)
        }
    }
}
