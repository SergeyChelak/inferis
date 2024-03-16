use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use super::storage::{ComponentStorage, EntityID};

pub struct EntityManager {
    storage: ComponentStorage,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            storage: ComponentStorage::new(),
        }
    }

    pub fn register_component<T: Any>(&mut self) {
        self.storage.register_component::<T>();
        // TODO: Ok(...)
    }

    pub fn create_entity(&mut self) -> EntityHandler {
        let id = self.storage.add_entity();
        self.entity(id)
    }

    pub fn entity(&mut self, entity_id: EntityID) -> EntityHandler {
        EntityHandler {
            id: entity_id,
            storage: &mut self.storage,
        }
    }

    pub fn remove_entity(&mut self, entity_id: EntityID) {
        self.storage.remove_entity(entity_id);
        todo!()
    }

    pub fn update(&mut self) {}
}

struct EntityHandler<'a> {
    id: EntityID,
    storage: &'a mut ComponentStorage,
}

impl<'a> EntityHandler<'a> {
    fn get<T: Any>(&self) -> Option<Ref<T>> {
        self.storage.get(self.id)
    }

    fn get_mut<T: Any>(&self) -> Option<RefMut<T>> {
        self.storage.get_mut(self.id)
    }

    // FIX: ignored result!
    fn set<T: Any>(self, value: T) -> Self {
        self.storage.set_value(self.id, Some(value));
        self
    }

    fn remove<T: Any>(&mut self) {
        self.storage.set_value::<T>(self.id, None);
    }

    fn id(self) -> EntityID {
        self.id
    }

    fn is_alive(&self) -> bool {
        self.storage.is_alive(self.id)
    }

    fn destroy(&mut self) {
        self.storage.remove_entity(self.id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn em_alive() {
        let mut em = EntityManager::new();
        let id_1 = em.create_entity().set(0i32).set(0.0f64);
        assert!(id_1.is_alive());
        let mut id_2 = em.create_entity();
        id_2.destroy();
        assert!(!id_2.is_alive());
    }

    #[derive(Debug, PartialEq)]
    struct C1(i32);
    #[derive(Debug, PartialEq)]
    struct C2(f32);

    #[test]
    fn em_component_modify() {
        let mut em = EntityManager::new();
        em.register_component::<C1>();
        em.register_component::<C2>();

        let mut entity = em.create_entity().set(C1(1)).set(C2(2.3));

        assert_eq!(*entity.get::<C1>().unwrap(), C1(1));
        assert_eq!(*entity.get::<C2>().unwrap(), C2(2.3));

        *entity.get_mut::<C1>().unwrap() = C1(5);
        assert_eq!(*entity.get::<C1>().unwrap(), C1(5));

        entity.remove::<C2>();
        assert!(entity.get::<C2>().is_none());
    }
}
