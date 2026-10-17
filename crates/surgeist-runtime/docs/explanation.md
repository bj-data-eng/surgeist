# Explanation

## Runtime and root adapters

`surgeist-runtime` owns app orchestration, events and effects, lifecycle,
invalidation, frame scheduling, and provenance. A reducer describes state changes
and work; runtime validates and records the orchestration result. Root `surgeist`
owns the facade and adapters that connect those results to concrete Surgeist
crates. Runtime consumes the lower-level contracts used by its own adapters
directly. The [public reexports](../src/lib.rs) expose runtime-owned contracts,
and the [manifest](../Cargo.toml) declares those dependencies.

The task boundary is concrete: runtime emits abstract task intents and accepts
task-originated app inputs. Concrete task execution, cancellation, lifecycle,
progress coalescing, and Tokio integration belong to `surgeist-task`. Root owns
the adapter that lowers runtime task intents into task crate requests and maps
task events back into runtime queues.

Root owns facade composition; each consuming subsystem owns its integration
with the lower-level contracts it uses. Parsing, style resolution, layout
algorithms, text shaping, rendering backends and OS window event loops remain
outside this crate. `AppLoop` is a deterministic application drain wrapper;
the bounded native DOM loop has its own lifecycle owner. This separation lets the
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

## DOM orchestration

`DomSession` owns the original canonical `Dom`, CSSOM resources, Details task
state and native hosted lifecycle. Its explicit `DomSessionLimits` admits each
domain's actual storage and work. `source_execution()`, `style_execution()`,
`details_execution()` and `hosted_execution()` return weak permissions into
that same session. Dropping or
reacquiring a permission preserves the original resources, queues and spent work.
The scoped `DomEventDom` view permits canonical edits without exposing a
replaceable mutable owner or its source issuer. Session construction refuses
prebound Style custody and prior nonempty standalone Details transfers whose
external task custody it cannot recover.

DOM owns each source operation's identity and published continuation. Runtime
drives one edit or named feature operation through that same canonical engine.
The Source driver routes Style requests through the CSS and CSSOM owners, retains acknowledged
publications and queues the actual notification tasks. A later cancellation or
refusal cannot undo an already published phase. Resuming uses the same operation
and retained resources. Current CSSOM disabled observations come from the actual
store and its revision, independently of an older association receipt.

The native dispatcher retains an immutable event path and once-only progress
across suspension. Style and Details use this shared dispatcher while retaining
their own task rules. Callbacks receive controlled canonical access. Public
observations do not retain the dispatcher or session, and terminal disposal does
not dispatch callbacks or manufacture a completion receipt. For graceful
shutdown, `try_shutdown` returns `DomShutdownOutcome::Pending` with the whole
session when obligations remain.

`HostedRuntime` creates a native Host and an initial top-level navigable through
the canonical DOM source protocol. `create_top_level` returns the actual top-level
result; `resume_top_level_creation` continues a refused creation using its original
operation. The resulting Document, browsing context,
history entry and Global have distinct checked identities. Settings read the
actual associated Document; a captured settings value remains historical after
later edits. The original Settings retain the constructor's actual creation and
top-level creation URL records before native execution is sealed. For the
no-opener top-level constructor, both refer to the same issued record. Child
Settings retain the original top-level origin; cross-site ancestry observations
instead walk the actual current canonical ancestry. Their
readbacks borrow `HtmlBaseUrlRecord`; use `serialization()` when the spelling is
needed. Current base selection still follows the associated Document. Storage
refusal preserves the original operation and its records for a bounded retry.
Creation uses a shared monotonic clock domain and fallible OS
entropy for the two history UUIDs. Unsupported clock or entropy providers return
typed errors. Neither UUID generation nor cancellation promises to interrupt an
OS entropy call.

Connected native Frame/Iframe hooks use the same Source driver to create an
initial child and queue its history attachment. Runtime computes the initial
Permissions Policy for the selected fullscreen-only profile from the actual
parent, container, sandbox and origin inputs. Reached URL parsing and origin
facts require an independently configured `DomSourceUrlService`; unavailable
facts park the original operation. This does not implement general navigation
or resource loading. Frame and Iframe creation preserve their distinct sandbox
rules, and later attribute changes do not retroactively rewrite a completed
Document's policy.

`destroy_top_level` returns `DomHostedDestructionQueued` after retaining the
actual destruction invocation and its child dependencies. It does not promise
that a waiting parent's Global cleanup task has already been queued. Use
`run_next` on the actual traversable to service history, Global jobs and cleanup
through the shared native task loop. A completed child removal can therefore
leave mandatory queued history or destruction work. Distinct overlapping
invocations join the same actual per-Document cleanup while retaining their own
captured dependencies and completion identity. The Document and its tree remain canonical
objects after destruction; current hosted authority becomes inactive while older
snapshots retain their original observations. A Host can own several top-levels,
so destroying one does not retire the shared Host. `destroy_host` retires an
actually empty Host after its top-level, operation and task obligations are
settled; graceful session shutdown still retains live hosted ownership. Canonical
entry Document fields become null at actual destruction completion; earlier
snapshots retain their original Document identities. Live-child membership and
retained history custody are separate: a queued creation-history closure still
runs after removal without restoring the live container relation.

Details ingestion admits the actual canonical pending intentions before taking
custody. Tasks preserve their original queued Document and Global across target
adoption; dispatch builds its path from the current tree. At task completion,
the native loop checks the canonical checkpoint state. Pending Slot work cannot
be delivered by this bounded contribution: it retains the started task and its
delivered-event progress with `CheckpointUnavailable`. An actually empty bank
can complete the native checkpoint. This does not implement mutation observers,
JavaScript jobs or Slot delivery.

Hosted dispatch admission checks the current association before starting. Each
listener then uses its current target's actual node Document: known inactive
hosted Documents skip the listener; unavailable authority returns
`RetainedHostedAuthority` with the same pending listener and cursor. Earlier
callbacks are not replayed, and a pending once registration is not consumed by
unavailable admission. Closed native Documents remain eligible. Their existing
started-dispatch retirement policy preserves historical exact-target delivery;
it never redirects an event to a replacement generation or establishes current
Window authority from a missing node.

These operations implement bounded native orchestration. A retained sheet or
completed canonical edit does not establish fetched-resource completion,
nonempty-media evaluation, rendering readiness, browser event-loop integration
or JavaScript execution. The [Style](../src/dom_style.rs),
[event](../src/dom_event.rs) and [Details](../src/dom_details.rs) modules define
their admitted operations and typed unavailable states; the
[hosted lifecycle](../src/dom_lifecycle.rs) owns native creation and teardown. The
[DOM owner](../../surgeist-dom/README.md) defines canonical source continuations.

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
