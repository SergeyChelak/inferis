use sdl2::mixer::AudioFormat;

use crate::{assets::AssetSource, geometry::SizeU32};

pub struct EngineSettings {
    pub window: WindowSettings,
    pub asset_source: AssetSource,
    pub audio_setting: AudioSettings,
}
pub struct WindowSettings {
    pub title: String,
    pub size: SizeU32,
}

pub struct AudioSettings {
    pub frequency: i32,
    pub format: AudioFormat,
    pub channels: i32,
    pub chunk_size: i32,
    // Number of mixing channels available for sound effect `Chunk`s to play simultaneously.
    pub mixing_channels: i32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            frequency: 44_100,
            format: sdl2::mixer::AUDIO_S16LSB, // signed 16 bit samples, in little-endian byte order
            channels: sdl2::mixer::DEFAULT_CHANNELS, // Stereo
            chunk_size: 1024,
            mixing_channels: 16,
        }
    }
}
