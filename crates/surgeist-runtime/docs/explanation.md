# Explanation

## Runtime and root adapters

`surgeist-runtime` owns app orchestration, events and effects, lifecycle,
invalidation, frame scheduling, and provenance. A reducer describes state changes
and work; runtime validates and records the orchestration result. Root `surgeist`
owns the facade and adapters that connect those results to concrete Surgeist
crates. The [public reexports](../src/lib.rs) expose runtime-owned contracts, and
the [manifest](../Cargo.toml) declares no sibling dependencies.

The task boundary is concrete: runtime emits abstract task intents and accepts
task-originated app inputs. Concrete task execution, cancellation, lifecycle,
progress coalescing, and Tokio integration belong to `surgeist-task`. Root owns
the adapter that lowers runtime task intents into task crate requests and maps
task events back into runtime queues.

Root also owns integration with template, CSS, style, retained, text, layout,
render, and window crates. Parser, style-resolution, layout-algorithm,
text-shaping, rendering-backend, retained-tree, and host implementation details
remain outside this crate. `AppLoop` is a deterministic drain wrapper; it does
not implement a native event loop. This separation lets the
[intent example](how-to.md#emit-work-for-root-adapters) exercise orchestration
without those concrete systems.

## Explicit commits and effect outcomes

A [reducer](../src/reducer.rs) borrows application state and input immutably.
It returns unchanged state with a commit, replacement state with a commit, or a
recoverable failure that cannot carry replacement state or effects. Reducers can
mutate their own implementation state; the application-state commit boundary
does not roll back arbitrary reducer internals.

[Runtime](../src/runtime.rs) preflights changed-state version and surface
invalidation advances before committing the input. A checked overflow restores
that input to its lane and exposes earlier committed work in a partial report.
A successful input's effects retain order and receive applied, forwarded, or
rejected outcomes. Diagnostics and eligible redraws are handled locally;
persistence, resource, task, and service work is forwarded as typed intents.
The resulting disposition describes runtime handling, not eventual adapter
completion.

[Provenance](../src/provenance.rs) records the input origin and its source-specific
identity. Task inputs identify a task and attempt; service inputs identify a
service; surface origins carry a generation-qualified reference. Current and
parent correlations are independently present or absent. Runtime chooses
effective provenance from an explicit reducer override or the triggering input.

## Identity, lifetime, and rendering progress

Window, surface, and element IDs are distinct runtime-owned types. A
`SurfaceRef` adds a generation to a surface ID, and element references and routes
retain that surface generation. Replacing a root or registering a previously
removed surface ID changes the generation, allowing runtime to reject delayed
work for the earlier registration.

The runtime registry owns surface mutation and observer validation. Surface
lifecycle determines whether interaction or rendering is eligible; only `Ready`
and `Resized` render. Terminal surfaces reject local changes. Staged updates
keep failed registry changes atomic, and generation changes, terminal
transitions, and removal clear the affected observer subscriptions.

Application state versions, surface generations, and invalidation generations
describe different changes. A render frame captures the surface, state version,
and invalidation boundary it covers. Acknowledgement consumes covered work
without discarding later invalidations or regressing rendered state. The
[surface contracts](../src/surface.rs) define the lifecycle transitions and
acknowledgement errors.

## Queues and host wakeups

Runtime has separate bounded UI, task, and service lanes. Admission errors return
the rejected input, and draining visits eligible lanes in persistent cyclic
order within global and per-lane budgets. Remaining counts cover all lanes, so
a host can distinguish a completed turn from an empty runtime.

`AppProxy` is a thread-safe staging queue for task and service input. Its wake
bridge requests a future host turn; it must neither synchronously send or drain
through the same proxy nor wait for that drain. Partial draining requests a
continuation wake. A failed continuation is reported with remaining work so the
root adapter can arrange another turn. The host owns the concrete scheduling
mechanism; [proxy.rs](../src/proxy.rs) owns the delivery contract.

## Resource values, subscriptions, and service mailboxes

[ResourceState](../src/resource.rs) separates lifecycle status from whether a
value remains available and fresh. Loads and refreshes issue operation tokens;
completion, failure, and cancellation must match the active resource, operation,
and generation. Refreshing can retain an existing value. A failure policy can
retain that value as stale, so an error does not necessarily remove renderable
data. Rejected transitions and checked counter overflow leave the resource
unchanged.

[CoordinationState](../src/coord.rs) owns subscription references. Target, scope,
generation-qualified observer, and priority all participate in a key. Repeated
references and distinct scopes or priorities remain distinguishable, while
aggregates provide a target-level view. Resource state does not keep a second
observer count.

[ServiceMailbox](../src/service.rs) is a bounded FIFO with explicit rejection or
oldest-message eviction outcomes. Registration metadata describes service
startup, shutdown, and restart choices; concrete service management stays with
the adapter. Mailbox policy reports what happened to each push without implying
coalescing or task execution.

## Authored declarations and snapshots

An authored `AppManifest` becomes an immutable `ValidatedAppManifest` only after
cross-descriptor validation. `App` owns that validated manifest. Snapshot
construction uses a declared root and copies its binding declarations;
subsequent entries must match those declarations. `SnapshotValue` carries opaque
serialized text rather than a codec or schema implementation. The
[descriptor](../src/descriptor.rs) and [snapshot](../src/snapshot.rs) modules
define the validation errors and successful construction paths.
