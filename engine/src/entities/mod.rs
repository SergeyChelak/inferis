pub mod handler;
pub mod storage;

#[derive(Debug)]
pub enum EcsError {
    ComponentNotRegistered,
}

pub type EcsResult<T> = Result<T, EcsError>;
