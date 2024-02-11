use crate::common::U32Size;

use self::packed_array::ValueID;

mod archetype;
mod asset_manager;
pub mod config;
pub mod entity_manager;
pub mod game_engine;
mod packed_array;

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

pub trait GameEngineContext {
    fn terminate(&mut self);
    fn screen_size(&self) -> U32Size;
    fn frame_counter(&self) -> u64;
}

pub trait Scene {
    fn update(&mut self, context: &mut dyn GameEngineContext);
}
