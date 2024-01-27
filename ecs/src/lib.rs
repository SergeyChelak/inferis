pub mod common;
mod packed_array;
mod state;

use std::any::Any;

use common::EntityProvider;
use state::StateManager;

#[derive(Default)]
pub struct Ecs {
    state: StateManager,
}

impl Ecs {
    /// System must be registered at initialization step
    pub fn register_system<T: Any>(&mut self, system: T) {
        todo!()
    }

    pub fn entities(&mut self) -> &mut impl EntityProvider {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
