use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::{
    common::{EcsError, EcsResult, EntityID},
    packed_array::PackedArray,
};

type AnyComponent = Rc<RefCell<dyn Any>>;
type ComponentRow = PackedArray<Option<AnyComponent>>;

pub struct StateManager {
    components: HashMap<TypeId, ComponentRow>,
}

impl Default for StateManager {
    fn default() -> Self {
        Self {
            components: Default::default(),
        }
    }
}

impl StateManager {
    /// This ECS designed to register all component before entities and systems will be introduced
    /// TODO: May return error if max components amount was exceeded
    pub fn register_component<T: Any>(&mut self) -> EcsResult<&mut Self> {
        let key = TypeId::of::<T>();
        self.components.insert(key, ComponentRow::new());
        Ok(self)
    }

    /// Creates new entity and return its identifier
    /// May return error if max entities amount exceeded
    pub fn create_entity(&mut self) -> EcsResult<EntityID> {
        let id = self
            .components
            .iter_mut()
            .map(|(_, row)| row.add(None))
            .collect::<Vec<EntityID>>();
        let Some(val) = id.first() else {
            return Err(EcsError::FailedAddEntity);
        };
        // check consistency
        if id.iter().all(|x| *x == *val) {
            Ok(*val)
        } else {
            return Err(EcsError::FailedAddEntity);
        }
    }

    /// Removes specified entity
    pub fn delete_entity(&mut self, entity: EntityID) -> EcsResult<()> {
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
    pub fn entity_add_component<T: Any>(
        &mut self,
        entity: EntityID,
        component: T,
    ) -> EcsResult<()> {
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
