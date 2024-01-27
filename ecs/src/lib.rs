pub mod common;
mod packed_array;
mod resource;
mod state;

use resource::AssetManager;
use state::StateManager;

#[derive(Default)]
pub struct Ecs {
    state_manager: StateManager,
    asset_manager: AssetManager,
}

impl Ecs {
    /// System must be registered at initialization step
    // pub fn register_system<T: Any>(&mut self, system: T) {
    //     todo!()
    // }

    pub fn entities(&mut self) -> &mut StateManager {
        &mut self.state_manager
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
