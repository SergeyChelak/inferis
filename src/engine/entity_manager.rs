use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{archetype::Archetype, packed_array::PackedArray, EngineError, EngineResult, EntityID};

type AnyComponent = Rc<RefCell<dyn Any>>;
type ComponentRow = PackedArray<Option<AnyComponent>>;

type QueryOutputComponentRow<T> = Vec<Rc<RefCell<T>>>;
type QueryOutputComponentMap = HashMap<TypeId, QueryOutputComponentRow<dyn Any>>;
type QueryOutputEntities = Vec<EntityID>;
type QueryInputTypes = HashSet<TypeId>;

#[derive(Default)]
pub struct QueryOutput {
    pub entities: QueryOutputEntities,
    pub components: QueryOutputComponentMap,
}

impl QueryOutput {
    fn get<T: Any>(&self) -> Option<&QueryOutputComponentRow<dyn Any>> {
        let key = TypeId::of::<T>();
        self.components.get(&key)
    }

    fn get_ref<T: Any>(&self) -> Vec<Ref<T>> {
        let mut result = Vec::new();
        let Some(array) = self.get::<T>() else {
            panic!("component not found");
        };
        for elem in array {
            let Ok(val) = elem.try_borrow() else {
                panic!("Failed to borrow");
            };
            let val = Ref::map(val, |x| x.downcast_ref::<T>().expect("Failed to downcast"));
            result.push(val);
        }
        result
    }

    fn get_mut<T: Any>(&self) -> Vec<RefMut<T>> {
        let mut result = Vec::new();
        let Some(array) = self.get::<T>() else {
            panic!("component not found");
        };
        for elem in array {
            let Ok(val) = elem.try_borrow_mut() else {
                panic!("Failed to borrow");
            };
            let val = RefMut::map(val, |x| x.downcast_mut::<T>().expect("Failed to downcast"));
            result.push(val);
        }
        result
    }
}

#[derive(Default)]
pub struct EntityManager {
    container: HashMap<TypeId, ComponentRow>,
    component_archetype: HashMap<TypeId, Archetype>,
    component_type: Vec<TypeId>,
    entity_archetype: PackedArray<Archetype>,
    insert_pool: Vec<EntityBuilder>,
    remove_pool: HashSet<EntityID>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Any>(&mut self) -> EngineResult<&mut Self> {
        let max = Archetype::max_items();
        let position = self.component_archetype.len();
        if position == max {
            return Err(EngineError::ComponentLimitExceeded(max));
        }
        let key = TypeId::of::<T>();
        self.container.insert(key, PackedArray::default());
        let arch = Archetype::with_position(position);
        self.component_archetype.insert(key, arch);
        self.component_type.push(key);
        Ok(self)
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

    fn fetch(&self, query: &Query) -> QueryOutput {
        let mut query_archetype = Archetype::default();
        query
            .types
            .iter()
            .filter_map(|key| self.component_archetype.get(key))
            .for_each(|arch| {
                query_archetype.add(arch);
            });
        let mut output = QueryOutput::default();
        for (id, val) in self.entity_archetype.iter() {
            if !query_archetype.matches(val) {
                continue;
            }
            output.entities.push(*id);
            query_archetype.all_enabled().iter().for_each(|pos| {
                let key = self.component_type[*pos];
                let comp = self
                    .container
                    .get(&key)
                    .and_then(|arr| arr.get(*id))
                    .and_then(|val| val.clone())
                    .expect("Component is missing");
                let entry = output.components.entry(key).or_default();
                entry.push(comp)
            });
        }
        output
    }

    pub fn apply(&mut self) -> Result<(), EngineError> {
        self.process_remove_pool()?;
        self.process_insert_pool()
    }

    fn process_remove_pool(&mut self) -> Result<(), EngineError> {
        let all_keys = self.all_keys();
        // remove all entities that're stored in remove pool
        for id in &self.remove_pool {
            if !self.entity_archetype.remove(*id) {
                continue;
            }
            for key in &all_keys {
                let Some(row) = self.container.get_mut(key) else {
                    self.remove_pool.clear();
                    return Err(EngineError::IntegrityFailed(
                        "Search row in remove pool".to_string(),
                    ));
                };
                assert!(row.remove(*id))
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
            let mut archetype = Archetype::default();
            for key in &all_keys {
                let Some(row) = self.container.get_mut(key) else {
                    self.insert_pool.clear();
                    return Err(EngineError::IntegrityFailed(
                        "Process insert pool".to_string(),
                    ));
                };
                if let Some(val) = entity.container.get(key) {
                    row.add(Some(val.clone()));
                    let comp_arch = self
                        .component_archetype
                        .get(key)
                        .expect("Archetype should be present");
                    archetype.add(comp_arch);
                } else {
                    row.add(None);
                }
            }
            self.entity_archetype.add(archetype);
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
        self.entity_archetype.ids()
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

#[derive(Default)]
pub struct Query {
    types: QueryInputTypes,
}

impl Query {
    pub fn with_component<C: Any>(mut self) -> Self {
        let val = TypeId::of::<C>();
        self.types.insert(val);
        self
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
    fn em_comp_register() -> EngineResult<()> {
        let mut em = EntityManager::new();
        em.register_component::<Position>()?
            .register_component::<Health>()?;
        assert_eq!(em.container.len(), 2, "Failed to register components");
        Ok(())
    }

    fn create_em() -> EngineResult<EntityManager> {
        let mut em = EntityManager::new();
        em.register_component::<Position>()?
            .register_component::<Health>()?
            .register_component::<PlayerMarker>()?;

        em.add()
            .set(Position { x: 1.2, y: 3.4 })?
            .set(Health(100))?
            .set(PlayerMarker)?;
        em.add()
            .set(Position { x: 5.6, y: 7.8 })?
            .set(Health(100))?;

        assert_eq!(0, em.entities_count());
        em.apply()?;
        Ok(em)
    }

    #[test]
    fn em_add_entity() -> EngineResult<()> {
        let mut em = create_em()?;
        assert_eq!(2, em.entities_count());
        em.apply()?;
        assert_eq!(2, em.entities_count());
        Ok(())
    }

    #[test]
    fn em_remove_entity() -> EngineResult<()> {
        let mut em = create_em()?;
        let ids = em.all_ids();
        em.remove(ids[0]);
        em.apply()?;

        assert_eq!(1, em.entities_count());
        em.apply()?;

        assert_eq!(1, em.entities_count());
        Ok(())
    }

    #[test]
    fn em_fetch() -> EngineResult<()> {
        let em = create_em()?;
        let query = Query::default()
            .with_component::<PlayerMarker>()
            .with_component::<Health>();
        let result = em.fetch(&query);
        assert_eq!(result.entities.len(), 1);
        {
            let health_arr = result.get_ref::<Health>();
            assert_eq!(health_arr.len(), 1);
            assert_eq!(health_arr[0].0, 100);
        }
        {
            let mut health_arr_mut = result.get_mut::<Health>();
            assert_eq!(health_arr_mut.len(), 1);
            health_arr_mut[0].0 = 50;
        }
        {
            let result = em.fetch(&query);
            let health_arr = result.get_ref::<Health>();
            assert_eq!(health_arr.len(), 1);
            assert_eq!(health_arr[0].0, 50);
        }
        Ok(())
    }
}
