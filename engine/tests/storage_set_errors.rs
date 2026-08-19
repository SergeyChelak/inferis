use engine::{ComponentStorage, EngineError};

struct Registered(u32);
struct Unregistered;

#[test]
fn set_with_unregistered_component_type_is_reported() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Registered>().unwrap();
    let id = storage.add_entity();

    // the component type is inferred from the value, so passing one that was
    // never registered is easy to do by accident
    assert!(matches!(
        storage.set(id, Some(Unregistered)),
        Err(EngineError::ComponentNotRegistered)
    ));
    assert!(storage.set(id, Some(Registered(1))).is_ok());
    assert_eq!(storage.get::<Registered>(id).map(|x| x.0), Some(1));
}

#[test]
fn set_on_a_removed_entity_is_reported() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Registered>().unwrap();
    let id = storage.add_entity();
    assert!(storage.remove_entity(id));

    assert!(matches!(
        storage.set(id, Some(Registered(1))),
        Err(EngineError::EntityNotAlive(_))
    ));
}

#[test]
fn set_through_a_stale_id_cannot_touch_the_recycled_slot() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Registered>().unwrap();
    let old_id = storage.add_entity();
    storage.remove_entity(old_id);

    // same slot, new generation: the stale handle must be rejected loudly
    // instead of writing into the entity that now owns the slot
    let new_id = storage.add_entity();
    assert!(matches!(
        storage.set(old_id, Some(Registered(9))),
        Err(EngineError::EntityNotAlive(_))
    ));
    assert!(storage.get::<Registered>(new_id).is_none());
}

#[test]
fn removing_a_component_reports_the_same_failures() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Registered>().unwrap();
    let id = storage.add_entity();
    storage.set(id, Some(Registered(1))).unwrap();

    assert!(storage.set::<Registered>(id, None).is_ok());
    assert!(storage.get::<Registered>(id).is_none());
    assert!(matches!(
        storage.set::<Unregistered>(id, None),
        Err(EngineError::ComponentNotRegistered)
    ));
}
