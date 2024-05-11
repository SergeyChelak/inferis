use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{ComponentEntry, ComponentStorage, EngineResult, EntityID};

#[derive(Default)]
pub struct EntityBundle {
    pub raw: HashMap<TypeId, ComponentEntry>,
}

impl EntityBundle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put<T: Any>(mut self, value: T) -> Self {
        let key = TypeId::of::<T>();
        self.raw.insert(key, Rc::new(RefCell::new(value)));
        self
    }
}

#[derive(Default)]
pub struct Query {
    pub types: HashSet<TypeId>,
}

impl Query {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_component<T: Any>(mut self) -> Self {
        let type_id = TypeId::of::<T>();
        self.types.insert(type_id);
        self
    }
}

pub fn fetch_first<T: Any>(storage: &ComponentStorage) -> Option<EntityID> {
    let query = Query::new().with_component::<T>();
    storage.fetch_entities(&query).first().copied()
}

pub fn cleanup_component<T: Any>(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<T>();
    let entities = storage.fetch_entities(&query);
    for id in entities {
        if !storage.set::<T>(id, None) {
            println!("[warn] failed to remove component {:?}", TypeId::of::<T>());
        }
    }
    Ok(())
}
