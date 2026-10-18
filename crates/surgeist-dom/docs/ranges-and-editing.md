# Live Ranges and designMode

Ranges, their Document association and designMode share the canonical tree owner. Reached focusing uses the [Source continuation](source-and-style.md) and [focus contracts](dialogs-and-focus.md#focusing-and-autofocus).

## Range identity and active association

Native live Ranges share the canonical owner, staged publication and revision.
Select finite retained-slot capacity with `DomLimits::with_ranges`; its default
is zero. Vacant and generation-exhausted slots count toward that capacity.
`create_range` starts both boundaries at the actual Document's offset zero
and resolves its batch ticket only after successful publication. Start/end
setters use exact UTF-16 offsets for Text, Comment and PI, ordinary child counts
for other admitted containers, and the DOM-prescribed opposite-endpoint
collapse. DocumentType cannot be a boundary container.

Each controlled native or initial-hosted Document starts with a known empty
native active-Range association. Unavailable imported state remains explicit.
`set_active_range` selects, replaces or clears that one association; initial
association requires the Range's ordinary root to be that Document. Detached,
fragment and ShadowRoot roots are initially ineligible even with the same
node-document. Later valid boundary setters and adoption preserve the same
association and Range identity across root or node-document changes.

Pre-removal, full-count insertion and CharacterData replacement maintain the
actual live boundaries at the shared mutation steps, including equal-text
replacement. Explicit native `retire_range` frees its live record and clears
every exact matching Document association while retaining boundary nodes.
Container destruction first runs its real removal maintenance, then retires
Ranges whose current boundary container is actually destroyed. Detach and
adoption preserve surviving identities. Retired identities never alias reused
generations; old snapshots retain their historical boundaries and associations.
Disassociation and copied-handle drop do not retire a Range.

Ordinary batches keep tree, Range and association changes atomic. Source edits
use that same owner and retain valid boundaries at every visible publication.
A multi-child insertion whose prescribed aggregate offset shift would make an
intermediate external publication invalid refuses with
`RangePublicationUnavailable` before any prefix or request is exposed.
Safe intermediate publications retain their actual Source-completeness qualification.
This bounded native association is distinct from complete browser Selection
bindings, selection events, editing and focus execution.

## designMode state and setter

`DomDocumentMetadata::design_mode_enabled` reads canonical Document state.
Native creators establish false; completed-report installation transfers each
owned ordinary, auxiliary and inert Document's original HTML-qualified state.
Captured existing Documents preserve their current state. Initial and Slot
semantic contexts feed these actual Document rows to the shared editability
kernel. The explicit subject/work limits also bound this fact formation and its
producer validation independently per formation. No cumulative producer budget
or external spellcheck default is inferred. Explicit spellcheck attributes can
answer; policy-dependent defaults remain missing until supplied by their owner.
`DomBatch::set_design_mode` and `DomSourceEdit::set_design_mode` execute the
HTML-owned setter against this same state. Keywords are ASCII case-insensitive
without trimming. A false-to-on transition enables the Document, reads its actual
active association, resets that same Range to `(Document, 0)` at both endpoints,
then reads the current document element for the prescribed focusing call. A known
null association skips only the reset; unavailable association remains a precise
requirement. Off, invalid input and already-on calls do not reset or focus.

A null document element permits local completion in one final publication.
`DomSourceCompletion::design_mode_range_disposition` reports `NoActiveRange` or
`RangeReset(id)` for the completed transition; other setter branches return
`None`. Ordinary batches refuse a reached unavailable dependency atomically.
Source publishes the reached enabled/reset prefix and retains the original
method, exact request occurrence and completed local work across lease drop,
cancellation and reacquisition. A final admission refusal retains the completed
method result. `DocumentDesignMode` records actual flag changes independently
from `RangeBoundary` changes, and historical snapshots retain their old values.

A present document element currently reaches the genuine generic focusing
eligibility request. `DomSourceRequest::DesignMode` identifies that action or
unavailable active-range access; it supplies no acknowledgment or fabricated
focus result. Positive focus eligibility, movement and reentry require their
downstream owners and remain outside this admitted completion surface. Canonical
enabled-state readback alone does not certify completion of a pending Source
setter.

[Back to surgeist-dom](../README.md).
