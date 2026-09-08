# Reference

## Package and public surface

[Cargo.toml](../Cargo.toml) defines package `surgeist-task` version `0.1.0`, library
`surgeist_task`, Rust edition 2024, and MIT licensing. It pins Tokio to `1.48.0`
with `rt-multi-thread`, `sync`, and `time`. It declares no crate features or
`rust-version` field.

[src/lib.rs](../src/lib.rs) is the public front door. Its implementation modules
are private and selected types are reexported at the crate root. It forbids unsafe
code. The fakes in `src/testing.rs` are internal test support, not public exports.

| Area | Principal types | Source |
| --- | --- | --- |
| Identity and provenance | `TaskId`, `TaskAttemptId`, `TaskKey`, `TaskName`, `ObserverId`, `CoalescingKey`, `CorrelationId`, `ResourceClassId`, `TaskProvenance` | [id.rs](../src/id.rs), [provenance.rs](../src/provenance.rs) |
| Scope | `TaskScope`, `TaskScopeSegment` | [scope.rs](../src/scope.rs) |
| Lifecycle | `TaskRecord`, `TaskSnapshot`, `TaskStatus`, `TaskTerminalStatus` | [lifecycle.rs](../src/lifecycle.rs) |
| Cancellation | `CancellationToken`, `CancellationView`, `CancelReason` | [cancel.rs](../src/cancel.rs) |
| Observation and registration | `TaskCoordination`, `TaskRegistration`, `ObservationChange`, `ObserverCountSnapshot` | [coordination.rs](../src/coordination.rs) |
| Policy | `TaskPolicy`, `TaskPriority`, `RetryPolicy`, `BlockingPolicy`, `UnobservedPolicy` | [policy.rs](../src/policy.rs) |
| Events | `TaskEvent`, `TaskJobEvent`, `TaskLifecycleEvent`, progress/output/diagnostic types | [event.rs](../src/event.rs) |
| Event queue | `TaskEventQueue`, `QueuePolicy`, `DrainBudget`, `QueueReport`, `DrainReport` | [queue.rs](../src/queue.rs) |
| Execution | `TaskExecutor`, `TaskSpawnRequest`, `TaskContext`, `TaskEventSink`, async/blocking job traits, handles, failures, reports | [executor.rs](../src/executor.rs) |
| Concrete runtime | `TokioTaskExecutor` | [runtime_tokio.rs](../src/runtime_tokio.rs) |

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
modules under `src/`. The tracked runnable example is
[basic_task.rs](../examples/basic_task.rs).

```sh
cargo check -p surgeist-task
cargo test -p surgeist-task
cargo clippy -p surgeist-task --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-task --check
```

The example's bounded offline invocation is documented in
[Getting started](getting-started.md).
