# Surgeist App Runtime Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first narrow but real `surgeist::app` layer that coordinates typed app state, retained UI surfaces, tasks, resources, services, queues, wakeups, and headless tests without putting app semantics into `surgeist::window`.

**Architecture:** `surgeist::app` sits above the existing `window`, `retained`, `layout`, `text`, `render`, `style`, and `css` modules. Reducers stay synchronous and deterministic: they receive typed app inputs, mutate owned app state through a `Reducer` contract, and return type-first `AppEffect` descriptors that the `Runtime` executes after state commits. Background tasks and services communicate through closed app-owned runtime lanes and an `AppProxy` wake bridge; the native `window::Proxy` is used only as a wake signal and native command lane.

**Tech Stack:** Rust, existing Surgeist `window` and `retained` modules, `std::sync` primitives for core queues and cancellation, optional `tokio` behind `app-runtime-tokio`, crate-local unit tests, `crates/surgeist/tests/app.rs` integration tests, and fake executor/window bridge test harnesses.

---

## File Structure

- Modify `crates/surgeist/src/lib.rs`: expose `pub mod app;` as the new front-door app runtime module.
- Modify `crates/surgeist/Cargo.toml`: add `app-runtime-tokio = ["dep:tokio"]` feature and add the small app example entry.
- Create `crates/surgeist/src/app/mod.rs`: public app module facade, re-exports, and module-level documentation.
- Create `crates/surgeist/src/app/ids.rs`: typed IDs and names for apps, roots, surfaces, resources, tasks, task attempts, services, correlations, and custom scopes.
- Create `crates/surgeist/src/app/coord.rs`: open `AppScope` descriptors, open subscription targets, observer sets, task/resource/service coordination state, dedupe, and coalescing policy.
- Create `crates/surgeist/src/app/command.rs`: typed command registry, command names, command descriptors, and command manifest entries.
- Create `crates/surgeist/src/app/event.rs`: typed app event registry, event names, event descriptors, and event manifest entries.
- Create `crates/surgeist/src/app/snapshot.rs`: `AppSnapshot`, snapshot versions, manifest bindings, and render-facing snapshot metadata.
- Create `crates/surgeist/src/app/descriptor.rs`: app identity, command/event/task/resource descriptors, window descriptors, root descriptors, startup window/root mapping, and app manifest.
- Create `crates/surgeist/src/app/provenance.rs`: `InputProvenance`, open source id, causal parent, and stale-event metadata.
- Create `crates/surgeist/src/app/diagnostic.rs`: structured diagnostics, diagnostic ring buffer, counters, and severity.
- Create `crates/surgeist/src/app/input.rs`: `AppInput` wrapper and typed input lanes for UI, retained, task, service, window, and system events.
- Create `crates/surgeist/src/app/effect.rs`: `AppEffect`, effect batches, redraw targets, timers, window commands, task/service effects, and diagnostic effects.
- Create `crates/surgeist/src/app/reducer.rs`: reducer trait, reducer result, state versioning, and purity-facing API.
- Create `crates/surgeist/src/app/resource.rs`: resource identity, freshness, status state machine, stale-while-refresh facts, and resource snapshots.
- Create `crates/surgeist/src/app/task.rs`: typed task registration, task keys, policies, handles, status, attempt lifecycle, cancellation token, and task events.
- Create `crates/surgeist/src/app/service.rs`: typed service registration, mailbox policy, service status, command/event envelope, overflow counters, and shutdown policy.
- Create `crates/surgeist/src/app/bridge.rs`: retained command bridge from `retained::Report` and `retained::Command` to typed app commands or diagnostics.
- Create `crates/surgeist/src/app/surface.rs`: `UiSurface`, `WindowRoot`, surface lifecycle state, retained model ownership, surface invalidation, and per-window isolation.
- Create `crates/surgeist/src/app/runtime.rs`: `Runtime`, input/effect queues, drain budgets, reducer execution, snapshot invalidation, task/service event draining, and redraw targeting.
- Create `crates/surgeist/src/app/proxy.rs`: `AppProxy`, shared app event queue sender, coalesced wake signal, and bridge trait over `window::Proxy`.
- Create `crates/surgeist/src/app/executor.rs`: backend-neutral executor adapter traits, fake executor hooks, blocking task contract, and executor event envelopes.
- Create `crates/surgeist/src/app/runtime_tokio.rs`: feature-gated Tokio executor adapter behind `app-runtime-tokio`.
- Create `crates/surgeist/src/app/loop_.rs`: `AppLoop` wrapper over `window::Loop`, native handler adapter, and app runtime dispatch boundaries.
- Create `crates/surgeist/src/app/testing.rs`: headless runtime harness, fake clock, fake executor, fake window bridge, assertion helpers, and prototype test fixtures.
- Create `crates/surgeist/src/app/tests.rs`: crate-local unit tests for IDs, provenance, diagnostics, reducer/effect behavior, tasks, resources, services, coordination, bridge, runtime, proxy, executor, and surfaces.
- Create `crates/surgeist/tests/app.rs`: public API integration tests that compile against `surgeist::app`.
- Create `crates/surgeist/examples/app-thumbnail-import.rs`: small fake thumbnail import example showing initial thumbnail tiles, progress, continuation policy, and targeted redraw.

## Implementation Boundaries

- Keep `surgeist::window` focused on native window identity, native commands, and event loop mechanics.
- Keep `surgeist::retained` focused on element identity, routing, focus, pointer capture, and retained command reports.
- Keep core `surgeist::app` usable without Tokio by default.
- Do not expose raw Tokio handles, raw channel types, `winit` types, renderer handles, or non-send native handles in app authoring APIs.
- Do not start durable task or service work from render/template code. Subscriptions declare observation; runtime coordination decides effects.
- Keep the first slice general-purpose and Surgeist-local. Do not add DES-specific project/runtime state.

## Type-First Runtime API Rules

- Public app APIs must prefer typed newtypes, descriptors, and open-vocabulary ids over closed enums. Use associated constants for built-in vocabulary, for example `DiagnosticCode::STALE_TASK_EVENT` on a string-backed `DiagnosticCode`.
- Enums are reserved for genuinely closed runtime protocols where Surgeist must exhaustively branch: `RedrawTarget`, `DiagnosticSeverity`, closed internal queue lanes, and task/resource/service status state machines are acceptable examples.
- Split public provenance from runtime scheduling. Public `InputProvenance` carries an open `InputSourceId`; runtime queue draining uses a closed internal `RuntimeLane` because the runtime must exhaustively branch over its own lanes.
- Model scope and subscriptions as open descriptors. `AppScope` is a path-like descriptor with typed helpers and accessors for built-ins such as app, window, surface, resource, workspace, document, and widget; subscription targets use `SubscriptionTarget { kind: SubscriptionTargetKindId, key: String }` with typed constructors instead of a closed task/resource/service enum.
- Model effects as a type-first descriptor, not a public variant list. `AppEffect` carries an `EffectKindId` and opaque `EffectPayload`; runtime-owned effects have typed payload structs and constructors. External app/toolkit layers can introduce new effect kinds without editing a public catch-all enum.
- Apply the Taffy calc lesson consistently: future computed values should use explicit handles such as `ExpressionId`, `CalcId`, or `ValueExprId` resolved by registries at the correct phase. Do not embed arbitrary expression ASTs into every value enum. This guidance applies to future template/style DSL values, snapshot bindings, resource keys, effect payloads, and layout/style calc integration.
- Keep zero lint allowances as a hard implementation constraint. Do not add lint suppression attributes or local lint-suppression calls in these tasks; reshape APIs and code until the compiler, formatter, and lints are satisfied without suppressions.

## Deferred Spec Requirements

- Full template or Smarty-like DSL syntax is deferred because this slice only needs the manifest-like target surface: registered commands, events, tasks, resources, windows, and roots.
- Multi-root composition, overlays, dialogs, split panes, z-order composition, and per-window multi-surface rendering are deferred because the first slice proves one primary active surface per window plus root replacement isolation.
- Real MCP, dbt, media, database, and process-supervisor service crates are deferred because this slice validates the service registry and mailbox protocol with fakes.
- Full retry/backoff strategy, pause/resume execution, and priority scheduling algorithms are deferred because this slice needs visible policy fields, honest statuses, and deterministic tests before advanced scheduling.
- CQRS/event-sourcing, replay logs, and durable audit persistence are deferred because reducers, diagnostics, and snapshots are enough for the first app-runtime boundary.

---

## Phase 1: Core App Runtime Contracts

Tasks 1 through 13 build the public app front door, deterministic reducer/effect boundary, manifest surface, resources, tasks, services, retained bridge, surfaces, runtime queues, wake bridge, executor adapter, and headless fakes. Workers should complete these tasks sequentially and request review after each commit because downstream tasks depend on the public API shape introduced earlier.

### Task 1: App Module Skeleton And Public API Fence

**Files:**
- Modify: `crates/surgeist/src/lib.rs`
- Modify: `crates/surgeist/Cargo.toml`
- Create: `crates/surgeist/src/app/mod.rs`
- Create: `crates/surgeist/src/app/ids.rs`
- Create: `crates/surgeist/src/app/coord.rs`
- Create: `crates/surgeist/src/app/command.rs`
- Create: `crates/surgeist/src/app/event.rs`
- Create: `crates/surgeist/src/app/snapshot.rs`
- Create: `crates/surgeist/src/app/descriptor.rs`
- Create: `crates/surgeist/tests/app.rs`

- [ ] **Step 1: Write the public API failing test**

Add `crates/surgeist/tests/app.rs`:

```rust
use surgeist::app::{
    App, AppCommand, AppDescriptor, AppEffect, AppEvent, AppId, AppLoop, AppManifest, AppScope,
    AppSnapshot, CommandDescriptor, EffectKindId, EventDescriptor, ExpressionId,
    ResourceDescriptor, ResourceId, RootDescriptor, RootId, Runtime, SnapshotBinding,
    StartupWindow, TaskDescriptor, TaskName, UiSurface, WindowDescriptor, WindowRoot,
};

#[test]
fn app_front_door_exports_expected_names() {
    let _scope = AppScope::app();
    let _ = std::mem::size_of::<App>();
    let _ = std::mem::size_of::<AppLoop>();
    let _ = std::mem::size_of::<Runtime<()>>();
    let _ = std::mem::size_of::<AppCommand>();
    let _ = std::mem::size_of::<AppEvent>();
    let _ = std::mem::size_of::<AppEffect>();
    let _ = std::mem::size_of::<EffectKindId>();
    let _ = std::mem::size_of::<ExpressionId>();
    let _ = std::mem::size_of::<AppSnapshot>();
    let _ = std::mem::size_of::<UiSurface>();
    let _ = std::mem::size_of::<WindowRoot>();
    let _ = std::mem::size_of::<AppDescriptor>();
    let _ = std::mem::size_of::<WindowDescriptor>();
    let _ = std::mem::size_of::<RootDescriptor>();
    let _ = std::mem::size_of::<StartupWindow>();
    let _ = std::mem::size_of::<AppManifest>();
    let _ = std::mem::size_of::<CommandDescriptor>();
    let _ = std::mem::size_of::<EventDescriptor>();
}

#[test]
fn app_manifest_registers_identity_windows_roots_commands_events_and_bindings() {
    let app = AppDescriptor::new(AppId::new("photo.lab"), "0.1.0");
    let command = CommandDescriptor::new("photos.import", "ImportPhotos");
    let event = EventDescriptor::new("photos.imported", "ImportFinished");
    let task = TaskDescriptor::new(TaskName::new("photos.import"), "ImportPhotos");
    let resource = ResourceDescriptor::new(ResourceId::new("photos"), "PhotoResource");
    let binding = SnapshotBinding::new("photos", "PhotoGridSnapshot");
    let root = RootDescriptor::new(RootId::new("gallery"))
        .requires_command(command.clone())
        .emits_event(event.clone())
        .binds_snapshot(binding.clone());
    let window = WindowDescriptor::new("main", "Photo Lab")
        .allows_root(RootId::new("gallery"));
    let startup = StartupWindow::new("main", RootId::new("gallery"), AppScope::app());
    let manifest = AppManifest::new(app)
        .command(command)
        .event(event)
        .task(task)
        .resource(resource)
        .window(window)
        .root(root)
        .startup(startup);

    assert_eq!(manifest.commands().len(), 1);
    assert_eq!(manifest.events().len(), 1);
    assert_eq!(manifest.tasks().len(), 1);
    assert_eq!(manifest.resources().len(), 1);
    assert_eq!(manifest.windows().len(), 1);
    assert_eq!(manifest.roots().len(), 1);
    assert_eq!(manifest.startup().len(), 1);
    assert_eq!(manifest.roots()[0].snapshot_bindings(), &[binding]);
}
```

- [ ] **Step 2: Run the failing test**

Run:

```sh
cargo test --package surgeist --test app app_front_door_exports_expected_names
```

Expected: fail with an unresolved `surgeist::app` import.

- [ ] **Step 3: Add the minimal module facade**

Implement only enough to expose the module and names:

- Add `pub mod app;` to `crates/surgeist/src/lib.rs`.
- Add `app-runtime-tokio = ["dep:tokio"]` to `crates/surgeist/Cargo.toml` under `[features]`.
- Create `crates/surgeist/src/app/mod.rs`, `ids.rs`, `coord.rs`, `command.rs`, `event.rs`, `snapshot.rs`, and `descriptor.rs` with the minimal public API shown below. These are concrete front-door files, not temporary marker definitions; follow-up tasks add fields and behavior without moving these names.

The initial `mod.rs` should include:

```rust
//! App runtime and authoring DSL boundary for Surgeist.
//!
//! This module coordinates deterministic app state, retained UI surfaces,
//! resources, tasks, services, native wakeups, and declared effects. Native
//! window mechanics stay in `surgeist::window`.

mod command;
mod coord;
mod descriptor;
mod event;
mod ids;
mod snapshot;

pub use command::{AppCommand, CommandDescriptor, CommandName};
pub use coord::{AppScope, ScopePathSegment};
pub use descriptor::{
    App, AppDescriptor, AppManifest, ResourceDescriptor, RootDescriptor, StartupWindow,
    TaskDescriptor, WindowDescriptor,
};
pub use event::{AppEvent, EventDescriptor, EventName};
pub use ids::{
    AppId, CalcId, CorrelationId, CustomScopeId, ExpressionId, ResourceId, RootId, ServiceId,
    SurfaceId, TaskAttemptId, TaskId, TaskKey, TaskName, ValueExprId,
};
pub use snapshot::{AppSnapshot, SnapshotBinding, StateVersion};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppLoop;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Runtime<State = ()> {
    state: State,
}
```

Add `ids.rs`:

```rust
macro_rules! string_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

macro_rules! numeric_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            #[must_use]
            pub const fn from_u64(value: u64) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn as_u64(self) -> u64 {
                self.0
            }
        }
    };
}

string_id!(AppId);
string_id!(RootId);
string_id!(ResourceId);
string_id!(TaskName);
string_id!(TaskKey);
string_id!(ServiceId);
string_id!(CustomScopeId);
string_id!(ExpressionId);
string_id!(CalcId);
string_id!(ValueExprId);

numeric_id!(SurfaceId);
numeric_id!(TaskId);
numeric_id!(TaskAttemptId);
numeric_id!(CorrelationId);
```

Add `command.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommandName(String);

impl CommandName {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppCommand {
    name: CommandName,
}

impl AppCommand {
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: CommandName::new(name),
        }
    }

    #[must_use]
    pub fn name(&self) -> &CommandName {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandDescriptor {
    name: CommandName,
    payload_type: &'static str,
}

impl CommandDescriptor {
    #[must_use]
    pub fn new(name: impl Into<String>, payload_type: &'static str) -> Self {
        Self {
            name: CommandName::new(name),
            payload_type,
        }
    }

    #[must_use]
    pub fn name(&self) -> &CommandName {
        &self.name
    }

    #[must_use]
    pub const fn payload_type(&self) -> &'static str {
        self.payload_type
    }
}
```

Add `event.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventName(String);

impl EventName {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppEvent {
    name: EventName,
}

impl AppEvent {
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: EventName::new(name),
        }
    }

    #[must_use]
    pub fn name(&self) -> &EventName {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventDescriptor {
    name: EventName,
    payload_type: &'static str,
}

impl EventDescriptor {
    #[must_use]
    pub fn new(name: impl Into<String>, payload_type: &'static str) -> Self {
        Self {
            name: EventName::new(name),
            payload_type,
        }
    }
}
```

Add `snapshot.rs`:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StateVersion(u64);

impl StateVersion {
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotBinding {
    pub name: String,
    pub source_type: &'static str,
}

impl SnapshotBinding {
    #[must_use]
    pub fn new(name: impl Into<String>, source_type: &'static str) -> Self {
        Self {
            name: name.into(),
            source_type,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppSnapshot {
    version: StateVersion,
    bindings: Vec<SnapshotBinding>,
}

impl AppSnapshot {
    #[must_use]
    pub fn new(version: StateVersion) -> Self {
        Self {
            version,
            bindings: Vec::new(),
        }
    }

    #[must_use]
    pub const fn version(&self) -> StateVersion {
        self.version
    }
}
```

Add `coord.rs` with the open scope descriptor immediately so Task 5 and downstream tasks do not depend on a temporary type:

```rust
use super::{CustomScopeId, ResourceId, SurfaceId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopePathSegment {
    namespace: String,
    value: String,
}

impl ScopePathSegment {
    #[must_use]
    pub fn new(namespace: impl Into<String>, value: impl Into<String>) -> Self {
        Self { namespace: namespace.into(), value: value.into() }
    }

    #[must_use]
    pub fn namespace(&self) -> &str { &self.namespace }

    #[must_use]
    pub fn value(&self) -> &str { &self.value }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppScope {
    segments: Vec<ScopePathSegment>,
}

impl AppScope {
    #[must_use]
    pub fn app() -> Self {
        Self { segments: vec![ScopePathSegment::new("app", "app")] }
    }

    #[must_use]
    pub fn window(id: crate::window::Id) -> Self {
        Self { segments: vec![ScopePathSegment::new("window", id.as_u64().to_string())] }
    }

    #[must_use]
    pub fn surface(id: SurfaceId) -> Self {
        Self { segments: vec![ScopePathSegment::new("surface", id.as_u64().to_string())] }
    }

    #[must_use]
    pub fn resource(id: ResourceId) -> Self {
        Self { segments: vec![ScopePathSegment::new("resource", id.as_str())] }
    }

    #[must_use]
    pub fn custom(id: impl Into<String>) -> Self {
        let id = CustomScopeId::new(id);
        Self { segments: vec![ScopePathSegment::new("custom", id.as_str())] }
    }

    #[must_use]
    pub fn workspace(id: impl Into<String>) -> Self {
        Self { segments: vec![ScopePathSegment::new("workspace", id)] }
    }

    #[must_use]
    pub fn document(id: impl Into<String>) -> Self {
        Self { segments: vec![ScopePathSegment::new("document", id)] }
    }

    #[must_use]
    pub fn widget(id: impl Into<String>) -> Self {
        Self { segments: vec![ScopePathSegment::new("widget", id)] }
    }

    #[must_use]
    pub fn then(mut self, segment: ScopePathSegment) -> Self {
        self.segments.push(segment);
        self
    }

    #[must_use]
    pub fn segments(&self) -> &[ScopePathSegment] {
        &self.segments
    }

    #[must_use]
    pub fn is_app(&self) -> bool {
        self.segments.len() == 1
            && self.segments[0].namespace() == "app"
            && self.segments[0].value() == "app"
    }

    #[must_use]
    pub fn resource_id(&self) -> Option<ResourceId> {
        self.segments
            .first()
            .filter(|segment| segment.namespace() == "resource")
            .map(|segment| ResourceId::new(segment.value()))
    }

    #[must_use]
    pub fn window_id(&self) -> Option<crate::window::Id> {
        self.segments
            .first()
            .filter(|segment| segment.namespace() == "window")
            .and_then(|segment| segment.value().parse::<u64>().ok())
            .map(crate::window::Id::from_u64)
    }

    #[must_use]
    pub fn surface_id(&self) -> Option<SurfaceId> {
        self.segments
            .first()
            .filter(|segment| segment.namespace() == "surface")
            .and_then(|segment| segment.value().parse::<u64>().ok())
            .map(SurfaceId::from_u64)
    }
}
```

Add `descriptor.rs`:

```rust
use super::{
    AppId, AppScope, CommandDescriptor, EventDescriptor, ResourceId, RootId, SnapshotBinding,
    TaskName,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct App {
    descriptor: AppDescriptor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppDescriptor {
    id: AppId,
    version: String,
    diagnostics_namespace: String,
}

impl AppDescriptor {
    #[must_use]
    pub fn new(id: AppId, version: impl Into<String>) -> Self {
        let diagnostics_namespace = id.as_str().to_owned();
        Self {
            id,
            version: version.into(),
            diagnostics_namespace,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowDescriptor {
    id: String,
    title: String,
    allowed_roots: Vec<RootId>,
}

impl WindowDescriptor {
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            allowed_roots: Vec::new(),
        }
    }

    #[must_use]
    pub fn allows_root(mut self, id: RootId) -> Self {
        self.allowed_roots.push(id);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootDescriptor {
    id: RootId,
    required_commands: Vec<CommandDescriptor>,
    required_events: Vec<EventDescriptor>,
    snapshot_bindings: Vec<SnapshotBinding>,
}

impl RootDescriptor {
    #[must_use]
    pub fn new(id: RootId) -> Self {
        Self {
            id,
            required_commands: Vec::new(),
            required_events: Vec::new(),
            snapshot_bindings: Vec::new(),
        }
    }

    #[must_use]
    pub fn requires_command(mut self, command: CommandDescriptor) -> Self {
        self.required_commands.push(command);
        self
    }

    #[must_use]
    pub fn emits_event(mut self, event: EventDescriptor) -> Self {
        self.required_events.push(event);
        self
    }

    #[must_use]
    pub fn binds_snapshot(mut self, binding: SnapshotBinding) -> Self {
        self.snapshot_bindings.push(binding);
        self
    }

    #[must_use]
    pub fn snapshot_bindings(&self) -> &[SnapshotBinding] {
        &self.snapshot_bindings
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDescriptor {
    name: TaskName,
    input_type: &'static str,
}

impl TaskDescriptor {
    #[must_use]
    pub fn new(name: TaskName, input_type: &'static str) -> Self {
        Self { name, input_type }
    }

    #[must_use]
    pub fn name(&self) -> &TaskName {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceDescriptor {
    id: ResourceId,
    value_type: &'static str,
}

impl ResourceDescriptor {
    #[must_use]
    pub fn new(id: ResourceId, value_type: &'static str) -> Self {
        Self { id, value_type }
    }

    #[must_use]
    pub fn id(&self) -> &ResourceId {
        &self.id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupWindow {
    pub window_id: String,
    pub root_id: RootId,
    pub scope: AppScope,
}

impl StartupWindow {
    #[must_use]
    pub fn new(window_id: impl Into<String>, root_id: RootId, scope: AppScope) -> Self {
        Self {
            window_id: window_id.into(),
            root_id,
            scope,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppManifest {
    pub app: Option<AppDescriptor>,
    pub commands: Vec<CommandDescriptor>,
    pub events: Vec<EventDescriptor>,
    pub tasks: Vec<TaskDescriptor>,
    pub resources: Vec<ResourceDescriptor>,
    pub windows: Vec<WindowDescriptor>,
    pub roots: Vec<RootDescriptor>,
    pub startup: Vec<StartupWindow>,
}

impl AppManifest {
    #[must_use]
    pub fn new(app: AppDescriptor) -> Self {
        Self {
            app: Some(app),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn command(mut self, command: CommandDescriptor) -> Self {
        self.commands.push(command);
        self
    }

    #[must_use]
    pub fn event(mut self, event: EventDescriptor) -> Self {
        self.events.push(event);
        self
    }

    #[must_use]
    pub fn task(mut self, task: TaskDescriptor) -> Self {
        self.tasks.push(task);
        self
    }

    #[must_use]
    pub fn resource(mut self, resource: ResourceDescriptor) -> Self {
        self.resources.push(resource);
        self
    }

    #[must_use]
    pub fn window(mut self, window: WindowDescriptor) -> Self {
        self.windows.push(window);
        self
    }

    #[must_use]
    pub fn root(mut self, root: RootDescriptor) -> Self {
        self.roots.push(root);
        self
    }

    #[must_use]
    pub fn startup(mut self, startup: StartupWindow) -> Self {
        self.startup.push(startup);
        self
    }

    #[must_use]
    pub fn commands(&self) -> &[CommandDescriptor] { &self.commands }
    #[must_use]
    pub fn events(&self) -> &[EventDescriptor] { &self.events }
    #[must_use]
    pub fn tasks(&self) -> &[TaskDescriptor] { &self.tasks }
    #[must_use]
    pub fn resources(&self) -> &[ResourceDescriptor] { &self.resources }
    #[must_use]
    pub fn windows(&self) -> &[WindowDescriptor] { &self.windows }
    #[must_use]
    pub fn roots(&self) -> &[RootDescriptor] { &self.roots }
    #[must_use]
    pub fn startup(&self) -> &[StartupWindow] { &self.startup }
}
```

Add temporary surface markers in `mod.rs` until Task 9 gives them real ownership:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiSurface;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WindowRoot;

#[derive(Clone, Debug)]
pub struct AppEffect {
    kind: EffectKindId,
}

impl AppEffect {
    #[must_use]
    pub fn kind(&self) -> &EffectKindId {
        &self.kind
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EffectKindId(String);
```

- [ ] **Step 4: Run the test and format**

Run:

```sh
cargo test --package surgeist --test app app_front_door_exports_expected_names
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/Cargo.toml crates/surgeist/src/lib.rs crates/surgeist/src/app/mod.rs crates/surgeist/src/app/ids.rs crates/surgeist/src/app/coord.rs crates/surgeist/src/app/command.rs crates/surgeist/src/app/event.rs crates/surgeist/src/app/snapshot.rs crates/surgeist/src/app/descriptor.rs crates/surgeist/tests/app.rs
```

Expected: test passes, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 5: Commit**

```sh
git add crates/surgeist/Cargo.toml crates/surgeist/src/lib.rs crates/surgeist/src/app/mod.rs crates/surgeist/src/app/ids.rs crates/surgeist/src/app/coord.rs crates/surgeist/src/app/command.rs crates/surgeist/src/app/event.rs crates/surgeist/src/app/snapshot.rs crates/surgeist/src/app/descriptor.rs crates/surgeist/tests/app.rs
git commit -m "Add Surgeist app module skeleton"
```

---

### Task 2: Typed IDs, Provenance, And Diagnostics

**Files:**
- Modify: `crates/surgeist/src/app/ids.rs`
- Create: `crates/surgeist/src/app/provenance.rs`
- Create: `crates/surgeist/src/app/diagnostic.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/tests.rs`
- Modify: `crates/surgeist/tests/app.rs`

- [ ] **Step 1: Write ID and provenance tests**

Create `crates/surgeist/src/app/tests.rs` with focused tests:

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

Add to `crates/surgeist/src/app/tests.rs`:

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
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/ids.rs crates/surgeist/src/app/provenance.rs crates/surgeist/src/app/diagnostic.rs crates/surgeist/src/app/tests.rs crates/surgeist/tests/app.rs
```

Expected: all three tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/ids.rs crates/surgeist/src/app/provenance.rs crates/surgeist/src/app/diagnostic.rs crates/surgeist/src/app/tests.rs crates/surgeist/tests/app.rs
git commit -m "Add app IDs provenance and diagnostics"
```

---

### Task 3: Reducer, Input, Effect, And Snapshot Contract

**Files:**
- Create: `crates/surgeist/src/app/input.rs`
- Create: `crates/surgeist/src/app/effect.rs`
- Create: `crates/surgeist/src/app/reducer.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

- [ ] **Step 1: Write reducer purity contract tests**

Add to `crates/surgeist/src/app/tests.rs`:

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
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/effect.rs crates/surgeist/src/app/input.rs crates/surgeist/src/app/reducer.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, the integration test still compiles, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 7: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/input.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/reducer.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app reducer and effect contract"
```

---

### Task 4: Resource State Machine And Snapshot Data

**Files:**
- Create: `crates/surgeist/src/app/resource.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/effect.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

- [ ] **Step 1: Write resource transition tests**

Add:

```rust
#[test]
fn resource_state_tracks_freshness_and_refreshing_independently() {
    let mut resource = ResourceState::<u32, String>::idle(ResourceId::new("thumb:1"));

    resource.starting();
    assert_eq!(resource.status(), ResourceStatus::Starting);
    assert!(!resource.is_renderable());

    resource.ready(7, Freshness::Fresh);
    assert_eq!(resource.status(), ResourceStatus::Ready);
    assert_eq!(resource.value(), Some(&7));
    assert!(resource.is_renderable());
    assert_eq!(resource.freshness(), Freshness::Fresh);

    resource.refreshing();
    assert_eq!(resource.status(), ResourceStatus::Refreshing);
    assert_eq!(resource.value(), Some(&7));
    assert!(resource.is_renderable());

    resource.mark_stale("source changed");
    assert_eq!(resource.freshness(), Freshness::Stale);
    assert_eq!(resource.stale_reason(), Some("source changed"));
}

#[test]
fn resource_failure_preserves_renderable_stale_value() {
    let mut resource = ResourceState::<u32, String>::ready(
        ResourceId::new("query:1"),
        10,
        Freshness::Fresh,
    );

    resource.refreshing();
    resource.failed("timeout".to_string(), FailureVisibility::KeepStaleValue);

    assert_eq!(resource.status(), ResourceStatus::Failed);
    assert_eq!(resource.value(), Some(&10));
    assert_eq!(resource.error(), Some(&"timeout".to_string()));
    assert!(resource.is_renderable());
}
```

- [ ] **Step 2: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing resource types.

- [ ] **Step 3: Implement resource state**

Add `resource.rs` with:

- `ResourceStatus::{Idle, Starting, Running, Refreshing, Ready, Failed, Cancelling, Cancelled, Stale}`;
- `Freshness::{Fresh, Stale}`;
- `FailureVisibility::{ClearValue, KeepStaleValue}`;
- generic `ResourceState<T, E>` storing id, status, value, error, freshness, stale reason, version, and observer count;
- transition methods used by tests;
- `ResourceSnapshot<T, E>` cloneable view for render/state binding.

- [ ] **Step 4: Connect resource effects**

Update `effect.rs` with:

- `EffectKindId::LOAD_RESOURCE` and `EffectKindId::INVALIDATE_RESOURCE`;
- `LoadResourceEffect { id: ResourceId, scope: AppScope }`;
- `InvalidateResourceEffect { id: ResourceId, reason: String }`;
- constructors `AppEffect::load_resource(id, scope)` and `AppEffect::invalidate_resource(id, reason)`.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/resource.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/resource.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app resource state machine"
```

---

### Task 5: Task Registry, Status, Attempts, And Cancellation

**Files:**
- Create: `crates/surgeist/src/app/task.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/effect.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

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
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/task.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 7: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/task.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app task registry and attempts"
```

---

### Task 6: Service Registry And Mailbox Policy

**Files:**
- Create: `crates/surgeist/src/app/service.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/effect.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

- [ ] **Step 1: Write service policy tests**

Add:

```rust
#[test]
fn service_registration_exposes_mailbox_policy() {
    let registration = ServiceRegistration::new(ServiceId::new("jsonrpc"))
        .scope(AppScope::app())
        .mailbox(MailboxPolicy::bounded(4).drop_oldest().observe_overflow())
        .startup(ServiceStartup::Lazy)
        .shutdown(ServiceShutdown::DrainThenStop);

    assert_eq!(registration.id(), &ServiceId::new("jsonrpc"));
    assert_eq!(registration.scope(), &AppScope::app());
    assert_eq!(registration.mailbox().capacity(), 4);
    assert_eq!(registration.mailbox().overflow(), MailboxOverflow::DropOldest);
    assert!(registration.mailbox().observe_overflow());
}

#[test]
fn service_mailbox_reports_overflow_and_keeps_capacity() {
    let policy = MailboxPolicy::bounded(2).drop_oldest().observe_overflow();
    let mut mailbox = ServiceMailbox::<u32>::new(ServiceId::new("rpc"), policy);

    mailbox.push(1);
    mailbox.push(2);
    mailbox.push(3);

    assert_eq!(mailbox.len(), 2);
    assert_eq!(mailbox.overflow_count(), 1);
    assert_eq!(mailbox.drain().collect::<Vec<_>>(), vec![2, 3]);
}
```

- [ ] **Step 2: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing service types.

- [ ] **Step 3: Implement service model**

Add `service.rs` with:

- `ServiceStatus::{Stopped, Starting, Running, Degraded, Stopping, Failed}`;
- `ServiceStartup::{Eager, Lazy}`;
- `ServiceShutdown::{Immediate, DrainThenStop}`;
- `ServiceRestart::{Never, OnFailure}`;
- `MailboxOverflow::{RejectNewest, DropNewest, DropOldest, CoalesceByKey}`;
- `MailboxPolicy` with bounded capacity, overflow behavior, observability flag, and constructors used by tests;
- `ServiceRegistration` with id, scope, mailbox, startup, shutdown, and restart policy;
- generic `ServiceMailbox<T>` backed by `VecDeque<T>` with push, drain, len, and overflow count.

- [ ] **Step 4: Add service effects**

Update `effect.rs` with:

- `EffectKindId::START_SERVICE`, `EffectKindId::STOP_SERVICE`, `EffectKindId::CALL_SERVICE`, and `EffectKindId::SERVICE_DIAGNOSTIC`;
- `StartServiceEffect { id: ServiceId }`;
- `StopServiceEffect { id: ServiceId }`;
- `CallServiceEffect { id: ServiceId, command: String, correlation: CorrelationId }`;
- `ServiceDiagnosticEffect { id: ServiceId, diagnostic: Diagnostic }`;
- typed constructors with matching names on `AppEffect`.

Use `String` for the first command payload lane so typed service command decoding can be layered without blocking the runtime skeleton.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/service.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/service.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app service mailbox policy"
```

---

### Task 7: Coordination, Scopes, Subscriptions, And Observer Policy

**Files:**
- Modify: `crates/surgeist/src/app/coord.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

- [ ] **Step 1: Write scope and subscription tests**

Add:

```rust
#[test]
fn app_scope_covers_runtime_ownership_kinds() {
    assert!(AppScope::app().is_app());
    assert_eq!(
        AppScope::window(surgeist::window::Id::from_u64(9)).window_id(),
        Some(surgeist::window::Id::from_u64(9))
    );
    assert_eq!(AppScope::surface(SurfaceId::from_u64(3)).surface_id(), Some(SurfaceId::from_u64(3)));
    assert_eq!(
        AppScope::resource(ResourceId::new("graph")).resource_id(),
        Some(ResourceId::new("graph"))
    );
    assert_eq!(AppScope::custom("workspace:alpha").segments()[0].namespace(), "custom");
    assert_eq!(
        AppScope::workspace("alpha")
            .then(ScopePathSegment::new("resource", "graph"))
            .segments()
            .len(),
        2
    );
}

#[test]
fn subscriptions_attach_and_detach_observers_without_owning_work() {
    let mut coord = CoordinationState::default();
    let sub = Subscription::task(TaskKey::new("compile:main"))
        .scope(AppScope::resource(ResourceId::new("project:main")))
        .observer(SurfaceId::from_u64(1));

    coord.subscribe(sub.clone());
    assert_eq!(coord.observer_count(&sub.target()), 1);
    assert!(coord.is_observed(&sub.target()));

    coord.unsubscribe(&sub);
    assert_eq!(coord.observer_count(&sub.target()), 0);
    assert!(!coord.is_observed(&sub.target()));
}
```

- [ ] **Step 2: Write coalescing tests**

Add:

```rust
#[test]
fn coordination_coalesces_progress_by_key() {
    let mut coord = CoordinationState::default();

    coord.record_progress(ProgressEvent::new(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
        CoalescingKey::new("bytes"),
        "10",
    ));
    coord.record_progress(ProgressEvent::new(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
        CoalescingKey::new("bytes"),
        "20",
    ));

    let drained = coord.drain_progress_budgeted(8);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].payload(), "20");
    assert_eq!(coord.coalesced_progress_count(), 1);
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing subscription, observer, and progress-coalescing types.

- [ ] **Step 4: Implement coordination**

Extend `coord.rs` with:

- `SubscriptionTargetKindId` as an open string-backed id with built-in constants `TASK`, `RESOURCE`, and `SERVICE`;
- `SubscriptionTarget { kind: SubscriptionTargetKindId, key: String }` with typed constructors `task(TaskKey)`, `resource(ResourceId)`, and `service(ServiceId)`;
- `Subscription` with target descriptor, scope, observer surface id, and priority;
- `CoordinationState` with observer sets by target and coalesced progress map;
- `CoalescingKey` and `ProgressEvent`;
- observer attach/detach methods and budgeted progress draining.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/coord.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/coord.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app coordination scopes and subscriptions"
```

---

### Task 8: Retained Bridge For Typed App Commands

**Files:**
- Create: `crates/surgeist/src/app/bridge.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/input.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

- [ ] **Step 1: Write retained bridge success test**

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
enum BridgeCommand {
    Open,
    OpenWithPayload(String),
}

#[test]
fn retained_bridge_decodes_registered_command() {
    let command_name = surgeist::retained::CommandName::new("open").unwrap();
    let mut bridge = RetainedBridge::<BridgeCommand>::new()
        .command(command_name.clone(), |_| Ok(BridgeCommand::Open));

    let retained = retained_command_for_test(command_name);
    let inputs = bridge
        .commands_to_inputs(
            SurfaceId::from_u64(1),
            retained.route().clone(),
            std::slice::from_ref(&retained),
        )
        .unwrap();

    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].payload(), &BridgeCommand::Open);
    assert_eq!(inputs[0].provenance().source(), &InputSourceId::RETAINED);
    assert_eq!(inputs[0].provenance().surface_id(), Some(SurfaceId::from_u64(1)));
}
```

- [ ] **Step 2: Write retained bridge diagnostic tests**

Add:

```rust
#[test]
fn retained_bridge_reports_unknown_command() {
    let command_name = surgeist::retained::CommandName::new("unknown").unwrap();
    let mut bridge = RetainedBridge::<BridgeCommand>::new();
    let retained = retained_command_for_test(command_name);

    let error = bridge
        .commands_to_inputs(
            SurfaceId::from_u64(1),
            retained.route().clone(),
            std::slice::from_ref(&retained),
        )
        .unwrap_err();

    assert_eq!(error.diagnostic().code(), &DiagnosticCode::UNKNOWN_RETAINED_COMMAND);
    assert_eq!(error.diagnostic().provenance().surface_id(), Some(SurfaceId::from_u64(1)));
}

#[test]
fn retained_bridge_reports_invalid_payload() {
    let command_name = surgeist::retained::CommandName::new("open").unwrap();
    let mut bridge = RetainedBridge::<BridgeCommand>::new()
        .command(command_name.clone(), |_| {
            Err(BridgeDecodeError::invalid_payload("expected folder id"))
        });
    let retained = retained_command_for_test(command_name);

    let error = bridge
        .commands_to_inputs(
            SurfaceId::from_u64(1),
            retained.route().clone(),
            std::slice::from_ref(&retained),
        )
        .unwrap_err();

    assert_eq!(error.diagnostic().code(), &DiagnosticCode::INVALID_RETAINED_PAYLOAD);
    assert!(error.diagnostic().message().contains("expected folder id"));
}
```

Add a crate-local helper at the bottom of the test module:

```rust
fn retained_command_for_test(name: surgeist::retained::CommandName) -> surgeist::retained::Command {
    let button = surgeist::retained::Element::tagged(
        surgeist::retained::Tag::new("button").unwrap(),
    )
    .with_hook(surgeist::retained::Hook::new(
        surgeist::retained::Trigger::Event(surgeist::retained::EventKind::Click),
        name,
    ));
    let mut model = surgeist::retained::Model::new(
        surgeist::retained::Element::root().with_child(button),
    )
    .unwrap();
    let target = model
        .snapshot()
        .children(model.root())
        .unwrap()
        .next()
        .unwrap();
    let report = model
        .dispatch(surgeist::retained::Event::new(
            target,
            surgeist::retained::EventKind::Click,
        ))
        .unwrap();
    report.commands()[0].clone()
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing `RetainedBridge`.

- [ ] **Step 4: Implement bridge**

Add `bridge.rs` with:

- `RetainedBridge<T>` storing command decoders by `retained::CommandName`;
- `command(name, decoder)` builder;
- `commands_to_inputs(surface_id, route, commands)` returning `Vec<AppInput<T>>`;
- `BridgeError` carrying a `Diagnostic`;
- `BridgeDecodeError::invalid_payload(message)` mapped to `DiagnosticCode::INVALID_RETAINED_PAYLOAD`;
- diagnostics for unknown command names and decoder failures;
- provenance with surface id, retained source, route phase sequence, and correlation id supplied by runtime in Task 10.

Keep payload decoding closure-based in this slice. A generated command registry can target the same registration map.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/bridge.rs crates/surgeist/src/app/input.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/bridge.rs crates/surgeist/src/app/input.rs crates/surgeist/src/app/tests.rs
git commit -m "Add retained command bridge"
```

---

### Task 9: UiSurface Lifecycle, Root Replacement, And Isolation

**Files:**
- Create: `crates/surgeist/src/app/surface.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/effect.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

- [ ] **Step 1: Write lifecycle tests**

Add:

```rust
#[test]
fn ui_surface_lifecycle_tracks_native_state() {
    let mut surface = UiSurface::new(
        SurfaceId::from_u64(1),
        surgeist::window::Id::from_u64(10),
        WindowRoot::new(RootId::new("main")),
    );

    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Created);
    surface.ready();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Ready);
    surface.resized(surgeist::window::size(640, 480));
    assert_eq!(surface.viewport().width, 640.0);
    surface.hidden();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Hidden);
    surface.suspended();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Suspended);
    surface.closing();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Closing);
    surface.closed();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Closed);
    surface.destroyed();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Destroyed);
}
```

- [ ] **Step 2: Write isolation tests**

Add:

```rust
#[test]
fn replacing_root_creates_distinct_retained_model() {
    let window_id = surgeist::window::Id::from_u64(20);
    let mut surface = UiSurface::new(
        SurfaceId::from_u64(1),
        window_id,
        WindowRoot::new(RootId::new("a")),
    );
    let old_retained_root = surface.retained().root();

    surface.replace_root(WindowRoot::new(RootId::new("b")));

    assert_eq!(surface.window_id(), window_id);
    assert_eq!(surface.root().id(), &RootId::new("b"));
    assert_ne!(surface.retained().root(), old_retained_root);
    assert!(surface.invalidations().contains(SurfaceInvalidation::RootReplaced));
}

#[test]
fn separate_surfaces_do_not_share_retained_or_invalidations() {
    let mut one = UiSurface::new(
        SurfaceId::from_u64(1),
        surgeist::window::Id::from_u64(1),
        WindowRoot::new(RootId::new("main")),
    );
    let two = UiSurface::new(
        SurfaceId::from_u64(2),
        surgeist::window::Id::from_u64(2),
        WindowRoot::new(RootId::new("main")),
    );

    one.invalidate(SurfaceInvalidation::SnapshotChanged(StateVersion::from_u64(2)));

    assert_ne!(one.retained().root(), two.retained().root());
    assert_eq!(one.invalidations().len(), 1);
    assert!(two.invalidations().is_empty());
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing surface types.

- [ ] **Step 4: Implement surface state**

Add `surface.rs` with:

- `WindowRoot` containing `RootId` and root `retained::Element` factory input for this slice;
- `UiSurface` fields for surface id, window id, root, retained model, lifecycle, viewport, invalidations, last rendered state version, hover/focus/native focus facts, and scroll offset;
- `SurfaceLifecycle::{Created, Ready, Resized, Hidden, Occluded, Suspended, Closing, Closed, Destroyed}`;
- `SurfaceInvalidation::{RootReplaced, SnapshotChanged(StateVersion), ViewportChanged, RetainedChanged}`;
- lifecycle methods from tests;
- accessors `id`, `window_id`, `root`, `retained`, `lifecycle`, `viewport`, and `invalidations`;
- `replace_root` creates a fresh `retained::Model::empty()` or root factory model and invalidates only this surface.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/surface.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/surface.rs crates/surgeist/src/app/effect.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app UI surface lifecycle"
```

---

### Task 10: Runtime Queues, Reducer Dispatch, Effect Execution, And Redraw Targeting

**Files:**
- Create: `crates/surgeist/src/app/runtime.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/tests.rs`

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

    runtime.enqueue(AppInput::new(CounterInput::Increment, InputProvenance::system()));
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
    runtime.enqueue_task(AppInput::new(CounterInput::Increment, InputProvenance::task(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
    )));
    runtime.enqueue(AppInput::new(CounterInput::Increment, InputProvenance::ui(
        SurfaceId::from_u64(1),
    )));

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

    runtime.enqueue_task(AppInput::new(
        CounterInput::Increment,
        InputProvenance::task(TaskId::from_u64(1), TaskAttemptId::from_u64(1)),
    ));
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
    runtime.enqueue(AppInput::new(CounterInput::Increment, InputProvenance::system()));

    let report = runtime.drain_once(RuntimeBudget::default());

    assert_eq!(runtime.state().value, 0);
    assert_eq!(report.reducer_errors(), 1);
    assert_eq!(runtime.diagnostics().count(&DiagnosticCode::REDUCER_ERROR), 1);
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
    AppEffect, AppInput, Diagnostic, DiagnosticCode, DiagnosticEffect, DiagnosticLog,
    EffectKindId, Reducer, ReducerResult, RedrawTarget, RequestRedrawEffect, StateVersion,
    SurfaceId, TaskId, TaskRecord, UiSurface,
};

pub trait RuntimeExecutor {
    fn execute(&mut self, effect: &AppEffect) -> Result<(), String>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLane {
    Ui,
    Task,
    Service,
}

pub struct Runtime<State = (), R = (), Input = ()> {
    state: State,
    reducer: R,
    executor: Option<Box<dyn RuntimeExecutor>>,
    state_version: StateVersion,
    surfaces: BTreeMap<SurfaceId, UiSurface>,
    tasks: BTreeMap<TaskId, TaskRecord>,
    diagnostics: DiagnosticLog,
    ui_queue: VecDeque<AppInput<Input>>,
    task_queue: VecDeque<AppInput<Input>>,
    service_queue: VecDeque<AppInput<Input>>,
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
    pub fn with_executor(mut self, executor: impl RuntimeExecutor + 'static) -> Self {
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

    pub fn enqueue(&mut self, input: AppInput<Input>) {
        self.ui_queue.push_back(input);
    }

    pub fn enqueue_task(&mut self, input: AppInput<Input>) {
        self.task_queue.push_back(input);
    }

    pub fn enqueue_service(&mut self, input: AppInput<Input>) {
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
                RuntimeLane::Ui => self.ui_queue.pop_front(),
                RuntimeLane::Task => self.task_queue.pop_front(),
                RuntimeLane::Service => self.service_queue.pop_front(),
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
            match effect.kind() {
                kind if kind == &EffectKindId::REQUEST_REDRAW => {
                    if let Some(payload) = effect.payload().downcast_ref::<RequestRedrawEffect>() {
                        report.record_redraw_target(payload.target().clone());
                    }
                }
                kind if kind == &EffectKindId::EMIT_DIAGNOSTIC => {
                    if let Some(payload) = effect.payload().downcast_ref::<DiagnosticEffect>() {
                        self.diagnostics.push(payload.diagnostic().clone());
                    }
                }
                kind if kind == &EffectKindId::START_TASK || kind == &EffectKindId::CANCEL_TASK => {
                    match self.executor.as_mut() {
                        Some(executor) => {
                            if let Err(message) = executor.execute(effect) {
                                self.diagnostics.push(Diagnostic::error(
                                    DiagnosticCode::EFFECT_FAILED,
                                    message,
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
                _ => {}
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

Do not spawn real background work in this task. Start/cancel effects must flow through the injected `RuntimeExecutor`; Task 12 supplies fake and Tokio-backed implementations of that trait.

- [ ] **Step 6: Re-export runtime types**

Update `mod.rs` with the concrete runtime module and public facade exports:

```rust
mod runtime;

pub use runtime::{Runtime, RuntimeBudget, RuntimeDrainReport, RuntimeExecutor, RuntimeLane};
```

Remove or replace the Task 1 temporary `Runtime<State = ()>` marker so only the concrete runtime type is exported.

- [ ] **Step 7: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/runtime.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, formatting succeeds, the integration-facing runtime exports compile through downstream tests, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/runtime.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app runtime queue draining"
```

---

### Task 11: AppProxy Wake Bridge And AppLoop Adapter

**Files:**
- Create: `crates/surgeist/src/app/proxy.rs`
- Create: `crates/surgeist/src/app/loop_.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/tests.rs`
- Modify: `crates/surgeist/src/app/testing.rs`

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

If `window::Proxy` lacks a public wake-only method, keep the `window::Proxy` implementation crate-private and drive tests through `WakeBridge`. Do not add typed task events to `window::UserEvent`.

- [ ] **Step 5: Add fake wake bridge support**

Update `testing.rs` with the fake bridge used by the Task 11 tests:

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
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/proxy.rs crates/surgeist/src/app/loop_.rs crates/surgeist/src/app/testing.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass, public app exports still compile, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/proxy.rs crates/surgeist/src/app/loop_.rs crates/surgeist/src/app/testing.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app proxy wake bridge"
```

---

### Task 12: Executor Adapter, Fake Executor, And Optional Tokio Feature

**Files:**
- Create: `crates/surgeist/src/app/executor.rs`
- Create: `crates/surgeist/src/app/runtime_tokio.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/runtime.rs`
- Modify: `crates/surgeist/Cargo.toml`
- Modify: `crates/surgeist/src/app/tests.rs`

- [ ] **Step 1: Write fake executor tests**

Add:

```rust
#[test]
fn fake_executor_records_spawn_and_cancel_requests() {
    let mut executor = FakeExecutor::default();
    let handle = executor.spawn_task(SpawnRequest::new(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
        TaskKey::new("search:rust"),
        AppScope::app(),
    ));

    assert_eq!(handle.task_id(), TaskId::from_u64(1));
    assert_eq!(executor.spawned().len(), 1);

    executor.cancel(handle);
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

Add `executor.rs` with:

- `Executor` trait with `spawn_task`, `spawn_blocking_task`, `cancel`, and `name`;
- `SpawnRequest` containing task id, attempt id, key, scope, and blocking flag;
- `ExecutorTaskHandle` containing task id and attempt id;
- `FakeExecutor` for tests;
- `BlockingPolicy::{Abortable, NonAbortableReportCancelling}`;
- `ExecutorEvent` envelope for task progress/completion sent through `AppProxy`.

Do not add Tokio types, Tokio imports, or Tokio-backed executors to `executor.rs`. This file is the backend-neutral contract used by fake executors and runtime tests.

- [ ] **Step 5: Implement Tokio executor behind feature**

Create `runtime_tokio.rs` behind the `app-runtime-tokio` feature:

```rust
#[cfg(feature = "app-runtime-tokio")]
pub struct TokioExecutor {
    runtime: tokio::runtime::Runtime,
}
```

Implement `new`, `name`, and adapter methods. Keep spawned closures owned and `Send + 'static`. Do not expose Tokio runtime or join handles through public app APIs. Re-export the module from `app::mod.rs` only under the feature:

```rust
#[cfg(feature = "app-runtime-tokio")]
pub mod runtime_tokio;
```

- [ ] **Step 6: Hook runtime effect execution to executor requests**

Implement `RuntimeExecutor` for the fake executor in `executor.rs` and for the feature-gated Tokio executor in `runtime_tokio.rs` so `Runtime::with_executor(...)` can execute task effects whose kind is `EffectKindId::START_TASK` or `EffectKindId::CANCEL_TASK` through the adapter. Preserve the runtime behavior for instances created without an executor: task effects emit a structured `DiagnosticCode::EFFECT_FAILED`.

- [ ] **Step 7: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo test --package surgeist --features app-runtime-tokio app::tests::tokio_executor_is_hidden_behind_adapter
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/executor.rs crates/surgeist/src/app/runtime_tokio.rs crates/surgeist/src/app/runtime.rs crates/surgeist/src/app/tests.rs
```

Expected: tests pass with and without the feature, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 8: Commit**

```sh
git add crates/surgeist/Cargo.toml crates/surgeist/src/app/mod.rs crates/surgeist/src/app/executor.rs crates/surgeist/src/app/runtime_tokio.rs crates/surgeist/src/app/runtime.rs crates/surgeist/src/app/tests.rs
git commit -m "Add app executor adapter"
```

---

### Task 13: Headless Runtime Harness And Fakes

**Files:**
- Create: `crates/surgeist/src/app/testing.rs`
- Modify: `crates/surgeist/src/app/mod.rs`
- Modify: `crates/surgeist/src/app/tests.rs`
- Modify: `crates/surgeist/tests/app.rs`

- [ ] **Step 1: Write headless harness integration test**

Add to `crates/surgeist/tests/app.rs`:

```rust
use surgeist::app::testing::HeadlessApp;

#[test]
fn headless_app_runs_without_winit_or_tokio() {
    let mut app = HeadlessApp::counter();

    app.open_surface("main");
    app.input_increment();
    app.drain();

    assert_eq!(app.counter(), 1);
    assert_eq!(app.fake_window().redraws(), &[app.surface_id("main")]);
    assert_eq!(app.fake_executor().spawned().len(), 0);
}
```

- [ ] **Step 2: Write fake clock test**

Add to `crates/surgeist/src/app/tests.rs`:

```rust
#[test]
fn fake_clock_advances_scheduled_effects_deterministically() {
    let mut harness = HeadlessHarness::counter();
    harness.schedule_timer("debounce", Duration::from_millis(50));

    assert!(harness.due_timers().is_empty());
    harness.clock_mut().advance(Duration::from_millis(50));

    assert_eq!(harness.due_timers(), vec!["debounce"]);
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test --package surgeist --test app headless_app_runs_without_winit_or_tokio
cargo test -p surgeist app::tests
```

Expected: fail with missing testing harness types.

- [ ] **Step 4: Implement test harness**

Add `testing.rs` with:

- `FakeClock` with deterministic `now`, `advance`, and scheduled timer drain;
- `FakeWindowBridge` recording redraw requests and native window commands;
- `FakeWakeBridge` used by Task 11 tests;
- `HeadlessHarness<State, Reducer, Input>` wrapping runtime, fake executor, fake window bridge, and fake clock;
- `HeadlessApp::counter()` convenience fixture used by public integration tests;
- fixture input methods that enqueue app inputs instead of calling reducer directly.

Re-export `testing` as `pub mod testing;` so integration tests can use it.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test --package surgeist --test app headless_app_runs_without_winit_or_tokio
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/testing.rs crates/surgeist/src/app/tests.rs crates/surgeist/tests/app.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add crates/surgeist/src/app/mod.rs crates/surgeist/src/app/testing.rs crates/surgeist/src/app/tests.rs crates/surgeist/tests/app.rs
git commit -m "Add headless app test harness"
```

---

## Phase 2: Prototype Validation And Example

Tasks 14 through 16 validate the architecture with deterministic prototypes, stress cases, and one small example. Keep production changes in this phase limited to fixes discovered by the prototypes; broad feature-crate behavior remains outside this first slice.

### Task 14: Stress And Prototype Tests

**Files:**
- Modify: `crates/surgeist/src/app/tests.rs`
- Modify: `crates/surgeist/src/app/testing.rs`
- Modify: `crates/surgeist/src/app/runtime.rs`
- Modify: `crates/surgeist/src/app/coord.rs`
- Modify: `crates/surgeist/src/app/task.rs`
- Modify: `crates/surgeist/src/app/service.rs`

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

Extend `testing.rs` with this prototype harness API. The method bodies can be compact fixture logic, but the public shape below must match the tests exactly:

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
    pub fn progress_event(&self, index: usize) -> AppInput<PrototypeInput>;

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

Keep prototype logic in `testing.rs`; production modules should only receive small fixes surfaced by these tests.

- [ ] **Step 8: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app/tests.rs crates/surgeist/src/app/testing.rs crates/surgeist/src/app/runtime.rs crates/surgeist/src/app/coord.rs crates/surgeist/src/app/task.rs crates/surgeist/src/app/service.rs
```

Expected: tests pass without enabling `app-runtime-tokio`, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 9: Commit**

```sh
git add crates/surgeist/src/app/tests.rs crates/surgeist/src/app/testing.rs crates/surgeist/src/app/runtime.rs crates/surgeist/src/app/coord.rs crates/surgeist/src/app/task.rs crates/surgeist/src/app/service.rs
git commit -m "Add app runtime stress prototypes"
```

---

### Task 15: Small Fake Thumbnail Import Example

**Files:**
- Create: `crates/surgeist/examples/app-thumbnail-import.rs`
- Modify: `crates/surgeist/Cargo.toml`
- Modify: `crates/surgeist/tests/app.rs`

- [ ] **Step 1: Write example compile test**

Add to `crates/surgeist/tests/app.rs`:

```rust
#[test]
fn thumbnail_import_example_contract_runs_headless() {
    let mut example = surgeist::app::testing::ThumbnailImportExample::new();

    example.choose_folder("/tmp/photos");
    example.drain_once();

    assert_eq!(example.initial_tile_count(), 3);
    assert_eq!(example.thumbnail_status(0), surgeist::app::ResourceStatus::Starting);

    example.finish_thumbnail(0);
    example.navigate_away();
    example.drain_all();

    assert_eq!(example.import_task_status(), surgeist::app::TaskStatus::Running);
    assert!(example.redrawn_surfaces().contains(&example.gallery_surface()));
}
```

- [ ] **Step 2: Add example manifest entry**

Add to `crates/surgeist/Cargo.toml`:

```toml
[[example]]
name = "app-thumbnail-import"
path = "examples/app-thumbnail-import.rs"
```

- [ ] **Step 3: Create example**

Create `crates/surgeist/examples/app-thumbnail-import.rs` demonstrating:

- an app-scoped fake import task registration;
- stable initial photo tile ids created synchronously;
- thumbnail resources moving through `Starting`, `Ready`, and `Refreshing`;
- task progress events becoming app inputs;
- navigation away removing observation while policy keeps useful work running;
- targeted redraw for the gallery surface.

The example should use the headless testing fixture for deterministic output. It should print a compact text summary:

```text
initial_tiles=3
thumbnail_0=ready
import=running
```

- [ ] **Step 4: Run failing test and example build**

Run:

```sh
cargo test --package surgeist --test app thumbnail_import_example_contract_runs_headless
cargo run -p surgeist --example app-thumbnail-import
```

Expected before implementation: the test or example compile fails because the example fixture is absent.

- [ ] **Step 5: Implement example fixture support**

Add `ThumbnailImportExample` to `testing.rs` if the example needs shared deterministic helpers. Keep the public example small and readable; the example should demonstrate the API, not contain test-only machinery.

- [ ] **Step 6: Verify**

Run:

```sh
cargo test --package surgeist --test app thumbnail_import_example_contract_runs_headless
cargo run -p surgeist --example app-thumbnail-import
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/examples/app-thumbnail-import.rs crates/surgeist/src/app/testing.rs crates/surgeist/tests/app.rs
```

Expected: test passes and example output contains:

```text
initial_tiles=3
thumbnail_0=ready
import=running
```

- [ ] **Step 7: Commit**

```sh
git add crates/surgeist/Cargo.toml crates/surgeist/examples/app-thumbnail-import.rs crates/surgeist/src/app/testing.rs crates/surgeist/tests/app.rs
git commit -m "Add app thumbnail import example"
```

---

### Task 16: Final Verification And Review Handoff

**Files:**
- Read-only verification across the Surgeist crate and new plan implementation.

- [ ] **Step 1: Run focused app tests**

Run:

```sh
cargo test -p surgeist app::tests
cargo test --package surgeist --test app
```

Expected: all app module unit tests and public app integration tests pass.

- [ ] **Step 2: Run feature-gated executor tests**

Run:

```sh
cargo test --package surgeist --features app-runtime-tokio app::tests::tokio_executor_is_hidden_behind_adapter
```

Expected: feature-gated test passes and no Tokio type appears in public integration test signatures.
The test should import `TokioExecutor` from `surgeist::app::runtime_tokio`, and default `cargo test -p surgeist` should not compile or export `runtime_tokio`.

- [ ] **Step 3: Verify thumbnail example contract**

Run the headless thumbnail example contract and example binary:

```sh
cargo test --package surgeist --test app thumbnail_import_example_contract_runs_headless
cargo run -p surgeist --example app-thumbnail-import
```

Expected: the contract test passes and the example prints the compact summary from Task 15.

- [ ] **Step 4: Run full Surgeist crate tests**

Run:

```sh
cargo test -p surgeist
```

Expected: all Surgeist crate tests pass.

- [ ] **Step 5: Run formatting**

Run:

```sh
cargo fmt
```

Expected: formatting completes with no diff outside intentional files.

- [ ] **Step 6: Verify zero lint suppressions**

Run:

```sh
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' crates/surgeist/src/app crates/surgeist/tests/app.rs crates/surgeist/examples/app-thumbnail-import.rs
```

Expected: the scan prints no matches. The implementation must keep zero lint suppression attributes or local lint-suppression calls.

- [ ] **Step 7: Inspect status**

Run:

```sh
git status --short
```

Expected: only files changed by the app implementation tasks appear.

- [ ] **Step 8: Commit verification fixes if needed**

If the verification steps required small fixes inside files already touched by this plan, commit them:

```sh
git status --short
git add -p
git commit -m "Stabilize app runtime tests"
```

Expected: only reviewed hunks from files shown by `git status --short` are staged. If every changed file is wholly owned by this app-runtime implementation, explicit path staging is also acceptable, for example `git add crates/surgeist/src/app/runtime.rs crates/surgeist/src/app/tests.rs`.

## Self-Review Checklist

- Spec coverage: tasks cover the app module skeleton, IDs/provenance/diagnostics, reducer/effect contract, resource state machine, task registry/status/attempt/cancellation, service registry/mailbox policy, coordination/subscriptions/scopes, retained bridge, `UiSurface` lifecycle/isolation, runtime queues/effect execution, `AppProxy` wake bridge, executor adapter and optional Tokio feature, headless fakes, stress/prototype tests, and a small example.
- Boundary check: no task places app semantics inside `surgeist::window`; the only window integration is through descriptors, ids, commands, and the wake bridge trait.
- Core compile check: default `cargo test -p surgeist` must compile without Tokio-backed app runtime types unless feature-gated tests request `app-runtime-tokio`.
- Deferred scope check: full template syntax, real feature services, multi-root composition, advanced scheduling, and replay/audit systems are intentionally outside this first slice.
- Worker handoff: each task has specific files, tests to write first, commands with expected results, implementation scope, verification commands, and a concrete commit message.
