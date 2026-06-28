use std::collections::{BTreeMap, VecDeque};

use super::{
    AppEffect, AppEffectPayload, AppInput, BlockingPolicy, Diagnostic, DiagnosticCode,
    DiagnosticLog, InputProvenance, RedrawTarget, Reducer, RuntimeExecutor, SpawnRequest,
    StateVersion, SurfaceId, TaskHandle, TaskId, TaskRecord, UiSurface,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLane {
    Ui,
    Task,
    Service,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiInput<Input> {
    input: AppInput<Input>,
}

impl<Input> UiInput<Input> {
    pub fn new(payload: Input, provenance: InputProvenance) -> Result<Self, RuntimeInputError> {
        if provenance.task_id().is_some() || provenance.service_id().is_some() {
            return Err(RuntimeInputError::wrong_lane(RuntimeLane::Ui, provenance));
        }

        Ok(Self {
            input: AppInput::new(payload, provenance),
        })
    }

    #[must_use]
    pub fn into_app_input(self) -> AppInput<Input> {
        self.input
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskInput<Input> {
    input: AppInput<Input>,
}

impl<Input> TaskInput<Input> {
    pub fn new(payload: Input, provenance: InputProvenance) -> Result<Self, RuntimeInputError> {
        if provenance.task_id().is_none() || provenance.task_attempt_id().is_none() {
            return Err(RuntimeInputError::wrong_lane(RuntimeLane::Task, provenance));
        }

        Ok(Self {
            input: AppInput::new(payload, provenance),
        })
    }

    #[must_use]
    pub fn into_app_input(self) -> AppInput<Input> {
        self.input
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceInput<Input> {
    input: AppInput<Input>,
}

impl<Input> ServiceInput<Input> {
    pub fn new(payload: Input, provenance: InputProvenance) -> Result<Self, RuntimeInputError> {
        if provenance.service_id().is_none() {
            return Err(RuntimeInputError::wrong_lane(
                RuntimeLane::Service,
                provenance,
            ));
        }

        Ok(Self {
            input: AppInput::new(payload, provenance),
        })
    }

    #[must_use]
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
    pub const fn provenance(&self) -> &InputProvenance {
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
    next_task_id: u64,
}

impl<State, R, Input> Runtime<State, R, Input> {
    #[must_use]
    pub fn new(state: State, reducer: R) -> Self {
        Self {
            state,
            reducer,
            executor: None,
            state_version: StateVersion::from_u64(0),
            surfaces: BTreeMap::new(),
            tasks: BTreeMap::new(),
            diagnostics: DiagnosticLog::with_capacity(256),
            ui_queue: VecDeque::new(),
            task_queue: VecDeque::new(),
            service_queue: VecDeque::new(),
            next_task_id: 1,
        }
    }

    #[must_use]
    pub fn with_executor(mut self, executor: Box<dyn RuntimeExecutor<Input>>) -> Self {
        self.executor = Some(executor);
        self
    }

    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    #[must_use]
    pub const fn state_version(&self) -> StateVersion {
        self.state_version
    }

    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticLog {
        &self.diagnostics
    }

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
}

impl<State, R, Input> Runtime<State, R, Input>
where
    R: Reducer<State, Input>,
{
    pub fn drain_once(&mut self, budget: RuntimeBudget) -> RuntimeDrainReport {
        let mut report = RuntimeDrainReport {
            remaining_task_inputs: self.task_queue.len(),
            ..RuntimeDrainReport::default()
        };

        while report.drained_inputs < budget.max_inputs && !self.ui_queue.is_empty() {
            let input = self
                .ui_queue
                .pop_front()
                .expect("queue was checked before pop");
            self.drain_input(RuntimeLane::Ui, input.into_app_input(), &mut report);
        }

        let mut drained_task_events = 0;
        while report.drained_inputs < budget.max_inputs
            && drained_task_events < budget.max_task_events
            && !self.task_queue.is_empty()
        {
            let input = self
                .task_queue
                .pop_front()
                .expect("queue was checked before pop");
            drained_task_events += 1;
            self.drain_task_input(input.into_app_input(), &mut report);
        }

        let mut drained_service_events = 0;
        while report.drained_inputs < budget.max_inputs
            && drained_service_events < budget.max_service_events
            && !self.service_queue.is_empty()
        {
            let input = self
                .service_queue
                .pop_front()
                .expect("queue was checked before pop");
            drained_service_events += 1;
            self.drain_input(RuntimeLane::Service, input.into_app_input(), &mut report);
        }

        report.remaining_task_inputs = self.task_queue.len();
        report
    }

    fn drain_task_input(&mut self, input: AppInput<Input>, report: &mut RuntimeDrainReport) {
        Self::record_drained_input(RuntimeLane::Task, report);
        if self.drop_stale_task_event(input.provenance(), report) {
            return;
        }

        self.reduce_input(input, report);
    }

    fn drop_stale_task_event(
        &mut self,
        provenance: &InputProvenance,
        report: &mut RuntimeDrainReport,
    ) -> bool {
        let Some(task_id) = provenance.task_id() else {
            return false;
        };
        let Some(attempt_id) = provenance.task_attempt_id() else {
            return false;
        };
        let Some(record) = self.tasks.get(&task_id) else {
            return false;
        };

        if record.accepts_attempt(attempt_id) {
            return false;
        }

        report.dropped_stale_task_events += 1;
        self.diagnostics.push(record.reject_stale(attempt_id));
        true
    }

    fn drain_input(
        &mut self,
        lane: RuntimeLane,
        input: AppInput<Input>,
        report: &mut RuntimeDrainReport,
    ) {
        Self::record_drained_input(lane, report);
        self.reduce_input(input, report);
    }

    fn record_drained_input(lane: RuntimeLane, report: &mut RuntimeDrainReport) {
        if report.first_drained_lane.is_none() {
            report.first_drained_lane = Some(lane);
        }
        report.drained_inputs += 1;
    }

    fn reduce_input(&mut self, input: AppInput<Input>, report: &mut RuntimeDrainReport) {
        let provenance = input.provenance().clone();
        let result = self.reducer.reduce(&mut self.state, input);
        if let Some(error) = result.recoverable_error() {
            report.reducer_errors += 1;
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::REDUCER_ERROR,
                error,
                result.provenance().cloned().unwrap_or(provenance),
            ));
            return;
        }

        if result.is_changed() {
            self.state_version = StateVersion::from_u64(self.state_version.as_u64() + 1);
        }

        for effect in result.effects() {
            self.execute_effect(effect, report);
        }
    }

    fn execute_effect(&mut self, effect: &AppEffect, report: &mut RuntimeDrainReport) {
        match effect.payload() {
            AppEffectPayload::RequestRedraw(effect) => {
                report.executed_effects += 1;
                match effect.target() {
                    RedrawTarget::All => {
                        report.redraw_requests.extend(self.surfaces.keys().copied());
                    }
                    RedrawTarget::Surface(id) => report.redraw_requests.push(*id),
                    RedrawTarget::Window(window_id) => {
                        report
                            .redraw_requests
                            .extend(self.surfaces.iter().filter_map(|(surface_id, surface)| {
                                (surface.window_id() == *window_id).then_some(*surface_id)
                            }));
                    }
                }
            }
            AppEffectPayload::Diagnostic(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(effect.diagnostic().clone());
            }
            AppEffectPayload::StartTask(effect) => {
                report.executed_effects += 1;
                let task_id = self.allocate_task_id();
                let attempt_id = super::TaskAttemptId::from_u64(1);
                let request = SpawnRequest::from_start_effect(task_id, attempt_id, effect);
                self.spawn_task(request, effect.name().as_str());
            }
            AppEffectPayload::CancelTask(effect) => {
                report.executed_effects += 1;
                self.cancel_task(effect.handle());
            }
            AppEffectPayload::LoadResource(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect_failed("resource registry is not available")
                        .with_resource(effect.id().clone())
                        .with_scope(effect.scope().clone())
                        .with_effect("runtime.load_resource"),
                );
            }
            AppEffectPayload::InvalidateResource(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect_failed("resource registry is not available")
                        .with_resource(effect.id().clone())
                        .with_effect("runtime.invalidate_resource"),
                );
            }
            AppEffectPayload::Persist(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect_failed("persistence registry is not available")
                        .with_scope(effect.scope().clone())
                        .with_effect("runtime.persist"),
                );
            }
            AppEffectPayload::ReprioritizeTask(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect_failed("task reprioritization is not available")
                        .with_task(effect.handle().task_id(), effect.handle().attempt_id())
                        .with_effect("runtime.reprioritize_task"),
                );
            }
            AppEffectPayload::StartService(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect_failed("service registry is not available")
                        .with_service(effect.id().clone())
                        .with_effect("runtime.start_service"),
                );
            }
            AppEffectPayload::StopService(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect_failed("service registry is not available")
                        .with_service(effect.id().clone())
                        .with_effect("runtime.stop_service"),
                );
            }
            AppEffectPayload::CallService(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect_failed("service registry is not available")
                        .with_service(effect.id().clone())
                        .with_effect("runtime.call_service"),
                );
            }
            AppEffectPayload::ServiceDiagnostic(effect) => {
                report.executed_effects += 1;
                self.diagnostics.push(
                    effect
                        .diagnostic()
                        .clone()
                        .with_service(effect.id().clone())
                        .with_effect("runtime.service_diagnostic"),
                );
            }
        }
    }

    fn spawn_task(&mut self, request: SpawnRequest<Input>, effect_name: &str) {
        let Some(executor) = &mut self.executor else {
            self.diagnostics
                .push(effect_failed("runtime executor is not available").with_effect(effect_name));
            return;
        };

        let result = match request.blocking_policy() {
            BlockingPolicy::Abortable => executor.spawn_task(request),
            BlockingPolicy::Blocking => executor.spawn_blocking_task(request),
        };

        if let Err(error) = result {
            self.diagnostics.push(
                effect_failed(format!("executor rejected task spawn: {error}"))
                    .with_effect(effect_name),
            );
        }
    }

    fn cancel_task(&mut self, handle: TaskHandle) {
        let Some(executor) = &mut self.executor else {
            self.diagnostics.push(
                effect_failed("runtime executor is not available")
                    .with_task(handle.task_id(), handle.attempt_id())
                    .with_effect("runtime.cancel_task"),
            );
            return;
        };

        if let Err(error) = executor.cancel(handle) {
            self.diagnostics.push(
                effect_failed(format!("executor rejected task cancel: {error}"))
                    .with_task(handle.task_id(), handle.attempt_id())
                    .with_effect("runtime.cancel_task"),
            );
        }
    }

    fn allocate_task_id(&mut self) -> TaskId {
        let id = TaskId::from_u64(self.next_task_id);
        self.next_task_id += 1;
        id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeBudget {
    max_inputs: usize,
    max_task_events: usize,
    max_service_events: usize,
}

impl RuntimeBudget {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_inputs: 64,
            max_task_events: 64,
            max_service_events: 32,
        }
    }

    #[must_use]
    pub const fn max_inputs(mut self, max_inputs: usize) -> Self {
        self.max_inputs = max_inputs;
        self
    }

    #[must_use]
    pub const fn max_task_events(mut self, max_task_events: usize) -> Self {
        self.max_task_events = max_task_events;
        self
    }

    #[must_use]
    pub const fn max_service_events(mut self, max_service_events: usize) -> Self {
        self.max_service_events = max_service_events;
        self
    }
}

impl Default for RuntimeBudget {
    fn default() -> Self {
        Self::new()
    }
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
    #[must_use]
    pub const fn drained_inputs(&self) -> usize {
        self.drained_inputs
    }

    #[must_use]
    pub const fn executed_effects(&self) -> usize {
        self.executed_effects
    }

    #[must_use]
    pub const fn reducer_errors(&self) -> usize {
        self.reducer_errors
    }

    #[must_use]
    pub const fn dropped_stale_task_events(&self) -> usize {
        self.dropped_stale_task_events
    }

    #[must_use]
    pub const fn remaining_task_inputs(&self) -> usize {
        self.remaining_task_inputs
    }

    #[must_use]
    pub const fn first_drained_lane(&self) -> Option<RuntimeLane> {
        self.first_drained_lane
    }

    #[must_use]
    pub fn redraw_requests(&self) -> &[SurfaceId] {
        &self.redraw_requests
    }
}

fn effect_failed(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::EFFECT_FAILED,
        message,
        InputProvenance::system(),
    )
}

impl Default for Runtime<(), (), ()> {
    fn default() -> Self {
        Self::new((), ())
    }
}
