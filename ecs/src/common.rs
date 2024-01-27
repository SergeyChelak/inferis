use std::any::TypeId;

use crate::packed_array::ValueID;

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
