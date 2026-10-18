# Details groups and toggle custody

Details changes retain ordered transition intentions. Runtime owns task coalescing and dispatch; the [Source](source-and-style.md) and [Dialog](dialogs-and-focus.md#notification-tasks) contracts describe the related execution boundaries.

## Group mutation and finite intentions

Native HTML Details derives open state solely from exact null-namespace `open`
presence. Generic name/open changes and per-node insertion consume HTML's shared
Details rules. Groups use equal nonempty exact UTF16 names in the same ordinary
tree, including detached trees and Template contents; distinct shadow trees stay
separate. Opening first records the initiating transition, then removes the first
open peer's attribute through the real mutation engine. Name changes and insertion
instead close the given Details. Detach/adopt preserve identities and pending
intentions; clone/import and native immutable installation run fresh copied
attribute/insertion hooks in their actual order.

`DomLimits::with_details_toggle_intentions` admits a concrete number of raw,
ordered, duplicate-preserving transition records. The default is zero; constructor
and closed-state edits need no intention capacity. Current-batch retirement
evidence shares that count so destroying a recipient cannot reset admission.
Group scans use the finite canonical node/attribute/string domains, poll each
visit/comparison, and allocate no traversal bank. Canonical strings retain their
existing unit accounting; fixed intention records and transfer output use their
separate count limits and fallible exact reservations.

## Transfer, retirement and Runtime custody

`take_details_toggle_intentions(max_output, cancel)` admits and builds the whole
output before atomically relinquishing DOM ownership. Its opaque owned transfer
and intention records cannot be cloned or publicly reconstructed; borrowed
inspection permits a consumer to plan admission before consuming the transfer. Every published transition has an opaque owner-qualified checked
operation identity, preserved through handoff and cancellation. Private creations
destroyed before publication consume no identity. Queue-only transitions and
draining have explicit `DetailsToggleIntentions` change coverage and revisions;
failed or cancelled staging preserves attributes, order and identities together.
Explicit destruction cancels only still-owned intentions for exact retired
targets, with finite `TargetDestroyed` evidence carrying the target, operation
identity and states. The transfer batch retains its historical source snapshot;
consuming iteration yields records retaining their operation, target and states.
A consumer must validate each exact target against a fresh current snapshot before
execution. DOM does not coalesce tasks or fire ToggleEvents. Runtime's shared
`DomSession` owns the shared DOM manipulation task bank, concrete Details tracker,
coalescing and native event dispatch. Its controlled ingestion admits the actual
pending bank before taking custody; a session-bound caller cannot bypass that
handoff with a raw transfer.
Tasks retain their original queued Document and hosted Global across adoption,
while dispatch uses the current tree's parent path. Standalone DOM consumers can
still use the raw transfer API; prior nonempty external transfers prevent later
session bootstrap because that external custody cannot be recovered.

[Back to surgeist-dom](../README.md).
