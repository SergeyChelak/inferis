use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

use super::storage::ComponentStorage;

pub struct Query<'a> {
    storage: &'a mut ComponentStorage,
    types: HashSet<TypeId>,
}

impl<'a> Query<'a> {
    pub fn new(storage: &'a mut ComponentStorage) -> Self {
        Self {
            storage,
            types: HashSet::default(),
        }
    }

    pub fn with_component<T: Any>(mut self) -> Self {
        let type_id = TypeId::of::<T>();
        self.types.insert(type_id);
        self
    }

    pub fn run(&self) {
        //
    }
}
