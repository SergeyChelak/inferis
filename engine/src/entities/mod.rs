pub mod entity_manager;
pub mod type_map;

// type GenerationalIndex = ();
// type GenerationalIndexArray<T> = Vec<T>;

pub type Entity = usize;
// pub type EntityMap<T> = GenerationalIndexArray<T>;

#[derive(Debug)]
pub enum EcsError {
    //
}

pub type EcsResult<T> = Result<T, EcsError>;
