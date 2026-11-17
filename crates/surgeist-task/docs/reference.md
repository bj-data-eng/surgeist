# Reference

## Package and public surface

[Cargo.toml](../Cargo.toml) defines package `surgeist-task` version `0.1.0`, library
`surgeist_task`, Rust edition 2024, and MIT licensing. It pins Tokio to `1.48.0`
with `rt-multi-thread`, `sync`, and `time`. It declares no crate features or
`rust-version` field.

[src/lib.rs](../src/lib.rs) is the public front door. The crate root declares
eleven public modules and the `CRATE_NAME` constant; it re-exports no type, and
every public item has one path, in its module. The implementation lives in the
private tree [src/internal/](../src/internal/). It forbids unsafe code. The fakes
in [src/internal/testing.rs](../src/internal/testing.rs) are internal test
support, not public exports.

| Module | Principal types | Implementation |
| --- | --- | --- |
| [`identity`](../src/identity.rs) | `TaskId`, `TaskAttemptId`, `TaskKey`, `TaskName`, `ObserverId`, `CoalescingKey`, `CorrelationId`, `ResourceClassId`, `TaskProvenance`, `TaskIdError` | [id.rs](../src/internal/id.rs), [provenance.rs](../src/internal/provenance.rs) |
| [`scope`](../src/scope.rs) | `TaskScope`, `TaskScopeSegment`, `TaskScopeError` | [scope.rs](../src/internal/scope.rs) |
| [`lifecycle`](../src/lifecycle.rs) | `TaskRecord`, `TaskSnapshot`, `TaskStatus`, `TaskTerminalStatus`, `TaskLifecycleError` | [lifecycle.rs](../src/internal/lifecycle.rs) |
| [`cancellation`](../src/cancellation.rs) | `CancellationToken`, `CancellationView`, `CancelReason` | [cancel.rs](../src/internal/cancel.rs) |
| [`coordination`](../src/coordination.rs) | `TaskCoordination`, `TaskRegistration`, `ObservationChange`, `ObserverCountSnapshot`, `TaskRegistrationError` | [coordination.rs](../src/internal/coordination.rs) |
| [`policy`](../src/policy.rs) | `TaskPolicy`, `TaskPriority`, `RetryPolicy`, `BlockingPolicy`, `UnobservedPolicy` | [policy.rs](../src/internal/policy.rs) |
| [`event`](../src/event.rs) | `TaskEvent`, `TaskJobEvent`, `TaskLifecycleEvent`, output and diagnostic events, `TaskEventSink`, `TaskEventSinkError` | [event.rs](../src/internal/event.rs), [executor.rs](../src/internal/executor.rs) |
| [`progress`](../src/progress.rs) | `TaskProgressEvent`, `TaskProgressValue`, `PercentComplete`, `PercentCompleteError` | [event.rs](../src/internal/event.rs) |
| [`queue`](../src/queue.rs) | `TaskEventQueue`, `QueuePolicy`, `DrainBudget`, `QueueReport`, `DrainReport`, `QueueError` | [queue.rs](../src/internal/queue.rs) |
| [`job`](../src/job.rs) | `TaskRunnable`, `TaskContext`, async/blocking job traits, `TaskFailure` | [executor.rs](../src/internal/executor.rs) |
| [`executor`](../src/executor.rs) | `TaskExecutor`, `TaskSpawnRequest`, handles, drain reports, `ExecutorError`, `TokioTaskExecutor` | [executor.rs](../src/internal/executor.rs), [runtime_tokio.rs](../src/internal/runtime_tokio.rs) |

Every public error type implements `Display` and `core::error::Error`.
`TaskRegistrationError::source` returns the identity or scope error that caused
it; the other error types wrap no error and return no source.

## Lifecycle states

`TaskRecord` associates an id, key, scope, policy, cancellation token, observer
count, and optional active attempt with a status.

| Status | Meaning |
| --- | --- |
| `Queued` | Awaiting execution; also the status immediately after `start_attempt`. |
| `Running` | The active attempt is executing. |
| `Waiting` | The active attempt is waiting on external work. |
| `Blocked` | The active attempt cannot make progress until a dependency changes. |
| `Cancelling` | Cancellation was requested while the task was non-terminal. |
| `Completed` | Terminal successful completion. |
| `Failed` | Terminal failure. |
| `Cancelled` | Terminal cancellation confirmation. |
| `FinishedAfterCancel` | Terminal successful completion after cancellation. |
| `FailedToCancel` | Terminal failure while cancellation was requested. |

Attempt ids must advance when `start_attempt` is called. Transitions reject stale
attempts and terminal records remain terminal. Starting an attempt leaves it
queued; normal completion and failure transitions require it to have left the
queued state. `finish_after_cancel` specifically requires `Cancelling`.

## Tokio terminal mapping

The executor samples the token after joining the job. On `drain_finished()`, it
removes the completion from active tracking, increments the corresponding report
counter, and attempts to emit one terminal event. Failure outcomes also attempt
an error diagnostic. Sink errors on these emissions are ignored.

| Joined outcome | Cancellation sampled | Policy condition | Terminal event |
| --- | --- | --- | --- |
| `Ok(())` | Requested | `NonAbortableReportCancelling` | `FinishedAfterCancel` |
| `Ok(())` | Requested | Any other blocking policy | `Completed` |
| `Ok(())` | Not requested | Any | `Completed` |
| `TaskFailureKind::Cancelled` | Either | Any | `Cancelled` |
| Other failure, panic, or join failure | Requested | Any | `FailedToCancel` |
| Other failure, panic, or join failure | Not requested | Any | `Failed` |

These are executor event outcomes. A `TaskRecord` is a separate state model;
the integrator owns applying relevant lifecycle changes to it.

## Policy and queue defaults

`TaskPolicy::continue_when_unobserved()` and the spawn-request default use no
deduplication, normal priority, one retry-policy attempt, and
`BlockingPolicy::Abortable`. `cancel_when_unobserved()` changes the unobserved
policy to `Cancel`. Builder methods can set deduplication, priority, maximum
attempts, and blocking policy. The executor does not enforce priority,
deduplication, retry scheduling, or automatic abortion from those settings.

`TaskCoordination` counts distinct observers by task key. Detaching the last
observer returns a policy decision; it does not itself execute that decision.

Queue capacity counts retained non-progress events plus distinct progress slots.
A slot is `(task id, attempt id, coalescing key)`. Replacing an existing progress
slot succeeds even at capacity; adding an event at capacity returns overflow and
increments the dropped count. Drains return non-progress FIFO events before
progress slots in their insertion order. Drop and coalescing counters are
cumulative. `DrainBudget::new()` defaults to an unlimited event count.

## Repository commands

These are the crate-local command inventory, not a separate delivery policy.
[AGENTS.md](../AGENTS.md) owns agent discovery and the command inventory; the
selected workflow determines which checks apply. Tests are inline `#[cfg(test)]`
modules under `src/` and public-API tests under `tests/`. The tracked runnable example is
[basic_task.rs](../examples/basic_task.rs).

```sh
cargo check -p surgeist-task
cargo test -p surgeist-task
cargo clippy -p surgeist-task --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-task --check
```

The example's bounded offline invocation is documented in
[Getting started](getting-started.md).
