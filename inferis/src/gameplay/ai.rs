use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Float, Query, Vec2f};

use super::{
    common::ray_cast_with_entity, Angle, CharacterState, NpcTag, Position, Shot, Transform,
    Velocity,
};

struct NpcData {
    player_id: EntityID,
    npc_id: EntityID,
    npc_position: Vec2f,
    delta_time: f32,
    angle: Float,
    vector: Vec2f,
}

impl NpcData {
    fn new(
        npc_id: EntityID,
        npc_position: Vec2f,
        player_id: EntityID,
        player_position: Vec2f,
        delta_time: f32,
    ) -> Self {
        let vector = player_position - npc_position;
        let angle = vector.y.atan2(vector.x);
        Self {
            npc_id,
            npc_position,
            player_id,
            delta_time,
            angle,
            vector,
        }
    }
}

pub fn ai_system(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    delta_time: f32,
) -> EngineResult<()> {
    let Some(player_position) = storage.get::<Position>(player_id).map(|x| x.0) else {
        return Ok(());
    };
    let query = Query::new().with_component::<NpcTag>();
    let entities = storage.fetch_entities(&query);
    for npc_id in entities {
        let Some(npc_position) = storage.get::<Position>(npc_id).map(|x| x.0) else {
            continue;
        };
        let data = NpcData::new(npc_id, npc_position, player_id, player_position, delta_time);
        update_npc_state(storage, &data)?;
        perform_npc_action(storage, &data)?;
    }
    Ok(())
}

fn update_npc_state(storage: &mut ComponentStorage, data: &NpcData) -> EngineResult<()> {
    let npc_id = data.npc_id;
    let Some(state) = storage.get::<CharacterState>(npc_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("CharacterState"));
    };
    use CharacterState::*;
    if matches!(state, Death | Damage) {
        return Ok(());
    }
    if let Some(distance) = vision(storage, data) {
        let new_state = if distance < 5.0 { Attack } else { Walk };
        storage.set(npc_id, Some(new_state));
        storage.set(npc_id, Some(Angle(data.angle)));
    } else {
        storage.set(npc_id, Some(Idle));
    }
    Ok(())
}

fn perform_npc_action(storage: &mut ComponentStorage, data: &NpcData) -> EngineResult<()> {
    let Some(state) = storage.get::<CharacterState>(data.npc_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("CharacterState"));
    };
    match state {
        CharacterState::Walk => {
            movement(storage, data);
        }
        CharacterState::Attack => {
            attack(storage, data);
        }
        _ => {
            // no op
        }
    }
    Ok(())
}

fn vision(storage: &mut ComponentStorage, data: &NpcData) -> Option<Float> {
    let Ok(result) = ray_cast_with_entity(storage, data.npc_id, data.npc_position, data.angle)
    else {
        return None;
    };
    let mut has_obstacles = false;
    if let Some(entity_id) = result {
        has_obstacles = entity_id != data.player_id;
    }
    if has_obstacles {
        None
    } else {
        Some(data.vector.hypotenuse())
    }
}

fn movement(storage: &mut ComponentStorage, data: &NpcData) {
    let Some(velocity) = storage.get::<Velocity>(data.npc_id).map(|x| x.0) else {
        return;
    };
    let angle = data.angle;
    let sin_a = angle.sin();
    let cos_a = angle.cos();
    let dist = velocity * data.delta_time;
    let transform = Transform {
        relative_x: dist * cos_a,
        relative_y: dist * sin_a,
        relative_angle: 0.0,
    };
    storage.set(data.npc_id, Some(transform));
}

fn attack(storage: &mut ComponentStorage, data: &NpcData) {
    let shot = Shot::new(data.npc_position, data.angle);
    storage.set(data.npc_id, Some(shot));
}
