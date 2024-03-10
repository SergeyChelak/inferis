use std::{
    any::{Any, TypeId},
    cell::{RefCell, RefMut},
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub trait Component: Any {}

type ComponentEntry = Rc<RefCell<dyn Component>>;
type Storage = HashMap<TypeId, Vec<Option<ComponentEntry>>>;

pub struct EntityManager {
    storage: Storage,
    insert_pool: Vec<Rc<RefCell<EntityBuilder>>>,
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
        // let builder = EntityBuilder::default();
        // let builder_ref = Rc::new(RefCell::new(builder));
        // self.insert_pool.push(builder_ref);
        // self.insert_pool.last().unwrap().borrow_mut()
        // builder
        todo!()
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
        todo!()
    }

    fn process_insert_pool(&mut self) {
        let all_keys = self.component_types();
        for entry in &self.insert_pool {
            let Ok(builder) = entry.try_borrow() else {
                continue;
            };
            if builder.is_dirty {
                continue;
            }
            builder
                .components
                .iter()
                .filter(|(key, _)| all_keys.contains(key))
                .for_each(|(key, value)| {
                    self.storage.get_mut(key).and_then(|arr| {
                        arr.push(Some(value.clone()));
                        Some(())
                    });
                });
        }
        self.insert_pool.clear();
    }

    fn component_types(&self) -> HashSet<TypeId> {
        self.storage.keys().copied().collect::<HashSet<TypeId>>()
    }
}

pub struct EntityBuilder {
    is_dirty: bool,
    components: HashMap<TypeId, ComponentEntry>,
}

impl Default for EntityBuilder {
    fn default() -> Self {
        Self {
            is_dirty: true,
            components: HashMap::default(),
        }
    }
}

impl EntityBuilder {
    pub fn with_component<C: Component>(mut self, value: C) -> Self {
        let key = TypeId::of::<C>();
        let elem = Rc::new(RefCell::new(value));
        self.components.insert(key, elem);
        self
    }

    pub fn store(mut self) {
        self.is_dirty = false;
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
        em.create_entity()
            .with_component(C1(123))
            .with_component(C2 { x: 1.0, y: 2.0 })
            .store();
    }
}
