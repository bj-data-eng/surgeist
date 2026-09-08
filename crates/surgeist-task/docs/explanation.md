# Explanation

## Task contracts and application ownership

`surgeist-task` owns the task-domain boundary: identity, scope, lifecycle,
cancellation, progress, observation, admission and queueing contracts,
executor-facing policy, and execution behind those contracts. The public front
door is [src/lib.rs](../src/lib.rs).

Root `surgeist` owns app reducers and effects, UI/window integration, retained
tree bridges, native wake bridges, app resource/service descriptors, and mapping
task events into app inputs. Surgeist-to-Surgeist adapters and lowering belong
there. This keeps app-specific concepts out of this crate and avoids duplicating
task semantics in root. Root also owns integration tests and tools, workspace
wiring, and the API generator and generated audit artifacts. See
[AGENTS.md](../AGENTS.md) for the repository ownership and discovery authority.

## A contract boundary around Tokio

Task authors express work with `TaskSpawnRequest`, `TaskRunnable`, `TaskContext`,
and event types. Integrators can depend on `TaskExecutor` and `TaskEventSink`.
`TokioTaskExecutor` owns a multi-thread Tokio runtime with time enabled, runs
async jobs on that runtime, and runs blocking jobs on Tokio's blocking pool.
Tokio runtime and join handles remain inside the implementation.

Jobs emit progress, output, and diagnostics through their context. The executor
joins completed jobs and queues completion records internally. A later
`drain_finished()` emits terminal lifecycle events and returns outcome counts.
The separation means the owning runtime must service completion drainage; job
return alone does not deliver its terminal event.

The policy types describe a broader scheduling boundary than the concrete
executor currently implements. Priority, deduplication, and retry values are
carried as data. Observation changes return decisions for callers to act on.
There is no priority scheduler, automatic retry loop, or deduplicating admission
algorithm in the Tokio executor.

## Attempts protect lifecycle state

A task has semantic identity and an active attempt id. Events carry that
provenance, and `TaskRecord` uses it to reject stale attempts. A new attempt must
advance its id, and a terminal record cannot be reopened. This lets an integrator
distinguish updates from different attempts without treating late events as
current task state.

`TaskRecord`, observation tracking, event queueing, and executor execution are
separate building blocks. The executor does not automatically update a record or
connect observation decisions to cancellation. The integration owns those
connections.

## Cancellation describes what happened

`CancellationToken` shares a request flag and preserves the first cancellation
reason. A job observes the flag through its context or cancellation view and
decides where it can stop. Executor cancellation requests cooperation; it does
not abort the Tokio job or stop a running blocking function.

Returning `TaskFailure::cancelled(...)` reports cooperative cancellation.
Successful work can report `FinishedAfterCancel` when the executor observed a
request and the policy is `NonAbortableReportCancelling`. Under other blocking
policies, success reports `Completed` even if cancellation was requested. Other
failures after an observed request report `FailedToCancel`. The exact mapping is
in [Reference](reference.md#tokio-terminal-mapping).

Terminal emission is attempted once per drained completion, but a rejecting sink
can lose that event: the executor ignores terminal sink errors. Outcome counts
therefore describe drained completions, not guaranteed event delivery.

## Bounded progress traffic

Progress frequently supersedes earlier progress for the same measure. The queue
uses an explicit coalescing key together with task and attempt ids to retain the
latest value in that slot. Non-progress events retain their FIFO order until
drained, provided their insertion succeeded. Capacity bounds retained events;
overflow is reported rather than silently growing the queue.

Draining non-progress events before progress favors those retained events but
does not preserve global arrival order. A count budget bounds each drain, and
reports expose pending, dropped, and coalesced events without exposing queue
internals. Callers own backpressure handling and any application presentation of
those counts.
