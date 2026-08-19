use super::{
    game_scene::GameScene,
    systems::{GameSystemCommand, RendererEffect, RendererLayersPtr, SoundEffect},
};
use crate::{
    game_scene::SceneEvent, systems::InputEvent, AssetManager, AudioSettings, EngineError,
    EngineResult, EngineSettings, SceneID, WindowSettings,
};
use log::warn;
use sdl2::{event::Event, mixer::InitFlag, pixels::Color, render::WindowCanvas, EventPump, Sdl};
use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

const TARGET_FPS: u64 = 60;
/// How much simulated time one gameplay step covers. Gameplay advances in
/// whole steps of this size, so the frame-based deadlines the game systems
/// use (weapon recharge, damage recovery, animation) always mean the same
/// amount of real time -- vsync at any refresh rate only changes how often
/// the world is drawn, never how fast it runs.
const FIXED_STEP: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);
const FIXED_STEP_SECS: f32 = 1.0 / TARGET_FPS as f32;
/// Upper bound on how much simulated time one iteration may catch up on. A
/// long stall -- level generation, a dragged window, a breakpoint -- would
/// otherwise queue a burst of steps and fast-forward the game.
const MAX_CATCH_UP: Duration = Duration::from_micros(5 * 1_000_000 / TARGET_FPS);
/// Upper bound on the render rate. Vsync usually paces the loop; this only
/// matters when it is unavailable or the display refreshes faster.
const FRAME_DURATION: Duration = FIXED_STEP;

#[derive(Default)]
pub struct GameWorld {
    scenes: Vec<GameScene>,
}

impl GameWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_scene(mut self, scene: GameScene) -> Self {
        self.scenes.push(scene);
        self
    }

    pub fn start(self, settings: EngineSettings) -> EngineResult<()> {
        let systems = SDLSystems::setup(&settings)?;
        let current_scene = self.scenes.first().map(|x| x.id()).unwrap_or_default();
        let scenes = self
            .scenes
            .into_iter()
            .map(|x| (x.id(), x))
            .collect::<HashMap<SceneID, GameScene>>();
        run(systems, &settings, scenes, current_scene)
    }
}

struct SDLSystems {
    canvas: WindowCanvas,
    event_pump: EventPump,
}

impl SDLSystems {
    fn setup(settings: &EngineSettings) -> EngineResult<Self> {
        let sdl = sdl2::init().map_err(EngineError::Sdl)?;
        let canvas = Self::setup_canvas(&sdl, &settings.window)?;
        Self::setup_audio(&sdl, &settings.audio_setting)?;
        let event_pump = sdl.event_pump().map_err(EngineError::Sdl)?;
        Ok(SDLSystems { canvas, event_pump })
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
}

fn run(
    systems: SDLSystems,
    settings: &EngineSettings,
    mut scenes: HashMap<SceneID, GameScene>,
    mut current_scene: SceneID,
) -> EngineResult<()> {
    let mut canvas = systems.canvas;
    let mut event_pump = systems.event_pump;
    let texture_creator = canvas.texture_creator();
    let mut asset_manager = AssetManager::default();
    asset_manager.setup(&settings.asset_source, &texture_creator)?;
    // setup all scenes
    for scene in scenes.values_mut() {
        scene.setup_systems(&asset_manager, settings.window.size)?;
    }
    let mut last_time = Instant::now();
    let mut accumulator = Duration::ZERO;
    let mut is_running = true;
    let mut events = Vec::with_capacity(32);
    while is_running {
        let frame_start = Instant::now();
        accumulator += frame_start.duration_since(last_time).min(MAX_CATCH_UP);
        last_time = frame_start;

        events.clear();
        let quit_requested = get_events(&mut event_pump, &mut events);
        let commands = {
            let Some(scene) = scenes.get_mut(&current_scene) else {
                return Err(EngineError::SceneNotFound);
            };
            scene.push_events(&events)?;
            let mut commands = Vec::new();
            while accumulator >= FIXED_STEP {
                accumulator -= FIXED_STEP;
                let step_commands = scene.update(FIXED_STEP_SECS, &asset_manager)?;
                if !step_commands.is_empty() {
                    // the scene is being left or the game is ending: run the
                    // command before simulating this scene any further
                    commands = step_commands;
                    break;
                }
            }
            let effects = scene.render(&asset_manager)?;
            render_effects(&mut canvas, &asset_manager, effects)?;
            let sound_effects = scene.sound_effects(&asset_manager)?;
            play_sound_effects(&sound_effects, &asset_manager)?;
            commands
        };
        for cmd in commands {
            match cmd {
                GameSystemCommand::Terminate => is_running = false,
                GameSystemCommand::SwitchScene { id, params } => {
                    let Some(scene) = scenes.get_mut(&id) else {
                        return Err(EngineError::SceneNotFound);
                    };
                    scene.send_event(SceneEvent::Change, &params)?;
                    current_scene = id;
                    // the handler may have rebuilt the level; start the new
                    // scene on a clean clock rather than catching up on the
                    // time the switch itself took
                    accumulator = Duration::ZERO;
                    last_time = Instant::now();
                }
                _ => {}
            }
        }
        // leave the loop instead of calling process::exit so that
        // destructors run and SDL shuts down cleanly
        if quit_requested {
            is_running = false;
        }
        frame_delay(&frame_start);
    }
    Ok(())
}

/// Sleeps out the rest of the frame if the loop ran ahead of the render cap.
///
/// Precision here no longer has to be exact: whatever the OS scheduler adds
/// is absorbed by the fixed-step accumulator, so it costs a slightly uneven
/// render cadence rather than a change in game speed. That makes a plain
/// sleep preferable to burning a core on a spin-wait.
#[inline(always)]
fn frame_delay(frame_start: &Instant) {
    let elapsed = frame_start.elapsed();
    if elapsed < FRAME_DURATION {
        std::thread::sleep(FRAME_DURATION - elapsed);
    }
}

/// Polls pending SDL events into `events`.
/// Returns true if the application was asked to quit.
fn get_events(event_pump: &mut EventPump, events: &mut Vec<InputEvent>) -> bool {
    let mut quit_requested = false;
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                quit_requested = true;
                events.push(InputEvent::Quit);
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
    quit_requested
}

fn play_sound_effects(effects: &[SoundEffect], asset_manager: &AssetManager) -> EngineResult<()> {
    for effect in effects {
        match effect {
            SoundEffect::PlaySound { asset_id, loops } => {
                // a missing chunk is a content bug, reported the same way
                // render_effect reports a missing texture
                let Some(chunk) = asset_manager.sound_chunk(asset_id) else {
                    let msg = format!("[run_loop] sound chunk not found {}", asset_id);
                    return Err(EngineError::ResourceNotFound(msg));
                };
                // every mixing channel being busy is ordinary saturation in a
                // loud scene, not a reason to end the run: drop the sound
                if let Err(err) = sdl2::mixer::Channel::all().play(chunk, *loops) {
                    warn!("failed to play {}: {}", asset_id, err);
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
        .sort_unstable_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(Ordering::Equal));
    for depth_effect in &layers.depth {
        render_effect(canvas, asset_manager, &depth_effect.effect)?;
    }

    for effect in &layers.hud {
        render_effect(canvas, asset_manager, effect)?;
    }
    canvas.present();
    Ok(())
}

#[inline(always)]
fn render_effect(
    canvas: &mut WindowCanvas,
    asset_manager: &AssetManager,
    effect: &RendererEffect,
) -> EngineResult<()> {
    use RendererEffect::*;
    match effect {
        Texture {
            texture,
            source,
            destination,
        } => {
            let Some(texture) = asset_manager.texture(*texture) else {
                let msg = format!("[run_loop] unknown texture handle {:?}", texture);
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
