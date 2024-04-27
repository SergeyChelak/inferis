use engine::{ComponentStorage, EngineError, EngineResult, EntityID, FrameCounter, Query, Vec2f};

use crate::resource::*;

use super::{AnimationData, CharacterState, Maze, NpcTag, Position};

pub fn ai_system() -> EngineResult<()> {
    Ok(())
}

pub fn npc_update(
    storage: &mut ComponentStorage,
    delta_time: f32,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    let mut npc = Npc::new(storage, delta_time, player_id, maze_id)?;
    npc.update()
}

struct Npc<'a> {
    storage: &'a mut ComponentStorage,
    delta_time: f32,
    player_id: EntityID,
    maze_id: EntityID,
    player_pos: Vec2f,
}

impl<'a> Npc<'a> {
    pub fn new(
        storage: &'a mut ComponentStorage,
        delta_time: f32,
        player_id: EntityID,
        maze_id: EntityID,
    ) -> EngineResult<Self> {
        let Some(player_pos) = storage.get::<Position>(player_id).map(|x| x.0) else {
            return Err(EngineError::component_not_found("Position"));
        };
        Ok(Self {
            storage,
            delta_time,
            player_id,
            maze_id,
            player_pos,
        })
    }

    pub fn update(&mut self) -> EngineResult<()> {
        let query: Query = Query::new().with_component::<NpcTag>();
        for entity_id in self.storage.fetch_entities(&query) {
            self.update_character(entity_id)?;
        }
        Ok(())
    }

    fn update_character(&mut self, entity_id: EntityID) -> EngineResult<()> {
        let Some(state) = self.storage.get::<CharacterState>(entity_id).map(|x| *x) else {
            return Err(engine::EngineError::component_not_found("CharacterState"));
        };
        use CharacterState::*;
        match state {
            Idle(mut progress) => {
                self.update_animation(entity_id, NPC_SOLDIER_IDLE, &mut progress, |p| Idle(p));
            }
            Death(mut progress) => {
                if progress.is_completed() {
                    self.storage.remove_entity(entity_id);
                } else {
                    self.update_animation(entity_id, NPC_SOLDIER_DEATH, &mut progress, |p| {
                        Death(p)
                    });
                }
            }
            Attack(mut progress) => {
                self.update_animation(entity_id, NPC_SOLDIER_ATTACK, &mut progress, |p| Attack(p));
            }
            Walk(mut progress) => {
                // NPC_SOLDIER_WALK
            }
            Damage(mut progress) => {
                if progress.is_completed() {
                    self.storage
                        .set(entity_id, Some(Idle(FrameCounter::infinite())));
                } else {
                    self.update_animation(entity_id, NPC_SOLDIER_DAMAGE, &mut progress, |p| {
                        Damage(p)
                    });
                }
            }
        }
        self.search_target(entity_id)?;
        Ok(())
    }

    fn update_animation(
        &mut self,
        entity_id: EntityID,
        animation_id: &str,
        progress: &mut FrameCounter,
        prod: impl FnOnce(FrameCounter) -> CharacterState,
    ) {
        if !progress.is_performing() {
            let data = AnimationData::new(animation_id);
            self.storage.set(entity_id, Some(data));
        }
        progress.teak();
        self.storage.set(entity_id, Some(prod(*progress)));
    }

    fn search_target(&mut self, entity_id: EntityID) -> EngineResult<()> {
        let Some(pos) = self.storage.get::<Position>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let is_idle = if let Some(CharacterState::Idle(_)) =
            self.storage.get::<CharacterState>(entity_id).map(|x| *x)
        {
            true
        } else {
            false
        };
        let is_close = (pos - self.player_pos).hypotenuse() < 5.0;
        match (is_idle, is_close) {
            (true, true) => {
                self.storage.set(
                    entity_id,
                    Some(CharacterState::Attack(FrameCounter::infinite())),
                );
            }
            (false, false) => {
                self.storage.set(
                    entity_id,
                    Some(CharacterState::Idle(FrameCounter::infinite())),
                );
            }
            _ => {}
        }
        Ok(())
    }
}
