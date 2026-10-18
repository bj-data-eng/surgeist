# Source continuations and Style ownership

Use Source when a canonical operation reaches an external service. The original continuation owns its receipts and mandatory tails; [Dialog methods](dialogs-and-focus.md) exercise event reentry and [hosted creation](hosted-documents.md) has its own controlled phases.

## Source edits and finite continuation admission

Native HTML Style retains its creation history and nullable CSSOM association.
Ordinary atomic edits refuse at an external processing boundary without
publishing a partial edit. `begin_source_edit` runs one `DomSourceEdit` through
the same canonical source engine and returns a lease for actual external
requests. Constructors take existing identities; callers sequence separate
operations explicitly. Native installation and Select completion retain their
own feature roots in that engine. Once a source phase publishes, dropping the
lease parks its mandatory continuation in the DOM; `resume_source_edit`
reacquires that exact operation. An acknowledged CSSOM publication survives a
failed later segment and is not reissued. Cancellation preserves an already
published phase and its remaining cleanup obligations.

Style captures select the current Document base through the Base owner. Hosted
Documents consume their actual constructor URL/about-base facts and maintained
first frozen Base; native closed Documents retain their established about:blank
fallback. A present but unfrozen first Base remains unavailable. The captured
base does not use a Settings creation-URL mirror or establish CSP permission.

`DomSourceLimits` bounds represented live bytes, UTF-16 backing capacity,
cumulative work, phase publications, output records, published native
identities and simultaneously outstanding nested operations. Storage includes
simultaneous private candidates, checkpoints, temporary banks and retained outputs.
Materialized payload fields are charged per retained occurrence; sharing a
persistent root does not materialize its records again. The
[storage explanation](storage.md) defines the ownership and charge table.
Allocator metadata and Arc control blocks are excluded.
Growth admits requested replacement storage while the old bank is live and
checks actual capacity afterwards. These are representation limits, not a
physical-memory ceiling. CSS and CSSOM keep their independent producer limits.

Before exposing an external phase, the source owner reserves checked revision,
invocation and association headroom for its mandatory current-command tail.
Finite work or storage refusal can preserve that tail for a raised-budget retry;
retry retains spent work and acknowledged effects. Independent Base, Meta,
Select and numerical limits remain separate and can be raised through
`set_limits` without resetting their retained usage. Changes to cumulative
OptionValue work and allocated-byte budgets must cover the amounts already
spent by the retained operation.

## Mutable CSSOM custody and disabled state

Style processing retains the actual mutable CSSOM store through
`DomStyleCssomRoot`, independently of an execution lease. Binding admits the
canonical root bank before external publication. `style_cssom_root` reissues
access to that exact live root's retained owner, returns `None` for a missing
binding and refuses an incomplete source tail or invalid root identity. Busy or
poisoned access preserves custody and any unattempted commit envelope. Root
retirement releases canonical custody; independently held access capabilities
may outlive it without pinning the tree. Snapshots preserve historical identities
but do not keep a mutable CSSOM owner alive.

`set_style_disabled` changes the existing associated sheet's flag through this
source protocol. A null-sheet assignment does nothing and cannot become a flag
for a later-created sheet. The operation preserves sheet identity and association
serial and queues no new load/error notification. Current CSSOM flags must be
read from the actual retained owner, independently of a historical association's
receipt revision; a flag read does not establish resource readiness. Pending
parsed Style processing remains explicit and cannot certify a null sheet.

## Standalone ownership transfer

The exclusive standalone `Dom` is `Send`, including a parked published source
tail, its acknowledged receipt and mutable CSSOM custody. Transfer moves the
whole owner; it does not reconstruct identity from a snapshot or authorize
concurrent writers. Private numerical accounting uses shared atomics while
owner-local cells remain, so this contract does not provide `Sync`. Runtime owns
its separate session, resource/task lifecycle and controlled event reentry.

## Dispatch target custody

A started dispatcher registers its actual Node invocation targets through
`register_dispatch_custody`. Configure aggregate
`DomDispatchCustodyLimits` with `DomLimits::with_dispatch_custody`; the default
has zero capacity. Custody retains exact target generations and records their
actual staged node-document only when retirement successfully publishes,
including adoption and destruction in one edit. It retains no mutable node or
tree. Live targets use their current association; a retirement stamp grants
neither mutation nor current Document/Global execution authority. Dropping the
affine scope releases its rows, while the bounded weak index charges remaining
headers until reclamation. This mechanism is independent of snapshots and
change history and preserves the standalone owner's `Send` contract.

[Back to surgeist-dom](../README.md).
