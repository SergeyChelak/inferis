use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::ComponentEntry;

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
