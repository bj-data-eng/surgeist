# Surgeist App Runtime Foundation 02: Typed IDs, Provenance, And Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed IDs, input provenance, and structured diagnostics.

**Architecture:** This is split 02 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 2: Typed IDs, Provenance, And Diagnostics

**Files:**
- Modify: `src/app/ids.rs`
- Create: `src/app/provenance.rs`
- Create: `src/app/diagnostic.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/tests.rs`
- Modify: `tests/app.rs`

- [ ] **Step 1: Write ID and provenance tests**

Create `src/app/tests.rs` with focused tests:

```rust
use super::*;

#[test]
fn typed_ids_are_stable_and_debuggable() {
    assert_eq!(AppId::new("photo.lab").as_str(), "photo.lab");
    assert_eq!(SurfaceId::from_u64(7).as_u64(), 7);
    assert_eq!(TaskAttemptId::from_u64(3).as_u64(), 3);
    assert_eq!(CorrelationId::from_u64(11).as_u64(), 11);
    assert_eq!(format!("{:?}", ResourceId::new("thumbs:42")), "ResourceId(\"thumbs:42\")");
}

#[test]
fn provenance_carries_causal_fields() {
    let parent = CorrelationId::from_u64(1);
    let child = InputProvenance::task(TaskId::from_u64(2), TaskAttemptId::from_u64(3))
        .with_surface(SurfaceId::from_u64(4))
        .with_correlation(CorrelationId::from_u64(5))
        .with_parent(parent);

    assert_eq!(child.source(), &InputSourceId::TASK);
    assert_eq!(child.task_id(), Some(TaskId::from_u64(2)));
    assert_eq!(child.task_attempt_id(), Some(TaskAttemptId::from_u64(3)));
    assert_eq!(child.surface_id(), Some(SurfaceId::from_u64(4)));
    assert_eq!(child.correlation_id(), CorrelationId::from_u64(5));
    assert_eq!(child.parent_correlation_id(), Some(parent));
}
```

- [ ] **Step 2: Write diagnostic tests**

Add to `src/app/tests.rs`:

```rust
#[test]
fn diagnostics_keep_recent_entries_and_counters() {
    let mut log = DiagnosticLog::with_capacity(2);
    log.push(Diagnostic::warning(
        DiagnosticCode::UNKNOWN_RETAINED_COMMAND,
        "missing binding",
        InputProvenance::ui(SurfaceId::from_u64(1)),
    ));
    log.push(Diagnostic::error(
        DiagnosticCode::STALE_TASK_EVENT,
        "attempt mismatch",
        InputProvenance::task(TaskId::from_u64(2), TaskAttemptId::from_u64(1)),
    )
    .with_app(AppId::new("photo.lab"))
    .with_window(surgeist::window::Id::from_u64(9))
    .with_root(RootId::new("gallery"))
    .with_scope(AppScope::resource(ResourceId::new("thumbs")))
    .with_resource(ResourceId::new("thumbs"))
    .with_queue(QueueDiagnostic::new("task-events", 128).with_age_ms(17))
    .with_effect("request_redraw"));
    log.push(Diagnostic::info(
        DiagnosticCode::QUEUE_COALESCED,
        "progress coalesced",
        InputProvenance::system(),
    ));

    let entries = log.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(log.dropped_oldest(), 1);
    assert_eq!(log.count(&DiagnosticCode::UNKNOWN_RETAINED_COMMAND), 1);
    assert_eq!(log.count(&DiagnosticCode::QUEUE_COALESCED), 1);
    assert_eq!(entries[0].code(), &DiagnosticCode::STALE_TASK_EVENT);
    assert_eq!(entries[0].app_id(), Some(&AppId::new("photo.lab")));
    assert_eq!(entries[0].window_id(), Some(surgeist::window::Id::from_u64(9)));
    assert_eq!(entries[0].root_id(), Some(&RootId::new("gallery")));
    assert_eq!(entries[0].resource_id(), Some(&ResourceId::new("thumbs")));
    assert_eq!(entries[0].emitted_effects(), &["request_redraw"]);
    assert_eq!(entries[0].queue().unwrap().capacity(), 128);
    assert_eq!(entries[0].queue().unwrap().age_ms(), Some(17));
}
```

- [ ] **Step 3: Run the failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing `AppId`, `InputProvenance`, and `DiagnosticLog` types.

- [ ] **Step 4: Implement ID newtypes**

Add `ids.rs` with:

Update `ids.rs` so string-backed ids also implement `Default` only where needed and have exact `Debug` output used by tests. Keep the Task 1 macros, and add this explicit `Default` impl:

```rust
impl Default for AppId {
    fn default() -> Self {
        Self::new("app")
    }
}
```

- [ ] **Step 5: Implement provenance**

Add `provenance.rs` with this public API:

```rust
use std::borrow::Cow;

use super::{CorrelationId, ServiceId, SurfaceId, TaskAttemptId, TaskId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputSourceId(Cow<'static, str>);

impl InputSourceId {
    pub const UI: Self = Self::from_static("ui");
    pub const RETAINED: Self = Self::from_static("retained");
    pub const TASK: Self = Self::from_static("task");
    pub const SERVICE: Self = Self::from_static("service");
    pub const WINDOW: Self = Self::from_static("window");
    pub const SYSTEM: Self = Self::from_static("system");

    #[must_use]
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputProvenance {
    source: InputSourceId,
    surface_id: Option<SurfaceId>,
    task_id: Option<TaskId>,
    task_attempt_id: Option<TaskAttemptId>,
    service_id: Option<ServiceId>,
    correlation_id: CorrelationId,
    parent_correlation_id: Option<CorrelationId>,
    sequence: Option<u64>,
}

impl InputProvenance {
    #[must_use]
    pub fn system() -> Self {
        Self::new(InputSourceId::SYSTEM)
    }

    #[must_use]
    pub fn ui(surface_id: SurfaceId) -> Self {
        Self::new(InputSourceId::UI).with_surface(surface_id)
    }

    #[must_use]
    pub fn retained(surface_id: SurfaceId) -> Self {
        Self::new(InputSourceId::RETAINED).with_surface(surface_id)
    }

    #[must_use]
    pub fn task(task_id: TaskId, attempt_id: TaskAttemptId) -> Self {
        Self::new(InputSourceId::TASK).with_task(task_id, attempt_id)
    }

    #[must_use]
    pub fn service(service_id: ServiceId) -> Self {
        let mut value = Self::new(InputSourceId::SERVICE);
        value.service_id = Some(service_id);
        value
    }

    #[must_use]
    pub fn window(surface_id: SurfaceId) -> Self {
        Self::new(InputSourceId::WINDOW).with_surface(surface_id)
    }

    #[must_use]
    pub fn with_surface(mut self, id: SurfaceId) -> Self {
        self.surface_id = Some(id);
        self
    }

    #[must_use]
    pub fn with_correlation(mut self, id: CorrelationId) -> Self {
        self.correlation_id = id;
        self
    }

    #[must_use]
    pub fn with_parent(mut self, id: CorrelationId) -> Self {
        self.parent_correlation_id = Some(id);
        self
    }

    #[must_use]
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    #[must_use]
    pub fn source(&self) -> &InputSourceId { &self.source }
    #[must_use]
    pub const fn surface_id(&self) -> Option<SurfaceId> { self.surface_id }
    #[must_use]
    pub const fn task_id(&self) -> Option<TaskId> { self.task_id }
    #[must_use]
    pub const fn task_attempt_id(&self) -> Option<TaskAttemptId> { self.task_attempt_id }
    #[must_use]
    pub const fn correlation_id(&self) -> CorrelationId { self.correlation_id }
    #[must_use]
    pub const fn parent_correlation_id(&self) -> Option<CorrelationId> { self.parent_correlation_id }

    #[must_use]
    pub fn new(source: InputSourceId) -> Self {
        Self {
            source,
            surface_id: None,
            task_id: None,
            task_attempt_id: None,
            service_id: None,
            correlation_id: CorrelationId::from_u64(0),
            parent_correlation_id: None,
            sequence: None,
        }
    }

    fn with_task(mut self, task_id: TaskId, attempt_id: TaskAttemptId) -> Self {
        self.task_id = Some(task_id);
        self.task_attempt_id = Some(attempt_id);
        self
    }
}
```

- [ ] **Step 6: Implement diagnostics**

Add `diagnostic.rs` with this public shape:

```rust
use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque},
};

use super::{
    AppId, AppScope, InputProvenance, ResourceId, RootId, ServiceId, TaskAttemptId, TaskId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticCode(Cow<'static, str>);

impl DiagnosticCode {
    pub const UNKNOWN_RETAINED_COMMAND: Self = Self::from_static("unknown_retained_command");
    pub const INVALID_RETAINED_PAYLOAD: Self = Self::from_static("invalid_retained_payload");
    pub const STALE_ELEMENT: Self = Self::from_static("stale_element");
    pub const INELIGIBLE_RETAINED_TARGET: Self = Self::from_static("ineligible_retained_target");
    pub const STALE_TASK_EVENT: Self = Self::from_static("stale_task_event");
    pub const QUEUE_OVERFLOW: Self = Self::from_static("queue_overflow");
    pub const QUEUE_COALESCED: Self = Self::from_static("queue_coalesced");
    pub const REDUCER_ERROR: Self = Self::from_static("reducer_error");
    pub const EFFECT_FAILED: Self = Self::from_static("effect_failed");
    pub const SERVICE_MAILBOX_OVERFLOW: Self = Self::from_static("service_mailbox_overflow");
    pub const SURFACE_DEGRADED: Self = Self::from_static("surface_degraded");

    #[must_use]
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueDiagnostic {
    name: String,
    capacity: usize,
    age_ms: Option<u64>,
}

impl QueueDiagnostic {
    #[must_use]
    pub fn new(name: impl Into<String>, capacity: usize) -> Self {
        Self { name: name.into(), capacity, age_ms: None }
    }

    #[must_use]
    pub fn with_age_ms(mut self, age_ms: u64) -> Self {
        self.age_ms = Some(age_ms);
        self
    }

    #[must_use]
    pub const fn capacity(&self) -> usize { self.capacity }
    #[must_use]
    pub const fn age_ms(&self) -> Option<u64> { self.age_ms }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    severity: DiagnosticSeverity,
    code: DiagnosticCode,
    message: String,
    provenance: InputProvenance,
    app_id: Option<AppId>,
    window_id: Option<crate::window::Id>,
    root_id: Option<RootId>,
    scope: Option<AppScope>,
    resource_id: Option<ResourceId>,
    task_id: Option<TaskId>,
    task_attempt_id: Option<TaskAttemptId>,
    service_id: Option<ServiceId>,
    emitted_effects: Vec<String>,
    queue: Option<QueueDiagnostic>,
}

impl Diagnostic {
    #[must_use]
    pub fn info(code: DiagnosticCode, message: impl Into<String>, provenance: InputProvenance) -> Self {
        Self::new(DiagnosticSeverity::Info, code, message, provenance)
    }

    #[must_use]
    pub fn warning(code: DiagnosticCode, message: impl Into<String>, provenance: InputProvenance) -> Self {
        Self::new(DiagnosticSeverity::Warning, code, message, provenance)
    }

    #[must_use]
    pub fn error(code: DiagnosticCode, message: impl Into<String>, provenance: InputProvenance) -> Self {
        Self::new(DiagnosticSeverity::Error, code, message, provenance)
    }

    #[must_use]
    pub fn with_app(mut self, id: AppId) -> Self { self.app_id = Some(id); self }
    #[must_use]
    pub fn with_window(mut self, id: crate::window::Id) -> Self { self.window_id = Some(id); self }
    #[must_use]
    pub fn with_root(mut self, id: RootId) -> Self { self.root_id = Some(id); self }
    #[must_use]
    pub fn with_scope(mut self, scope: AppScope) -> Self { self.scope = Some(scope); self }
    #[must_use]
    pub fn with_resource(mut self, id: ResourceId) -> Self { self.resource_id = Some(id); self }
    #[must_use]
    pub fn with_task(mut self, id: TaskId, attempt: TaskAttemptId) -> Self {
        self.task_id = Some(id);
        self.task_attempt_id = Some(attempt);
        self
    }
    #[must_use]
    pub fn with_service(mut self, id: ServiceId) -> Self { self.service_id = Some(id); self }
    #[must_use]
    pub fn with_effect(mut self, effect: impl Into<String>) -> Self { self.emitted_effects.push(effect.into()); self }
    #[must_use]
    pub fn with_queue(mut self, queue: QueueDiagnostic) -> Self { self.queue = Some(queue); self }

    #[must_use]
    pub fn code(&self) -> &DiagnosticCode { &self.code }
    #[must_use]
    pub const fn provenance(&self) -> &InputProvenance { &self.provenance }
    #[must_use]
    pub fn message(&self) -> &str { &self.message }
    #[must_use]
    pub fn app_id(&self) -> Option<&AppId> { self.app_id.as_ref() }
    #[must_use]
    pub const fn window_id(&self) -> Option<crate::window::Id> { self.window_id }
    #[must_use]
    pub fn root_id(&self) -> Option<&RootId> { self.root_id.as_ref() }
    #[must_use]
    pub fn resource_id(&self) -> Option<&ResourceId> { self.resource_id.as_ref() }
    #[must_use]
    pub const fn task_id(&self) -> Option<TaskId> { self.task_id }
    #[must_use]
    pub const fn task_attempt_id(&self) -> Option<TaskAttemptId> { self.task_attempt_id }
    #[must_use]
    pub fn service_id(&self) -> Option<&ServiceId> { self.service_id.as_ref() }
    #[must_use]
    pub fn emitted_effects(&self) -> &[String] { &self.emitted_effects }
    #[must_use]
    pub const fn queue(&self) -> Option<&QueueDiagnostic> { self.queue.as_ref() }

    fn new(
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: impl Into<String>,
        provenance: InputProvenance,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            provenance,
            app_id: None,
            window_id: None,
            root_id: None,
            scope: None,
            resource_id: None,
            task_id: None,
            task_attempt_id: None,
            service_id: None,
            emitted_effects: Vec::new(),
            queue: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticLog {
    capacity: usize,
    entries: VecDeque<Diagnostic>,
    dropped_oldest: usize,
    counts: BTreeMap<DiagnosticCode, usize>,
}

impl DiagnosticLog {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { capacity, entries: VecDeque::new(), dropped_oldest: 0, counts: BTreeMap::new() }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        *self.counts.entry(diagnostic.code().clone()).or_default() += 1;
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
            self.dropped_oldest += 1;
        }
        self.entries.push_back(diagnostic);
    }

    #[must_use]
    pub fn entries(&self) -> Vec<Diagnostic> {
        self.entries.iter().cloned().collect()
    }

    #[must_use]
    pub const fn dropped_oldest(&self) -> usize { self.dropped_oldest }

    #[must_use]
    pub fn count(&self, code: &DiagnosticCode) -> usize {
        self.counts.get(code).copied().unwrap_or(0)
    }
}
```

- [ ] **Step 7: Re-export and verify**

Update `mod.rs` to declare `mod ids; mod provenance; mod diagnostic;` and re-export their public types. Add `#[cfg(test)] mod tests;`.

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/ids.rs src/app/provenance.rs src/app/diagnostic.rs src/app/tests.rs tests/app.rs
```

Expected: all three tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add src/app/mod.rs src/app/ids.rs src/app/provenance.rs src/app/diagnostic.rs src/app/tests.rs tests/app.rs
git commit -m "Add app IDs provenance and diagnostics"
```

---

