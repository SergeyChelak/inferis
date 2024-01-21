use std::any::Any;

#[derive(Debug)]
pub enum EcsError {
    TooManyComponents,
    TooManyEntities,
}

pub type EcsResult<T> = Result<T, EcsError>;

/// Entity is just an identifier that used to group required components
pub type Entity = usize;

pub struct Ecs {
    // TODO: add fields
}

impl Ecs {
    /// This ECS designed to register all component before entities and systems will be introduced
    /// May return error if max components amount exceeded
    pub fn register_component<T: Any>(&mut self) {
        todo!()
    }

    /// System must be registered at initialization step
    /// By default, any system isn't permitted to read or write components
    /// Corresponding methods in API are should be called to declare which
    /// component could be accessed for reading and which ones for writing
    pub fn register_system<T: Any>(&mut self, system: T) {
        todo!()
    }

    /// Enables read component permission for the system
    pub fn system_permit_read<S: Any, C: Any>(&mut self, system: S, component: C) {
        todo!()
    }

    /// Enables write component permission for the system
    pub fn system_permit_write<S: Any, C: Any>(&mut self, system: S, component: C) {
        todo!()
    }

    /// Creates new entity and return its identifier
    /// May return error if max entities amount exceeded
    pub fn create_entity(&mut self) -> EcsResult<Entity> {
        todo!()
    }

    /// Removes specified entity
    pub fn delete_entity(&mut self, entity: Entity) {
        todo!()
    }

    /// Sets given component to the specified entity
    pub fn entity_add_component<T: Any>(&mut self, entity: Entity, component: T) -> EcsResult<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
