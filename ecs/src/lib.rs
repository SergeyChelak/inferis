/// ECS, part of Inferis Project
mod type_map;

// TODO: replace
type GenerationalIndex = ();
type GenerationalIndexArray<T> = Vec<T>;

pub type Entity = GenerationalIndex;
pub type EntityMap<T> = GenerationalIndexArray<T>;

#[derive(Debug)]
pub enum EcsError {
    //
}

pub type EcsResult<T> = Result<T, EcsError>;
