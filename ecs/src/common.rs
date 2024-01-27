use std::any::{Any, TypeId};

use crate::{packed_array::ValueID, state::Entity};

#[derive(Debug)]
pub enum EcsError {
    FailedAddEntity,
    ComponentNotFound(TypeId),
    AccessComponentFailure(EntityID),
    EntityNotFound(EntityID),
    TooManyComponents,
    TooManyEntities,
}

pub type EcsResult<T> = Result<T, EcsError>;

/// Entity is just an identifier that used to group required components
pub type EntityID = ValueID;

pub trait EntityProvider {
    fn new_entity(&mut self) -> EcsResult<Entity>;
    fn entity(&mut self, id: EntityID) -> EcsResult<Entity>;
    fn register_component<T: Any>(&mut self) -> EcsResult<&mut Self>;
}
