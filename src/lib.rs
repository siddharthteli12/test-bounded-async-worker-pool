pub mod error;
use std::future::Future;
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::error::{BoundedResult, PoolError};

pub struct WorkerPool {
    semaphore: Arc<Semaphore>,
    tasks: JoinSet<()>,
    is_shutdown: bool,
}

impl WorkerPool {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            semaphore: Arc::new(Semaphore::new(capacity)),
            tasks: JoinSet::new(),
            is_shutdown: false,
        }
    }

    pub async fn submit<F, Fut>(&mut self, factory: F) -> BoundedResult<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.is_shutdown {
            return Err(PoolError::Shutdown);
        }

        while self.tasks.try_join_next().is_some() {}

        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore closed");

        self.tasks.spawn(async move {
            factory().await;
            drop(permit);
        });

        Ok(())
    }

    pub async fn shutdown(mut self) {
        self.is_shutdown = true;

        while self.tasks.join_next().await.is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_submit_and_run() {
        let mut pool = WorkerPool::new(2);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..5 {
            let c = counter.clone();
            pool.submit(move || async move {
                c.fetch_add(1, Ordering::SeqCst);
            })
            .await
            .unwrap();
        }

        pool.shutdown().await;
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_submit_after_shutdown_returns_error() {
        let mut pool = WorkerPool::new(2);
        pool.is_shutdown = true;

        let result = pool.submit(|| async {}).await;

        assert_eq!(result, Err(PoolError::Shutdown));
    }

    #[tokio::test]
    #[should_panic]
    async fn test_zero_capacity_panics() {
        WorkerPool::new(0);
    }

    #[tokio::test]
    async fn integration_all_jobs_complete_under_load() {
        let total = 200usize;
        let mut pool = WorkerPool::new(8);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..total {
            let c = counter.clone();
            pool.submit(move || async move {
                sleep(Duration::from_micros(100)).await;
                c.fetch_add(1, Ordering::Relaxed);
            })
            .await
            .unwrap();
        }

        pool.shutdown().await;
        assert_eq!(counter.load(Ordering::Relaxed), total);
    }
}
