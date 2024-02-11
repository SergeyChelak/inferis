use self::packed_array::ValueID;

mod archetype;
pub mod config;
pub mod entity_manager;
pub mod game_engine;
mod packed_array;
pub mod scene;

pub type EntityID = ValueID;

#[derive(Debug)]
pub enum EngineError {
    SDLError(String),
    ComponentNotRegistered,
    ComponentLimitExceeded(usize),
    ComponentBorrowFailed,
    ComponentCastFailed,
    IntegrityFailed(String),
}

pub type EngineResult<T> = Result<T, EngineError>;

pub trait GameEngineContext {}
