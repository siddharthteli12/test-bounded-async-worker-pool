use thiserror::Error;
use tokio::sync::AcquireError;

pub type BoundedResult<T> = Result<T, PoolError>;

#[derive(Debug, Error)]
pub enum PoolError {
    #[error("worker pool has been shut down")]
    Shutdown,

    #[error("acquire error {0}")]
    AcquireError(#[from] AcquireError),
}
