use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub trait Component: Any {}

type ComponentEntry = Rc<RefCell<dyn Component>>;
type Storage = HashMap<TypeId, Vec<Option<ComponentEntry>>>;
type EntityBundle = HashMap<TypeId, ComponentEntry>;

pub struct EntityManager {
    storage: Storage,
    insert_pool: Vec<EntityBundle>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            storage: Storage::default(),
            insert_pool: Vec::new(),
        }
    }

    pub fn register_component<C: Component>(&mut self) {
        let id = TypeId::of::<C>();
        // TODO: convert to result
        assert!(
            self.storage.get(&id).is_none(),
            "Component already registered"
        );
        self.storage.insert(id, Vec::new());
        // TODO: Ok(...)
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }

    fn add_entity(&mut self, bundle: EntityBundle) {
        self.insert_pool.push(bundle);
    }

    pub fn entity(&self) {
        // unknown return type yet
        todo!()
    }

    pub fn remove_entity(&mut self) {
        // not clear what is expected argument and result
        todo!()
    }

    pub fn update(&mut self) {
        self.process_remove_pool();
        self.process_insert_pool();
    }

    fn process_remove_pool(&mut self) {
        //todo!()
    }

    fn process_insert_pool(&mut self) {
        let all_keys = self.component_types();
        for entry in &self.insert_pool {
            for key in all_keys.iter() {
                let Some(row) = self.storage.get_mut(&key) else {
                    panic!("Component not found");
                };
                if let Some(value) = entry.get(&key) {
                    row.push(Some(value.clone()));
                } else {
                    row.push(None);
                }
            }
        }
        self.insert_pool.clear();
    }

    fn component_types(&self) -> HashSet<TypeId> {
        self.storage.keys().copied().collect::<HashSet<TypeId>>()
    }
}

pub struct EntityBuilder<'a> {
    components: EntityBundle,
    entity_manager: &'a mut EntityManager,
}

impl<'a> EntityBuilder<'a> {
    pub fn new(entity_manager: &'a mut EntityManager) -> Self {
        Self {
            components: EntityBundle::default(),
            entity_manager,
        }
    }

    pub fn with_component<C: Component>(mut self, value: C) -> Self {
        let key = TypeId::of::<C>();
        let elem = Rc::new(RefCell::new(value));
        self.components.insert(key, elem);
        self
    }

    pub fn build(self) {
        self.entity_manager.add_entity(self.components);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct C1(i32);
    impl Component for C1 {}
    struct C2 {
        x: f32,
        y: f32,
    }
    impl Component for C2 {}

    #[test]
    fn em_create() {
        let mut em = EntityManager::new();
        em.register_component::<C1>();
        em.register_component::<C2>();
        em.create_entity()
            .with_component(C1(123))
            .with_component(C2 { x: 1.0, y: 2.0 })
            .build();
        em.create_entity()
            .with_component(C1(234))
            .with_component(C2 { x: 2.3, y: 4.5 })
            .build();
        em.update();
        for (_, row) in em.storage.iter() {
            assert_eq!(row.len(), 2);
        }
    }
}
