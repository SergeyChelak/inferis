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
    ray_cast, Float, RayCastResult, Rectangle, Size, SizeFloat, SizeU32, Vec2f,
    RAY_CASTER_MAX_DEPTH, RAY_CASTER_TOL,
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

pub type EngineResult<T> = Result<T, EngineError>;
