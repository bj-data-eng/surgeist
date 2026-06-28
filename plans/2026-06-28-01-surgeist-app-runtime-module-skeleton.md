# Surgeist App Runtime Foundation 01: Module Skeleton And Public API Fence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the initial app module front door and public API fence.

**Architecture:** This split creates only the stable `surgeist::app` front door and manifest vocabulary that later runtime pieces extend. It intentionally does not add the executor backend, optional Tokio feature, or real background execution yet; those arrive after the app/task/service contracts exist.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, and placeholder-free public API scaffolding.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

## Sequence File Map

This map names the files that the full 01-16 sequence will create or modify. Task 1 only owns the subset listed in its own `Files` section below. Workers must not create later-split files from this map unless they are executing that later numbered plan.

- Modify `src/lib.rs`: expose `pub mod app;` as the new front-door app runtime module.
- Create `src/app/mod.rs`: public app module facade, re-exports, and module-level documentation.
- Create `src/app/ids.rs`: typed IDs and names for apps, roots, surfaces, resources, tasks, task attempts, services, correlations, and custom scopes.
- Create `src/app/coord.rs`: open `AppScope` descriptors, open subscription targets, observer sets, task/resource/service coordination state, dedupe, and coalescing policy.
- Create `src/app/command.rs`: typed command registry, command names, command descriptors, and command manifest entries.
- Create `src/app/event.rs`: typed app event registry, event names, event descriptors, and event manifest entries.
- Create `src/app/snapshot.rs`: `AppSnapshot`, snapshot versions, manifest bindings, and render-facing snapshot metadata.
- Create `src/app/descriptor.rs`: app identity, command/event/task/resource descriptors, window descriptors, root descriptors, startup window/root mapping, and app manifest.
- Create `src/app/provenance.rs`: `InputProvenance`, open source id, causal parent, and stale-event metadata.
- Create `src/app/diagnostic.rs`: structured diagnostics, diagnostic ring buffer, counters, and severity.
- Create `src/app/input.rs`: `AppInput` wrapper and typed input lanes for UI, retained, task, service, window, and system events.
- Create `src/app/effect.rs`: `AppEffect`, effect batches, redraw targets, timers, window commands, task/service effects, and diagnostic effects.
- Create `src/app/reducer.rs`: reducer trait, reducer result, state versioning, and purity-facing API.
- Create `src/app/resource.rs`: resource identity, freshness, status state machine, stale-while-refresh facts, and resource snapshots.
- Create `src/app/task.rs`: typed task registration, task keys, policies, handles, status, attempt lifecycle, cancellation token, and task events.
- Create `src/app/service.rs`: typed service registration, mailbox policy, service status, command/event envelope, overflow counters, and shutdown policy.
- Create `src/app/bridge.rs`: retained command bridge from `retained::Report` and `retained::Command` to typed app commands or diagnostics.
- Create `src/app/surface.rs`: `UiSurface`, `WindowRoot`, surface lifecycle state, retained model ownership, surface invalidation, and per-window isolation.
- Create `src/app/runtime.rs`: `Runtime`, input/effect queues, drain budgets, reducer execution, snapshot invalidation, task/service event draining, and redraw targeting.
- Create `src/app/proxy.rs`: `AppProxy`, shared app event queue sender, coalesced wake signal, and bridge trait over `window::Proxy`.
- Create `src/app/executor.rs`: backend-neutral executor adapter traits, fake executor hooks, blocking task contract, and executor event envelopes.
- Create `src/app/loop_.rs`: `AppLoop` wrapper over `window::Loop`, native handler adapter, and app runtime dispatch boundaries.
- Create `src/app/testing.rs`: headless runtime harness, fake clock, fake executor, fake window bridge, assertion helpers, and prototype test fixtures.
- Create `src/app/tests.rs`: crate-local unit tests for IDs, provenance, diagnostics, reducer/effect behavior, tasks, resources, services, coordination, bridge, runtime, proxy, executor, and surfaces.
- Create `tests/app.rs`: public API integration tests that compile against `surgeist::app`.
- Create `examples/app-thumbnail-import.rs`: small fake thumbnail import example showing initial thumbnail tiles, progress, continuation policy, and targeted redraw.

## Implementation Boundaries

- Keep `surgeist::window` focused on native window identity, native commands, and event loop mechanics.
- Keep `surgeist::retained` focused on element identity, routing, focus, pointer capture, and retained command reports.
- Keep core `surgeist::app` usable without Tokio by default. This first split must not add a Tokio dependency or `app-runtime-tokio` feature.
- Do not expose raw Tokio handles, raw channel types, `winit` types, renderer handles, or non-send native handles in app authoring APIs.
- Do not start durable task or service work from render/template code. Subscriptions declare observation; runtime coordination decides effects.
- Keep the first slice general-purpose and Surgeist-local. Do not add DES-specific project/runtime state.

## Async Boundary Policy

Surgeist's app runtime uses architectural asynchrony, not an async-first UI model. The UI plane receives typed inputs, runs deterministic reducers, commits state, and declares effects. Work planes may be in-process tasks, blocking pools, one-shot sidecars, app-managed local servers, external services, or true project daemons, but all of them communicate with the app through typed commands, events, attempts, cancellation state, and diagnostics.

Prefer the least operationally expensive work plane that preserves correctness:

1. in-process deterministic code;
2. in-process task or blocking task lane;
3. in-process long-lived service;
4. one-shot sidecar process;
5. app-managed local server;
6. project daemon only when it must outlive the UI or coordinate multiple clients.

This sequence is policy for later task/service/executor plans. Do not introduce daemon-specific assumptions into the public app API.

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
- Modify: `src/lib.rs`
- Create: `src/app/mod.rs`
- Create: `src/app/ids.rs`
- Create: `src/app/coord.rs`
- Create: `src/app/command.rs`
- Create: `src/app/event.rs`
- Create: `src/app/snapshot.rs`
- Create: `src/app/descriptor.rs`
- Create: `tests/app.rs`

- [ ] **Step 1: Write the public API failing test**

Add `tests/app.rs`:

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

- Add `pub mod app;` to `src/lib.rs`.
- Create `src/app/mod.rs`, `ids.rs`, `coord.rs`, `command.rs`, `event.rs`, `snapshot.rs`, and `descriptor.rs` with the minimal public API shown below. These are concrete front-door files, not temporary marker definitions; follow-up tasks add fields and behavior without moving these names.

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
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/lib.rs src/app/mod.rs src/app/ids.rs src/app/coord.rs src/app/command.rs src/app/event.rs src/app/snapshot.rs src/app/descriptor.rs tests/app.rs
```

Expected: test passes, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 5: Commit**

```sh
git add src/lib.rs src/app/mod.rs src/app/ids.rs src/app/coord.rs src/app/command.rs src/app/event.rs src/app/snapshot.rs src/app/descriptor.rs tests/app.rs
git commit -m "Add Surgeist app module skeleton"
```

---
