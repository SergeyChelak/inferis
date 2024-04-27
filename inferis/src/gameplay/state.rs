use engine::{ComponentStorage, EngineResult, Query};

use super::Shot;

pub fn state_system(storage: &mut ComponentStorage) -> EngineResult<()> {
    cleanup_shots(storage)?;
    Ok(())
}

fn cleanup_shots(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<Shot>();
    let entities = storage.fetch_entities(&query);
    for id in entities {
        storage.set::<Shot>(id, None);
    }
    Ok(())
}
