# Surgeist App Runtime Foundation 10: Runtime Queues, Reducer Dispatch, Effect Execution, And Redraw Targeting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add runtime input queues, reducer dispatch, effect execution, and redraw targeting.

**Architecture:** This split is the first real runtime turn loop: queued UI/task/service inputs drain through a deterministic reducer, state commits before effects execute, and task/service output cannot mutate state except by returning through the app input lanes. It preserves the separate-plane model while keeping execution itself delegated to later adapters.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 10: Runtime Queues, Reducer Dispatch, Effect Execution, And Redraw Targeting

**Runtime lane policy:** UI/window/retained/system inputs and task/service events are sibling inputs to the reducer, but they are not equal scheduling risks. UI-sensitive lanes drain first, task/service lanes are budgeted, and floods must remain observable through drain reports and diagnostics. This split must not spawn real background work; it only routes already-enqueued inputs and converts committed effects into executor requests when an executor exists.

**Files:**
- Create: `src/app/runtime.rs`
- Create: `src/app/executor.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write runtime reducer dispatch tests**

Add:

```rust
#[test]
fn runtime_commits_state_before_executing_effects() {
    let mut runtime = Runtime::new(CounterState::default(), CounterReducer);
    runtime.add_surface(UiSurface::new(
        SurfaceId::from_u64(1),
        surgeist::window::Id::from_u64(1),
        WindowRoot::new(RootId::new("main")),
    ));

    runtime.enqueue_ui(UiInput::new(
        CounterInput::Increment,
        InputProvenance::system(),
    ).unwrap());
    let report = runtime.drain_once(RuntimeBudget::default());

    assert_eq!(runtime.state().value, 1);
    assert_eq!(runtime.state_version(), StateVersion::from_u64(1));
    assert_eq!(report.executed_effects(), 1);
    assert_eq!(report.redraw_requests(), &[SurfaceId::from_u64(1)]);
}
```

- [ ] **Step 2: Write queue budget and priority tests**

Add:

```rust
#[test]
fn runtime_drains_ui_before_task_events_and_respects_budget() {
    let mut runtime = Runtime::new(CounterState::default(), CounterReducer);
    runtime.enqueue_task(TaskInput::new(
        CounterInput::Increment,
        InputProvenance::task(TaskId::from_u64(1), TaskAttemptId::from_u64(1)),
    ).unwrap());
    runtime.enqueue_ui(UiInput::new(
        CounterInput::Increment,
        InputProvenance::ui(SurfaceId::from_u64(1)),
    ).unwrap());

    let report = runtime.drain_once(RuntimeBudget::new().max_inputs(1));

    assert_eq!(runtime.state().value, 1);
    assert_eq!(report.drained_inputs(), 1);
    assert_eq!(report.remaining_task_inputs(), 1);
    assert_eq!(report.first_drained_lane(), Some(RuntimeLane::Ui));
}
```

Budget semantics for this test: `RuntimeBudget::max_inputs` is the total number of inputs drained across all lanes in one loop turn. UI/window/retained/system inputs drain first; task inputs drain only if total budget remains, and `max_task_events` caps how many task-lane inputs may be drained from that remaining total.

- [ ] **Step 3: Write stale task event runtime test**

Add:

```rust
#[test]
fn runtime_drops_stale_task_events_with_diagnostics() {
    let mut runtime = Runtime::new(CounterState::default(), CounterReducer);
    runtime.register_task_record(TaskRecord::running_for_test(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(2),
        TaskKey::new("search:rust"),
    ));

    runtime.enqueue_task(TaskInput::new(
        CounterInput::Increment,
        InputProvenance::task(TaskId::from_u64(1), TaskAttemptId::from_u64(1)),
    ).unwrap());
    let report = runtime.drain_once(RuntimeBudget::default());

    assert_eq!(runtime.state().value, 0);
    assert_eq!(report.dropped_stale_task_events(), 1);
    assert_eq!(runtime.diagnostics().count(&DiagnosticCode::STALE_TASK_EVENT), 1);
}

struct FailingReducer;

impl Reducer<CounterState, CounterInput> for FailingReducer {
    fn reduce(
        &mut self,
        _state: &mut CounterState,
        _input: AppInput<CounterInput>,
    ) -> ReducerResult {
        ReducerResult::recoverable_failure("counter reducer rejected input")
    }
}

#[test]
fn runtime_turns_recoverable_reducer_errors_into_diagnostics() {
    let mut runtime = Runtime::new(CounterState::default(), FailingReducer);
    runtime.enqueue_ui(UiInput::new(
        CounterInput::Increment,
        InputProvenance::system(),
    ).unwrap());

    let report = runtime.drain_once(RuntimeBudget::default());

    assert_eq!(runtime.state().value, 0);
    assert_eq!(report.reducer_errors(), 1);
    assert_eq!(runtime.diagnostics().count(&DiagnosticCode::REDUCER_ERROR), 1);
}

#[test]
fn runtime_rejects_work_lane_provenance_for_ui_queue() {
    let error = match UiInput::new(
        CounterInput::Increment,
        InputProvenance::task(TaskId::from_u64(1), TaskAttemptId::from_u64(1)),
    ) {
        Ok(_) => panic!("task provenance should not enter the UI queue"),
        Err(error) => error,
    };

    assert_eq!(error.lane(), RuntimeLane::Ui);
}
```

- [ ] **Step 4: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing runtime implementation.

- [ ] **Step 5: Implement runtime**

Add `runtime.rs` with this public API shape:

```rust
use std::collections::{BTreeMap, VecDeque};

use super::{
    AppEffect, AppEffectPayload, AppInput, BlockingPolicy, Diagnostic, DiagnosticCode,
    DiagnosticLog, ExecutorError, Reducer, ReducerResult, RuntimeExecutor, RedrawTarget,
    InputProvenance, SpawnRequest, StateVersion, SurfaceId, TaskHandle, TaskId, TaskRecord,
    UiSurface,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLane {
    Ui,
    Task,
    Service,
}

pub struct UiInput<Input> {
    input: AppInput<Input>,
}

impl<Input> UiInput<Input> {
    pub fn new(payload: Input, provenance: InputProvenance) -> Result<Self, RuntimeInputError> {
        if provenance.task_id().is_some() || provenance.service_id().is_some() {
            return Err(RuntimeInputError::wrong_lane(RuntimeLane::Ui, provenance));
        }
        Ok(Self { input: AppInput::new(payload, provenance) })
    }

    pub fn into_app_input(self) -> AppInput<Input> {
        self.input
    }
}

pub struct TaskInput<Input> {
    input: AppInput<Input>,
}

impl<Input> TaskInput<Input> {
    pub fn new(payload: Input, provenance: InputProvenance) -> Result<Self, RuntimeInputError> {
        if provenance.task_id().is_none() || provenance.task_attempt_id().is_none() {
            return Err(RuntimeInputError::wrong_lane(RuntimeLane::Task, provenance));
        }
        Ok(Self { input: AppInput::new(payload, provenance) })
    }

    pub fn into_app_input(self) -> AppInput<Input> {
        self.input
    }
}

pub struct ServiceInput<Input> {
    input: AppInput<Input>,
}

impl<Input> ServiceInput<Input> {
    pub fn new(payload: Input, provenance: InputProvenance) -> Result<Self, RuntimeInputError> {
        if provenance.service_id().is_none() {
            return Err(RuntimeInputError::wrong_lane(RuntimeLane::Service, provenance));
        }
        Ok(Self { input: AppInput::new(payload, provenance) })
    }

    pub fn into_app_input(self) -> AppInput<Input> {
        self.input
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeInputError {
    lane: RuntimeLane,
    provenance: InputProvenance,
}

impl RuntimeInputError {
    fn wrong_lane(lane: RuntimeLane, provenance: InputProvenance) -> Self {
        Self { lane, provenance }
    }

    #[must_use]
    pub const fn lane(&self) -> RuntimeLane {
        self.lane
    }

    #[must_use]
    pub fn provenance(&self) -> &InputProvenance {
        &self.provenance
    }
}

pub struct Runtime<State = (), R = (), Input = ()> {
    state: State,
    reducer: R,
    executor: Option<Box<dyn RuntimeExecutor<Input>>>,
    state_version: StateVersion,
    surfaces: BTreeMap<SurfaceId, UiSurface>,
    tasks: BTreeMap<TaskId, TaskRecord>,
    diagnostics: DiagnosticLog,
    ui_queue: VecDeque<UiInput<Input>>,
    task_queue: VecDeque<TaskInput<Input>>,
    service_queue: VecDeque<ServiceInput<Input>>,
}

impl<State, R, Input> Runtime<State, R, Input>
where
    R: Reducer<State, Input>,
{
    #[must_use]
    pub fn new(state: State, reducer: R) -> Self {
        Self {
            state,
            reducer,
            executor: None,
            state_version: StateVersion::initial(),
            surfaces: BTreeMap::new(),
            tasks: BTreeMap::new(),
            diagnostics: DiagnosticLog::with_capacity(256),
            ui_queue: VecDeque::new(),
            task_queue: VecDeque::new(),
            service_queue: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn with_executor(mut self, executor: impl RuntimeExecutor<Input> + 'static) -> Self {
        self.executor = Some(Box::new(executor));
        self
    }

    #[must_use]
    pub const fn state(&self) -> &State { &self.state }
    #[must_use]
    pub const fn state_version(&self) -> StateVersion { self.state_version }
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticLog { &self.diagnostics }

    pub fn add_surface(&mut self, surface: UiSurface) {
        self.surfaces.insert(surface.id(), surface);
    }

    pub fn register_task_record(&mut self, record: TaskRecord) {
        self.tasks.insert(record.id(), record);
    }

    pub fn enqueue_ui(&mut self, input: UiInput<Input>) {
        self.ui_queue.push_back(input);
    }

    pub fn enqueue_task(&mut self, input: TaskInput<Input>) {
        self.task_queue.push_back(input);
    }

    pub fn enqueue_service(&mut self, input: ServiceInput<Input>) {
        self.service_queue.push_back(input);
    }

    pub fn drain_once(&mut self, budget: RuntimeBudget) -> RuntimeDrainReport {
        let mut report = RuntimeDrainReport::default();
        let mut remaining_total = budget.max_inputs;
        remaining_total -= self.drain_queue(RuntimeLane::Ui, remaining_total, &mut report);
        if remaining_total > 0 {
            let lane_budget = remaining_total.min(budget.max_task_events);
            remaining_total -= self.drain_queue(RuntimeLane::Task, lane_budget, &mut report);
        }
        if remaining_total > 0 {
            let lane_budget = remaining_total.min(budget.max_service_events);
            self.drain_queue(RuntimeLane::Service, lane_budget, &mut report);
        }
        report.remaining_task_inputs = self.task_queue.len();
        report
    }

    fn drain_queue(
        &mut self,
        lane: RuntimeLane,
        budget: usize,
        report: &mut RuntimeDrainReport,
    ) -> usize {
        let start_drained = report.drained_inputs;
        for _ in 0..budget {
            let input = match lane {
                RuntimeLane::Ui => self.ui_queue.pop_front().map(UiInput::into_app_input),
                RuntimeLane::Task => self.task_queue.pop_front().map(TaskInput::into_app_input),
                RuntimeLane::Service => self.service_queue.pop_front().map(ServiceInput::into_app_input),
            };
            let Some(input) = input else { break; };
            report.drained_inputs += 1;
            if report.first_drained_lane.is_none() {
                report.first_drained_lane = Some(lane);
            }
            if self.is_stale_task_input(&input) {
                report.dropped_stale_task_events += 1;
                self.diagnostics.push(Diagnostic::warning(
                    DiagnosticCode::STALE_TASK_EVENT,
                    "dropped stale task event",
                    input.provenance().clone(),
                ));
                continue;
            }
            let result = self.reducer.reduce(&mut self.state, input);
            self.apply_reducer_result(result, report);
        }
        report.drained_inputs - start_drained
    }

    fn is_stale_task_input(&self, input: &AppInput<Input>) -> bool {
        match (input.provenance().task_id(), input.provenance().task_attempt_id()) {
            (Some(task_id), Some(attempt_id)) => self
                .tasks
                .get(&task_id)
                .is_some_and(|record| !record.accepts_attempt(attempt_id)),
            _ => false,
        }
    }

    fn apply_reducer_result(&mut self, result: ReducerResult, report: &mut RuntimeDrainReport) {
        if result.is_changed() {
            self.state_version = self.state_version.next();
        }
        if let Some(message) = result.recoverable_error() {
            report.reducer_errors += 1;
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::REDUCER_ERROR,
                message,
                result.provenance().cloned().unwrap_or_else(super::InputProvenance::system),
            ));
            return;
        }
        for effect in result.effects() {
            report.executed_effects += 1;
            match effect.payload() {
                AppEffectPayload::RequestRedraw(payload) => {
                    report.record_redraw_target(payload.target().clone());
                }
                AppEffectPayload::Diagnostic(payload) => {
                    self.diagnostics.push(payload.diagnostic().clone());
                }
                AppEffectPayload::StartTask(payload) => {
                    match self.executor.as_mut() {
                        Some(executor) => {
                            let request = SpawnRequest::from_start_effect(payload);
                            let outcome = match payload.blocking_policy() {
                                BlockingPolicy::Abortable => executor.spawn_task(request),
                                BlockingPolicy::NonAbortableReportCancelling => {
                                    executor.spawn_blocking_task(request)
                                }
                            };
                            if let Err(error) = outcome.map(|_| ()) {
                                self.diagnostics.push(Diagnostic::error(
                                    DiagnosticCode::EFFECT_FAILED,
                                    error.to_string(),
                                    super::InputProvenance::system(),
                                ));
                            }
                        }
                        None => self.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::EFFECT_FAILED,
                            "task effect emitted without runtime executor",
                            super::InputProvenance::system(),
                        )),
                    }
                }
                AppEffectPayload::CancelTask(payload) => {
                    match self.executor.as_mut() {
                        Some(executor) => {
                            if let Err(error) = executor.cancel(payload.handle()) {
                                self.diagnostics.push(Diagnostic::error(
                                    DiagnosticCode::EFFECT_FAILED,
                                    error.to_string(),
                                    super::InputProvenance::system(),
                                ));
                            }
                        }
                        None => self.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::EFFECT_FAILED,
                            "task effect emitted without runtime executor",
                            super::InputProvenance::system(),
                        )),
                    }
                }
                AppEffectPayload::LoadResource(payload) => {
                    self.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::EFFECT_FAILED,
                        format!(
                            "resource load requested before resource registry integration: {}",
                            payload.id().as_str()
                        ),
                        super::InputProvenance::system(),
                    ));
                }
                AppEffectPayload::InvalidateResource(payload) => {
                    self.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::EFFECT_FAILED,
                        format!(
                            "resource invalidation requested before resource registry integration: {}",
                            payload.id().as_str()
                        ),
                        super::InputProvenance::system(),
                    ));
                }
                AppEffectPayload::Persist(_)
                | AppEffectPayload::ReprioritizeTask(_)
                | AppEffectPayload::StartService(_)
                | AppEffectPayload::StopService(_)
                | AppEffectPayload::CallService(_)
                | AppEffectPayload::ServiceDiagnostic(_) => {}
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeBudget {
    max_inputs: usize,
    max_task_events: usize,
    max_service_events: usize,
}

impl Default for RuntimeBudget {
    fn default() -> Self {
        Self { max_inputs: 64, max_task_events: 64, max_service_events: 32 }
    }
}

impl RuntimeBudget {
    #[must_use]
    pub fn new() -> Self { Self::default() }
    #[must_use]
    pub const fn max_inputs(mut self, value: usize) -> Self { self.max_inputs = value; self }
    #[must_use]
    pub const fn max_task_events(mut self, value: usize) -> Self { self.max_task_events = value; self }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeDrainReport {
    drained_inputs: usize,
    executed_effects: usize,
    reducer_errors: usize,
    dropped_stale_task_events: usize,
    remaining_task_inputs: usize,
    first_drained_lane: Option<RuntimeLane>,
    redraw_requests: Vec<SurfaceId>,
}

impl RuntimeDrainReport {
    pub fn record_redraw_target(&mut self, target: RedrawTarget) {
        match target {
            RedrawTarget::All => {
                self.redraw_requests.clear();
            }
            RedrawTarget::Surface(id) => self.redraw_requests.push(id),
            RedrawTarget::Window(_) => {}
        }
    }

    #[must_use]
    pub const fn drained_inputs(&self) -> usize { self.drained_inputs }
    #[must_use]
    pub const fn executed_effects(&self) -> usize { self.executed_effects }
    #[must_use]
    pub const fn reducer_errors(&self) -> usize { self.reducer_errors }
    #[must_use]
    pub const fn dropped_stale_task_events(&self) -> usize { self.dropped_stale_task_events }
    #[must_use]
    pub const fn remaining_task_inputs(&self) -> usize { self.remaining_task_inputs }
    #[must_use]
    pub const fn first_drained_lane(&self) -> Option<RuntimeLane> { self.first_drained_lane }
    #[must_use]
    pub fn redraw_requests(&self) -> &[SurfaceId] { &self.redraw_requests }
}
```

Create `executor.rs` in this task with only the backend-neutral request contract:

```rust
pub trait RuntimeExecutor<Input> {
    fn spawn_task(&mut self, request: SpawnRequest<Input>) -> Result<ExecutorTaskHandle, ExecutorError>;
    fn spawn_blocking_task(&mut self, request: SpawnRequest<Input>) -> Result<ExecutorTaskHandle, ExecutorError>;
    fn cancel(&mut self, handle: TaskHandle) -> Result<(), ExecutorError>;
    fn name(&self) -> &'static str;
}

pub struct SpawnRequest<Input> {
    task_id: TaskId,
    attempt_id: TaskAttemptId,
    key: TaskKey,
    scope: AppScope,
    blocking: BlockingPolicy,
    input: Option<Input>,
}
```

Include:

- `SpawnRequest::new(task_id, attempt_id, key, scope)` defaulting to `BlockingPolicy::Abortable` and `input: None`;
- `SpawnRequest::from_start_effect(&StartTaskEffect<Input>)`;
- accessors for task id, attempt id, key, scope, blocking policy, and input;
- `BlockingPolicy`, `ExecutorTaskHandle`, and `ExecutorError` as data-only types;
- `ExecutorTaskHandle::task_id()` and `ExecutorTaskHandle::attempt_id()` accessors;
- `ExecutorError::invalid_request(message)`.

`ExecutorTaskHandle` is the executor's spawn acknowledgement and may be recorded on the runtime task record. App-facing cancellation still uses `TaskHandle`, matching the `CancelTaskEffect` from Task 5.

Do not add fake executor behavior, Tokio, child processes, closures, or real spawned work in this task. Start/cancel effects must lower into `SpawnRequest` or `cancel(TaskHandle)` calls on the injected `RuntimeExecutor`; Task 12 supplies fake and Tokio-backed implementations of the same trait. This keeps the app runtime compatible with future sidecar/process/service adapters without adding a second effect-lowering path.

For resource effects, `LoadResource` should mark or report the resource as requested once the resource registry exists; `InvalidateResource` should mark or report stale state. If the required registry is absent in this slice, emit `DiagnosticCode::EFFECT_FAILED` with the resource id rather than silently ignoring the effect. Create a focused branch for every known effect family that the runtime deliberately observes but does not fully execute yet; do not use a wildcard arm that can silently swallow newly added `AppEffectPayload` variants.

- [ ] **Step 6: Re-export runtime types**

Update `mod.rs` with the concrete runtime module and public facade exports:

```rust
mod executor;
mod runtime;

pub use executor::{BlockingPolicy, ExecutorError, ExecutorTaskHandle, RuntimeExecutor, SpawnRequest};
pub use runtime::{Runtime, RuntimeBudget, RuntimeDrainReport, RuntimeInputError, RuntimeLane, ServiceInput, TaskInput, UiInput};
```

Remove or replace the Task 1 temporary `Runtime<State = ()>` marker so only the concrete runtime type is exported.

- [ ] **Step 7: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/executor.rs src/app/runtime.rs src/app/tests.rs
```

Expected: tests pass, formatting succeeds, the integration-facing runtime exports compile through downstream tests, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add src/app/mod.rs src/app/executor.rs src/app/runtime.rs src/app/tests.rs
git commit -m "Add app runtime queue draining"
```

---
