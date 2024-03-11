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

    pub fn add(&mut self) -> EntityID {
        let id = self.allocator.allocate();
        self.raw.iter_mut().for_each(|(_, v)| {
            let size = v.len();
            // if id.index() <
        });
        todo!()
    }

    pub fn remove(&mut self, entity_id: EntityID) -> bool {
        todo!()
    }
}
