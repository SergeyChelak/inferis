use std::any::Any;

use super::{type_map::TypeMap, Entity};

pub struct EntityBundle {
    id: Option<Entity>,
    map: TypeMap,
}

impl EntityBundle {
    pub fn new() -> Self {
        Self {
            id: None,
            map: TypeMap::default(),
        }
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        todo!()
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        todo!()
    }

    pub fn set<T: Any>(mut self, value: T) -> Self {
        self
    }

    pub fn remove<T: Any>(mut self) -> Self {
        self
    }
}
