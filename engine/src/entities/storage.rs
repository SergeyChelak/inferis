use std::{any::TypeId, cell::RefCell, collections::HashMap, rc::Rc};

use super::entity_manager::Component;

mod allocator {
    #[derive(Default)]
    pub struct Allocator {
        entries: Vec<Entry>,
        recycled: Vec<usize>,
        count: usize,
    }

    impl Allocator {
        pub fn allocate(&mut self) -> Index {
            let index = if let Some(idx) = self.recycled.pop() {
                idx
            } else {
                self.entries.push(Entry::default());
                self.entries.len() - 1
            };
            let entry = self
                .entries
                .get_mut(index)
                .expect("[Allocator] failed to get allocated item");
            entry.is_alive = true;
            entry.generation += 1;
            self.count += 1;
            Index {
                generation: entry.generation,
                index,
            }
        }

        pub fn deallocate(&mut self, index: Index) -> bool {
            let idx = index.index;
            if self.recycled.contains(&idx) {
                return false;
            }
            let Some(entry) = self.entries.get_mut(idx) else {
                return false;
            };
            self.count -= 1;
            entry.is_alive = false;
            self.recycled.push(idx);
            true
        }

        pub fn is_alive(&self, index: Index) -> bool {
            self.entries
                .get(index.index)
                .and_then(|x| Some(x.is_alive && x.generation == index.generation))
                .unwrap_or_default()
        }

        pub fn len(&self) -> usize {
            self.count
        }
    }

    #[derive(Default)]
    struct Entry {
        is_alive: bool,
        generation: u64,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Index {
        index: usize,
        generation: u64,
    }

    impl Index {
        pub fn index(&self) -> usize {
            self.index
        }
    }
}

type ComponentEntry = Rc<RefCell<dyn Component>>;
pub type EntityID = allocator::Index;

const STORAGE_CAPACITY: usize = 1000;

pub struct ComponentStorage {
    raw: HashMap<TypeId, Vec<Option<ComponentEntry>>>,
    allocator: allocator::Allocator,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            allocator: allocator::Allocator::default(),
            raw: HashMap::new(),
        }
    }

    pub fn register_component<T: Component>(&mut self) {
        let key = TypeId::of::<T>();
        if self.raw.get(&key).is_none() {
            self.raw.insert(key, Vec::with_capacity(STORAGE_CAPACITY));
        }
    }

    pub fn add_entity(&mut self) -> EntityID {
        let entity_id = self.allocator.allocate();
        self.raw.iter_mut().for_each(|(_, v)| {
            if entity_id.index() < v.len() {
                v[entity_id.index()] = None;
            } else {
                v.push(None);
            }
        });
        entity_id
    }

    pub fn remove_entity(&mut self, entity_id: EntityID) -> bool {
        if !self.allocator.is_alive(entity_id) {
            return false;
        }
        self.raw.iter_mut().for_each(|(_, v)| {
            v[entity_id.index()] = None;
        });
        self.allocator.deallocate(entity_id);
        true
    }

    pub fn len(&self) -> usize {
        self.allocator.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct C1(i32);
    impl Component for C1 {}
    struct C2 {
        a: f32,
        b: char,
    }
    impl Component for C2 {}

    #[test]
    fn cs_add_remove() {
        let mut storage = ComponentStorage::new();
        storage.register_component::<C1>();
        storage.register_component::<C2>();
        let entity_a = storage.add_entity();
        assert_eq!(1, storage.len());
        let entity_b = storage.add_entity();
        assert_eq!(2, storage.len());
        storage.remove_entity(entity_a);
        assert!(!storage.remove_entity(entity_a));
        assert_eq!(1, storage.len());
        storage.remove_entity(entity_b);
        assert_eq!(0, storage.len());
    }
}
