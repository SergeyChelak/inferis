use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use super::storage::{ComponentStorage, EntityID};

pub struct EntityHandler<'a> {
    id: EntityID,
    storage: &'a mut ComponentStorage,
}

impl<'a> EntityHandler<'a> {
    pub fn new(id: EntityID, storage: &'a mut ComponentStorage) -> Self {
        Self { id, storage }
    }

    fn with(storage: &'a mut ComponentStorage) -> Self {
        Self {
            id: storage.add_entity(),
            storage,
        }
    }

    fn get<T: Any>(&self) -> Option<Ref<T>> {
        self.storage.get(self.id)
    }

    fn get_mut<T: Any>(&self) -> Option<RefMut<T>> {
        self.storage.get_mut(self.id)
    }

    // FIX: ignored result!
    fn with_component<T: Any>(mut self, value: T) -> Self {
        self.add::<T>(value);
        self
    }

    fn add<T: Any>(&mut self, value: T) -> bool {
        self.storage.set(self.id, Some(value))
    }

    fn remove<T: Any>(&mut self) {
        self.storage.set::<T>(self.id, None);
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
        let mut cs = ComponentStorage::new();
        let id_1 = EntityHandler::with(&mut cs)
            .with_component(0i32)
            .with_component(0.0f64);
        assert!(id_1.is_alive());
        let mut id_2 = EntityHandler::with(&mut cs);
        id_2.destroy();
        assert!(!id_2.is_alive());
    }

    #[derive(Debug, PartialEq)]
    struct C1(i32);
    #[derive(Debug, PartialEq)]
    struct C2(f32);

    #[test]
    fn em_component_modify() {
        let mut cs = ComponentStorage::new();
        cs.register_component::<C1>();
        cs.register_component::<C2>();

        let mut entity = EntityHandler::with(&mut cs)
            .with_component(C1(1))
            .with_component(C2(2.3));

        assert_eq!(*entity.get::<C1>().unwrap(), C1(1));
        assert_eq!(*entity.get::<C2>().unwrap(), C2(2.3));

        *entity.get_mut::<C1>().unwrap() = C1(5);
        assert_eq!(*entity.get::<C1>().unwrap(), C1(5));

        entity.remove::<C2>();
        assert!(entity.get::<C2>().is_none());
    }

    #[test]
    fn em_multiple_entities() {
        let mut cs = ComponentStorage::new();
        cs.register_component::<C1>();
        cs.register_component::<C2>();

        let _ = EntityHandler::with(&mut cs).with_component(C1(1));
        let _ = EntityHandler::with(&mut cs).with_component(C2(2.0));

        assert_eq!(cs.len(), 2);
    }
}
