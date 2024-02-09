use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{packed_array::PackedArray, EngineError, EngineResult, EntityID};

type AnyComponent = Rc<RefCell<dyn Any>>;
type ComponentRow = PackedArray<Option<AnyComponent>>;

#[derive(Default)]
pub struct EntityManager {
    container: HashMap<TypeId, ComponentRow>,
    insert_pool: Vec<EntityBuilder>,
    remove_pool: HashSet<EntityID>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Any>(&mut self) {
        let key = TypeId::of::<T>();
        self.container.insert(key, PackedArray::default());
    }

    pub fn add(&mut self) -> &mut EntityBuilder {
        let allowed_types = self.all_keys();
        // push a new entity to insert pool
        // return entity builder
        let builder = EntityBuilder::new(allowed_types);
        self.insert_pool.push(builder);
        self.insert_pool.last_mut().unwrap()
    }

    fn all_keys(&self) -> HashSet<TypeId> {
        self.container.keys().copied().collect::<HashSet<TypeId>>()
    }

    pub fn remove(&mut self, id: EntityID) {
        // push entity id to remove pool
        self.remove_pool.insert(id);
    }

    pub fn query(&self) {
        // create the query builder
    }

    pub fn apply(&mut self) -> Result<(), EngineError> {
        self.process_remove_pool()?;
        self.process_insert_pool()
    }

    fn process_remove_pool(&mut self) -> Result<(), EngineError> {
        let all_keys = self.all_keys();
        // remove all entities that're stored in remove pool
        'outer: for id in &self.remove_pool {
            for key in &all_keys {
                let Some(row) = self.container.get_mut(key) else {
                    self.remove_pool.clear();
                    return Err(EngineError::IntegrityFailed(
                        "Search row in remove pool".to_string(),
                    ));
                };
                if !row.remove(*id) {
                    // expecting that entity not present, so there is no reason to process a rest of the rows
                    continue 'outer;
                }
            }
        }
        self.remove_pool.clear();
        Ok(())
    }

    fn process_insert_pool(&mut self) -> Result<(), EngineError> {
        let all_keys = self.all_keys();
        // move all entities from insert pool
        for entity in &self.insert_pool {
            if entity.is_invalidated {
                continue;
            }
            for key in &all_keys {
                let Some(row) = self.container.get_mut(key) else {
                    self.insert_pool.clear();
                    return Err(EngineError::IntegrityFailed(
                        "Process insert pool".to_string(),
                    ));
                };
                if let Some(val) = entity.container.get(key) {
                    row.add(Some(val.clone()));
                } else {
                    row.add(None);
                }
            }
        }
        self.insert_pool.clear();
        Ok(())
    }

    pub fn entities_count(&self) -> usize {
        let mut iter = self.container.values().map(|arr| arr.len());
        let Some(first) = iter.next() else {
            return 0;
        };
        if iter.all(|elem| elem == first) {
            first
        } else {
            0
        }
    }

    pub fn all_ids(&self) -> &[EntityID] {
        let mut iter = self.container.iter();
        let Some(row) = iter.next() else {
            return &[];
        };
        row.1.ids()
    }
}

pub struct EntityBuilder {
    allowed_types: HashSet<TypeId>,
    is_invalidated: bool,
    container: HashMap<TypeId, Rc<RefCell<dyn Any>>>,
}

impl EntityBuilder {
    pub fn new(allowed_types: HashSet<TypeId>) -> Self {
        Self {
            allowed_types,
            is_invalidated: false,
            container: HashMap::default(),
        }
    }

    pub fn set<T: Any>(&mut self, value: T) -> EngineResult<&mut Self> {
        let key = TypeId::of::<T>();
        if !self.allowed_types.contains(&key) {
            return Err(super::EngineError::ComponentNotRegistered);
        }
        self.container.insert(key, Rc::new(RefCell::new(value)));
        Ok(self)
    }

    pub fn invalidate(&mut self) -> EngineResult<&mut Self> {
        self.is_invalidated = true;
        Ok(self)
    }
}

pub struct QueryBuilder {
    //
}

impl QueryBuilder {
    pub fn with_component<T: Any>(&mut self) -> &mut Self {
        self
    }

    pub fn exec(&self) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // component types
    struct Position {
        x: f32,
        y: f32,
    }

    struct Health(u16);

    struct PlayerMarker;

    #[test]
    fn em_comp_register() {
        let mut em = EntityManager::new();
        em.register_component::<Position>();
        em.register_component::<Health>();
        assert_eq!(em.container.len(), 2, "Failed to register components")
    }

    #[test]
    fn em_add_entity() -> EngineResult<()> {
        let mut em = EntityManager::new();
        em.register_component::<Position>();
        em.register_component::<Health>();
        em.register_component::<PlayerMarker>();

        em.add()
            .set(Position { x: 1.2, y: 3.4 })?
            .set(Health(100))?
            .set(PlayerMarker)?;
        em.add()
            .set(Position { x: 5.6, y: 7.8 })?
            .set(Health(100))?;

        assert_eq!(0, em.entities_count());
        em.apply()?;
        assert_eq!(2, em.entities_count());
        em.apply()?;
        assert_eq!(2, em.entities_count());
        Ok(())
    }

    #[test]
    fn em_remove_entity() -> EngineResult<()> {
        let mut em = EntityManager::new();
        em.register_component::<Position>();
        em.register_component::<Health>();
        em.register_component::<PlayerMarker>();

        em.add()
            .set(Position { x: 1.2, y: 3.4 })?
            .set(Health(100))?
            .set(PlayerMarker)?;
        em.add()
            .set(Position { x: 5.6, y: 7.8 })?
            .set(Health(100))?;

        assert_eq!(0, em.entities_count());
        em.apply()?;

        let ids = em.all_ids();
        em.remove(ids[0]);
        em.apply()?;

        assert_eq!(1, em.entities_count());
        em.apply()?;

        assert_eq!(1, em.entities_count());
        Ok(())
    }
}
