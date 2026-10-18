# Fullscreen model and exit continuation

Fullscreen shares raw and pending top-layer state with [Dialogs](dialogs-and-focus.md#showmodal-and-canonical-top-layer). Model state, synchronous exit and Runtime-owned parallel completion have separate qualifications.

## Structural model and getters

`DomBatch::fullscreen_element_model`, `unfullscreen_element_model` and
`unfullscreen_document_model`, with matching `DomSourceEdit` constructors,
implement the [native Fullscreen model](../../../references/fullscreen--7c38d773117aa1e6bfa13754afe77483f40c908f--7d8cd68e63c0.md).
Every Element namespace has initially unset fullscreen and keyboard-lock flags;
only an actual HTML Iframe has its separate iframe fullscreen flag. Immutable
Element getters expose those values. DOM clone/import initializes fresh flags,
while snapshots and adoption preserve current state.

The model shares the canonical raw/pending top layer and UA-rule producer.
The current closed HTML common state proves empty showing Auto/Hint lists when
the Document's list history is qualified. Fullscreening then sets the flag,
removes membership immediately and appends at the top; repeating it reorders the
member. Unfullscreening clears all applicable flags and removes membership.
Document unfullscreening visits every flagged raw member. Unknown history
remains unavailable. These helpers admit detached Elements and implement neither
`requestFullscreen` eligibility nor platform execution or event delivery.

`DomDocument::fullscreen_model_element`, legacy `fullscreen`,
`is_simple_fullscreen_document` and `keyboard_lock_active` derive from flagged
raw membership. `DomDocument::fullscreen_element` and
`DomShadowRoot::fullscreen_element` instead apply the standard retargeting and
ordinary-tree checks, including closed shadows and disconnected hosts. Detached
raw membership can therefore make legacy `fullscreen` true while the standard
getter returns null. These views require a complete Source basis. Flag changes
produce `ElementState` and derived `DocumentFullscreenState` evidence, including
when raw membership itself is unchanged.

## Removal and unloading cleanup

Ordinary removal visits the static flagged shadow-inclusive list in source
order. The current fullscreen element invokes exit; other flagged entries
unfullscreen. Inactive exit rejects before clearing its flags, after which the
removing steps still remove lingering membership. Active atomic removal refuses
the eventful boundary without publication; Source can retain the actual parallel
exit described below. Cross-Document adoption refuses residual old-Document raw or pending
membership that its actual source path cannot clean. The check covers ordinary
and shadow descendants and separately adopted Template contents, including
insertion/replacement. Same-Document adoption and flags without membership do
not trigger that refusal. Explicit model unfullscreening can permit a later
adoption; adoption does not silently add that operation.

Hosted Document destruction runs unloading cleanup even when unload events are
suppressed. Before retiring the Document or clearing history, its original
cleanup owner requires a null raw-derived fullscreen element. A non-null result
retains `FullyExitFullscreen` and the same queued cleanup for retry; flag-only
Elements outside raw membership do not block it. Actual unfullscreening permits
the queued destruction to continue. General full-exit, platform, orientation
and fullscreen event delivery are not admitted by the structural model.

## Exit and pending parallel work

`DomSourceEdit::exit_fullscreen` implements the synchronous exit prefix and its
native parallel handoff. Configure finite `DomLimits::with_fullscreen_exits`
and `with_fullscreen_events` allowances before admitting background work; both
default to zero. Runtime also reserves its actual job under its existing task,
storage, scratch and work limits. A source-defined inactive or null-fullscreen
branch returns `DomFullscreenExitReturn::RejectedTypeError` locally, without
requiring a background slot. Missing qualified facts remain unavailable.

The active path privately collects actual ancestor Documents, selects the
source's Document and resize decision, and queues a change tuple before
unfullscreening a disconnected current Element. A known-null ancestor violates
the source assertion; the model's ability to flag a child alone does not invent
ancestor flags. The prepared handoff secures the same DOM-owned invocation and
real Runtime scheduling custody before publishing the prefix. Failure before
publication leaves no prefix or runnable job.

`DomSourceCompletion::fullscreen_exit_return` exposes the immutable native
return. `Pending(DomFullscreenExitId)` finishes the synchronous call while the
parallel exit remains unfinished. `Dom::fullscreen_exit_status` provides a fresh
exact-identity observation separately. Neither observation authorizes execution
or acknowledges an external action. Result or executor-handle drop parks the
same invocation. Nested removal continues its real synchronous tail after the
pending return, preserving any enclosing Source exclusion until its own boundary.

Runtime's scheduler drives the parallel invocation to its actual
`FullyUnlockScreenOrientation` requirement and parks it without retaining an
ordinary Source guard or current WindowLoop task. Independent edits and real
History checkpoints remain available. `FullscreenExitPending` reports the first
reached wait; `Quiescent` distinguishes retained passive work from `Idle`'s
no-remaining-obligation result. No accepted orientation or normal-viewport
service exists yet, so a later flag change cannot complete or restart the exit.
Orientation and conditional viewport restoration must precede the later fresh
fullscreen read. The pending result supplies no platform completion.

## Queued event records

`DomDocument::pending_fullscreen_events` observes its canonical ordered tuples.
The real producer deduplicates event kind and exact subject generation in source
order. `Dom::take_fullscreen_events` preadmits the complete finite immutable
batch before emptying the original queue, preserving old snapshots and recorded
Document/subject identities. These records establish queued work; they do not
establish dispatch or repeatable delivery permission. Subject destruction does
not erase the recorded event. Hosted `DocumentPhase::Retired`, inactivity or
service-owner loss is distinct from canonical Document identity retirement;
the native owner currently exposes no Document identity-retirement ingress.

[Back to surgeist-dom](../README.md).
