pub mod assets;
pub mod entities;
pub mod geometry;
pub mod runloop;
pub mod settings;

pub use assets::AssetManager;
pub use entities::utils::*;
pub use entities::*;
pub use geometry::*;
pub use runloop::*;
pub use sdl2::*;
pub use settings::*;
pub use storage::*;

#[derive(Debug)]
pub enum EngineError {
    ComponentNotRegistered,
    ComponentAlreadyRegistered,
    ComponentCountOverflow,
    ComponentNotFound(String),
    TextureNotFound(String),
    SceneNotFound,
    FileAccessError(String),
    ResourceParseError(String),
    ResourceNotFound(String),
    Sdl(String),
    MazeGenerationFailed(String),
    UnexpectedState(String),
}

impl EngineError {
    pub fn component_not_found(name: impl Into<String>) -> EngineError {
        Self::ComponentNotFound(name.into())
    }

    pub fn sdl(name: impl Into<String>) -> EngineError {
        Self::Sdl(name.into())
    }

    pub fn unexpected_state(message: impl Into<String>) -> EngineError {
        Self::UnexpectedState(message.into())
    }
}

pub type EngineResult<T> = Result<T, EngineError>;
