# Surgeist App DSL Architecture

## Purpose

Surgeist needs an app layer before a higher-level authoring DSL can settle. The
existing modules already separate native windows, retained element state, CSS,
style, text, layout, and rendering. What is missing is the reusable app runtime
that coordinates those pieces into a fluid, typed UI framework.

The app layer should make immediate UI work and long-running background work
feel like one coherent application without making either side pretend to be the
other. UI work must stay fast, local, and frame-sensitive. Task work must be
asynchronous, cancellable, observable, and allowed to continue when views come
and go. The app runtime consumes both as typed inputs and produces snapshots,
effects, and redraw requests.

The app layer is not an async runtime facade. It is a deterministic app-state
boundary that receives typed inputs and returns declared effects. Async runtimes,
native wakeups, services, and task workers are implementation machinery behind
that boundary.

## Current Layer Fit

The current Surgeist module split is close enough to support this cleanly.

- `surgeist::window` owns the native `winit` boundary: window identity, native
  events, commands, wakeups, handles, and draw scheduling.
- `surgeist::retained` owns element identity, projection, focus, pointer
  capture, input routing, and retained command emission.
- `surgeist::layout`, `surgeist::text`, `surgeist::render`, `surgeist::style`,
  and `surgeist::css` own focused subsystems.

The missing layer should not live inside `window`. `window` should remain the
native boundary. The new layer should live under `surgeist::app`, which becomes
the front-door runtime and DSL home.

## Top-Level Shape

`surgeist::app` should own the app-facing runtime concepts:

```text
surgeist::app
  App
  AppLoop
  Runtime
  AppCommand
  AppEvent
  AppEffect
  AppSnapshot
  AppScope
  UiSurface
  WindowRoot
  task
  coord
```

`AppLoop` is the bridge from the native `window::Loop` into app semantics.
`Runtime` owns live app state, registered tasks, task handles, surfaces, and
queues. `UiSurface` owns per-window or per-root UI machinery: retained model,
layout state, text state, render state, input state, focus, scroll, hover, and
other transient UI state.

The public DSL should generally speak in terms of apps, windows, surfaces,
commands, events, tasks, and snapshots. Users should not need to write against
`winit`, raw Tokio channels, or renderer internals.

Several names overlap existing lower-level modules. `window::App`,
`window::Command`, `window::Scope`, `retained::Command`, `retained::Event`,
`retained::Snapshot`, and render surfaces remain valid lower-level concepts.
The app layer should either use explicit app-prefixed names in the first
implementation or clearly require qualification in public examples. `app::App`
is the preferred authoring front door; `window::App` remains a native-window
bootstrap helper.

`app` is also the right home for the higher-level Rust DSL because it can expose
registration and runtime concepts together. The DSL should describe what the app
can do, what can run in the background, what windows and roots exist, and how
state becomes snapshots.

## App And Task Are Siblings

The central architecture should treat app/UI and task as sibling systems.

```text
App/UI side
  immediate, frame-sensitive, nonblocking
  window events, pointer events, keyboard events, focus, scroll, redraw
  retained state, surface state, snapshots, commands

Task side
  asynchronous, durable, cancellable
  progress, partial output, retries, process IO, network IO, file IO
  services, subscriptions, background work, completed results

Coordination
  routes app commands into task requests
  routes task events into app events
  maps task state into renderable snapshots
  scopes, deduplicates, prioritizes, cancels, and observes work
```

The app observes and commands tasks. Tasks emit events. Tasks never directly
mutate app state, retained state, surfaces, render caches, or windows.

The app loop receives sibling event sources:

```rust
enum Input<AppUiEvent, AppTaskEvent, AppSystemEvent> {
    Ui(AppUiEvent),
    Task(AppTaskEvent),
    Window(surgeist::window::InputEvent),
    System(AppSystemEvent),
}
```

The concrete API does not need to expose this exact enum, but it should preserve
the same boundary: UI events and task events are both inputs to app state.

Every input delivered to the app runtime should carry provenance. At minimum,
runtime diagnostics and stale-event filtering need source kind, surface id when
applicable, task id when applicable, task attempt or generation, correlation id,
and causal parent when an input was produced by a prior effect. These fields are
what make async behavior inspectable instead of spooky.

## Reducer Purity

Reducers should be synchronous and deterministic from the app runtime's point of
view. A reducer receives current state plus a typed app input, computes state
changes, and returns declared effects. It must not spawn tasks, await futures,
read wall-clock time, generate random ids, call services, mutate native windows,
or touch render resources directly.

When a reducer needs time, ids, persistence, a service call, a task start, a
timer, a window operation, or a redraw, it returns an `AppEffect`. The runtime
executes effects after committing state and snapshot invalidations. This keeps
the core app behavior replayable, testable without `winit` or Tokio, and safe
from hidden blocking work.

## Nonblocking Rule

The app loop must protect UI fluidity as a first-class invariant.

- Window input and draw callbacks must not wait for expensive work.
- App state mutation happens on the app loop thread.
- Background work communicates through typed queued events.
- Per-window surfaces are isolated so one busy or invalidated surface does not
  poison another.
- Draw requests are targeted to affected surfaces when possible.
- Task event floods are coalesced or batched before they overwhelm rendering.

Tokio is a good invisible backend for the default desktop app runtime, but core
app concepts should not require Tokio to compile. The app layer should define an
executor/runtime adapter boundary, with a feature such as `app-runtime-tokio` or
`app-desktop-runtime` enabling the Tokio-backed implementation. The public API
should expose Surgeist concepts such as tasks, services, subscriptions, events,
cancellation, and scopes rather than Tokio handles or raw channels.

## Threading Boundary

The app runtime should make the ownership rule boring and strict.

- App state, surface state, retained state, layout caches, text caches, and
  render state live on the app/window owner thread.
- Background tasks receive owned, sendable inputs and emit owned, sendable
  events.
- Task closures and service handlers that run off-thread must satisfy the
  runtime's sendability requirements.
- Non-send native handles, renderer handles, and retained ids are not smuggled
  into worker tasks.
- If a task needs UI-derived data, the app reducer converts current state into a
  task input first.
- If a task produces UI-visible data, it returns typed events; the reducer
  integrates those events into app state.

This keeps the task system powerful without making the app a shared mutable
state graph.

## Native Wakeup Bridge

The current `window::Proxy` can wake the native loop with native window commands
and internal actions. It does not yet expose a public app/task event lane. The
app implementation must not assume task events can ride the existing proxy
unchanged.

The first implementation should choose one of two compatible approaches:

- extend the native user-event bridge with an app-owned wake token that lets the
  app runtime drain its own task event queue on the window thread; or
- introduce an `app::AppProxy` layered over `window::Proxy`, where background
  tasks enqueue typed app events into the app runtime and use the native proxy
  only as a wake signal.

The preferred direction is an `AppProxy` because it keeps typed task/app events
owned by `surgeist::app` and leaves `surgeist::window` focused on native loop
mechanics.

## Task Registration

Tasks must be registered as typed app capabilities. Registration gives the
runtime enough information to identify work, validate inputs, display status,
deduplicate starts, route progress, cancel safely, and expose task state to DSL
or template-generated UI.

A task registration should describe:

- stable task id or name for diagnostics and bindings;
- typed input parameters;
- typed progress/event payloads;
- typed output payload;
- structured error payload;
- scope, such as app, workspace, document, resource, window, or custom id;
- identity key for deduplication and status lookup;
- policy for priority, cancellation, retry, observation, and continuation;
- executor function or service hook;
- mapping from task events into app events or commands;
- sendability and lifetime requirements for inputs, events, outputs, and
  executor closures.

Illustrative shape:

```rust
app.task::<ImportPhotos>()
    .id("library.import_photos")
    .scope(|input| AppScope::resource(input.library_id))
    .key(|input| input.folder.clone())
    .policy(TaskPolicy::continue_when_unobserved().dedupe_by_key())
    .run(import_photos)
    .on_event(AppEvent::PhotoImport)
    .on_complete(AppEvent::PhotoImportFinished)
    .on_error(AppEvent::PhotoImportFailed);
```

The exact builder names can change, but the registered information should not be
implicit or hidden inside arbitrary closures.

Task parameters should be ordinary typed Rust values. They may include ids,
paths, options, query descriptions, limits, cache policy, or feature-specific
configuration, but they should not include borrowed references into app state or
direct handles to surfaces/windows/renderers.

## Task Attempts And Cancellation

Task identity needs both a stable key and an attempt identity. The stable key
answers "is this equivalent work?" Attempt identity answers "is this event still
from the current run?"

The runtime should track:

- `TaskKey` for dedupe and lookup;
- `TaskId` or `TaskHandle` for the live task record;
- `TaskAttemptId` for each start/retry generation;
- cooperative cancellation token for running work;
- observer set or subscription count for interested views/resources;
- terminal state and last accepted event version.

Cancellation is cooperative. A cancelled task receives a cancellation signal and
may still produce late events. Runtime status should distinguish `Cancelling`,
`Cancelled`, `FinishedAfterCancel`, and `FailedToCancel` when a backend cannot
stop immediately. The runtime must drop or quarantine events whose attempt id no
longer matches the live task attempt, unless a task policy explicitly allows a
late successful result to be adopted. Deduped observers attach to the same live
task record. Detaching the last observer applies the task's continuation policy:
continue, lower priority, pause, or cancel.

Blocking and CPU-heavy work needs an explicit policy. Such work should run on a
bounded blocking pool, a dedicated worker, or a feature-provided service, and it
must report cancellation truthfully if the work cannot be aborted once started.
The app should never promise that cancellation has completed until the task or
service has actually stopped publishing accepted events.

## Service Registration

Long-lived async capabilities should be registered as services. Services are
siblings of tasks, not hidden globals. A service may own durable async resources
such as a process supervisor, MCP session, file watcher, local server, database
connection pool, or project daemon.

A service registration should describe:

- stable service id or name;
- typed commands accepted by the service;
- typed events emitted by the service;
- startup and shutdown policy;
- scope and ownership;
- failure and restart policy;
- bounded command/event mailbox policy, including overflow behavior and
  observability counters;
- task registrations or workflow hooks provided by the service;
- app event mappings for service events.

Feature crates can provide service registrations plus widgets and blocks that
know how to observe their service state. The app runtime still owns integration:
services emit events, and app state changes through the reducer.

## Retained Bridge

`surgeist::retained` already emits retained commands from element hooks during
input routing. The app layer needs an explicit bridge from retained command
reports into typed app inputs.

The bridge should:

- receive hit-tested retained events and retained command reports from a
  `UiSurface`;
- validate that the retained command name or id is registered for the current
  app/root;
- decode or construct the typed `AppCommand` or `AppEvent` payload;
- attach route, element id, surface id, and phase diagnostics;
- report unknown commands, invalid payloads, stale element ids, and ineligible
  targets as structured diagnostics;
- provide a compile-time target for future template-generated command names.

Retained remains responsible for routing and retained state. The app bridge is
responsible for turning retained intent into typed app input.

## Coordination Layer

`surgeist::app::coord` should sit between app state and task execution. It does
not replace either side. It provides the typed vocabulary that lets UI consume
task state safely.

Core concepts:

- `AppScope`: owns lifetime and routing, e.g. app, workspace, document, window,
  surface, widget, resource, or custom domain id.
- `TaskStatus`: queued, running, waiting, blocked, completed, failed,
  cancelling, cancelled, finished after cancel, or failed to cancel.
- `TaskHandle`: opaque handle for cancellation, reprioritization, and
  diagnostics.
- `TaskKey`: stable identity for dedupe and lookup.
- `Subscription`: declares interest in task/resource/service updates without
  owning the work.
- `Resource`: cacheable app data whose state may be loading, partial, fresh,
  stale, failed, or evicted.
- `Workflow`: named multi-step process composed of tasks and app events.
- `Service`: long-lived async capability such as an MCP server/client, file
  watcher, process supervisor, database bridge, or project daemon.

The coordination layer should support:

- deduplication of equivalent task starts;
- coalescing and batching of noisy progress events;
- backpressure for large streams;
- priority changes based on visible UI;
- explicit cancellation scopes;
- continuation of useful tasks when no view is observing them;
- partial failure as renderable state;
- stable resource identity;
- cache invalidation and freshness policies;
- transactional app updates from task event batches.

The coordination layer should expose task and resource state as snapshot data.
Views subscribe to or bind against snapshot state; they do not hold task handles
as hidden ownership. A view disappearing removes observation interest, not
necessarily the task itself. The continuation policy decides whether the task
keeps running, lowers priority, pauses, or cancels.

Resource state should be explicit rather than a scattering of booleans. The
first resource model should cover `Idle`, `Starting`, `Running`, `Refreshing`,
`Ready`, `Failed`, `Cancelling`, `Cancelled`, and `Stale`. A resource may be
fresh and still refreshing, or stale but still renderable. That distinction is
critical for media imports, dbt project state, MCP tool/resource calls, and
template-generated status bindings.

## Event And Effect Flow

The app reducer should be the single place where durable app state changes.

```text
window input / retained command / task event / system event
  -> app input
  -> reducer computes state changes and effects
  -> runtime commits app state, snapshot versions, and surface invalidations
  -> runtime executes effects
  -> task requests, window requests, persistence requests
  -> redraw requests from committed versions
```

Effects should describe side effects without performing them inside the reducer.
Useful effect families:

- start, cancel, pause, resume, or reprioritize task;
- open, close, resize, or focus window;
- request surface redraw;
- schedule timer or debounce;
- persist or load app data;
- emit diagnostics;
- call service method;
- update task/resource priority based on visible surface demand.

This keeps the reducer testable and prevents hidden blocking work from entering
the immediate UI path.

## Surfaces And Windows

Each active root should have its own `UiSurface` state. In the first
implementation, a normal native window hosts one primary active surface at a
time. Replacing the root creates or restores a different surface record rather
than reusing retained/layout/render state accidentally. Future overlay, dialog,
split-pane, or multi-root composition can add multiple active surfaces per
window with explicit z-order and composition rules.

Suggested runtime relationship:

```text
Runtime
  app state
  task registry
  task runtime
  coordination state
  window registry bridge
  surfaces: SurfaceId -> UiSurface
  window_surfaces: WindowId -> primary SurfaceId

UiSurface
  surface id
  window id
  root id
  retained model
  layout cache
  text system/cache
  render surface/cache
  input/focus/hover/scroll state
  last rendered snapshot version
```

Window events enter through `window::Loop`, are scoped by `window::Id`, routed to
the matching `UiSurface`, and then converted into app input or retained input.
Application state changes produce snapshots. Snapshots feed surfaces. Surfaces
request redraw when their visible state changes.

Surface lifecycle states:

- created when a window/root pair is opened or restored;
- ready after the native window is available and renderer/text resources can be
  attached;
- resized when native metrics or content size change;
- hidden or occluded when the host reports it should deprioritize redraw;
- suspended when native lifecycle requires resources to be released or paused;
- closing during close-request handling while app commands may still veto or
  transform close behavior;
- closed after the native window is gone and native resources must be released;
- destroyed when retained/layout/text/render/input state is intentionally
  discarded.

State ownership should be split cleanly. Retained owns element identity,
presence, focusability, routed hooks, and pointer capture semantics. `UiSurface`
owns window-local input facts such as cursor position, hover target, scroll
offsets, active pointer device state, last layout metrics, render resources, and
the currently projected retained model for that root. Focus changes flow through
retained state, while the surface tracks the native focus/window context needed
to route them.

Native lifecycle events that must be represented in the app layer include
resume, suspend, close request, closed, exit, resize, scale/DPI change,
visibility/occlusion changes, theme changes, and redraw readiness.

## Photo Library Example

A photo library import shows the desired split.

Fast app/UI path:

1. User chooses a folder.
2. App performs only bounded, known-cheap synchronous work. If counting or
   enumeration might block unpredictably, it creates an initial pending resource
   record and starts a task.
3. App creates placeholder records with stable photo ids.
4. Grid layout renders placeholders immediately.
5. Each placeholder observes thumbnail resource status.

Task path:

1. Registered import task receives folder and library id.
2. Task decodes photos, extracts metadata, generates thumbnails, and writes
   cache entries.
3. Task emits typed events such as `Counted`, `ThumbnailReady`,
   `MetadataReady`, `ItemFailed`, and `Completed`.
4. App reducer associates each event with the existing placeholder identity.
5. Affected surfaces redraw; other UI remains interactive.

If the user navigates away, the task may continue according to policy. If the
user returns, the fast UI path renders the current snapshot and already-finished
thumbnails without restarting the import.

## Future Feature Layers

The app runtime should make industrial feature crates natural without placing
those features in Surgeist core.

Examples:

- dbt-core-v2-style feature crate: project parsing, model graph compilation,
  lineage, diagnostics, run orchestration, logs, process supervision, database
  previews, and cancellation.
- MCP feature crate: JSON-RPC sessions, local server/client transports, tool
  registry, resource registry, streaming tool progress, and typed UI widgets for
  connected capabilities.
- Media feature crate: thumbnailing, metadata extraction, cache management,
  import workflows, visible-item prioritization, and partial failures.

These should plug into `surgeist::app` as tasks, services, resources,
subscriptions, commands, and widgets. They should not require a custom app loop.

## App Identity, Windows, And Roots

The app DSL should register app identity separately from native windows and
roots.

- App identity names the app, version, diagnostics namespace, and default
  runtime policies.
- Window descriptors describe native window role, title policy, size policy,
  startup behavior, close behavior, and allowed root ids.
- Root descriptors describe the retained root factory, initial surface state,
  required commands/tasks/resources, and snapshot bindings.
- Startup configuration maps app identity to one or more initial window/root
  pairs.
- Runtime commands can open additional registered windows/roots by descriptor
  id, not by ad hoc native construction.

This keeps the later template DSL and feature crates pointed at stable app
descriptors rather than native window details.

## Diagnostics And Error Policy

The app runtime should treat failures as structured data whenever possible.

Diagnostics should include app id, window id, surface id, root id, task id, task
attempt id, scope id, resource id, reducer input, emitted effects, dropped or
stale task events, and bridge errors. Development tooling should be able to
inspect recent events, effects, task transitions, and surface invalidations.

Error policy:

- reducer errors become structured app diagnostics and may produce app-level
  failure events;
- task panics become failed task attempts and diagnostics;
- service crashes follow service restart policy and emit service failure events;
- render, layout, and text failures become surface diagnostics and may mark the
  affected surface degraded;
- fatal native-loop failures remain fatal runtime errors.

The implementation plan should define which errors are recoverable in the first
slice and how tests assert those paths.

## Backpressure And Resource Ownership

The task coordination layer should define queue and resource behavior early.

- UI input has priority over task progress draining.
- Task event queues are bounded by default.
- Noisy progress streams declare coalescing keys.
- Large outputs can be chunked and applied over multiple loop turns.
- Fairness rules prevent one task or service from starving other task events.
- Resources declare ownership, freshness, invalidation source, memory pressure
  behavior, and stale-while-refresh policy.
- Eviction removes cached data, not durable app identity.
- Wakeups are coalesced so a burst of task events does not schedule thousands of
  native loop wakeups.
- Each loop turn drains task/service events with a budget and schedules another
  wake when work remains.

Task and service protocols should make queue policy visible in diagnostics:
capacity, dropped events, coalesced events, delayed events, and the oldest
queued event age. The public DSL can hide channels, but runtime policy should
not be invisible.

## Template And Subscription Rule

Templates and render code must not start durable work as a side effect of being
drawn. The safe rule is: templates declare desired command bindings,
subscriptions, task/resource identities, and widget blocks; the app runtime diffs
those declarations against current scopes and decides which effects to start,
continue, reprioritize, or stop.

This avoids render-driven async work, fallback churn, and widget-owned durable
state. Widgets may own local retained state such as focus, hover, text cursor,
scroll position, animation, and measurement caches. Durable business state,
service ownership, and task ownership belong to app state and coordination.

## Relationship To Future Template DSL

The future template/Smarty-like DSL should compile into typed Rust that targets
`surgeist::app` and retained roots. Template syntax should not own runtime
semantics. It should be able to reference registered commands, tasks, resources,
widgets, and status bindings by typed names generated or validated at compile
time.

The app layer therefore needs a stable manifest-like surface:

- registered command names and input types;
- registered app event names and payload types;
- registered task names, scopes, keys, input types, and status types;
- registered resource names and status types;
- registered windows and roots.

This lets templates remain declarative while preserving Rust compile-time
checks.

The first implementation only needs commands, app events, tasks, resources,
windows, and roots in this manifest. Services, widgets, and reusable template
blocks should be planned as extension points, not required in the first slice.

Template-generated subscriptions should be identity based. Re-rendering the same
template block should not restart equivalent work. Removing a block should remove
that observation interest, then apply the registered continuation policy instead
of blindly cancelling app-level work.

## Research-Informed Risks

The app architecture borrows useful lessons from Elm/Redux-style effects,
Iced-style task and subscription identity, React's stale async and Suspense
pitfalls, Tokio's cooperative cancellation and bounded-channel guidance, and
native GUI event-loop constraints. The mitigations that matter for Surgeist are:

- keep reducers deterministic and side-effect free;
- preserve task/resource identity and attempt generations;
- reject stale task events by default;
- make cancellation states honest rather than binary;
- use bounded queues and coalesced wakeups;
- keep all native UI handles on the owner thread;
- prevent task callbacks from re-entering reducer or render code;
- keep durable resource/task state out of widget mount/render state;
- make service mailboxes explicit, bounded, observable, and shut down through
  policy;
- avoid turning command/event/snapshot vocabulary into full CQRS/event sourcing
  unless a feature crate genuinely needs replay or audit.

References for the risk model:

- Elm effects: <https://guide.elm-lang.org/effects/>
- Redux side effects: <https://redux.js.org/usage/side-effects-approaches>
- Iced subscriptions: <https://docs.rs/iced/latest/iced/struct.Subscription.html>
- React effect race guidance: <https://react.dev/reference/react/useEffect>
- React Suspense caveats: <https://react.dev/reference/react/Suspense>
- Tokio graceful shutdown: <https://tokio.rs/tokio/topics/shutdown>
- Tokio `mpsc`: <https://docs.rs/tokio/latest/tokio/sync/mpsc/>
- Tokio `spawn_blocking`: <https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html>
- `winit` event-loop proxy: <https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopProxy.html>
- JSON-RPC correlation ids: <https://www.jsonrpc.org/specification>
- MCP lifecycle: <https://modelcontextprotocol.io/specification/2025-03-26/basic/lifecycle>
- MCP cancellation: <https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/cancellation>

## Non-Goals

- Do not make Tokio visible in the public authoring DSL.
- Do not let tasks mutate UI, app state, retained state, or render state
  directly.
- Do not put app semantics into `surgeist::window`.
- Do not require Studio or DES-specific app state as the proving target.
- Do not add arbitrary runtime template behavior to solve app coordination.
- Do not require every task to be cancelled when a view disappears.
- Do not force every task to survive navigation; continuation is policy.

## First Implementation Direction

The first implementation plan should build a narrow but real app layer rather
than a full industrial runtime all at once.

Initial slice:

1. Add `surgeist::app` with public module boundaries and minimal typed IDs.
2. Add `AppLoop` wrapper over `window::Loop`.
3. Add `Runtime` with app state, surface registry, input queue, effect queue,
   and redraw targeting.
4. Add typed task registry and task status model.
5. Add executor/runtime adapter traits so core app types do not expose Tokio.
6. Add a Tokio-backed internal task executor behind app abstractions under an
   explicit app runtime feature.
7. Add `AppProxy` or an equivalent app-owned wake token layered over
   `window::Proxy` so task events wake the native loop without becoming native
   window events.
8. Add coordination primitives for task keys, scopes, statuses, and
   subscriptions.
9. Add retained bridge tests that prove retained commands become typed app
   commands with diagnostics for unknown or invalid commands.
10. Add tests that prove UI events remain immediate while task events update
   snapshots asynchronously.
11. Add fake executor and fake window bridge tests so reducer and coordination
   logic can run without `winit` or Tokio.
12. Add surface isolation tests for separate windows and root replacement.
13. Add a headless app runtime harness with deterministic event queue, fake
   clock, fake executor, and fake window bridge.
14. Add a "latest search wins" prototype task to verify stale completion
   rejection.
15. Add an append-only log-stream prototype task to verify ordered stream
   accumulation, coalescing, and backpressure.
16. Stress the wake bridge with at least 10,000 task events, coalesced wakeups,
   frame-budgeted draining, closed-loop handling, and no reducer reentrancy.
17. Add a two-surface shared-service prototype where closing one surface does
   not kill an app-scoped task still observed elsewhere.
18. Add a fake MCP/JSON-RPC service prototype with out-of-order responses,
   notifications, progress, cancellation, timeout, and reconnect.
19. Add a blocking media-import prototype that truthfully reports cancellation
   requested while non-abortable work finishes.
20. Add a small example, such as a fake thumbnail import or fake project compile,
   that demonstrates placeholders, task progress, continuation when unobserved,
   and redraw of affected surfaces.

This slice is enough to validate the architecture without committing to the full
future DSL syntax.

## Review Checklist

Before implementation planning, validate:

- `surgeist::app` is clearly above `window`, `retained`, `layout`, `text`, and
  `render`.
- UI/app events and task events are sibling inputs.
- Tokio is invisible to app authors.
- Tokio/runtime policy is visible in diagnostics and tests.
- Reducers are synchronous and return declared effects.
- App inputs carry enough provenance for stale-event filtering and diagnostics.
- Task registration is typed and explicit.
- Services have an extension path without becoming hidden globals.
- Task state can be rendered without blocking.
- Tasks can continue or cancel according to policy.
- Cancellation is represented honestly for cooperative, blocking, and late-event
  cases.
- Task attempts, stale events, cancellation, and deduped observers have defined
  behavior.
- Resource state has a first-class state machine rather than ad hoc booleans.
- Backpressure, coalescing, bounded queues, and wake budgets are specified.
- Surfaces isolate per-window retained/layout/render/input state and have clear
  lifecycle states.
- Retained command reports have a typed app bridge.
- The native wakeup path does not pretend `window::Proxy` already carries typed
  task events.
- Executor feature gates keep Tokio invisible and optional at the core app
  boundary.
- Future feature crates can plug in through tasks, services, resources,
  subscriptions, commands, and widgets.
- Future template codegen has a typed app manifest to target.

## Review Cycle Notes

The first review pass checked the draft for unresolved placeholders, ownership
contradictions, missing registration pieces, and gaps between task state and
renderable app snapshots. The review added the explicit threading boundary,
service registration, snapshot-observation rule, task sendability requirements,
and template manifest entries for app events and services.

A clean-context subagent review then found that the wakeup bridge, app-layer
name collisions, surface lifecycle, effect ordering, Tokio feature boundary,
retained bridge, task cancellation/stale-event model, diagnostics, backpressure,
and template manifest scope needed sharper treatment. Those findings were
incorporated directly into the sections above.

A deeper research pass compared this architecture with Elm/Redux effects, Iced
subscriptions, React async state and Suspense behavior, Tokio cancellation and
backpressure, native event-loop constraints, actor/service mailbox risks, and
JSON-RPC/MCP lifecycle semantics. The resulting mitigations were incorporated as
requirements for reducer purity, input provenance, explicit resource state,
bounded queues, honest cancellation, render-safe subscriptions, runtime
diagnostics, and early stress prototypes.
