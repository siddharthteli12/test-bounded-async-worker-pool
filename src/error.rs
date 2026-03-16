use thiserror::Error;

pub type BoundedResult<T> = Result<T, PoolError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PoolError {
    #[error("worker pool has been shut down")]
    Shutdown,
}
