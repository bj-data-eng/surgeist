# Surgeist App Runtime Foundation 12: Executor Adapter, Fake Executor, And Optional Tokio Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add backend-neutral executor contracts, a fake executor, and optional Tokio adapter support.

**Architecture:** This split adds the transport-neutral work-plane adapter after tasks, services, runtime queues, and the app proxy already exist. The public app model stays executor-agnostic: Tokio is one optional backend, while in-process workers, blocking pools, sidecars, local servers, and external services remain compatible with the same task/event envelope.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 12: Executor Adapter, Fake Executor, And Optional Tokio Feature

**Executor boundary policy:** The executor starts or cancels work on behalf of committed runtime effects. It must not mutate app state directly, hold non-send UI/window/render handles, or expose runtime-specific handles to app authors. All worker output returns through `ExecutorEvent` and `AppProxy` with task id, attempt id, correlation/provenance data, and cancellation truth.

**Files:**
- Modify: `src/app/executor.rs`
- Create: `src/app/runtime_tokio.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/runtime.rs`
- Modify: `Cargo.toml`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write fake executor tests**

Add:

```rust
#[test]
fn fake_executor_records_spawn_and_cancel_requests() {
    let mut executor = FakeExecutor::default();
    let handle = executor
        .spawn_task(SpawnRequest::new(
            TaskId::from_u64(1),
            TaskAttemptId::from_u64(1),
            TaskKey::new("search:rust"),
            AppScope::app(),
        ))
        .expect("fake spawn should succeed");

    assert_eq!(handle.task_id(), TaskId::from_u64(1));
    assert_eq!(executor.spawned().len(), 1);

    executor
        .cancel(TaskHandle::new(handle.task_id(), handle.attempt_id()))
        .expect("fake cancel should succeed");
    assert_eq!(executor.cancelled(), &[TaskId::from_u64(1)]);
}
```

- [ ] **Step 2: Write Tokio feature compile test**

Add a feature-gated test:

```rust
#[cfg(feature = "app-runtime-tokio")]
#[test]
fn tokio_executor_is_hidden_behind_adapter() {
    use surgeist::app::runtime_tokio::TokioExecutor;

    let executor = TokioExecutor::new();
    assert_eq!(executor.name(), "tokio");
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
cargo test --package surgeist --features app-runtime-tokio app::tests::tokio_executor_is_hidden_behind_adapter
```

Expected: fail with missing executor types.

- [ ] **Step 4: Implement executor adapter**

Extend the `executor.rs` contract from Task 10 with:

- `FakeExecutor` for tests, implementing the existing `RuntimeExecutor` trait;
- `ExecutorEvent` envelope for task progress/completion sent through `AppProxy`.

Preserve the Task 10 `RuntimeExecutor` trait shape. Do not define a second executor trait. `SpawnRequest` already includes `BlockingPolicy`; use it so non-abortable work can report `Cancelling` honestly until it finishes. `ExecutorEvent` must include the producing task id and attempt id so Task 10 stale-event filtering can reject late events from old attempts.

Do not add Tokio types, Tokio imports, child process types, sockets, or Tokio-backed executors to `executor.rs`. This file is the backend-neutral contract used by fake executors and runtime tests.

- [ ] **Step 5: Implement Tokio executor behind feature**

Create `runtime_tokio.rs` behind the `app-runtime-tokio` feature:

```rust
#[cfg(feature = "app-runtime-tokio")]
pub struct TokioExecutor {
    runtime: tokio::runtime::Runtime,
}
```

Update `Cargo.toml` in this split, not earlier splits:

```toml
tokio = { version = "=1.48.0", optional = true, features = ["rt-multi-thread", "sync", "time"] }

[features]
app-runtime-tokio = ["dep:tokio"]
```

Implement `new`, `name`, and the `RuntimeExecutor` methods. `spawn_task` is for ordinary executor work described by a typed `SpawnRequest`; `spawn_blocking_task` is for blocking or CPU-heavy work that must not run on the UI/app owner thread. Do not accept raw closures in the public app API, and do not expose the Tokio runtime, `JoinHandle`, `mpsc` channels, or cancellation internals through public app APIs. Re-export the module from `app::mod.rs` only under the feature:

```rust
#[cfg(feature = "app-runtime-tokio")]
pub mod runtime_tokio;
```

- [ ] **Step 6: Hook runtime effect execution to executor requests**

Implement `RuntimeExecutor` for the fake executor in `executor.rs` and for the feature-gated Tokio executor in `runtime_tokio.rs` so `Runtime::with_executor(...)` can execute task effects whose kind is `EffectKindId::START_TASK` or `EffectKindId::CANCEL_TASK` through the adapter. Preserve the runtime behavior for instances created without an executor: task effects emit a structured `DiagnosticCode::EFFECT_FAILED`.

This split should not add a process supervisor. Future process/sidecar adapters should implement the same backend-neutral executor or service contracts instead of introducing a second lowering layer.

- [ ] **Step 7: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo test --package surgeist --features app-runtime-tokio app::tests::tokio_executor_is_hidden_behind_adapter
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/executor.rs src/app/runtime_tokio.rs src/app/runtime.rs src/app/tests.rs
```

Expected: tests pass with and without the feature, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add Cargo.toml src/app/mod.rs src/app/executor.rs src/app/runtime_tokio.rs src/app/runtime.rs src/app/tests.rs
git commit -m "Add app executor adapter"
```

---
