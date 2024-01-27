mod packed_array;

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use packed_array::{PackedArray, ValueID};

#[derive(Debug)]
pub enum EcsError {
    FailedAddEntity,
    ComponentNotFound(TypeId),
    AccessComponentFailure(Entity),
    EntityNotFound(Entity),
    TooManyComponents,
    TooManyEntities,
}

pub type EcsResult<T> = Result<T, EcsError>;

/// Entity is just an identifier that used to group required components
pub type Entity = ValueID;
type AnyComponent = Rc<RefCell<dyn Any>>;
type ComponentRow = PackedArray<Option<AnyComponent>>;

pub struct Ecs {
    components: HashMap<TypeId, ComponentRow>,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    /// This ECS designed to register all component before entities and systems will be introduced
    /// TODO: May return error if max components amount was exceeded
    pub fn register_component<T: Any>(&mut self) -> EcsResult<()> {
        let key = TypeId::of::<T>();
        self.components.insert(key, ComponentRow::new());
        Ok(())
    }

    /// System must be registered at initialization step
    pub fn register_system<T: Any>(&mut self, system: T) {
        todo!()
    }

    /// Creates new entity and return its identifier
    /// May return error if max entities amount exceeded
    pub fn create_entity(&mut self) -> EcsResult<Entity> {
        let id = self
            .components
            .iter_mut()
            .map(|(_, row)| row.add(None))
            //.take(1)
            .collect::<Vec<Entity>>();
        let Some(val) = id.first() else {
            return Err(EcsError::FailedAddEntity);
        };
        Ok(*val)
    }

    /// Removes specified entity
    pub fn delete_entity(&mut self, entity: Entity) -> EcsResult<()> {
        let result = self
            .components
            .iter_mut()
            .map(|(_, row)| row.remove(entity))
            .all(|r| r);
        if result {
            Ok(())
        } else {
            Err(EcsError::EntityNotFound(entity))
        }
    }

    /// Sets given component to the specified entity
    pub fn entity_add_component<T: Any>(&mut self, entity: Entity, component: T) -> EcsResult<()> {
        let key = TypeId::of::<T>();
        let Some(row) = self.components.get_mut(&key) else {
            return Err(EcsError::ComponentNotFound(key));
        };
        // println!("row len {}", row.len());
        let Some(item) = row.get_mut(entity) else {
            return Err(EcsError::AccessComponentFailure(entity));
        };
        *item = Some(Rc::new(RefCell::new(component)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
