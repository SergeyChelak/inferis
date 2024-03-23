use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

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
