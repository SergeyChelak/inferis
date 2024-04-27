use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{utils::*, EngineError, EngineResult};

use super::footprint::Footprint;

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
                .map(|x| x.is_alive && x.generation == index.generation)
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

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct Index {
        index: usize,
        generation: u64,
    }

    impl Index {
        pub fn index(&self) -> usize {
            self.index
        }

        pub fn id_key(&self) -> String {
            format!("{}:{}", self.generation, self.index)
        }
    }
}

pub type ComponentEntry = Rc<RefCell<dyn Any>>;
pub type EntityID = allocator::Index;

const STORAGE_CAPACITY: usize = 1000;

#[derive(Default)]
pub struct ComponentStorage {
    raw: HashMap<TypeId, Vec<Option<ComponentEntry>>>,
    allocator: allocator::Allocator,
    // type-footprint position mapping
    type_position_map: HashMap<TypeId, usize>,
    entity_footprint: HashMap<EntityID, Footprint>,
    indices: HashSet<EntityID>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Any>(&mut self) -> EngineResult<()> {
        let key = TypeId::of::<T>();
        if self.raw.get(&key).is_some() {
            return Err(EngineError::ComponentAlreadyRegistered);
        }
        let position = self.type_position_map.len();
        if position >= Footprint::max_items() {
            return Err(EngineError::ComponentCountOverflow);
        }
        self.type_position_map.insert(key, position);
        self.raw.insert(key, Vec::with_capacity(STORAGE_CAPACITY));
        Ok(())
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
        self.indices.insert(entity_id);
        entity_id
    }

    pub fn append(&mut self, bundle: &EntityBundle) -> EntityID {
        let id = self.add_entity();
        for (key, value) in bundle.raw.iter() {
            let Some(row) = self.raw.get_mut(key) else {
                println!("[ComponentStorage] failed to get component's row");
                continue;
            };
            let Some(&position) = self.type_position_map.get(key) else {
                println!("[ComponentStorage] failed to get component's position");
                continue;
            };
            row[id.index()] = Some(value.clone());
            let footprint = self.entity_footprint.entry(id).or_insert(Footprint::new());
            footprint.set(position, true);
        }
        id
    }

    pub fn remove_entity(&mut self, entity_id: EntityID) -> bool {
        if !self.is_alive(entity_id) {
            return false;
        }
        self.raw.iter_mut().for_each(|(_, v)| {
            v[entity_id.index()] = None;
        });
        self.allocator.deallocate(entity_id);
        self.entity_footprint.remove(&entity_id);
        self.indices.remove(&entity_id);
        true
    }

    pub fn get<T: Any>(&self, entity_id: EntityID) -> Option<Ref<T>> {
        let val = self.get_component::<T>(entity_id)?;
        let Ok(borrowed) = val.try_borrow() else {
            return None;
        };
        let ref_val = Ref::map(borrowed, |item| {
            item.downcast_ref::<T>()
                .expect("[ComponentStorage] Failed to downcast_ref")
        });
        Some(ref_val)
    }

    pub fn get_mut<T: Any>(&self, entity_id: EntityID) -> Option<RefMut<T>> {
        let val = self.get_component::<T>(entity_id)?;
        let Ok(borrowed) = val.try_borrow_mut() else {
            return None;
        };
        let ref_val = RefMut::map(borrowed, |item| {
            item.downcast_mut::<T>()
                .expect("[ComponentStorage] Failed to downcast_mut")
        });
        Some(ref_val)
    }

    pub fn set<T: Any>(&mut self, entity_id: EntityID, value: Option<T>) -> bool {
        if !self.is_alive(entity_id) {
            return false;
        }
        let key = TypeId::of::<T>();
        let Some(row) = self.raw.get_mut(&key) else {
            return false;
        };
        let footprint = self
            .entity_footprint
            .entry(entity_id)
            .or_insert(Footprint::new());
        let Some(&position) = self.type_position_map.get(&key) else {
            return false;
        };
        row[entity_id.index()] = if let Some(x) = value {
            footprint.set(position, true);
            Some(Rc::new(RefCell::new(x)))
        } else {
            footprint.set(position, false);
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn footprint(&self, types: &HashSet<TypeId>) -> Footprint {
        let mut footprint = Footprint::new();
        for item in types {
            let Some(&pos) = self.type_position_map.get(item) else {
                continue;
            };
            footprint.set(pos, true);
        }
        footprint
    }

    pub fn fetch_entities(&self, query: &Query) -> Vec<EntityID> {
        let mut entities = Vec::new();
        let query_footprint = self.footprint(&query.types);
        for entity_id in &self.indices {
            let Some(entity_footprint) = self
                .indices
                .get(entity_id)
                .and_then(|index| self.entity_footprint.get(index))
            else {
                println!("[ComponentStorage] fetch: footprint isn't registered for entity");
                continue;
            };
            if query_footprint.is_matches(entity_footprint) {
                entities.push(*entity_id);
            }
        }
        entities
    }

    pub fn fetch_components(&self, query: &Query) -> HashMap<TypeId, Vec<ComponentEntry>> {
        let idx_array = self
            .fetch_entities(query)
            .iter()
            .map(|item| item.index())
            .collect::<Vec<usize>>();
        let mut result: HashMap<TypeId, Vec<ComponentEntry>> = HashMap::new();
        for type_id in query.types.iter() {
            let Some(row) = self.raw.get(type_id) else {
                panic!("[ComponentStorage] fetch: failed to get component row");
            };
            let entry = result.entry(*type_id).or_default();
            for entity_id in &idx_array {
                let Some(component) = row.get(*entity_id).and_then(|x| x.as_ref()) else {
                    panic!("[ComponentStorage] fetch: component is none");
                };
                entry.push(component.clone());
            }
        }
        result
    }
}
