use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

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

type ComponentEntry = Rc<RefCell<dyn Any>>;
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

    pub fn register_component<T: Any>(&mut self) -> bool {
        let key = TypeId::of::<T>();
        if self.raw.get(&key).is_none() {
            self.raw.insert(key, Vec::with_capacity(STORAGE_CAPACITY));
            true
        } else {
            false
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
        if !self.is_alive(entity_id) {
            return false;
        }
        self.raw.iter_mut().for_each(|(_, v)| {
            v[entity_id.index()] = None;
        });
        self.allocator.deallocate(entity_id);
        true
    }

    pub fn get<T: Any>(&self, entity_id: EntityID) -> Option<Ref<T>> {
        let Some(val) = self.get_component::<T>(entity_id) else {
            return None;
        };
        let Ok(borrowed) = val.try_borrow() else {
            return None;
        };
        let ref_val = Ref::map(borrowed, |item| {
            item.downcast_ref::<T>()
                .expect("[ComponentStorage] Failed to downcast")
        });
        Some(ref_val)
    }

    pub fn get_mut<T: Any>(&self, entity_id: EntityID) -> Option<RefMut<T>> {
        let Some(val) = self.get_component::<T>(entity_id) else {
            return None;
        };
        let Ok(borrowed) = val.try_borrow_mut() else {
            return None;
        };
        let ref_val = RefMut::map(borrowed, |item| {
            item.downcast_mut::<T>()
                .expect("[ComponentStorage] Failed to downcast")
        });
        Some(ref_val)
    }

    pub fn set_value<T: Any>(&mut self, entity_id: EntityID, value: Option<T>) -> bool {
        if !self.is_alive(entity_id) {
            return false;
        }
        let key = TypeId::of::<T>();
        let Some(row) = self.raw.get_mut(&key) else {
            return false;
        };
        row[entity_id.index()] = if let Some(x) = value {
            Some(Rc::new(RefCell::new(x)))
        } else {
            None
        };
        true
    }

    fn get_component<T: Any>(&self, entity_id: EntityID) -> Option<&ComponentEntry> {
        if !self.is_alive(entity_id) {
            return None;
        }
        let key = TypeId::of::<T>();
        self.raw
            .get(&key)
            .and_then(|arr| arr.get(entity_id.index()))
            .and_then(|item| item.as_ref())
    }

    pub fn is_alive(&self, entity_id: EntityID) -> bool {
        self.allocator.is_alive(entity_id)
    }

    pub fn len(&self) -> usize {
        self.allocator.len()
    }
}
