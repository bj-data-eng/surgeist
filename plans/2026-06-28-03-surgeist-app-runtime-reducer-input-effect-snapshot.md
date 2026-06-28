# Surgeist App Runtime Foundation 03: Reducer, Input, Effect, And Snapshot Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the deterministic reducer boundary, typed inputs, effects, and snapshot contract.

**Architecture:** This is split 03 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 3: Reducer, Input, Effect, And Snapshot Contract

**Files:**
- Create: `src/app/input.rs`
- Create: `src/app/effect.rs`
- Create: `src/app/reducer.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write reducer purity contract tests**

Add to `src/app/tests.rs`:

```rust
#[derive(Default)]
struct CounterState {
    value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CounterInput {
    Increment,
    Save,
}

struct CounterReducer;

impl Reducer<CounterState, CounterInput> for CounterReducer {
    fn reduce(
        &mut self,
        state: &mut CounterState,
        input: AppInput<CounterInput>,
    ) -> ReducerResult {
        match input.payload() {
            CounterInput::Increment => {
                state.value += 1;
                ReducerResult::changed().with_effect(AppEffect::request_redraw(
                    RedrawTarget::surface(SurfaceId::from_u64(1)),
                ))
            }
            CounterInput::Save => ReducerResult::unchanged().with_effect(AppEffect::persist(
                "counter",
                AppScope::app(),
            )),
        }
    }
}

#[test]
fn reducer_returns_effects_without_executing_them() {
    let mut reducer = CounterReducer;
    let mut state = CounterState::default();
    let result = reducer.reduce(
        &mut state,
        AppInput::new(CounterInput::Increment, InputProvenance::system()),
    );

    assert_eq!(state.value, 1);
    assert!(result.is_changed());
    assert_eq!(result.effects().len(), 1);
    assert_eq!(result.effects()[0].kind(), &EffectKindId::REQUEST_REDRAW);
}
```

- [ ] **Step 2: Write effect batch tests**

Add:

```rust
#[test]
fn effect_batches_preserve_order() {
    let effects = EffectBatch::new()
        .push(AppEffect::diagnostic(Diagnostic::info(
            DiagnosticCode::QUEUE_COALESCED,
            "coalesced",
            InputProvenance::system(),
        )))
        .push(AppEffect::request_redraw(RedrawTarget::all()));

    assert_eq!(effects.effects().len(), 2);
    assert_eq!(effects.effects()[0].kind(), &EffectKindId::EMIT_DIAGNOSTIC);
    assert_eq!(effects.effects()[1].kind(), &EffectKindId::REQUEST_REDRAW);
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing `Reducer`, `AppInput`, `ReducerResult`, `AppEffect`, `RedrawTarget`, and `EffectBatch`.

- [ ] **Step 4: Implement input and effect types**

Add `input.rs` with generic:

```rust
use super::InputProvenance;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppInput<T> {
    payload: T,
    provenance: InputProvenance,
}

impl<T> AppInput<T> {
    #[must_use]
    pub fn new(payload: T, provenance: InputProvenance) -> Self {
        Self { payload, provenance }
    }

    #[must_use]
    pub const fn payload(&self) -> &T {
        &self.payload
    }

    #[must_use]
    pub fn into_payload(self) -> T {
        self.payload
    }

    #[must_use]
    pub const fn provenance(&self) -> &InputProvenance {
        &self.provenance
    }
}
```

Add `effect.rs` with this public shape:

```rust
use std::{any::Any, borrow::Cow, sync::Arc};

use super::{AppScope, Diagnostic, SurfaceId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RedrawTarget {
    All,
    Surface(SurfaceId),
    Window(crate::window::Id),
}

impl RedrawTarget {
    #[must_use]
    pub const fn all() -> Self {
        Self::All
    }

    #[must_use]
    pub const fn surface(id: SurfaceId) -> Self {
        Self::Surface(id)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EffectKindId(Cow<'static, str>);

impl EffectKindId {
    pub const REQUEST_REDRAW: Self = Self::from_static("runtime.request_redraw");
    pub const PERSIST: Self = Self::from_static("runtime.persist");
    pub const EMIT_DIAGNOSTIC: Self = Self::from_static("runtime.emit_diagnostic");
    pub const START_TASK: Self = Self::from_static("runtime.start_task");
    pub const CANCEL_TASK: Self = Self::from_static("runtime.cancel_task");
    pub const START_SERVICE: Self = Self::from_static("runtime.start_service");
    pub const STOP_SERVICE: Self = Self::from_static("runtime.stop_service");
    pub const SCHEDULE_TIMER: Self = Self::from_static("runtime.schedule_timer");
    pub const WINDOW_COMMAND: Self = Self::from_static("runtime.window_command");

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

#[derive(Clone)]
pub struct EffectPayload {
    value: Arc<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for EffectPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectPayload").finish_non_exhaustive()
    }
}

impl EffectPayload {
    #[must_use]
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Send + Sync + 'static,
    {
        Self { value: Arc::new(value) }
    }

    #[must_use]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Any + Send + Sync + 'static,
    {
        self.value.downcast_ref::<T>()
    }
}

#[derive(Clone, Debug)]
pub struct AppEffect {
    kind: EffectKindId,
    payload: EffectPayload,
}

impl AppEffect {
    #[must_use]
    pub fn new(kind: EffectKindId, payload: EffectPayload) -> Self {
        Self { kind, payload }
    }

    #[must_use]
    pub fn request_redraw(target: RedrawTarget) -> Self {
        Self::new(
            EffectKindId::REQUEST_REDRAW,
            EffectPayload::new(RequestRedrawEffect { target }),
        )
    }

    #[must_use]
    pub fn persist(key: impl Into<String>, scope: AppScope) -> Self {
        Self::new(
            EffectKindId::PERSIST,
            EffectPayload::new(PersistEffect { key: key.into(), scope }),
        )
    }

    #[must_use]
    pub fn diagnostic(diagnostic: Diagnostic) -> Self {
        Self::new(
            EffectKindId::EMIT_DIAGNOSTIC,
            EffectPayload::new(DiagnosticEffect { diagnostic }),
        )
    }

    #[must_use]
    pub fn kind(&self) -> &EffectKindId {
        &self.kind
    }

    #[must_use]
    pub fn payload(&self) -> &EffectPayload {
        &self.payload
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestRedrawEffect {
    target: RedrawTarget,
}

impl RequestRedrawEffect {
    #[must_use]
    pub const fn target(&self) -> &RedrawTarget {
        &self.target
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistEffect {
    key: String,
    scope: AppScope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticEffect {
    diagnostic: Diagnostic,
}

impl DiagnosticEffect {
    #[must_use]
    pub const fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

#[derive(Clone, Debug, Default)]
pub struct EffectBatch {
    effects: Vec<AppEffect>,
}

impl EffectBatch {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn push(mut self, effect: AppEffect) -> Self {
        self.effects.push(effect);
        self
    }

    #[must_use]
    pub fn effects(&self) -> &[AppEffect] {
        &self.effects
    }
}
```

- [ ] **Step 5: Implement reducer result**

Add `reducer.rs` with:

```rust
use super::{AppEffect, AppInput, EffectBatch, InputProvenance};

pub trait Reducer<State, Input> {
    fn reduce(&mut self, state: &mut State, input: AppInput<Input>) -> ReducerResult;
}

#[derive(Clone, Debug, Default)]
pub struct ReducerResult {
    changed: bool,
    effects: EffectBatch,
    recoverable_error: Option<String>,
    provenance: Option<InputProvenance>,
}

impl ReducerResult {
    #[must_use]
    pub fn changed() -> Self {
        Self { changed: true, ..Self::default() }
    }

    #[must_use]
    pub fn unchanged() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn recoverable_failure(message: impl Into<String>) -> Self {
        Self {
            recoverable_error: Some(message.into()),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_effect(mut self, effect: AppEffect) -> Self {
        self.effects = self.effects.push(effect);
        self
    }

    #[must_use]
    pub fn with_effects(mut self, effects: EffectBatch) -> Self {
        self.effects = effects;
        self
    }

    #[must_use]
    pub fn with_provenance(mut self, provenance: InputProvenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    #[must_use]
    pub const fn is_changed(&self) -> bool {
        self.changed
    }

    #[must_use]
    pub fn effects(&self) -> &[AppEffect] {
        self.effects.effects()
    }

    #[must_use]
    pub fn recoverable_error(&self) -> Option<&str> {
        self.recoverable_error.as_deref()
    }

    #[must_use]
    pub fn provenance(&self) -> Option<&InputProvenance> {
        self.provenance.as_ref()
    }
}
```

- [ ] **Step 6: Re-export and verify**

Update `mod.rs` re-exports and remove the Task 1 marker definitions that now have concrete homes.

Run:

```sh
cargo test -p surgeist app::tests
cargo test --package surgeist --test app app_front_door_exports_expected_names
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/effect.rs src/app/input.rs src/app/reducer.rs src/app/tests.rs
```

Expected: tests pass, the integration test still compiles, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 7: Commit**

```sh
git add src/app/mod.rs src/app/input.rs src/app/effect.rs src/app/reducer.rs src/app/tests.rs
git commit -m "Add app reducer and effect contract"
```

---

