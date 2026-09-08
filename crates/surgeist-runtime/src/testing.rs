use std::{
    error::Error,
    fmt,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::Duration,
};

use super::{
    AppInput, AppProxy, InputProvenance, ProxyInput, QueuePolicy, Reducer, ReducerCommit,
    ReducerResult, Runtime, RuntimeBudget, RuntimeDrainReport, ServiceInput, TaskInput,
    TaskIntentAttemptId, TaskIntentId, UiInput, WakeBridge, WakeError,
};

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
    fn wake(&self) -> Result<(), WakeError> {
        let mut state = self.state.lock().expect("fake wake bridge lock");
        if state.closed {
            return Err(WakeError::new("fake native wake bridge is closed"));
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

pub struct HeadlessHarness {
    clock: FakeClock,
}

impl HeadlessHarness {
    #[must_use]
    pub fn counter() -> Self {
        Self {
            clock: FakeClock::default(),
        }
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
}

pub(super) mod concurrency;

use concurrency::ScenarioDeadline;

pub struct PrototypeApp {
    budget: RuntimeBudget,
    runtime: Runtime<PrototypeState, PrototypeReducer, PrototypeInput>,
    wake: FakeWakeBridge,
    proxy: AppProxy<PrototypeInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrototypeInput {
    SearchStarted {
        query: String,
        attempt: TaskIntentAttemptId,
    },
    SearchComplete {
        attempt: TaskIntentAttemptId,
        results: Vec<String>,
    },
    LogLine(String),
    Progress(usize),
    Service(PrototypeServiceEvent),
}

/// Application payloads recorded exactly as delivered; this fixture does not
/// implement request lifecycle, cancellation, timeout, or reconnect policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrototypeServiceEvent {
    Progress { request: u64, message: String },
    Response { request: u64, message: String },
    Cancelled { request: u64 },
    TimedOut { request: u64 },
    Reconnected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrototypeDrainFailure {
    Deadline,
    NoProgress,
    TurnLimit,
}

#[derive(Debug)]
pub struct PrototypeDrainError {
    failure: PrototypeDrainFailure,
    budget: RuntimeBudget,
    max_turns: NonZeroUsize,
    completed_turns: usize,
    proxy_pending: usize,
    last_report: Option<Box<RuntimeDrainReport>>,
    elapsed: Duration,
}

impl fmt::Display for PrototypeDrainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "prototype drain {:?}: budget={:?}, completed_turns={}/{}, \
             proxy_pending={}, elapsed={:?}, last_report={:?}",
            self.failure,
            self.budget,
            self.completed_turns,
            self.max_turns,
            self.proxy_pending,
            self.elapsed,
            self.last_report,
        )
    }
}

impl Error for PrototypeDrainError {}

impl PrototypeApp {
    #[must_use]
    pub fn latest_search() -> Self {
        Self::new(RuntimeBudget::default())
    }

    #[must_use]
    pub fn log_stream(budget: RuntimeBudget) -> Self {
        Self::new(budget)
    }

    #[must_use]
    pub fn progress_counter(budget: RuntimeBudget) -> Self {
        Self::new(budget)
    }

    #[must_use]
    pub fn service_recorder(budget: RuntimeBudget) -> Self {
        Self::new(budget)
    }

    pub fn start_search(&mut self, query: &str, attempt: TaskIntentAttemptId) {
        self.enqueue_ui(PrototypeInput::SearchStarted {
            query: query.to_owned(),
            attempt,
        });
    }

    pub fn complete_search(&mut self, attempt: TaskIntentAttemptId, results: Vec<&str>) {
        self.complete_search_with_provenance(attempt, attempt, results);
    }

    pub fn complete_search_with_provenance(
        &mut self,
        provenance_attempt: TaskIntentAttemptId,
        payload_attempt: TaskIntentAttemptId,
        results: Vec<&str>,
    ) {
        let results = results.into_iter().map(str::to_owned).collect::<Vec<_>>();
        self.proxy
            .send_task(
                TaskInput::new(
                    PrototypeInput::SearchComplete {
                        attempt: payload_attempt,
                        results,
                    },
                    InputProvenance::task(SEARCH_TASK_ID, provenance_attempt),
                )
                .expect("prototype search completion should be a task input"),
            )
            .expect("prototype search completion should enqueue");
    }

    pub fn push_log_line(&mut self, line: String) {
        self.proxy
            .send_task(
                TaskInput::new(
                    PrototypeInput::LogLine(line),
                    InputProvenance::task(LOG_TASK_ID, TaskIntentAttemptId::from_u64(1)),
                )
                .expect("prototype log line should be a task input"),
            )
            .expect("prototype log line should enqueue");
    }

    pub fn drain(&mut self) -> RuntimeDrainReport {
        self.flush_proxy();
        self.runtime
            .drain_once(self.budget)
            .expect("prototype fixtures do not construct overflowing runtime transactions")
    }

    pub fn drain_all(
        &mut self,
        max_turns: NonZeroUsize,
    ) -> Result<Vec<RuntimeDrainReport>, PrototypeDrainError> {
        self.drain_all_with_deadline(max_turns, &ScenarioDeadline::new())
    }

    fn drain_all_with_deadline(
        &mut self,
        max_turns: NonZeroUsize,
        deadline: &ScenarioDeadline,
    ) -> Result<Vec<RuntimeDrainReport>, PrototypeDrainError> {
        let mut reports = Vec::new();
        for _ in 0..max_turns.get() {
            if deadline.remaining("prototype drain turn").is_err() {
                return Err(self.drain_error(
                    PrototypeDrainFailure::Deadline,
                    max_turns,
                    &reports,
                    deadline,
                ));
            }

            reports.push(self.drain());
            let report = reports.last().expect("the turn just produced a report");
            if deadline.remaining("prototype drain completion").is_err() {
                return Err(self.drain_error(
                    PrototypeDrainFailure::Deadline,
                    max_turns,
                    &reports,
                    deadline,
                ));
            }
            if self.proxy.pending_len() == 0 && !report.has_pending_inputs() {
                return Ok(reports);
            }
            if report.drained_inputs() == 0 {
                return Err(self.drain_error(
                    PrototypeDrainFailure::NoProgress,
                    max_turns,
                    &reports,
                    deadline,
                ));
            }
        }

        Err(self.drain_error(
            PrototypeDrainFailure::TurnLimit,
            max_turns,
            &reports,
            deadline,
        ))
    }

    fn drain_error(
        &self,
        failure: PrototypeDrainFailure,
        max_turns: NonZeroUsize,
        reports: &[RuntimeDrainReport],
        deadline: &ScenarioDeadline,
    ) -> PrototypeDrainError {
        PrototypeDrainError {
            failure,
            budget: self.budget,
            max_turns,
            completed_turns: reports.len(),
            proxy_pending: self.proxy.pending_len(),
            last_report: reports.last().cloned().map(Box::new),
            elapsed: deadline.elapsed(),
        }
    }

    #[must_use]
    pub fn search_results(&self) -> &[String] {
        &self.runtime.state().search_results
    }

    #[must_use]
    pub fn log_lines(&self) -> &[String] {
        &self.runtime.state().log_lines
    }

    #[must_use]
    pub fn progress_indices(&self) -> &[usize] {
        &self.runtime.state().progress_indices
    }

    #[must_use]
    pub fn service_events(&self) -> &[AppInput<PrototypeServiceEvent>] {
        &self.runtime.state().service_events
    }

    #[must_use]
    pub const fn fake_wake(&self) -> &FakeWakeBridge {
        &self.wake
    }

    #[must_use]
    pub const fn proxy(&self) -> &AppProxy<PrototypeInput> {
        &self.proxy
    }

    #[must_use]
    pub fn progress_event(&self, index: usize) -> TaskInput<PrototypeInput> {
        TaskInput::new(
            PrototypeInput::Progress(index),
            InputProvenance::task(PROGRESS_TASK_ID, TaskIntentAttemptId::from_u64(1)),
        )
        .expect("prototype progress should be a task input")
    }

    pub fn send_service(&self, event: PrototypeServiceEvent, provenance: InputProvenance) {
        self.proxy
            .send_service(
                ServiceInput::new(PrototypeInput::Service(event), provenance)
                    .expect("prototype service event should be a service input"),
            )
            .expect("prototype service event should enqueue");
    }

    fn new(budget: RuntimeBudget) -> Self {
        let wake = FakeWakeBridge::default();
        let proxy = AppProxy::new(wake.clone(), QueuePolicy::bounded(20_000));

        Self {
            budget,
            runtime: Runtime::new(PrototypeState::default(), PrototypeReducer),
            wake,
            proxy,
        }
    }

    fn flush_proxy(&mut self) {
        for input in self.proxy.drain_pending(NonZeroUsize::MAX).into_drained() {
            match input {
                ProxyInput::Task(input) => self
                    .runtime
                    .enqueue_task(input)
                    .expect("prototype task input should fit the runtime queue"),
                ProxyInput::Service(input) => self
                    .runtime
                    .enqueue_service(input)
                    .expect("prototype service input should fit the runtime queue"),
            }
        }
    }

    fn enqueue_ui(&mut self, input: PrototypeInput) {
        self.runtime
            .enqueue_ui(UiInput::new(input, InputProvenance::system()).expect(
                "prototype setup action should be accepted as deterministic UI/system input",
            ))
            .expect("prototype UI input should fit the runtime queue");
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PrototypeState {
    active_search_query: Option<String>,
    active_search_attempt: Option<TaskIntentAttemptId>,
    search_results: Vec<String>,
    log_lines: Vec<String>,
    progress_indices: Vec<usize>,
    service_events: Vec<AppInput<PrototypeServiceEvent>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PrototypeReducer;

impl Reducer<PrototypeState, PrototypeInput> for PrototypeReducer {
    fn reduce(
        &mut self,
        state: &PrototypeState,
        input: &AppInput<PrototypeInput>,
    ) -> ReducerResult<PrototypeState> {
        let mut next_state = state.clone();
        let changed = match input.payload() {
            PrototypeInput::SearchStarted { query, attempt } => {
                next_state.active_search_query = Some(query.clone());
                next_state.active_search_attempt = Some(*attempt);
                true
            }
            PrototypeInput::SearchComplete { attempt, results } => {
                if next_state.active_search_attempt == Some(*attempt) {
                    next_state.search_results.clone_from(results);
                    true
                } else {
                    false
                }
            }
            PrototypeInput::LogLine(line) => {
                next_state.log_lines.push(line.clone());
                true
            }
            PrototypeInput::Progress(index) => {
                next_state.progress_indices.push(*index);
                true
            }
            PrototypeInput::Service(event) => {
                next_state
                    .service_events
                    .push(AppInput::new(event.clone(), input.provenance().clone()));
                true
            }
        };

        if changed {
            ReducerResult::changed(next_state, ReducerCommit::new())
        } else {
            ReducerResult::unchanged(ReducerCommit::new())
        }
    }
}

const SEARCH_TASK_ID: TaskIntentId = TaskIntentId::from_u64(1);
const LOG_TASK_ID: TaskIntentId = TaskIntentId::from_u64(2);
const PROGRESS_TASK_ID: TaskIntentId = TaskIntentId::from_u64(3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_work_with_zero_budget_reports_no_progress_after_one_turn() {
        let budget = RuntimeBudget::new(0, 0, 0, 0);
        let mut app = PrototypeApp::log_stream(budget);
        app.push_log_line("waiting".to_owned());

        let error = app.drain_all(NonZeroUsize::new(3).unwrap()).unwrap_err();

        assert_eq!(error.failure, PrototypeDrainFailure::NoProgress);
        assert_eq!(error.budget, budget);
        assert_eq!(error.completed_turns, 1);
        assert_eq!(error.max_turns.get(), 3);
        assert_eq!(error.proxy_pending, 0);
        let last = error.last_report.as_ref().unwrap();
        assert_eq!(last.drained_inputs(), 0);
        assert_eq!(last.remaining_task_inputs(), 1);
        assert!(last.has_pending_inputs());
        assert!(app.log_lines().is_empty());
    }

    #[test]
    fn insufficient_turn_limit_reports_remaining_work_after_completed_turns() {
        let budget = RuntimeBudget::new(1, 0, 1, 0);
        let mut app = PrototypeApp::log_stream(budget);
        for line in ["first", "second", "third"] {
            app.push_log_line(line.to_owned());
        }

        let error = app.drain_all(NonZeroUsize::new(2).unwrap()).unwrap_err();

        assert_eq!(error.failure, PrototypeDrainFailure::TurnLimit);
        assert_eq!(error.budget, budget);
        assert_eq!(error.completed_turns, 2);
        assert_eq!(error.max_turns.get(), 2);
        assert_eq!(error.proxy_pending, 0);
        let last = error.last_report.as_ref().unwrap();
        assert_eq!(last.drained_inputs(), 1);
        assert_eq!(last.remaining_task_inputs(), 1);
        assert!(last.has_pending_inputs());
        assert_eq!(app.log_lines(), &["first", "second"]);
    }

    #[test]
    fn expired_drain_deadline_preserves_queued_work_and_reports_zero_turns() {
        let mut app = PrototypeApp::log_stream(RuntimeBudget::new(1, 0, 1, 0));
        app.push_log_line("waiting".to_owned());
        let deadline = ScenarioDeadline::with_timeout(Duration::ZERO);

        let error = app
            .drain_all_with_deadline(NonZeroUsize::new(1).unwrap(), &deadline)
            .unwrap_err();

        assert_eq!(error.failure, PrototypeDrainFailure::Deadline);
        assert_eq!(error.completed_turns, 0);
        assert_eq!(error.proxy_pending, 1);
        assert!(error.last_report.is_none());
        assert!(app.log_lines().is_empty());
        assert_eq!(app.proxy().pending_len(), 1);
    }
}
