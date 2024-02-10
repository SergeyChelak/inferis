use self::packed_array::ValueID;

mod archetype;
pub mod config;
pub mod entity_manager;
mod packed_array;
pub mod scene;
pub mod world;

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
