pub mod assets;
pub mod entities;
pub mod geometry;
pub mod prelude;
pub mod runloop;
pub mod settings;

pub use assets::AssetManager;
pub use entities::storage::{ComponentEntry, ComponentStorage, EntityID};
pub use entities::utils::{
    cleanup_component, fetch_first, refresh_cached_entity, EntityBundle, Query,
};
pub use geometry::{
    ray_cast, ray_cast_dir, Float, RayCastResult, Rectangle, Size, SizeFloat, SizeU32, Vec2f,
    RAY_CASTER_TOL,
};
pub use runloop::{game_scene, systems, world, SceneID};
pub use settings::{AudioSettings, EngineSettings, WindowSettings};

#[derive(Debug)]
pub enum EngineError {
    ComponentNotRegistered,
    ComponentAlreadyRegistered,
    ComponentCountOverflow,
    ComponentNotFound(String),
    EntityNotAlive(EntityID),
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

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EngineError::*;
        match self {
            ComponentNotRegistered => write!(f, "component type is not registered"),
            ComponentAlreadyRegistered => write!(f, "component type is already registered"),
            ComponentCountOverflow => write!(f, "too many registered component types"),
            ComponentNotFound(name) => write!(f, "component not found: {name}"),
            EntityNotAlive(id) => write!(f, "entity {} is not alive", id.index()),
            TextureNotFound(name) => write!(f, "texture not found: {name}"),
            SceneNotFound => write!(f, "scene not found"),
            FileAccessError(path) => write!(f, "failed to access file: {path}"),
            ResourceParseError(msg) => write!(f, "failed to parse resource: {msg}"),
            ResourceNotFound(name) => write!(f, "resource not found: {name}"),
            Sdl(msg) => write!(f, "SDL error: {msg}"),
            MazeGenerationFailed(msg) => write!(f, "maze generation failed: {msg}"),
            UnexpectedState(msg) => write!(f, "unexpected state: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;
