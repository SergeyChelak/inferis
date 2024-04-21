use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Query, Rectangle, Vec2f};

use crate::resource::PLAYER_SHOTGUN_SHOT_ANIM;

use super::{
    controller::ControllerState, ray_caster::ray_cast, Angle, AnimationData, BoundingBox, Maze,
    NpcState, NpcTag, Position,
};

pub fn perform_shots(
    storage: &mut ComponentStorage,
    controller: &ControllerState,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    if controller.shot_pressed {
        handle_player_shot(storage, player_id, maze_id)?;
    }
    Ok(())
}

fn handle_player_shot(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    // TODO: is it smart enough?
    if storage.get::<AnimationData>(player_id).is_some() {
        return Ok(());
    };
    let data = AnimationData {
        frame_counter: 0,
        target_frames: 60,
        animation_id: PLAYER_SHOTGUN_SHOT_ANIM.to_string(),
    };
    storage.set(player_id, Some(data));
    cast_shoot(storage, player_id, maze_id)
}

#[derive(Debug)]
enum CastItem {
    Wall,
    Enemy(EntityID),
}

fn cast_shoot(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    let Some(player_pos) = storage.get::<Position>(player_id).map(|x| x.0) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(player_angle) = storage.get::<Angle>(player_id).map(|x| x.0) else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let query: Query = Query::new()
        .with_component::<NpcTag>()
        .with_component::<BoundingBox>();
    let enemies = storage.fetch_entities(&query);
    let check = |point: Vec2f| {
        for entity_id in &enemies {
            let Some(pos) = storage.get::<Position>(*entity_id).map(|x| x.0) else {
                continue;
            };
            let Some(bounding_box) = storage.get::<BoundingBox>(*entity_id).map(|x| x.0) else {
                continue;
            };
            let rect = Rectangle::with_pole(pos, bounding_box);
            if rect.contains(point) {
                println!("[cast_shoot] attacked enemy with id {}", entity_id.index());
                return Some(CastItem::Enemy(*entity_id));
            }
        }
        if let Some(true) = storage.get::<Maze>(maze_id).map(|x| x.is_wall(point)) {
            println!("[cast_shoot] shoot in the wall");
            return Some(CastItem::Wall);
        };
        None
    };
    let item = ray_cast(player_pos, player_angle, &check);
    match item.value {
        Some(CastItem::Enemy(id)) => {
            println!("[shot] id {} updated with damage", id.index());
            storage.set::<NpcState>(id, Some(NpcState::Damage));
        }
        _ => {
            println!("Result value: {:?}", item.value);
        }
    }
    Ok(())
}
