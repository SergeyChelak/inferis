pub mod common;
mod packed_array;
mod state;

use std::any::Any;

use common::{EcsResult, EntityID};
use state::StateManager;

pub struct Ecs {
    state: StateManager,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            state: StateManager::default(),
        }
    }

    /// System must be registered at initialization step
    pub fn register_system<T: Any>(&mut self, system: T) {
        todo!()
    }

    pub fn entity(&mut self) -> EcsResult<Entity> {
        let id = self.state.create_entity()?;
        Ok(Entity {
            id,
            state: &mut self.state,
        })
    }

    pub fn state(&mut self) -> &mut StateManager {
        &mut self.state
    }
}

pub struct Entity<'a> {
    id: EntityID,
    state: &'a mut StateManager,
}

impl<'a> Entity<'a> {
    pub fn add_component<T: Any>(&mut self, component: T) -> EcsResult<&mut Self> {
        self.state.entity_add_component(self.id, component)?;
        Ok(self)
    }

    pub fn remove_component<T: Any>(&mut self) -> EcsResult<&mut Self> {
        self.state.entity_remove_component::<T>(self.id)?;
        Ok(self)
    }

    pub fn dispose(self) -> EcsResult<()> {
        self.state.delete_entity(self.id)
    }

    pub fn as_id(&self) -> EntityID {
        self.id
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
