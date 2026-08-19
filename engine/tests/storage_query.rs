use engine::{ComponentStorage, Query};

struct Registered;
struct Unregistered;

fn storage_with_one_entity() -> ComponentStorage {
    let mut storage = ComponentStorage::new();
    storage.register_component::<Registered>().unwrap();
    let id = storage.add_entity();
    storage.set(id, Some(Registered)).unwrap();
    storage
}

#[test]
fn query_with_unregistered_component_matches_nothing() {
    let storage = storage_with_one_entity();
    let query = Query::new().with_component::<Unregistered>();
    assert!(storage.fetch_entities(&query).is_empty());
    assert!(storage.fetch_first_entity(&query).is_none());
}

#[test]
fn mixed_query_with_unregistered_component_matches_nothing() {
    let storage = storage_with_one_entity();
    let query = Query::new()
        .with_component::<Registered>()
        .with_component::<Unregistered>();
    assert!(storage.fetch_entities(&query).is_empty());
    assert!(storage.fetch_first_entity(&query).is_none());
}

#[test]
fn registered_query_still_matches() {
    let storage = storage_with_one_entity();
    let query = Query::new().with_component::<Registered>();
    assert_eq!(storage.fetch_entities(&query).len(), 1);
    assert!(storage.fetch_first_entity(&query).is_some());
}
