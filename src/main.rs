use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use test_bounded_async_worker_pool::WorkerPool;

#[tokio::main]
async fn main() {
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
