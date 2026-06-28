# Surgeist App Runtime Foundation 11: AppProxy Wake Bridge And AppLoop Adapter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add app-owned proxy wake bridging and the native loop adapter boundary.

**Architecture:** This is split 11 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 11: AppProxy Wake Bridge And AppLoop Adapter

**Files:**
- Create: `src/app/proxy.rs`
- Create: `src/app/loop_.rs`
- Create: `src/app/testing.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write AppProxy wake coalescing test**

Add:

```rust
#[test]
fn app_proxy_coalesces_wakeups_while_queue_is_non_empty() {
    let wake = FakeWakeBridge::default();
    let proxy = AppProxy::<CounterInput>::new(wake.clone(), QueuePolicy::bounded(16));

    proxy.send_task(AppInput::new(CounterInput::Increment, InputProvenance::task(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
    ))).unwrap();
    proxy.send_task(AppInput::new(CounterInput::Increment, InputProvenance::task(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
    ))).unwrap();

    assert_eq!(wake.wake_count(), 1);
    assert_eq!(proxy.pending_len(), 2);

    let drained = proxy.drain_pending(8);
    assert_eq!(drained.len(), 2);
    assert_eq!(proxy.pending_len(), 0);
}
```

- [ ] **Step 2: Write closed-loop handling test**

Add:

```rust
#[test]
fn app_proxy_reports_closed_native_wake_bridge() {
    let wake = FakeWakeBridge::closed();
    let proxy = AppProxy::<CounterInput>::new(wake, QueuePolicy::bounded(16));

    let error = proxy
        .send_task(AppInput::new(CounterInput::Increment, InputProvenance::system()))
        .unwrap_err();

    assert_eq!(error.code(), AppProxyErrorCode::WakeFailed);
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing proxy and fake bridge.

- [ ] **Step 4: Implement proxy**

Add `proxy.rs` with:

- `WakeBridge` trait with `wake(&self) -> Result<(), AppProxyError>`;
- implementation for a wrapper around `window::Proxy` that calls a native wake command or request action once the native side exposes a suitable public call;
- `AppProxy<T>` owning `Arc<Mutex<VecDeque<AppInput<T>>>>`, bounded policy, coalesced pending wake flag, and bridge;
- `send_task`, `send_service`, `drain_pending`, and `pending_len`;
- `QueuePolicy` bounded capacity and overflow diagnostics;
- `AppProxyError` and `AppProxyErrorCode`.

`drain_pending` must clear the coalesced pending wake flag when the queue becomes empty. A later `send_task` or `send_service` after a full drain must wake again. Keep this behavior explicit because Task 14 stress tests rely on sustained wake coalescing across multiple drain cycles.

If `window::Proxy` lacks a public wake-only method, keep the `window::Proxy` implementation crate-private and drive tests through `WakeBridge`. Do not add typed task events to `window::UserEvent`.

- [ ] **Step 5: Add fake wake bridge support**

Create `testing.rs` with the fake bridge used by the Task 11 tests. Task 13 will extend this same file with the headless harness; it must not recreate or replace `FakeWakeBridge`.

```rust
#[derive(Clone, Debug, Default)]
pub struct FakeWakeBridge {
    state: std::sync::Arc<std::sync::Mutex<FakeWakeState>>,
}

#[derive(Clone, Debug, Default)]
struct FakeWakeState {
    closed: bool,
    wakes: usize,
}

impl FakeWakeBridge {
    #[must_use]
    pub fn closed() -> Self {
        let bridge = Self::default();
        bridge.state.lock().expect("fake wake bridge lock").closed = true;
        bridge
    }

    #[must_use]
    pub fn wake_count(&self) -> usize {
        self.state.lock().expect("fake wake bridge lock").wakes
    }
}

impl WakeBridge for FakeWakeBridge {
    fn wake(&self) -> Result<(), AppProxyError> {
        let mut state = self.state.lock().expect("fake wake bridge lock");
        if state.closed {
            return Err(AppProxyError::new(AppProxyErrorCode::WakeFailed));
        }
        state.wakes += 1;
        Ok(())
    }
}
```

Keep this fake in `app::testing` so subsequent runtime harness tests can reuse it instead of introducing a second wake fake.

- [ ] **Step 6: Implement AppLoop wrapper**

Add `loop_.rs` with:

- `AppLoop` marker/wrapper for `window::Loop`;
- `AppHandler` adapter trait documenting the native-to-runtime flow;
- a narrow constructor that stores app runtime and native loop pieces without changing window internals.

Keep this task compile-focused. Native loop behavior is verified through fakes until a demo app needs full execution.

- [ ] **Step 7: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo test --package surgeist --test app app_front_door_exports_expected_names
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/proxy.rs src/app/loop_.rs src/app/testing.rs src/app/tests.rs
```

Expected: tests pass, public app exports still compile, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add src/app/mod.rs src/app/proxy.rs src/app/loop_.rs src/app/testing.rs src/app/tests.rs
git commit -m "Add app proxy wake bridge"
```

---
