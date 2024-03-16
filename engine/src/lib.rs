mod assets;
mod entities;
pub mod geometry;
pub mod prelude;
pub mod settings;
mod world;

#[derive(Debug)]
pub enum EngineError {
    ComponentNotRegistered,
    Sdl(String),
}

pub type EngineResult<T> = Result<T, EngineError>;
