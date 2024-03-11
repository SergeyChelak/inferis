pub mod entity_manager;
mod storage;

#[derive(Debug)]
pub enum EcsError {
    //
}

pub type EcsResult<T> = Result<T, EcsError>;
