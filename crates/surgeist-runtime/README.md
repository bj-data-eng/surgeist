# surgeist-runtime

Rust orchestration contracts for Surgeist application and adapter authors.
Runtime coordinates application state, events/effects, surface lifecycle,
invalidation and queued inputs. It emits abstract work intents and owns its
adapters to lower-level contracts, including the bounded native DOM task loop.
Concrete task execution, rendering and OS window event loops belong to their
respective owners.

The [DOM orchestration guide](docs/explanation.md#dom-orchestration) explains
`DomSession`, Source operations, event/task custody and hosted lifecycle limits.
These native operations do not supply complete browser services, positive focus
movement or rendering readiness.
The actual Style association owner also retains [Document stylesheet-set
preferences](docs/explanation.md#document-stylesheet-set-preferences), including
empty-list history and source-defined enable/select behavior. `DomSession::new`
returns the original DOM intact with `PendingParsedAssociations` when parsed
association history remains pending under canonical custody. Explicit
`DomSession::admit_parsed` adopts complete supported contributions into the real
Session before readiness and before any CSSOM operation. Original contribution
work and raise-only processing policy survive a refused attempt; structural
state, identities and effects remain unchanged. Successful admission carries its
actual Runtime discovery work into that Session. Parsed processing uses the
shared `DomParsedHtmlAssociationLimits` envelope; its admission errors and
processing resources are distinct from native Meta limits.

The Session's borrowed `pending_parsed_html_associations` iterator discovers
original canonical custody even after installation outputs are dropped; it lends
no execution authority and performs no eager scan. The parsed document/fragment
installation fronts publish original structure and genuine Pending evidence
separately from execution.
`DomStyleRuntime::consume_parsed_meta` consumes a whole supported contribution
exactly once through the existing Document stylesheet-set owner. It preserves
original occurrence identity/order and UTF-16 names, including removed subjects
and zero-sheet history. Every producer flag, prepared name and canonical pending
acknowledgement is admitted before publication; names install while the actual
producer locks remain held. Failed preparation keeps original work and custody,
without acknowledging or replaying effects. Completing one contribution leaves
other contributions on the same Document guarded.

Current public parsing produces executable Meta history for fresh Documents.
Contextual fragment Meta is an original no-effect path, preserving an existing
owner's preferred/last-selected names and sheet flags. Historical Meta effects on
existing selected owners remain conditional on a producer that can genuinely
capture that profile. A later captured shadow-Style contribution can already
share the same Document; completing its earlier Meta contribution preserves the
remaining Style guard.

Influencing Style histories remain Pending. On an existing Session's structural
installation, `prepare_parsed_style` can prepare the first influencing Style of
a fresh original Document through its historical inline-policy request. DOM
advances HTML's cursor using the original retained inputs; Runtime checks its
actual Session, idle custody and finite work. This prefix needs no CSSOM root
capability and creates no sheet or preference effect.

The explicit `bind_parsed_style_initial_policy` command accepts only the exact
constructor-declared unbound slot and genuine immutable `HtmlCspAssociation`
custody. A supplied constructor-empty artifact is explicit input, never an
inference from current referrer/default policy. Accepted binding survives later
request-admission refusal. The separate `parsed_html_style_prefix` Session query
lends the same canonical request with original identities, UTF-16 text/nonce
and policy identity; it grants no reply or execution authority. Drop the borrowed
view before binding or advancing. Original work, phase and raise-only limits
remain canonical across retries. Raw `admit_parsed` still refuses influencing
Style histories.

An external evaluator can inspect that immutable request and construct a
`DomParsedHtmlStyleInlinePolicyObservation` from the same request. Drop the view
before passing the decision envelope to `accept_parsed_style_inline_policy`.
The fixed observation qualifies the original request and exact bound policy;
it retains no text, nonce or policy graph. A missing evaluator supplies no
observation and never implies permission. No CSP parser or evaluator is selected
by Runtime. Explicit constructor-empty policy differs from an opaque external
artifact and from unbound or unavailable authority.

Acceptance feeds HTML's retained cursor once: allow stops at
`CreateSheetPending`, while block records `InlinePolicyBlocked` for only that
original occurrence. Both keep the whole contribution Pending. Cancellation or
admission refusal keeps the caller's actual decision envelope for retry without
evaluating again; original work, policy and output accounting remain cumulative.

`create_parsed_style_sheet` executes the reached original CreateSheet action
using the actual bound CSSOM root and the sole add-sheet/preference algorithm.
The first insertion's text is empty: it precedes parser push and text
consumption. Its authentic ParserPopped successor uses its own retained text;
later final markup is never substituted. Original title, media,
parser Document and mode qualify the actual nonconstructed sheet. CSSOM and DOM
prepare all fallible work before a joint publication fence; Runtime installs
preformed names/serial while the genuine root lock remains held. Returned
completion exposes the actual sheet ID/revision, DOM commit, CSSOM commit and
retained media diagnostics. Success stops at `ReadScriptBlockingPending`;
whole history and Style readiness remain Pending. Repeated creation of the same
invocation refuses; a separately allowed ParserPopped invocation creates a new
sheet and association with its own processing receipt.
The same fence retains a compact original processing receipt in the Session's
completion lifecycle bank. It records import dependencies from the actual parsed
rules, original qualification, execution profile and genuine sheet/association
identity and creation revision. Caller output Drop and later CSSOM edits do not
rewrite it. This pre-task record is included in Runtime storage and live-resource
custody; it is neither an import-resource wait nor a networking task. It issues
no notification permission or task serial. Parsed dispatch still requires the
separate original synchronous frontier and remaining influencing history.
`queue_parsed_style_notification` consumes that receipt once. It checks the
actual original sheet's current owner against its retained creation-time label
under the genuine root guard. A cleared owner returns `OwnerAssociationRemoved`
without a task; another nonnull owner refuses. Later revisions alone do not
invalidate original processing. A queued task retains its exact parsed origin,
target and receipt through subsequent sheet changes/removal and Session moves.
The real networking FIFO returns `OriginalSynchronousPending` before event or
unblock work for these tasks, naming actual remaining synchronous work or the
unavailable frontier. It cannot skip that head to deliver a later task. No positive
frontier is supplied by this checkpoint; native task behavior is preserved.
`advance_parsed_style_successor` selects the next original ParserPopped update
inside the retained contribution. If the predecessor created a sheet, it jointly
removes that sheet and clears the canonical association before reaching a fresh
inline-policy request for the original text and nonce. The old sheet and rules
remain readable. An actually blocked predecessor has no sheet or creation owner;
advancement needs no CSSOM root and fabricates no producer publication. The
completion's borrowed `removal()` view distinguishes actual DOM/CSSOM commits
from `NoAssociatedSheet` while keeping the new qualification available in both.
Preferred/last-selected history and root/task serials stay with their existing
owners; previous processing receipts and queued tasks retain their own original
qualification. Queue delivery and import completion are not prerequisites for
this synchronous step. The new request needs its own policy decision. An allowed
successor can then create, exclude script bookkeeping and complete its rendering
tail through the same owners. A blocked successor creates no sheet or task;
real imports remain `PendingImports` until genuine resource outcomes exist.
Further influencing history and a positive networking frontier remain unavailable.
`exclude_parsed_style_script_bookkeeping` binds the actual Session's
no-executable-JavaScript profile to the exact original occurrence. It advances
that retained cursor to `ReadRenderBlockingPending`, without inventing a false
script predicate or running deferred script-blocking sets. Render/media/context,
resource completion and later genuine synchronous history remain
independent original actions. Whole history stays Pending; captured-context
recovery remains unavailable. Native completion notifications use the same
explicit profile exclusion after actual event delivery and before real rendering
unblocking. Rust event listeners do not provide a JavaScript execution engine.

`process_parsed_style_rendering` continues that same original cursor. Absent,
empty and ASCII-whitespace media use HTML's classification; nontrivial media
returns `MediaEnvironment` without inventing a match. Original Document class
and body-presence facts govern rendering membership, even when the installed
final tree already has a body. Completion carries the actual DOM commit and
retirement custody plus a narrow membership witness. It creates no sheet or
preference effect and leaves the whole history and general Style readiness
Pending; resource completion and later original actions still remain.

Each public CSS preparation/pre-grammar/post-grammar poll charges the same
original contribution and Runtime work before publication. CSS owns its finite
UTF16 preparation and existing grammar limits. Private grammar work, temporary
allocation and fine-grained interruption latency are outside the consumer
bound; cancellation during grammar is observed at the next public poll.
Temporary parse reports transfer directly into CSSOM's private batch and its
logical quotas. Actual returned media diagnostic backing and source union,
consumer labels/names/output holders, replacement overlap and DOM publication
are admitted under their owning contracts. Source retention measures source
backing, not complete report/AST size. A retained-result refusal after parsing
publishes nothing and preserves spent work; retry polls and charges parsing
again. No unused URL/Base/media binder is introduced, and accepting a decision
does not acknowledge its contribution.
Meta change-preferred does not read those external context slots. Read-only
captures remain historical; standalone enable/select/change-preferred mutations
refuse an affected Document while its original contribution is pending.
`try_shutdown` returns the same Session with
`DomShutdownReason::ParsedAssociations(id)` until that custody is consumed;
creation refusal uses `DomSessionCreateFailure::Parsed(error)`. These variants
extend existing exhaustive enums, so exhaustive callers must handle the parsed
case. Retaining or structurally installing history does not by itself establish
association readiness.

## Start

From a prepared checkout with the [prerequisites](docs/getting-started.md#prerequisites),
prove that a reducer's task request becomes a runtime intent:

```sh
cargo test --offline --locked -j 1 -p surgeist-runtime --lib tests::runtime_forwards_task_work_as_intents_without_executing_it -- --exact
```

Expected: one passing test checking a forwarded task intent and no diagnostics.
[Getting started](docs/getting-started.md) explains the source and result.

## Documentation

- [Getting started](docs/getting-started.md): run and understand the first proof.
- [How-to](docs/how-to.md): emit intents, handle queues, render surfaces and build snapshots.
- [Reference](docs/reference.md): package facts, public entry points, defaults and checks.
- [Explanation](docs/explanation.md): ownership, state transitions, DOM orchestration and wake delivery.

### Ordinary native Meta preferred-name composition

`DomStyleRuntime::apply_meta` executes an explicit finite ordinary batch against
its existing Document stylesheet-set owners. It consumes only actual reached
nonempty default-style insertion occurrences, preserving UTF-16, invocation
order and existing or original batch-qualified allocation identity even when a
Meta is removed or destroyed later in that batch. Editing content/http-equiv alone does not
replay insertion. Zero-sheet history remains in the same Document preference
state and controls later real Style additions; explicit last-selected history
continues to govern the conditional flag algorithm.

The operation prepares every reached Document producer and optional inline
producer before publishing any owner. Runtime supplies stable external root
capabilities and admits their backing, names, temporary/output banks and old/new
retention under its own limits and the explicit `DomMetaAtomicLimits`. Initial
Runtime work is prepaid into that same native envelope before DOM preparation;
subsequent work charges both owners. No private producer graph tariff is added.
The final fallible DOM commit publishes the actual prepared producers and
canonical tail. Runtime installs already-qualified name rows while root guards
remain held, extracts authentic outputs, then fills only preallocated output
revision fields. Old Runtime names and CSSOM retirement backing survive that
whole fence. This does not claim bounded disposal for every existing DOM value.

The returned outcome owns the real canonical/root/optional-inline commits and
ordered preference observations. `DomStyleMetaError` preserves either the original
Runtime refusal or the native root-qualified producer error; these native errors
do not widen unrelated `DomStyleRuntimeError` results. Multiple operations on one Document report one
authentic final producer revision, never fabricated intermediate CSSOM commits.
Returned immutable observations are caller-owned history; current-bank storage
admission does not cap all observations a caller may retain.

Source and staged Style publication custody must both be idle for this ordinary
front.
Context-free DOM calls still refuse reached default-style without the owner.
The native capability does not consume pending parsed associations or waive
Runtime's intact-DOM Pending bootstrap refusal. Source continuation and public
Source-driver integration remain unavailable through this ordinary front.

### Source native Meta composition

`DomSourceServices.meta` explicitly selects finite native composition for the
existing Session Source driver. `None` preserves reached unavailable behavior;
it cannot discharge a parked native Meta publication. `Some` uses the same
Document preference owner and actual original operation, retaining its inline
profiles, native policy, cumulative work and pending publication across retries.
Native limits can increase without resetting original spent work or outputs.

The driver flushes the preceding actual Style receipt, prepays watcher settlement,
and borrows the operation's exclusive Meta hold before forming external root
banks. Every Runtime scan and allocation prepays the original Source, all active
ancestors and the native envelope. Temporary CSS source-collector scratch keeps
its existing distinction from persistent Runtime `storage_bytes`; its real
formation and old/replacement backing are additionally admitted by Source/native
limits before allocation. The exact persistent Runtime bank and new name/output
union are checked before the new banks form. Caller-owned immutable outputs have
the same output boundary as ordinary observations.

Each actual joined publication returns immediately as
`DomSourceDriveProgress::MetaSource`, including same-name or producer-unchanged
occurrences. It owns the authentic next request or completion, root commits,
optional inline evidence and ordered preference observations. Watcher settlement
finishes before any result is propagated. Complete finishes the actual Style
guard while retaining the whole outcome; Request is returned unserviced. A later
explicit resume processes later Style/policy/event work against the names already
installed. An advance without a Meta occurrence continues the existing service
flow and fabricates no native commit. `DomSourceDriveError` preserves either the
original service error or the native composition error at this driver boundary;
non-driving service APIs retain their existing errors.

## Stewardship

[MIT license](LICENSE) · [Attribution](NOTICE.md)
