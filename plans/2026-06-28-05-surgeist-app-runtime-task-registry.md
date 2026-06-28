# Surgeist App Runtime Foundation 05: Task Registry, Status, Attempts, And Cancellation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add task registration, status tracking, attempts, and cancellation contracts.

**Architecture:** This is split 05 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 5: Task Registry, Status, Attempts, And Cancellation

**Files:**
- Create: `src/app/task.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/effect.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write task registration and dedupe tests**

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
struct SearchInput {
    query: String,
}

#[test]
fn task_registry_records_identity_scope_key_and_policy() {
    let registration = TaskRegistration::<SearchInput>::new("search")
        .scope(|_| AppScope::resource(ResourceId::new("search-results")))
        .key(|input| TaskKey::new(format!("search:{}", input.query)))
        .with_policy(TaskPolicy::continue_when_unobserved().dedupe_by_key());

    let input = SearchInput { query: "rust".into() };
    assert_eq!(registration.id().as_str(), "search");
    assert_eq!(registration.scope_for(&input), AppScope::resource(ResourceId::new("search-results")));
    assert_eq!(registration.key_for(&input), TaskKey::new("search:rust"));
    assert!(registration.policy().dedupes_by_key());
    assert_eq!(registration.policy().unobserved(), UnobservedPolicy::Continue);
}
```

- [ ] **Step 2: Write attempt and stale event tests**

Add:

```rust
#[test]
fn task_record_rejects_events_from_stale_attempts() {
    let mut record = TaskRecord::queued(
        TaskId::from_u64(1),
        TaskKey::new("search:rust"),
        AppScope::app(),
        TaskPolicy::cancel_when_unobserved(),
    );

    let first = record.start_attempt(TaskAttemptId::from_u64(1));
    assert_eq!(first, TaskAttemptId::from_u64(1));
    record.mark_running();
    record.start_attempt(TaskAttemptId::from_u64(2));

    assert!(record.accepts_attempt(TaskAttemptId::from_u64(2)));
    assert!(!record.accepts_attempt(TaskAttemptId::from_u64(1)));
    assert_eq!(
        record.reject_stale(TaskAttemptId::from_u64(1)).code(),
        &DiagnosticCode::STALE_TASK_EVENT
    );
}

#[test]
fn cancellation_status_is_honest_until_terminal_event_arrives() {
    let mut record = TaskRecord::queued(
        TaskId::from_u64(2),
        TaskKey::new("media:import"),
        AppScope::app(),
        TaskPolicy::continue_when_unobserved(),
    );

    record.start_attempt(TaskAttemptId::from_u64(1));
    record.mark_running();
    let token = record.request_cancel();

    assert!(token.is_cancelled());
    assert_eq!(record.status(), TaskStatus::Cancelling);

    record.mark_finished_after_cancel();
    assert_eq!(record.status(), TaskStatus::FinishedAfterCancel);
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing task types.

- [ ] **Step 4: Implement task model**

Add `task.rs` with this public API shape:

```rust
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use super::{
    AppScope, Diagnostic, DiagnosticCode, InputProvenance, TaskAttemptId, TaskId, TaskKey,
    TaskName,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskStatus {
    Queued,
    Running,
    Waiting,
    Blocked,
    Completed,
    Failed,
    Cancelling,
    Cancelled,
    FinishedAfterCancel,
    FailedToCancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnobservedPolicy {
    Continue,
    LowerPriority,
    Pause,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPolicy {
    dedupe_by_key: bool,
    unobserved: UnobservedPolicy,
    priority: TaskPriority,
    retry_limit: u8,
}

impl TaskPolicy {
    #[must_use]
    pub const fn continue_when_unobserved() -> Self {
        Self {
            dedupe_by_key: false,
            unobserved: UnobservedPolicy::Continue,
            priority: TaskPriority::Normal,
            retry_limit: 0,
        }
    }

    #[must_use]
    pub const fn cancel_when_unobserved() -> Self {
        Self {
            dedupe_by_key: false,
            unobserved: UnobservedPolicy::Cancel,
            priority: TaskPriority::Normal,
            retry_limit: 0,
        }
    }

    #[must_use]
    pub const fn dedupe_by_key(mut self) -> Self {
        self.dedupe_by_key = true;
        self
    }

    #[must_use]
    pub const fn dedupes_by_key(&self) -> bool {
        self.dedupe_by_key
    }

    #[must_use]
    pub const fn unobserved(&self) -> UnobservedPolicy {
        self.unobserved
    }
}

#[derive(Clone)]
pub struct TaskRegistration<Input> {
    id: TaskName,
    scope: Arc<dyn Fn(&Input) -> AppScope + Send + Sync>,
    key: Arc<dyn Fn(&Input) -> TaskKey + Send + Sync>,
    policy: TaskPolicy,
}

impl<Input> TaskRegistration<Input> {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: TaskName::new(id),
            scope: Arc::new(|_| AppScope::app()),
            key: Arc::new(|_| TaskKey::new("task")),
            policy: TaskPolicy::continue_when_unobserved(),
        }
    }

    #[must_use]
    pub fn scope(mut self, f: impl Fn(&Input) -> AppScope + Send + Sync + 'static) -> Self {
        self.scope = Arc::new(f);
        self
    }

    #[must_use]
    pub fn key(mut self, f: impl Fn(&Input) -> TaskKey + Send + Sync + 'static) -> Self {
        self.key = Arc::new(f);
        self
    }

    #[must_use]
    pub const fn policy(&self) -> &TaskPolicy {
        &self.policy
    }

    #[must_use]
    pub fn with_policy(mut self, policy: TaskPolicy) -> Self {
        self.policy = policy;
        self
    }

    #[must_use]
    pub fn id(&self) -> &TaskName {
        &self.id
    }

    #[must_use]
    pub fn scope_for(&self, input: &Input) -> AppScope {
        (self.scope)(input)
    }

    #[must_use]
    pub fn key_for(&self, input: &Input) -> TaskKey {
        (self.key)(input)
    }
}

#[derive(Clone, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaskHandle {
    task_id: TaskId,
    attempt_id: TaskAttemptId,
}

impl TaskHandle {
    #[must_use]
    pub const fn new(task_id: TaskId, attempt_id: TaskAttemptId) -> Self {
        Self { task_id, attempt_id }
    }

    #[must_use]
    pub const fn task_id(self) -> TaskId {
        self.task_id
    }
}

#[derive(Clone, Debug)]
pub struct TaskRecord {
    id: TaskId,
    key: TaskKey,
    scope: AppScope,
    policy: TaskPolicy,
    status: TaskStatus,
    attempt_id: Option<TaskAttemptId>,
    cancellation: CancellationToken,
    observers: usize,
}

impl TaskRecord {
    #[must_use]
    pub fn queued(id: TaskId, key: TaskKey, scope: AppScope, policy: TaskPolicy) -> Self {
        Self {
            id,
            key,
            scope,
            policy,
            status: TaskStatus::Queued,
            attempt_id: None,
            cancellation: CancellationToken::new(),
            observers: 0,
        }
    }

    #[must_use]
    pub fn running_for_test(id: TaskId, attempt_id: TaskAttemptId, key: TaskKey) -> Self {
        let mut record = Self::queued(id, key, AppScope::app(), TaskPolicy::continue_when_unobserved());
        record.start_attempt(attempt_id);
        record.mark_running();
        record
    }

    pub fn start_attempt(&mut self, attempt_id: TaskAttemptId) -> TaskAttemptId {
        self.attempt_id = Some(attempt_id);
        self.cancellation = CancellationToken::new();
        attempt_id
    }

    pub fn mark_running(&mut self) {
        self.status = TaskStatus::Running;
    }

    pub fn request_cancel(&mut self) -> CancellationToken {
        self.cancellation.cancel();
        self.status = TaskStatus::Cancelling;
        self.cancellation.clone()
    }

    pub fn mark_finished_after_cancel(&mut self) {
        self.status = TaskStatus::FinishedAfterCancel;
    }

    #[must_use]
    pub const fn id(&self) -> TaskId {
        self.id
    }

    #[must_use]
    pub const fn status(&self) -> TaskStatus {
        self.status
    }

    #[must_use]
    pub fn accepts_attempt(&self, attempt_id: TaskAttemptId) -> bool {
        self.attempt_id == Some(attempt_id)
    }

    #[must_use]
    pub fn reject_stale(&self, stale_attempt: TaskAttemptId) -> Diagnostic {
        Diagnostic::warning(
            DiagnosticCode::STALE_TASK_EVENT,
            format!("dropped event from stale attempt {:?}", stale_attempt),
            InputProvenance::task(self.id, stale_attempt),
        )
    }
}
```

- [ ] **Step 5: Add task effects**

Update `effect.rs` with:

- `EffectKindId::START_TASK`, `EffectKindId::CANCEL_TASK`, and `EffectKindId::REPRIORITIZE_TASK`;
- `StartTaskEffect { name: TaskName, key: TaskKey, scope: AppScope }`;
- `CancelTaskEffect { handle: TaskHandle }`;
- `ReprioritizeTaskEffect { handle: TaskHandle, priority: TaskPriority }`;
- constructors `AppEffect::start_task(...)`, `AppEffect::cancel_task(...)`, and `AppEffect::reprioritize_task(...)` used by runtime tests in Task 10.

- [ ] **Step 6: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/task.rs src/app/effect.rs src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 7: Commit**

```sh
git add src/app/mod.rs src/app/task.rs src/app/effect.rs src/app/tests.rs
git commit -m "Add app task registry and attempts"
```

---

