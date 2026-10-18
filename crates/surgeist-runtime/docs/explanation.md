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

`DomSession` owns the original canonical `Dom`, CSSOM resources, DOM manipulation
task state, Source event continuations and native hosted lifecycle. Its explicit
`DomSessionLimits` admits each domain's actual storage and work.
`source_execution()`, `style_execution()`,
`element_task_execution()` and `hosted_execution()` return weak permissions into
that same session. Dropping or reacquiring a permission preserves the original
resources, queues and spent work.
The scoped `DomEventDom` view permits canonical edits without exposing a
replaceable mutable owner or its source issuer. Session construction refuses
prebound Style custody, an already bound DOM-manipulation task source and prior
nonempty standalone Details transfers whose external task custody it cannot
recover. An already bound Source event executor also refuses Session construction.

DOM owns each source operation's identity and published continuation. Runtime
drives one edit or named feature operation through that same canonical engine.
The Source driver routes Style requests through the CSS and CSSOM owners, retains acknowledged
publications and queues the actual notification tasks. A later cancellation or
refusal cannot undo an already published phase. Resuming uses the same operation
and retained resources. Current CSSOM disabled observations come from the actual
store and its revision, independently of an older association receipt.

The same driver returns actual local designMode completion, including DOM's
`NoActiveRange` or exact `RangeReset` result. An unavailable active association
or generic focusing action is forwarded unchanged as a pending designMode
request with its typed unavailable reason. Runtime preserves DOM's published
enabled/reset prefix and original operation across cancellation and resumption;
it does not infer a focus result from surface selections or acknowledge a
missing service. Positive generic focus execution remains a separate
contribution.

Style capture uses the canonical Document's current Base selection, including
the actual inherited about-base of an initial child Document. An empty bound
CSSOM root still retains lifecycle custody: native hosted Document destruction
refuses while that binding remains. The current public surface has no root
unbind operation. With a live Host, graceful Session shutdown reports
`HostedLifecycle`; terminal Session drop releases custody without claiming
destruction or completion.

For Dialog Show's initial beforetoggle request, call
`DomSourceRuntime::service_event` with the original request, actual dispatcher
and explicit canonical/dispatch limits. The Session's finite
`source_events` limits bound adapter jobs, retained bytes and cumulative adapter
work. Canonical Source work and dispatcher work retain their separate limits.
The durable job preserves the actual event, dispatch permit and completion receipt
across refusal or replacement of a weak execution permission. Servicing accepts
the event result; ordinary `resume` then runs the remaining Source cursor.
Callbacks can make ordinary edits or start nested Source work under the exact
event scope. An unfinished nested operation blocks the outer dispatch until the
child is serviced; completed callbacks are not replayed. Uncanceled Show queues
its actual null-source toggle task and publishes the canonical tracker before
adding `open=""` through the ordinary DOM attribute hooks. The reached prefix
publishes before the next service. An inactive or disconnected receiver reaches
RememberFocusedElement. A connected, fully active receiver enters its Document's
open-dialog list and establishes a close watcher through the actual original
Window manager before requesting that same focus observation. Direct Source attribute
and tree edits retain the same watcher hook and remaining source steps. Ordinary
atomic edits refuse the service boundary with `NeedsEventfulOperation` and no
publication. The queued task cannot run through an unfinished synchronous tail.
These publications do not establish completed Show or its event delivery.

The original Source operation retains its entry execution scope independently
of later node-Document changes. A hosted entry captures its original Global and
Host association; an unavailable association does not become a native entry.
For a native entry without a Global, obtain a weak `DomSourceExecutionScope`
through `DomSession::source_execution_scope` and supply it in
`DomSourceServices::execution_scope` at entry. The first advance seals an
unbound native entry. Resume cannot add or replace its scope: a supplied scope
must match the retained original, while `None` preserves that binding. Nested
operations select their own scope rather than inheriting a parent's proof.

`HostedRuntime::create_headless_host` establishes an immutable complete-Host
restriction excluding HTML content system focus throughout its admitted
lifetime, including later top-levels and children. Its actual producer can
therefore establish that no Element is focused. Ordinary Hosts have no focus
observation yet. Native Window focus, UiSurface selections, inactivity and
missing hosting are not substitutes for this proof. A headless entry's known
absence writes null to the canonical common HTML state through the original
Source receipt. Refused acceptance or
publication retains the observed result without sampling again. Saving null
over an already-null slot creates no artificial DOM revision. This observation
supplies neither a positive focused Element nor focus movement. Exact live
issuer/Host identity and the enforced lifetime restriction qualify this producer;
unrelated issuer progress does not invalidate its immutable absence fact. A
retired/reused Host, disposed authority or stale canonical basis still refuses.
Future mutable adapters must validate their actual source and scope revisions.

The following Document capture and popover steps run in DOM's original Source
continuation. Qualified canonical raw membership and actual hidden popover state
establish empty Auto and Hint showing lists; nonpopover members do not prevent
this proof. DOM retains the actual captured Document and null endpoint, then
rechecks that proof before completing the empty hide step. Such Show paths reach
RunDialogFocusingSteps. Unknown history or unqualified nonhidden state retains
the precise popover wait. Headlessness alone supplies no top-layer evidence.

For the focusing action, Runtime qualifies the receiver's current canonical
Document independently from its original execution scope and queued-task
Global. An explicitly retired Document denies feature permission before policy
evaluation. Its actual retained original Window can then establish no transient
activation from its constructor-owned never-activated state. Runtime publishes
that precise negative allow-focus result through the same affine focus bridge;
accepted denial completes the called permission step and its genuine early-return
branch without moving focus. Missing
Window state remains unavailable. The live execution checks for retired Globals
are preserved.

For a current live Document, DOM qualifies allowed-to-use in source order:
browsing context, full activity, then its actual focus-without-user-activation
policy. Runtime reads the current associated Window's activation only after
permission is known false. This Window may differ from the method's original
queue Global or remembered-focus Host. Live roles and the exact canonical basis
qualify the new receipt; the retired-Window path keeps its historical checks.
Enabled permission advances the HTML focusing child locally in DOM, with no
Window activation read or Runtime observation/header formation.
Unknown permission stays unavailable even when activation is known false.
Forming a required Runtime qualification consumes the existing finite temporary/header and
work allowance; refusal retains the original operation and no phantom result.

The original method retains one complete HTML focusing child. Actual autofocus
selects the subject; otherwise the HTML delegate protocols run their separate
ordinary-descendant passes in the same DOM operation. Empty traversals return
null and select the subject. An unavailable descendant query returns a precise
`FocusDelegate` request, retaining the actual root, queried node, language step
and original occurrence across retry. Runtime reports the missing query without
inventing a focusability observation or acknowledging called resolution.

Native own-inert, non-root and nondelegating facts can prove the selected subject's called
focusing makes no change. That proof does not complete the outer call: DOM
next resolves the selected control's actual top Document and compares actual
origins. A difference returns before autofocus-state writes; the same-origin
path clears that captured top Document's actual candidates, sets processed=true
and completes the child. Other unavailable work is exposed as the precise
`DialogFocusing` child action. No positive focus movement or focused-area history
is inferred.

Actual ordinary Document-tree insertion runs HTML's separate autofocus cursor
inside DOM's original effect stack. DOM owns the independently qualified
per-Document candidate history, actual insertion guards and top-Document
selection. Enabled permission advances locally. A Window-dependent allow-focus
step uses the same `DomFocusRequest`, preparation, publication and receipt owner
as Dialog, qualified to its real insertion occurrence. Runtime retains the
original result across refusal and retry. Setting the attribute without insertion
does not enqueue, and rendering-time autofocus flushing is a separate operation.

Each admitted native Window starts never activated. The current HTML event
surface admits load, error, toggle, beforetoggle and close; physical Window input and
application UiInput have no canonical HTML activation-event join. Future
trusted activation-triggering input must update the actual Window state and
perform HTML's propagation and close-watcher notification before admission.
Headless focus absence is a separate Host fact.

Watcher custody can retain an original Window after execution teardown.
An unfinished Source tail prevents its real cleanup from overtaking a pending
focus receipt; it cannot manufacture a missing Window. After the tail completes,
ordinary watcher cleanup can release that retired Window. Native no-Global
Documents, unknown feature branches and unsupported focusing actions still
retain their precise service wait.

Each native Agent owns its Window records separately from execution Globals.
The Window manager retains actual ordered watcher groups and Dialog-bound action
descriptors; registration does not require a nonmodal watcher to be enabled.
Establishment checks the Dialog's original relevant Global and its associated
Document, so adoption cannot silently select the destination's Window. Source
accepts the actual registration or destruction through its existing retained
receipt. Runtime keeps the corresponding record until a read through the current
canonical owner confirms publication of the exact watcher association or its
removal. Refused acceptance or publication retains custody without repeating the
manager effect.

Retiring execution removes the live Global without clearing the surviving
Dialog's watcher. Open removal and source-required tree cleanup can therefore
destroy it on the retained original Window even after the Document becomes
inactive. A retired Window releases when its watcher, publication and started-invocation
custody is empty; the existing host lifecycle then permits composition cleanup.
Watcher registration and destruction supply no focus service or platform
close-request processing. A started invocation and actual pending hook removal
retain the same original record independently of manager membership.
Show's canceled beforetoggle returns normally even if its callback
retired the receiver; an uncanceled read of that retired receiver is a typed
Source failure.

If a retained child exhausts an ancestor's Source allowance, use
`DomSourceRuntime::raise_event_limits` with the ancestor's original event request.
This monotonically raises policy through the same Session owner without resuming
the parent or disturbing the child, receipt or spent work. A replacement weak
permission can perform the same recovery.

The adapter's own cumulative work has a separate recovery front:
`DomSourceRuntime::raise_event_runtime_limits`. It monotonically raises the
Session's event-job, byte and work limits while retaining the original jobs,
permits, delivered receipts and spent work. Validation uses the proposed work
allowance, so an exhausted old allowance cannot prevent recovery. This update
runs no callback or canonical Source step. In particular, a receipt retained
after listener delivery can complete acceptance after a raise without replaying
that listener. Linked job bodies retain their original Source storage charge
until their actual custody is dropped.

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
Permissions Policy for the fixed fullscreen and focus-without-user-activation
profile, both with default allowlist `'self'`, from actual parent, container,
sandbox and origin inputs. The complete two-feature value publishes once;
neither feature can install independently. The one parser retains each exact
feature's last declaration, and `allowfullscreen` supplies only fullscreen's
legacy fallback. Reached URL parsing and origin
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

Surviving Documents retain their canonical metadata, registry and fixed platform
policy after hosted teardown. Ordinary DOM edits can use those facts while
current hosted settings and execution remain unavailable. Actual retirement
proves inactivity for the existing Dialog hooks and standalone Img wait; a missing
or expired live issuer does not supply that proof. Resource-bearing edits still
need their actual composed service, including removal of an associated Style
sheet through the Source driver.

Configure the shared element-task owner's aggregate count, storage and cumulative
work through `DomSessionLimits::element_tasks`. Each `run_next` selects
`DomElementTaskSourceKind::DomManipulation` or `UserInteraction` and receives
explicit `DomElementTaskRunLimits`: dispatcher limits bound event delivery, and
Source limits bound canonical cleanup. Selection preserves FIFO within that
source; physical bank order promises no ordering between sources. Filtering
uses the original work allowance. Replacing a weak execution permission resets
neither domain. Queued-task cleanup uses the original Source engine and reserves
its closing revision before callbacks can run.

Details and Dialog toggles use the DOM-manipulation source; Dialog Close
notifications use the user-interaction source. Their closed payloads derive the
source, while one actual owner, binding, serial and admission ledger retain all
queued and executing work. Details ingestion admits actual canonical pending
intentions before taking custody; Dialog publication joins through its exact
Source continuation. Toggle coalescing removes the tracked task and appends its
replacement at that source's current tail, retaining first old and latest new
state. Close tasks have no tracker and do not coalesce. Tasks preserve their
original queued Document and Global across adoption; dispatch builds its path
from the current tree. After a Toggle dispatch, the surviving Dialog's current
tracker is cleared even when a callback replaced it; that clear does not remove
the newer queued task. Close dispatch performs no toggle-tracker write and
replays none of the initiating method's effects. A retired target never redirects
cleanup to a replacement generation.

For Dialog toggles, `DomElementTaskProgress::Complete` pairs the actual receipt
with `dialog_toggle_disposition`: `TrackerCleared` means canonical cleanup cleared
the surviving target's tracker; `TargetRetiredDuringDispatch` means cleanup found
the exact target retired and performed no write to it. This result comes from
the original DOM Source operation. The task owner retains it with the receipt
across a later checkpoint refusal, so retry preserves both without replaying
listeners or inferring a new outcome from a later snapshot. Close and Details
completion carry no Dialog toggle disposition. Retirement before dispatch retains
the separate `SkippedTargetDestroyed` outcome and supplies no delivery receipt.

The standard `close_dialog(receiver, result)` Source operation represents
`close(optional returnValue)`, whose source is null. It retains omitted versus
supplied-empty result through the original method continuation. Initial absent
open returns; otherwise authentic noncancelable beforetoggle precedes a new
current open read. A surviving callback that removes open supplies the genuine
early return. The method publishes the toggle, runs complete open-removal hooks,
applies only a supplied result, clears actual request-close slots and reads
canonical previous focus. Known absence reaches real Close task publication.
Modal Close requests canonical deferred top-layer removal and clears the modal
flag. Unsupported positive restoration stays at its exact pending action.
The method completes only after its actual final QueueClose acknowledgment;
subsequent notification delivery has independent task custody.

If the original receiver is actually retired during beforetoggle, the dispatcher
finishes its retained path and authentic receipt before the method reports
`SourceReceiverRetired` at its required ReadOpen. Retirement is not an absent-open
result or cancellation. Detach, adoption and failed destruction retain their
ordinary source checks. Internal command or watcher Close callers with nonnull
source, source retargeting and their additional lifetime requirements remain
contribution work.

`request_close_dialog(receiver, result)` runs standard null-source requestClose
through the same Source owner. Current open, connectedness and Document activity
guards precede force-enable and nullable request-slot publication. Source retains
the outer Dialog and called watcher cursors; Runtime supplies the original
Window's actual membership, enabled/running state, activity and history effects.
Plain Cancel dispatch uses the shared dispatcher, with its actual cancelability
and retained path/receipt. Runtime releases its mutable borrows before listeners.
Its current hosted reads use DOM's qualified dispatch fronts while the actual
Source event scope is active. The published request state remains marked
incomplete; ordinary complete-source reads retain their guards. Dispatch resumes
with the current caller's cancellation input and the original Source accounting.

Each Window owns bounded invocation and pending-removal custody referencing the
same actual watcher records as its manager groups. Physical membership removal
does not destroy a started invocation. A reentrant request can return while the
outer cancel action still runs; it owns no reset obligation for that outer guard.
After authentic cancel return, only the owning invocation resets its guard.
Cancellation consumes history-action activation through a qualified canonical
capture: a real null Window navigable returns early; otherwise the actual
shadow-including descendant Window closure receives its timestamp assignments.
Never-activated timestamps do not by themselves establish an empty operation.

An uncanceled watcher rechecks current guards and removes membership before
Source invokes the existing Close frame with freshly captured request operands.
The watcher invocation settles before the outer literal force=false write.
If the receiver retired, started events and incurred cleanup still finish against
their original records. Source retains the genuine stopped action, exposes typed
pending cleanup, clears its canonical incomplete marker at actual terminal
settlement and reports the native failure. It does not turn cancellation into
an outer early return or manufacture Applied after a failed nested Close.

Modal requestClose uses this same watcher and nested Close path. Preventing
Cancel preserves the modal, watcher and top-layer input. An uncanceled result
can close it, retaining raw membership with pending removal and queuing separate
Toggle and Close notifications. If a reentrant request resets force-enable to
false, the outer fresh enabled check still permits modal Auto's Close Request
state; explicit `closedby="none"` instead keeps the modal open. The actual latest
request operands determine the nested Close result.

`show_modal_dialog(receiver)` reuses the same Source driver, dispatcher, element
task queue and watcher setup. After actual open hooks and a valid watcher
assertion, DOM publishes the coherent modal/raw-top-layer phase and its generated
UA overlay input before observing remembered focus. With actual remembered-focus
absence and empty showing-popover lists, it reaches allow-focus qualification.
A real inherited denial and the actual Window's false activation complete the
method before focus-target selection. Permitted calls run the actual child
protocol and can complete the own-inert, distinct-origin no-change path described
above. Other inputs retain their precise child wait. The structural publication supplies no computed-style or rendered result;
completion requires the actual producer terminal branch. Source-completeness guards protect ordinary
top-layer views and semantic captures. Canceled and post-event early returns use
the actual producer result. Callback adoption into a connected inactive Document
can finish the open hook without a watcher. DOM publishes that real prefix and
returns the selected typed invariant failure, preserving the original task
association and independent queued work. The original Source operation retains
the first failure and any incurred cleanup through cancellation, admission
refusal and lease reacquisition. It does not create a watcher invocation to
settle an absent watcher or replay completed setup. Receiver retirement during
started beforetoggle retains authentic dispatch; cancellation and the next
required live read keep their distinct outcomes.

At task completion, the native loop checks the canonical checkpoint state.
Pending Slot work cannot be delivered by this bounded contribution: it retains
the started task and its delivered-event progress with `CheckpointUnavailable`. An actually empty bank
can complete the native checkpoint. This does not implement mutation observers,
JavaScript jobs or Slot delivery.

The canonical DOM holds a weak binding to the actual task owner. Its live
storage projection remains part of every applicable Source admission after the
creating operation closes. Shared event and path backing stays charged until
its actual last observer releases it; removing a task row alone does not release
that backing. Canonical dispatch custody is counted once in Source admission,
while Runtime's own storage limit covers its full retained payload. Transferred
caller-owned diagnostic receipts retain their existing ownership boundary.

A retry prices the event's actual retained path, scope and listener state along
with any replacement failure bank. Lowering a new path limit does not shrink an
already captured path. Growing a retained bank admits the old and new backing
together before callbacks resume. A refused attempt preserves the original
event and delivered progress for a later policy raise.

Hosted dispatch admission checks the current association before starting. Each
listener then uses its current target's actual node Document: known inactive
hosted Documents skip the listener; unavailable authority returns
`RetainedHostedAuthority` with the same pending listener and cursor. Earlier
callbacks are not replayed, and a pending once registration is not consumed by
unavailable admission. Closed native Documents remain eligible. Before the
first callback, the dispatcher registers exact invocation targets with the
canonical DOM's bounded dispatch custody. If a callback retires one, successful
canonical publication retains its actual retirement-time node-document,
including adoption and destruction in one edit. Later listeners use that
association with the Document's current activity and issuer. They never infer it
from a missing node, an earlier path association or an earlier closed environment.
The same started event retains its path, registrations and cancellation state
across quota refusal and resume; it cannot redirect to a replacement generation.
The original queue association remains distinct from each listener's current
eligibility. Configure explicit `DomDispatchCustodyLimits` on the DOM as well as
the dispatcher's limits; the DOM's aggregate capacity defaults to zero.

These operations implement bounded native orchestration. A retained sheet or
completed canonical edit does not establish fetched-resource completion,
nonempty-media evaluation, rendering readiness, browser event-loop integration
or JavaScript execution. The [Style](../src/dom_style.rs),
[event](../src/dom_event.rs) and
[Element task](../src/element_tasks.rs) modules define their admitted
operations and typed unavailable states; the
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
