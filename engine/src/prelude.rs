//! Types of the underlying backend (SDL2) that leak into the engine's public
//! API: they appear in [`crate::systems::InputEvent`],
//! [`crate::systems::RendererEffect`] and [`crate::settings::AudioSettings`],
//! so games have to name them.
//!
//! This module is the whole engine/SDL boundary. Nothing else of SDL is
//! re-exported, so swapping the backend means replacing this file (and the
//! engine internals) instead of auditing every game module.

pub use sdl2::keyboard::Keycode;
pub use sdl2::mixer::{AudioFormat, AUDIO_S16LSB, DEFAULT_CHANNELS};
pub use sdl2::pixels::Color;
pub use sdl2::rect::{Point, Rect};
pub use sdl2::render::BlendMode;
