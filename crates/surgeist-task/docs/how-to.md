# How-to

These procedures are for task authors and runtime integrators who have completed
[Getting started](getting-started.md). They describe the existing contracts;
the caller supplies application state, event consumption, and scheduling policy.

## Execute an async or blocking job

Start with an input type and output type that are `Send + 'static`, plus a
`TaskEventSink<Output>` that can safely receive events from worker threads.

1. Build a `TaskSpawnRequest` with task and attempt ids, a key, scope,
   cancellation token, input, event sink, and runnable. Use `unit_input()` for
   `()` input. Handle the error from `build()` if a required field is missing.
2. Choose `async_fn` or `blocking_fn` on the builder, or provide an implementation
   of `AsyncTaskJob` or `BlockingTaskJob` through `TaskRunnable`.
3. Create `TokioTaskExecutor` and handle its construction error. Through
   `TaskExecutor<Input, Output>`, call `spawn_task` for an async runnable or
   `spawn_blocking_task` for a blocking runnable. A mismatched runnable returns
   `ExecutorErrorKind::InvalidRequest`.
4. Retain the returned handle and call `drain_finished()` as part of the owning
   runtime's servicing loop. Where type inference needs help, use the fully
   qualified `TaskExecutor<Input, Output>` method.

Verify job progress/output at the sink, and terminal outcomes in both the sink
and `ExecutorDrainReport`. Completion drainage performs terminal emission;
successful spawning alone does not prove completion. The focused async and
blocking executor tests in [runtime_tokio.rs](../src/runtime_tokio.rs) show both
paths.

## Make cancellation observable

Start with the token used by the request and a job that can reach cooperative
cancellation checks.

1. Check `TaskContext::is_cancelled()` or its `CancellationView` at appropriate
   points in the job. To report that work stopped due to cancellation, return
   `TaskFailure::cancelled(...)`.
2. Request cancellation using the shared token and a `CancelReason`, or call
   `TaskExecutor::cancel` with the active `TaskHandle`. Executor cancellation
   records a user-requested reason.
3. For work that cannot be aborted and should report successful completion after
   cancellation distinctly, set
   `BlockingPolicy::NonAbortableReportCancelling` on the request's `TaskPolicy`.
4. Continue servicing completion drainage until the job settles.

Verify the token's cancellation view and the eventual outcome using the
[terminal mapping](reference.md#tokio-terminal-mapping). A cancellation request
does not stop a blocking function. If `TaskCoordination::unobserve` returns
`ObservationChange::CancelRequested`, the caller must connect that decision to
the task's cancellation mechanism.

## Bound and drain task events

Start with a `TaskEventQueue<Output>` owned by the event-consuming integration.
The queue itself is separate from `TaskEventSink`; provide the synchronization
and sink adapter needed by that integration.

1. Construct the queue with `QueuePolicy::bounded(capacity)`.
2. Push task events and handle `QueueErrorCode::Overflow`. Use a stable
   `CoalescingKey` for successive updates to the same progress measure.
3. Drain with `DrainBudget::new().max_events(limit)` to bound the number of
   returned events. Consume the returned events or take them with `into_events()`.
4. Inspect `QueueReport` and `DrainReport` for pending events and cumulative
   dropped/coalesced counts.

Verify that repeated progress updates for the same task, attempt, and key retain
the newest value, while different attempts remain separate. Non-progress events
drain first in FIFO order; the queue does not promise one global arrival order
across progress and non-progress events. See [queue.rs](../src/queue.rs) for the
focused capacity, coalescing, and drainage tests.
