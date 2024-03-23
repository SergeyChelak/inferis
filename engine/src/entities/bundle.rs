use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
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

    pub fn add<T: Any>(mut self, value: T) -> Self {
        let key = TypeId::of::<T>();
        self.raw.insert(key, Rc::new(RefCell::new(value)));
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
