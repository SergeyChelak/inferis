use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    common::{EcsError, EcsResult, EntityID},
    packed_array::PackedArray,
};

type AnyComponent = Rc<RefCell<dyn Any>>;
type ComponentRow = PackedArray<Option<AnyComponent>>;

#[derive(Default)]
pub struct EntityManager {
    entity_ids: HashSet<EntityID>,
    components: HashMap<TypeId, ComponentRow>,
    entity_delete_pool: HashSet<EntityID>,
}

impl EntityManager {
    pub fn update(&mut self) -> EcsResult<()> {
        let pool = self.entity_delete_pool.to_owned();
        self.entity_delete_pool.clear();
        for id in pool {
            self.delete_entity(id)?;
        }
        Ok(())
    }

    /// This ECS designed to register all component before entities and systems will be introduced
    /// TODO: May return error if max components amount was exceeded
    pub fn register_component<T: Any>(&mut self) -> EcsResult<&mut Self> {
        let key = TypeId::of::<T>();
        self.components.insert(key, ComponentRow::default());
        Ok(self)
    }

    pub fn new_entity(&mut self) -> EcsResult<Entity> {
        let id = self.create_entity()?;
        self.entity(id)
    }

    pub fn entity(&mut self, id: EntityID) -> EcsResult<Entity> {
        if !self.is_valid_id(id) {
            return Err(EcsError::EntityNotFound(id));
        }
        Ok(Entity { id, state: self })
    }

    /// Creates new entity and return its identifier
    /// May return error if max entities amount exceeded
    fn create_entity(&mut self) -> EcsResult<EntityID> {
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
            self.entity_ids.insert(*val);
            Ok(*val)
        } else {
            return Err(EcsError::FailedAddEntity);
        }
    }

    /// Removes specified entity
    fn delete_entity(&mut self, entity: EntityID) -> EcsResult<()> {
        let result = self
            .components
            .iter_mut()
            .map(|(_, row)| row.remove(entity))
            .all(|r| r);
        if result {
            self.entity_ids.remove(&entity);
            Ok(())
        } else {
            Err(EcsError::EntityNotFound(entity))
        }
    }

    /// Sets given component to the specified entity
    fn entity_add_component<T: Any>(&mut self, entity: EntityID, component: T) -> EcsResult<()> {
        let key = TypeId::of::<T>();
        let Some(row) = self.components.get_mut(&key) else {
            return Err(EcsError::ComponentNotFound(key));
        };
        let Some(item) = row.get_mut(entity) else {
            return Err(EcsError::AccessComponentFailure(entity));
        };
        *item = Some(Rc::new(RefCell::new(component)));
        Ok(())
    }

    fn entity_remove_component<T: Any>(&mut self, entity: EntityID) -> EcsResult<()> {
        let key = TypeId::of::<T>();
        let Some(row) = self.components.get_mut(&key) else {
            return Err(EcsError::ComponentNotFound(key));
        };
        let Some(item) = row.get_mut(entity) else {
            return Err(EcsError::AccessComponentFailure(entity));
        };
        *item = None;
        Ok(())
    }

    fn is_valid_id(&self, entity: EntityID) -> bool {
        self.entity_ids.contains(&entity)
    }

    fn push_to_delete_pool(&mut self, entity: EntityID) {
        self.entity_delete_pool.insert(entity);
    }
}

pub struct Entity<'a> {
    id: EntityID,
    state: &'a mut EntityManager,
}

impl<'a> Entity<'a> {
    pub fn add_component<T: Any>(&mut self, component: T) -> EcsResult<&mut Self> {
        self.state.entity_add_component(self.id, component)?;
        Ok(self)
    }

    pub fn remove_component<T: Any>(&mut self) -> EcsResult<&mut Self> {
        self.state.entity_remove_component::<T>(self.id)?;
        Ok(self)
    }

    pub fn as_removed(&mut self) -> EcsResult<&mut Self> {
        self.state.push_to_delete_pool(self.id);
        Ok(self)
    }

    pub fn as_id(&self) -> EntityID {
        self.id
    }
}
