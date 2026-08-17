use engine::ComponentStorage;

struct Comp(u32);

#[test]
fn double_remove_is_rejected() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Comp>().unwrap();
    let id = storage.add_entity();
    assert!(storage.remove_entity(id));
    assert!(!storage.remove_entity(id));
    assert_eq!(storage.len(), 0);
}

#[test]
fn stale_id_does_not_affect_recycled_slot() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Comp>().unwrap();
    let old_id = storage.add_entity();
    storage.remove_entity(old_id);

    // the slot is recycled under a new generation
    let new_id = storage.add_entity();
    assert!(storage.set(new_id, Some(Comp(5))));

    // the stale handle must be rejected and must not kill the new entity
    assert!(!storage.remove_entity(old_id));
    assert!(storage.is_alive(new_id));
    assert_eq!(storage.get::<Comp>(new_id).map(|x| x.0), Some(5));
    assert_eq!(storage.len(), 1);
}

#[test]
fn remove_all_entities_clears_storage() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Comp>().unwrap();
    for i in 0..100 {
        let id = storage.add_entity();
        storage.set(id, Some(Comp(i)));
    }
    assert_eq!(storage.len(), 100);
    storage.remove_all_entities();
    assert_eq!(storage.len(), 0);
    assert!(storage.is_empty());
}
