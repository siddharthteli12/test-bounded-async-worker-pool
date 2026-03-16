# Design: Bounded Async Worker Pool

## Overview

This project implements a bounded asynchronous worker pool using Tokio.

The pool accepts jobs dynamically and executes them asynchronously while ensuring that at most **N jobs run concurrently**. If the pool reaches its capacity, new submissions wait until a running job finishes.

The implementation uses Tokio primitives to ensure the system remains fully non-blocking and memory usage stays bounded.

---

## Architecture and Concurrency Model

The worker pool is built using two main components.

**Semaphore**

A `tokio::sync::Semaphore` is used to enforce the concurrency limit.  
The semaphore is initialized with `N` permits, where `N` is the maximum number of concurrent jobs.

Before a job starts executing, it must acquire a permit from the semaphore.  
When the job finishes, the permit is released automatically.

This guarantees that no more than `N` jobs run at the same time.

**JoinSet**

A `tokio::task::JoinSet` is used to track all spawned tasks.  
Each submitted job is executed inside a Tokio task that is added to the `JoinSet`.

The `JoinSet` allows the worker pool to wait for all running tasks to complete during shutdown.

---

## Backpressure

Backpressure is implemented using the semaphore.

When the pool is at capacity, all permits are already in use. In this situation, the call to `acquire_owned().await` waits asynchronously until a permit becomes available.

This causes the `submit()` call to pause until a running job finishes and releases a permit.

Because of this design:

- No unbounded queue is created
- Memory usage remains bounded
- Callers automatically wait when the pool is full

---

## Graceful Shutdown

Graceful shutdown is handled in two steps.

First, the pool marks itself as shut down using an internal flag. After this point, new calls to `submit()` return `PoolError::Shutdown`.

Second, the pool waits for all running tasks to finish by repeatedly awaiting tasks in the `JoinSet`.

Shutdown only completes once every accepted job has finished execution.

This ensures that no in-flight work is lost.

---

## Trade-offs

**Simplicity**

The implementation uses standard Tokio primitives (`Semaphore` and `JoinSet`). This keeps the design simple and easy to reason about.

**Performance**

The semaphore-based design avoids additional queues and keeps the critical path short. Tasks are spawned directly once a permit is acquired.

**Correctness**

The semaphore guarantees the concurrency limit, while `JoinSet` ensures all tasks are properly tracked and awaited during shutdown.

---

## Known Limitations

Jobs currently return `()`, so results are not returned to the caller. Applications that need results must use shared state or channels.

Jobs cannot be cancelled once submitted; they always run to completion.

All jobs share the same pool and concurrency limit. Priority scheduling is not supported.

Panics inside jobs are not explicitly handled and will be ignored when the task completes.