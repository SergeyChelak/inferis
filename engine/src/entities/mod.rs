pub mod handler;
pub mod storage;

#[derive(Debug)]
pub enum EcsError {
    //
}

pub type EcsResult<T> = Result<T, EcsError>;
