pub mod assets;
pub mod entities;
pub mod geometry;
pub mod prelude;
pub mod settings;
pub mod world;

#[derive(Debug)]
pub enum EngineError {
    ComponentNotRegistered,
    SceneNotFound,
    FileAccessError(String),
    ResourceParseError(String),
    Sdl(String),
}

pub type EngineResult<T> = Result<T, EngineError>;
