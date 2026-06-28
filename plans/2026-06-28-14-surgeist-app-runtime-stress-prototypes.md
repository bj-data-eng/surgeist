# Surgeist App Runtime Foundation 14: Stress And Prototype Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stress and prototype validation tests for runtime behavior.

**Architecture:** This split validates the runtime shape with focused prototypes instead of production integrations. The prototypes exercise stale-event rejection, backpressure, shared service observation, JSON-RPC-like correlation, cancellation truth, and non-abortable blocking work without introducing real daemons or external dependencies.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

## Phase 2: Prototype Validation And Example

Tasks 14 through 16 validate the architecture with deterministic prototypes, stress cases, and one small example. Keep production changes in this phase limited to fixes discovered by the prototypes; broad feature-crate behavior remains outside this first slice.

### Task 14: Stress And Prototype Tests

**Prototype policy:** These tests model the contracts that real app features will later depend on. Keep them fake and deterministic: no network, no child process, no local server, no clock sleeps, and no real Tokio runtime requirement. The JSON-RPC prototype should prove request/response/event correlation, not implement an MCP client.

**Files:**
- Modify: `src/app/tests.rs`
- Modify: `src/app/testing.rs`
- Modify: `src/app/runtime.rs`
- Modify: `src/app/coord.rs`
- Modify: `src/app/task.rs`
- Modify: `src/app/service.rs`

- [ ] **Step 1: Add latest-search-wins stale completion test**

Add:

```rust
#[test]
fn prototype_latest_search_wins_rejects_stale_completion() {
    let mut app = PrototypeApp::latest_search();

    app.start_search("rust", TaskAttemptId::from_u64(1));
    app.start_search("rust async", TaskAttemptId::from_u64(2));
    app.complete_search(TaskAttemptId::from_u64(1), vec!["old"]);
    app.complete_search(TaskAttemptId::from_u64(2), vec!["new"]);
    app.drain();

    assert_eq!(app.search_results(), &["new"]);
    assert_eq!(app.diagnostics().count(&DiagnosticCode::STALE_TASK_EVENT), 1);
}
```

- [ ] **Step 2: Add append-only log stream backpressure test**

Add:

```rust
#[test]
fn prototype_log_stream_accumulates_ordered_entries_with_budgeted_draining() {
    let mut app = PrototypeApp::log_stream(RuntimeBudget::new().max_task_events(10));

    for index in 0..35 {
        app.push_log_line(format!("line-{index:02}"));
    }
    app.drain();

    assert_eq!(app.log_lines().len(), 10);
    assert_eq!(app.remaining_task_inputs(), 25);

    app.drain_all();
    assert_eq!(app.log_lines().first().unwrap(), "line-00");
    assert_eq!(app.log_lines().last().unwrap(), "line-34");
}
```

- [ ] **Step 3: Add wake bridge stress test**

Add:

```rust
#[test]
fn stress_ten_thousand_task_events_use_coalesced_wakeups_and_budgeted_drains() {
    let mut app = PrototypeApp::progress_counter(RuntimeBudget::new().max_task_events(128));

    for index in 0..10_000 {
        app.proxy().send_task(app.progress_event(index)).unwrap();
    }

    assert!(app.fake_wake().wake_count() < 100);
    app.drain_all();
    assert_eq!(app.progress_count(), 10_000);
    assert_eq!(app.reducer_reentry_count(), 0);
}
```

- [ ] **Step 4: Add shared-service and fake JSON-RPC tests**

Add:

```rust
#[test]
fn prototype_two_surfaces_share_app_scoped_task_until_last_observer_detaches() {
    let mut app = PrototypeApp::shared_compile_service();
    let left = app.open_surface("left");
    let right = app.open_surface("right");

    app.observe_compile(left);
    app.observe_compile(right);
    app.close_surface(left);

    assert_eq!(app.compile_task_status(), TaskStatus::Running);

    app.close_surface(right);
    assert_eq!(app.compile_task_status(), TaskStatus::Cancelling);
}

#[test]
fn prototype_jsonrpc_service_handles_out_of_order_progress_cancel_timeout_and_reconnect() {
    let mut app = PrototypeApp::jsonrpc_service();

    let first = app.call_tool("compile");
    let second = app.call_tool("docs");
    app.notify_progress(second, "half");
    app.respond(first, "compiled");
    app.cancel(second);
    app.timeout(second);
    app.reconnect();
    app.drain_all();

    assert_eq!(app.response(first), Some("compiled"));
    assert_eq!(app.request_status(second), ServiceRequestStatus::TimedOutAfterCancel);
    assert_eq!(app.service_status(ServiceId::new("jsonrpc")), ServiceStatus::Running);
}
```

- [ ] **Step 5: Add blocking media cancellation truth test**

Add:

```rust
#[test]
fn prototype_blocking_media_import_reports_cancelling_until_non_abortable_work_finishes() {
    let mut app = PrototypeApp::blocking_media_import();

    let handle = app.start_import("photos");
    app.cancel_import(handle);
    app.drain();

    assert_eq!(app.import_status(handle), TaskStatus::Cancelling);

    app.finish_non_abortable_import(handle);
    app.drain();

    assert_eq!(app.import_status(handle), TaskStatus::FinishedAfterCancel);
}
```

- [ ] **Step 6: Run failing prototype tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing prototype harness support.

- [ ] **Step 7: Implement prototype harness support**

Extend `testing.rs` with this prototype harness API. The method bodies can be compact fixture logic, but each prototype must enqueue work-plane output through `AppProxy` or the runtime task/service queues and then call runtime drain methods to integrate state. Do not update the observed domain state directly from helper methods such as `complete_search`, `push_log_line`, `notify_progress`, or `finish_non_abortable_import`; those helpers enqueue events, and `drain`/`drain_all` applies them through the reducer path.

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ServiceRequestId(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceRequestStatus {
    Pending,
    Completed,
    Cancelled,
    TimedOutAfterCancel,
}

pub struct PrototypeApp {
    budget: RuntimeBudget,
    diagnostics: DiagnosticLog,
    search_results: Vec<String>,
    log_lines: Vec<String>,
    remaining_task_inputs: usize,
    progress_count: usize,
    reducer_reentry_count: usize,
    wake: FakeWakeBridge,
    proxy: AppProxy<PrototypeInput>,
    compile_task: TaskRecord,
    jsonrpc_status: ServiceStatus,
    request_status: BTreeMap<ServiceRequestId, ServiceRequestStatus>,
    responses: BTreeMap<ServiceRequestId, String>,
    imports: BTreeMap<TaskHandle, TaskStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrototypeInput {
    SearchComplete { attempt: TaskAttemptId, results: Vec<String> },
    LogLine(String),
    Progress(usize),
    ServiceProgress { request: ServiceRequestId, message: String },
}

impl PrototypeApp {
    #[must_use]
    pub fn latest_search() -> Self;
    #[must_use]
    pub fn log_stream(budget: RuntimeBudget) -> Self;
    #[must_use]
    pub fn progress_counter(budget: RuntimeBudget) -> Self;
    #[must_use]
    pub fn shared_compile_service() -> Self;
    #[must_use]
    pub fn jsonrpc_service() -> Self;
    #[must_use]
    pub fn blocking_media_import() -> Self;

    pub fn start_search(&mut self, query: &str, attempt: TaskAttemptId);
    pub fn complete_search(&mut self, attempt: TaskAttemptId, results: Vec<&str>);
    pub fn push_log_line(&mut self, line: String);
    pub fn drain(&mut self);
    pub fn drain_all(&mut self);

    #[must_use]
    pub fn search_results(&self) -> &[String];
    #[must_use]
    pub fn diagnostics(&self) -> &DiagnosticLog;
    #[must_use]
    pub fn log_lines(&self) -> &[String];
    #[must_use]
    pub const fn remaining_task_inputs(&self) -> usize;
    #[must_use]
    pub const fn progress_count(&self) -> usize;
    #[must_use]
    pub const fn reducer_reentry_count(&self) -> usize;
    #[must_use]
    pub const fn fake_wake(&self) -> &FakeWakeBridge;
    #[must_use]
    pub fn proxy(&self) -> &AppProxy<PrototypeInput>;
    #[must_use]
    pub fn progress_event(&self, index: usize) -> TaskInput<PrototypeInput>;

    pub fn open_surface(&mut self, name: &str) -> SurfaceId;
    pub fn observe_compile(&mut self, surface: SurfaceId);
    pub fn close_surface(&mut self, surface: SurfaceId);
    #[must_use]
    pub const fn compile_task_status(&self) -> TaskStatus;

    pub fn call_tool(&mut self, name: &str) -> ServiceRequestId;
    pub fn notify_progress(&mut self, request: ServiceRequestId, message: &str);
    pub fn respond(&mut self, request: ServiceRequestId, message: &str);
    pub fn cancel(&mut self, request: ServiceRequestId);
    pub fn timeout(&mut self, request: ServiceRequestId);
    pub fn reconnect(&mut self);
    #[must_use]
    pub fn response(&self, request: ServiceRequestId) -> Option<&str>;
    #[must_use]
    pub fn request_status(&self, request: ServiceRequestId) -> ServiceRequestStatus;
    #[must_use]
    pub fn service_status(&self, service: ServiceId) -> ServiceStatus;

    pub fn start_import(&mut self, name: &str) -> TaskHandle;
    pub fn cancel_import(&mut self, handle: TaskHandle);
    pub fn finish_non_abortable_import(&mut self, handle: TaskHandle);
    #[must_use]
    pub fn import_status(&self, handle: TaskHandle) -> TaskStatus;
}
```

Prototype semantics to implement:

- latest search state tracks the active attempt and accepts only matching completion events;
- append-only log stream preserves event order while `RuntimeBudget` limits each drain;
- progress counter sends through `AppProxy`, uses coalesced wakeups, and rejects reducer reentry;
- shared compile service keeps an app-scoped task running until the last observer detaches;
- fake JSON-RPC service uses `CorrelationId`-like request ids, out-of-order responses, notifications, cancellation, timeout, and reconnect status;
- blocking media import uses `BlockingPolicy::NonAbortableReportCancelling` and remains `Cancelling` until the fixture marks work finished.

Treat these as regression prototypes for the separate-plane architecture: work-plane events enter through `AppProxy`/runtime queues, reducers integrate them synchronously, and runtime diagnostics expose stale or overflowing events.

Add explicit assertions inside the prototype tests or harness accessors that prove the runtime path was used:

- latest search stale completion increments `RuntimeDrainReport::dropped_stale_task_events` or `DiagnosticCode::STALE_TASK_EVENT`;
- log stream leaves events queued according to `RuntimeBudget` before `drain_all`;
- progress counter increases `FakeWakeBridge::wake_count` through `AppProxy` rather than direct reducer calls;
- shared compile service changes observer count before task cancellation status changes;
- JSON-RPC responses are keyed by request id/correlation, not arrival order;
- blocking import remains `Cancelling` until a queued completion event is drained.

Keep prototype logic in `testing.rs`; production modules should only receive small fixes surfaced by these tests.

- [ ] **Step 8: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/tests.rs src/app/testing.rs src/app/runtime.rs src/app/coord.rs src/app/task.rs src/app/service.rs
```

Expected: tests pass without enabling `app-runtime-tokio`, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 9: Commit**

```sh
git add src/app/tests.rs src/app/testing.rs src/app/runtime.rs src/app/coord.rs src/app/task.rs src/app/service.rs
git commit -m "Add app runtime stress prototypes"
```

---
