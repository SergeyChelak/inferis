/// ECS, part of Inferis Project

#[derive(Debug)]
pub enum EcsError {
    //
}

pub type EcsResult<T> = Result<T, EcsError>;
