use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

pub trait Component: Any {}

type ComponentEntry = Rc<RefCell<dyn Component>>;
type Storage = HashMap<TypeId, Vec<Option<ComponentEntry>>>;

pub struct EntityManager {
    storage: Storage,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            storage: Storage::default(),
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

    pub fn create_entity(&mut self) {
        // unknown return type yet
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
        todo!()
    }
}
