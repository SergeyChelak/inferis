use engine::{ComponentStorage, EngineResult, EntityID, Float, Query, Vec2f};

use super::{Maze, NpcTag, Position};

pub fn ai_system(storage: &mut ComponentStorage, player_id: EntityID) -> EngineResult<()> {
    let query = Query::new().with_component::<NpcTag>();
    let entities = storage.fetch_entities(&query);
    for npc_id in entities {
        npc_behavior(storage, npc_id, player_id)?;
    }
    Ok(())
}

fn npc_behavior(
    storage: &mut ComponentStorage,
    npc_id: EntityID,
    player_id: EntityID,
) -> EngineResult<()> {
    Ok(())
}

fn vision(storage: &mut ComponentStorage, npc_id: EntityID, player_id: EntityID) -> Option<Float> {
    let Some(npc_position) = storage.get_mut::<Position>(npc_id).map(|x| x.0) else {
        return None;
    };
    let Some(player_position) = storage.get_mut::<Position>(npc_id).map(|x| x.0) else {
        return None;
    };
    let vector = player_position - npc_position;
    let angle = vector.y.atan2(vector.x);
    //
    None
}
