use engine::{
    fetch_first,
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float, Query,
};

use super::{components, subsystems::ray_cast_from_entity};

const ATTACK_DETECTION_SENSITIVITY: Float = 0.3;

pub struct DamageSystem {
    frames: usize,
    maze_id: EntityID,
}

impl DamageSystem {
    pub fn new() -> Self {
        Self {
            frames: Default::default(),
            maze_id: Default::default(),
        }
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        if storage.is_alive(self.maze_id) {
            return Ok(());
        }
        self.maze_id = fetch_first::<components::Maze>(storage).ok_or(
            EngineError::unexpected_state("[v2.damage] maze entity not found"),
        )?;
        Ok(())
    }

    fn process_shot(
        &self,
        storage: &mut ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let Some(shot) = storage.get::<components::Shot>(entity_id).map(|x| *x) else {
            return Ok(());
        };
        if shot.deadline != self.frames {
            return Ok(());
        }
        // TODO: it's a lazy implementation to obtain the shot damage value
        // The correct approach is to provide the damage value as part of the Shot component
        // In the future, user can change weapon type but damaged will be calculated based on
        // the currently selected type but not that one which used for shooting
        // Now I don't think that is a problem because there is no option to change ammo
        let Some(weapon_damage) = storage
            .get::<components::Weapon>(entity_id)
            .map(|x| x.damage)
        else {
            return Ok(());
        };
        let Some(target_id) = ray_cast_from_entity(
            entity_id,
            storage,
            self.maze_id,
            shot.position,
            shot.angle,
            ATTACK_DETECTION_SENSITIVITY,
        )?
        else {
            return Ok(());
        };
        // println!("[v2.damage] targeted {}", target_id.index());
        // accumulate damages
        let total_damage = weapon_damage
            + storage
                .get::<components::Damage>(target_id)
                .map(|x| x.0)
                .unwrap_or_default();
        storage.set::<components::Damage>(target_id, Some(components::Damage(total_damage)));
        Ok(())
    }
}

impl GameSystem for DamageSystem {
    fn setup(
        &mut self,
        storage: &mut ComponentStorage,
        _asset_manager: &AssetManager,
    ) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
        println!("[v2.damage] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        _delta_time: Float,
        storage: &mut ComponentStorage,
        _asset_manager: &AssetManager,
    ) -> EngineResult<GameSystemCommand> {
        self.update_storage_cache(storage)?;
        self.frames = frames;

        let query = Query::new().with_component::<components::Shot>();
        let entities = storage.fetch_entities(&query);
        for entity_id in entities {
            self.process_shot(storage, entity_id)?;
        }
        Ok(GameSystemCommand::Nothing)
    }
}
