pub mod assets;
pub mod entities;
pub mod geometry;
pub mod settings;
pub mod world;

pub use assets::AssetManager;
pub use entities::utils::*;
pub use entities::*;
pub use geometry::*;
pub use sdl2::*;
pub use settings::*;
pub use storage::*;
pub use world::game_world::GameWorld;
pub use world::*;

#[derive(Debug)]
pub enum EngineError {
    ComponentNotRegistered,
    ComponentAlreadyRegistered,
    ComponentCountOverflow,
    ComponentNotFound(String),
    SceneNotFound,
    FileAccessError(String),
    ResourceParseError(String),
    ResourceNotFound(String),
    Sdl(String),
    MazeGenerationFailed(String),
}

pub type EngineResult<T> = Result<T, EngineError>;
