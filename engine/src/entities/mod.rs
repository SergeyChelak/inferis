pub mod entity_manager;

pub type EntityID = usize;

#[derive(Debug)]
pub enum EcsError {
    //
}

pub type EcsResult<T> = Result<T, EcsError>;
