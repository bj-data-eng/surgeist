use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use super::{
    AppEffect, AppProxyError, AppProxyErrorCode, FakeExecutor, InputProvenance, RedrawTarget,
    Reducer, ReducerResult, RootId, Runtime, RuntimeBudget, RuntimeDrainReport, RuntimeExecutor,
    RuntimeInputError, SpawnRequest, SurfaceId, TaskHandle, UiInput, UiSurface, WakeBridge,
    WindowRoot,
};
use crate::window;

#[derive(Clone, Debug, Default)]
pub struct FakeWakeBridge {
    state: Arc<Mutex<FakeWakeState>>,
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeClock {
    now: Duration,
    next_sequence: u64,
    timers: Vec<ScheduledTimer>,
}

impl FakeClock {
    #[must_use]
    pub const fn now(&self) -> Duration {
        self.now
    }

    pub fn advance(&mut self, duration: Duration) {
        self.now += duration;
    }

    pub fn schedule_timer(&mut self, id: impl Into<String>, delay: Duration) {
        self.timers.push(ScheduledTimer {
            id: id.into(),
            due_at: self.now + delay,
            sequence: self.next_sequence,
        });
        self.next_sequence += 1;
    }

    #[must_use]
    pub fn drain_due_timers(&mut self) -> Vec<String> {
        let mut due = Vec::new();
        let mut pending = Vec::new();

        for timer in self.timers.drain(..) {
            if timer.due_at <= self.now {
                due.push(timer);
            } else {
                pending.push(timer);
            }
        }

        due.sort_by_key(|timer| (timer.due_at, timer.sequence));
        self.timers = pending;
        due.into_iter().map(|timer| timer.id).collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScheduledTimer {
    id: String,
    due_at: Duration,
    sequence: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeWindowBridge {
    redraws: Vec<SurfaceId>,
    commands: Vec<FakeWindowCommand>,
}

impl FakeWindowBridge {
    pub fn request_redraw(&mut self, surface_id: SurfaceId) {
        self.redraws.push(surface_id);
    }

    pub fn record_native_command(&mut self, window_id: window::Id, command: impl Into<String>) {
        self.commands.push(FakeWindowCommand::Native {
            window_id,
            command: command.into(),
        });
    }

    #[must_use]
    pub fn redraws(&self) -> &[SurfaceId] {
        &self.redraws
    }

    #[must_use]
    pub fn commands(&self) -> &[FakeWindowCommand] {
        &self.commands
    }

    fn record_open_surface(
        &mut self,
        name: impl Into<String>,
        surface_id: SurfaceId,
        window_id: window::Id,
        root_id: RootId,
    ) {
        self.commands.push(FakeWindowCommand::OpenSurface {
            name: name.into(),
            surface_id,
            window_id,
            root_id,
        });
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FakeWindowCommand {
    OpenSurface {
        name: String,
        surface_id: SurfaceId,
        window_id: window::Id,
        root_id: RootId,
    },
    Native {
        window_id: window::Id,
        command: String,
    },
}

pub struct HeadlessHarness<State, R, Input = ()> {
    runtime: Runtime<State, R, Input>,
    fake_executor: Arc<Mutex<FakeExecutor<Input>>>,
    fake_window: FakeWindowBridge,
    clock: FakeClock,
    surfaces: BTreeMap<String, SurfaceId>,
    next_surface_id: u64,
    next_window_id: u64,
    last_report: Option<RuntimeDrainReport>,
}

impl<State, R, Input> HeadlessHarness<State, R, Input>
where
    Input: 'static,
{
    #[must_use]
    pub fn new(state: State, reducer: R) -> Self {
        let fake_executor = Arc::new(Mutex::new(FakeExecutor::default()));
        let runtime = Runtime::new(state, reducer).with_executor(Box::new(
            SharedFakeExecutor::new(Arc::clone(&fake_executor)),
        ));

        Self {
            runtime,
            fake_executor,
            fake_window: FakeWindowBridge::default(),
            clock: FakeClock::default(),
            surfaces: BTreeMap::new(),
            next_surface_id: 1,
            next_window_id: 1,
            last_report: None,
        }
    }

    #[must_use]
    pub const fn runtime(&self) -> &Runtime<State, R, Input> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Runtime<State, R, Input> {
        &mut self.runtime
    }

    #[must_use]
    pub const fn state(&self) -> &State {
        self.runtime.state()
    }

    pub fn fake_executor(&self) -> MutexGuard<'_, FakeExecutor<Input>> {
        self.fake_executor
            .lock()
            .expect("headless fake executor lock")
    }

    #[must_use]
    pub const fn fake_window(&self) -> &FakeWindowBridge {
        &self.fake_window
    }

    #[must_use]
    pub const fn clock(&self) -> &FakeClock {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut FakeClock {
        &mut self.clock
    }

    pub fn schedule_timer(&mut self, id: impl Into<String>, delay: Duration) {
        self.clock.schedule_timer(id, delay);
    }

    #[must_use]
    pub fn due_timers(&mut self) -> Vec<String> {
        self.clock.drain_due_timers()
    }

    pub fn open_surface(&mut self, name: impl Into<String>) -> SurfaceId {
        let name = name.into();
        if let Some(surface_id) = self.surfaces.get(&name) {
            return *surface_id;
        }

        let surface_id = SurfaceId::from_u64(self.next_surface_id);
        self.next_surface_id += 1;
        let window_id = window::Id::from_u64(self.next_window_id);
        self.next_window_id += 1;
        let root_id = RootId::new(name.clone());

        self.runtime.add_surface(UiSurface::new(
            surface_id,
            window_id,
            WindowRoot::new(root_id.clone()),
        ));
        self.fake_window
            .record_open_surface(name.clone(), surface_id, window_id, root_id);
        self.surfaces.insert(name, surface_id);
        surface_id
    }

    #[must_use]
    pub fn surface_id(&self, name: &str) -> SurfaceId {
        *self
            .surfaces
            .get(name)
            .expect("headless surface should be open")
    }

    pub fn enqueue_ui(
        &mut self,
        input: Input,
        provenance: InputProvenance,
    ) -> Result<(), RuntimeInputError> {
        self.runtime.enqueue_ui(UiInput::new(input, provenance)?);
        Ok(())
    }

    #[must_use]
    pub const fn last_report(&self) -> Option<&RuntimeDrainReport> {
        self.last_report.as_ref()
    }
}

impl<State, R, Input> HeadlessHarness<State, R, Input>
where
    R: Reducer<State, Input>,
    Input: 'static,
{
    pub fn drain(&mut self) -> RuntimeDrainReport {
        let report = self.runtime.drain_once(RuntimeBudget::default());
        for surface_id in report.redraw_requests() {
            self.fake_window.request_redraw(*surface_id);
        }
        self.last_report = Some(report.clone());
        report
    }
}

impl HeadlessHarness<CounterState, CounterReducer, CounterInput> {
    #[must_use]
    pub fn counter() -> Self {
        Self::new(CounterState::default(), CounterReducer)
    }

    pub fn input_increment(&mut self) {
        let surface_id = self.primary_surface_id();
        self.enqueue_ui(CounterInput::Increment, InputProvenance::ui(surface_id))
            .expect("counter input should be valid ui input");
    }

    #[must_use]
    pub fn counter_value(&self) -> u32 {
        self.state().value
    }

    fn primary_surface_id(&self) -> SurfaceId {
        self.surfaces
            .values()
            .next()
            .copied()
            .unwrap_or_else(|| SurfaceId::from_u64(1))
    }
}

pub struct HeadlessApp;

impl HeadlessApp {
    #[must_use]
    pub fn counter() -> CounterApp {
        CounterApp {
            harness: HeadlessHarness::counter(),
        }
    }
}

pub struct CounterApp {
    harness: HeadlessHarness<CounterState, CounterReducer, CounterInput>,
}

impl CounterApp {
    pub fn open_surface(&mut self, name: &str) -> SurfaceId {
        self.harness.open_surface(name)
    }

    pub fn input_increment(&mut self) {
        self.harness.input_increment();
    }

    pub fn drain(&mut self) -> RuntimeDrainReport {
        self.harness.drain()
    }

    #[must_use]
    pub fn counter(&self) -> u32 {
        self.harness.counter_value()
    }

    #[must_use]
    pub fn surface_id(&self, name: &str) -> SurfaceId {
        self.harness.surface_id(name)
    }

    #[must_use]
    pub const fn fake_window(&self) -> &FakeWindowBridge {
        self.harness.fake_window()
    }

    pub fn fake_executor(&self) -> MutexGuard<'_, FakeExecutor<CounterInput>> {
        self.harness.fake_executor()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CounterState {
    value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CounterInput {
    Increment,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CounterReducer;

impl Reducer<CounterState, CounterInput> for CounterReducer {
    fn reduce(
        &mut self,
        state: &mut CounterState,
        input: super::AppInput<CounterInput>,
    ) -> ReducerResult {
        match input.payload() {
            CounterInput::Increment => {
                state.value += 1;
                let surface_id = input
                    .provenance()
                    .surface_id()
                    .unwrap_or_else(|| SurfaceId::from_u64(1));
                ReducerResult::changed()
                    .with_effect(AppEffect::request_redraw(RedrawTarget::surface(surface_id)))
            }
        }
    }
}

#[derive(Clone, Debug)]
struct SharedFakeExecutor<Input> {
    inner: Arc<Mutex<FakeExecutor<Input>>>,
}

impl<Input> SharedFakeExecutor<Input> {
    fn new(inner: Arc<Mutex<FakeExecutor<Input>>>) -> Self {
        Self { inner }
    }
}

impl<Input> RuntimeExecutor<Input> for SharedFakeExecutor<Input> {
    fn spawn_task(
        &mut self,
        request: SpawnRequest<Input>,
    ) -> Result<super::ExecutorTaskHandle, super::ExecutorError> {
        self.inner
            .lock()
            .expect("headless fake executor lock")
            .spawn_task(request)
    }

    fn spawn_blocking_task(
        &mut self,
        request: SpawnRequest<Input>,
    ) -> Result<super::ExecutorTaskHandle, super::ExecutorError> {
        self.inner
            .lock()
            .expect("headless fake executor lock")
            .spawn_blocking_task(request)
    }

    fn cancel(&mut self, handle: TaskHandle) -> Result<(), super::ExecutorError> {
        self.inner
            .lock()
            .expect("headless fake executor lock")
            .cancel(handle)
    }

    fn name(&self) -> &'static str {
        "shared-fake"
    }
}
