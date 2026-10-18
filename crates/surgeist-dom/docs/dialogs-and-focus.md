# Dialogs, focusing and top-layer state

Dialog methods retain the actual HTML-owned continuation across event dispatch, focus observations and watcher services. Read [Source and Style](source-and-style.md) for shared admission and custody, and [fullscreen](fullscreen.md) for the other canonical top-layer consumer.

## Dialog state and Show

Native HTML Dialog exposes retained `returnValue` through `DomNode::as_dialog`.
`set_dialog_return_value` preserves exact UTF-16 and old snapshots; equal values
are no-ops. Native clone/import starts with the source-defined fresh return value,
while adoption preserves the existing value. Ordinary `open` attribute and tree
hooks consume HTML's rules against the actual Document environment and canonical
Document lists. `DomDialog::close_watcher` observes the stored optional watcher
identity; historical snapshots retain their captured association. Runtime owns
the actual Window manager and its watcher objects. Ordinary atomic edits refuse
a reached watcher service with `NeedsEventfulOperation` before publication.
Tracker and modal state retain their own source-defined rules. Request-close
readbacks retain the actual nullable request result/source and force-enable flag
at the snapshot's revision; they grant no current invocation authority.

`DomSourceEdit::show_dialog` starts the shared HTML Show cursor and can expose
its actual cancelable beforetoggle request. A bound event executor acknowledges
only a completed dispatch. Its scoped permission permits callback edits and
nested Source operations; after dispatch the original cursor resumes against
fresh canonical state, preserving those edits. The initial event observation
has no unpublished Draft and does not mark the snapshot incomplete. Dropping
its lease retains the exposed request even before the first tree publication.
Each outstanding ancestor constrains real nested work and retained storage.
Publication, identity and output counters remain owned by each original
operation; callback edits preserve its remaining mandatory publication revisions
without pretending that it published those edits.
If a pending child exhausts an observing ancestor's allowance, retain that
ancestor's issued event request and use `raise_source_event_limits` to raise
its policy monotonically. This does not execute the parent or replace the top
child. Spent work and accepted receipts survive the update; ordinary top resume
still performs full admission for policy reductions and preserves the required
closing phase. Canonical `set_limits` refuses while an event observation or its
retained ancestor is outstanding.
Task transfers, notification completion and other atomic controllers whose
outputs have not joined this admission retain their strict Source-idle guard.
They preserve pending obligations for later service.

Canceling beforetoggle is a normal HTML return, distinct from canceling the
Source operation. If the callback retires the receiver, genuine event
cancellation completes without reading it again. An uncanceled post-event read
instead reports the original receiver's retirement. QueueToggle publishes the
real task and canonical tracker through the original bound DOM-manipulation
owner. Its exact publication is accepted once; refusal retains it for retry
without queueing again. Show adds `open=""` through the ordinary attribute hooks
and publishes the reached prefix. An inactive or disconnected receiver then
reaches RememberFocusedElement. A connected, fully active receiver enters its
Document's open-dialog list and requests EstablishCloseWatcher. Runtime's Source
driver registers the actual watcher and accepts its exact result through the
original Source receipt; Show then requests RememberFocusedElement. Direct
Source attribute and tree operations use the same hook and retained suffix.
Without the actual service, the request remains pending. Registration uses the
Dialog's original relevant Global, independently of its current node-Document;
adoption does not substitute a new Window.

## Entry scope and remembered focus

Each Source operation retains its actual entry scope outside the mutable Draft.
Hosted entries preserve their original Global and optional Host association;
they cannot fall back to native binding when that association is unavailable.
A native entry may receive one explicit Host binding from the original trusted
issuer before its first advance. Advancing without one seals it unbound, and
resume cannot change that choice. Nested roots select independently. The
read-only `DomSourceExecutionEntryScope` observation grants no execution or
publication authority.

At RememberFocusedElement, the original trusted Host producer prepares an
operation/action/occurrence-qualified observation against the actual canonical
basis and original live Host identity. Runtime's sealed headless Host can attest known
absence; missing or unsupported observations leave the request pending. Accepted
absence writes null to the shared common HTML state and acknowledges the cursor
once. Retained publication custody survives
refusal and retry without sampling again. An already-null slot remains a
semantic no-op. No current positive focused-Element producer or arbitrary DOM
focus setter is exposed. Inactivity, no Global and surface selection alone
cannot establish absence. The headless proof covers the complete admitted Host
lifetime, so unrelated issuer progress does not invalidate it. Actual Host
retirement/reuse, disposed authority or a stale canonical basis still refuses.
Mutable positive-focus observations are not admitted by this Host profile.

Show then captures the receiver's current node-Document in its original
continuation. The ancestry query reads that Document's qualified raw top layer
and the members' actual popover visibility. The admitted common state proves
hidden, so both Auto and Hint showing lists are empty and the endpoint is null.
Hide rechecks the captured Document and that proof before advancing to
RunDialogFocusingSteps. Nonpopover top-layer entries do not prevent it.
Unavailable history or unqualified nonhidden state leaves the corresponding
action pending. These checks use
canonical Document state independently of the Host's headless focus proof;
they perform no popover dispatch or focus movement.

## Focusing and autofocus

At RunDialogFocusingSteps, the original method retains the complete HTML-owned
focusing child cursor. `DomSourceRequest::DialogFocusing` exposes its exact
unfulfilled action and original invocation. The allow-focus decision reads the
receiver's then-current Document. It checks browsing-context presence and full activity
before consulting that Document's actual focus-without-user-activation policy.
A denied feature then requires the actual current Window's activation state.
Runtime's constructor-owned never-activated state supplies false activation,
so the shared HTML rule returns before focus-target selection or focus effects.
The original affine receipt fulfills this exact child permission step. Denial
completes the child and then the outer Show or ShowModal. A permitted feature
advances the child locally before reading activation or forming a Runtime receipt.

Actual `autofocus` selects the subject. Without it, the original focusing frame
runs the HTML-owned delegate protocols with two separate ordinary-descendant
passes. An admitted indexed ancestor stack preserves preorder with linear work
and storage proportional to depth. Actual shadow-root selection follows the
producer; ordinary traversal does not enter shadow or hosted relations implicitly.
An empty descendant domain genuinely completes both scans and selects the subject.
Unknown focusability or called-area resolution retains a `FocusDelegate` request
with its actual root, queried node, step and original operation occurrence.
Checkpoint refusal, cancellation and lease replacement preserve the original
traversal and cumulative admission; they do not supply missing facts.

The native adapter can fulfill called
focusing without changing focus when the subject has applicable own `inert`
and actual non-root, nondelegating state. The shared HTML rule covers the
subject's own scroll regions; it supplies no geometry or guessed focusability.
The child then reads the selected control's actual current top Document and
compares origins. A genuine origin difference completes it before autofocus
writes. For the same origin, the child clears that captured top Document's
actual candidates, sets its processed flag and completes. Missing no-change proof
waits at `RunFocusingSteps`. Positive focus movement and restoration remain unavailable.

Each Document retains independently qualified autofocus history. Fresh native
and hosted Documents begin with an empty candidate list and processed=false;
parsed interaction provenance alone does not establish that history.
`DomDocument::autofocus()` reads the candidates, processed flag and snapshot
stamp from a complete Source basis, with an explicit error for unavailable
history. It grants no execution authority over a historical candidate.

Actual ordinary Document-tree insertion consumes the HTML insertion cursor,
qualifying the Element's HTML/SVG/MathML namespace and exact autofocus attribute,
then checking full activity, sandbox automatic features and
allow-focus in source order before resolving the actual top Document. If its
processed flag is false, the exact candidate is removed and appended. The native
owner elects to continue past the standard's optional user-preference return.
No focusability check or attribute-only enqueue is added. Shadow-connectedness
alone is insufficient. Ordinary detach, adoption, attribute removal and inactivity
preserve candidate history; distinct Documents can retain the same candidate.
Rendering-time flushing remains unavailable.

Use `DomLimits::with_autofocus_candidates` for a finite aggregate installed
candidate allowance; the default is zero. `DomUsage::autofocus_candidates()`
reports actual references across Documents. Source separately admits actual
backing capacities, checkpoint overlap and cumulative work. Successful explicit
Element destruction removes its exact identity from every surviving Document's
candidate list in the same publication, preserving processed flags and survivor
order. Failed unpublished destruction changes none of those lists; historical
snapshots retain their original identities without authorizing live access.

The shared `DomFocusRequest` distinguishes remembered Dialog focus, the Dialog
child's allow-focus action and an actual autofocus insertion allow-focus action.
`prepare_focus` and `accept_focus` use the same original affine observation
and receipt owner for all three. An insertion never impersonates a Dialog call.

An explicitly retired hosted Document has no browsing context, so feature
permission is denied without evaluating policy. Its historical original Window
can still supply its actual activation state while retained in watcher custody.
This historical path remains distinct from the live current-Document qualification.
The receiver's original Global still owns any previously queued task; adoption
does not substitute it for the current Document's Window.

A missing or pruned Window supplies no activation observation. Native Documents
without a Global and branches requiring an unknown permission or actual focus
movement retain their service wait. This negative branch provides no arbitrary
focus setter or positive focus implementation.

Open removal and source-required tree cleanup destroy the actual watcher before
clearing the canonical association. This cleanup remains available after hosted
execution retirement, which leaves the surviving Dialog and its original Window
custody intact. Retries retain the original receipt without repeating manager
effects. Queued delivery cannot bypass an unfinished synchronous tail.
Show completion requires the actual producer Complete branch, including the
proven negative allow-focus path above.

## Close and requestClose

`DomSourceEdit::close_dialog(receiver, result)` represents the standard
close method with its actual null source. An omitted result preserves returnValue;
a supplied empty string writes empty. The original HTML cursor orders authentic
noncancelable beforetoggle, the postcallback open read, toggle publication,
complete open-removal/watcher hooks and result/request-state updates. A surviving
callback that removes open supplies the genuine early return. Known previous-focus
absence reaches actual final Close task publication. Modal Close requests deferred
top-layer removal and clears the modal flag; unsupported focus restoration
retains its precise action wait. Method Complete requires the
real QueueClose acknowledgment.

`DomSourceEdit::request_close_dialog(receiver, result)` represents standard
requestClose with its null source. Closed, disconnected or inactive receivers
return at their actual source guards. Otherwise the original Source continuation
captures the real watcher, enables it for the request and writes the nullable
request operands before invoking the watcher. The watcher fires an actual
cancelable plain `cancel` event. A normal watcher result always precedes the
literal final force=false write, including genuine cancellation. Cancellation
does not clear the request slots or undo callback edits.

Source retains both HTML cursors and invokes the existing Close frame when the
watcher's current guards permit it. At that call it captures the then-current
request operands, so reentrant requests can affect the supplied Close result.
A reentrant request which observes running cancel returns without running another
cancel action, then performs its own literal force=false. The outer request's
fresh enabled check can consequently skip Close. Omitted and supplied-empty
request values remain distinct throughout.

For a modal Dialog, computed Auto still enables Close Request after that force
reset, while explicit `closedby="none"` can stop the outer Close. Genuine Cancel
prevention preserves modal state; an uncanceled nested Close performs the same
deferred top-layer removal and independent notification queuing described above.

Runtime retains each started invocation's exact original watcher record after
manager membership removal. Guard reset, reached activation consumption and
invocation settlement complete through action-qualified affine receipts. Lease
drop, cancellation or admission refusal preserves the original mandatory tail.
A retired receiver cannot receive the final force write: the owner retains its
authentic stopped action and any real cleanup, exposes `PendingCleanup` with a
typed cause while settlement remains, then clears the canonical incomplete-source
marker and returns the native failure. A surviving receiver receives final false.
Neither path fabricates a normal nested Close return or event receipt.

## ShowModal and canonical top layer

`DomSourceEdit::show_modal_dialog(receiver)` runs the standard null-source
ShowModal prefix. Initial invalid-state checks precede real cancelable
beforetoggle; after an uncanceled event it checks current open, connection and
popover-showing state without repeating the activity guard. Genuine canceled
and post-event early-return branches can complete. The positive prefix queues
the actual toggle and adds open through its complete ordinary hooks. After the
valid watcher assertion, one callback-free canonical phase sets the modal flag,
establishes Document blocking through raw membership, and adds the Dialog to the
top layer if it is absent. That phase publishes before the remembered-focus
observation. An already-raw Dialog takes the source's literal skip-Add branch,
including when pending removal. Supported remembered-focus and empty-popover
branches reach `RunDialogFocusingSteps`. Proven allow-focus denial can complete
the method without selecting or moving focus. Unknown inputs or positive
focusing retain the genuine wait and incomplete Source marker; structural modal
publication alone does not certify completion.

`DomDocument::top_layer()` borrows immutable raw membership, pending removals
and the Document stamp. Its allocation-free `ua_rules()` projection emits the
universal UA-important `overlay:none` baseline, followed by exact-Element
UA-important `overlay:auto` inputs in raw order for members not pending removal.
These are canonical producer declarations for downstream Style collection and
cascade. They do not establish computed overlay values or rendering. The getter
requires a complete Source basis and qualified Document history; an unfinished
ShowModal therefore cannot use it or semantic capture to bypass that guard.
An earlier complete snapshot retains its historical lists and inputs.

Raw membership and ordered pending removals share the canonical Document owner.
Requesting removal retains raw membership and removes the dynamic auto input;
re-adding a pending member removes both entries before appending it at the top.
Immediate source removal clears both lists and the input. Ordinary removal,
adoption and destruction use the same cleanup against the old Document; fresh
clone/import initializes independent nonmodal state. Manually removing `open`
does not stand in for Close and can leave modal/raw blocking state. Actual
rendering-checkpoint removal still requires separately qualified rendering and
computed-style evidence, which this surface does not admit.

The watcher assertion observes the published canonical prefix. If a callback
adopted the receiver into a connected inactive Document, its completed open hook
can leave the watcher null. Proven null yields `SourceInvariantFailed`, carrying
the exact receiver, ShowModal action, observed stamp and
`MissingDialogCloseWatcher` reason. It preserves real open, queued work and
callback effects. Incurred cleanup retains the same failure through
`PendingCleanup`; terminal release returns that failure rather than retrying
the assertion after later adoption. A new invocation observes retained open
nonmodal state and follows the initial InvalidState branch. Missing or unfinished
setup, stale facts, inconsistent watcher identity and receiver retirement remain
distinct from this proven invariant failure.

Started ShowModal beforetoggle dispatch retains its real path and receipt through
receiver retirement. Actual cancellation can complete normally; an uncanceled
event instead fails at the required ReadOpen. Later prefix failures preserve
published effects and settle only their incurred original-owner work.

## Qualified dispatch reads

The `dispatch_relevant_global`, `dispatch_global_document` and
`dispatch_document_fully_active` fronts qualify current hosted reads during an
actual active Source event scope. They validate the original executor and
published observer chain, charge its Source work and preserve the incomplete
marker. Outside that scope they require a completed Source basis; an unfinished
nonobserving operation cannot use them to bypass ordinary completeness guards.

## Notification tasks

Dialog toggles use the DOM-manipulation task source and its tracker. Close
notifications are separate user-interaction tasks, with no toggle tracker or
coalescing. Runtime's one element-task owner selects the actual source explicitly;
both preserve the queued target and original Global/Document association.
Successful Toggle dispatch clears the surviving Dialog's current tracker,
including a replacement installed by a callback, without canceling that queued
replacement. `DomSourceCompletion::dialog_toggle_disposition` reports
`TrackerCleared` or `TargetRetiredDuringDispatch` from the actual cleanup branch.
Cancellation before dispatch and other Source operations return no Toggle
disposition; a serviced Toggle never substitutes absence for its outcome.
Close dispatch does not clear the tracker or repeat method effects. Real receiver
retirement during beforetoggle preserves the started dispatcher and authentic
receipt, then yields `SourceReceiverRetired` at ReadOpen rather than a synthetic
absent-open return. Internal Close callers with nonnull source, complete modal execution,
positive focusing, platform close requests and general popover
processing are not admitted by this surface.

[Back to surgeist-dom](../README.md).
