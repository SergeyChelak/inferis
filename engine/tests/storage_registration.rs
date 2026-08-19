use engine::{ComponentStorage, Query};

struct CompA(u32);
struct CompB(u32);

#[test]
fn register_component_after_entities_exist() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<CompA>().unwrap();
    let e1 = storage.add_entity();
    let e2 = storage.add_entity();

    // late registration: rows must stay indexable by existing entity ids
    storage.register_component::<CompB>().unwrap();
    storage.set(e1, Some(CompB(7))).unwrap();
    assert_eq!(storage.get::<CompB>(e1).map(|x| x.0), Some(7));
    storage.set(e2, Some(CompB(8))).unwrap();
    assert_eq!(storage.get::<CompB>(e2).map(|x| x.0), Some(8));

    let query = Query::new().with_component::<CompB>();
    assert_eq!(storage.fetch_entities(&query).len(), 2);
}

#[test]
fn recycled_entity_works_with_late_registered_component() {
    let mut storage = ComponentStorage::new();
    storage.register_component::<CompA>().unwrap();
    let e1 = storage.add_entity();
    storage.register_component::<CompB>().unwrap();

    // recycle the slot and use both rows through the new entity
    assert!(storage.remove_entity(e1));
    let e2 = storage.add_entity();
    storage.set(e2, Some(CompA(1))).unwrap();
    storage.set(e2, Some(CompB(2))).unwrap();
    assert_eq!(storage.get::<CompA>(e2).map(|x| x.0), Some(1));
    assert_eq!(storage.get::<CompB>(e2).map(|x| x.0), Some(2));
}
