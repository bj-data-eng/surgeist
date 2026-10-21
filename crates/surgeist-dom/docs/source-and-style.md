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

The native Style cursor intentionally selects HTML's
`NoExecutableJavaScript` execution profile. At its script-bookkeeping action it
uses the specific exclusion reply for both the normative executable-script
predicate and associated sets; parser provenance and media are not converted
into false predicate results. The next genuine render action still uses its
own Document/body/blocking/media facts. Native and Runtime notification cursors
use the same profile distinction while preserving prior event dispatch and
actual later render unblocking. This does not implement executable JavaScript.
The [original parsed Style consumer](html-installation.md#explicit-exclusion-of-executable-script-bookkeeping)
binds this disposition to its exact retained occurrence rather than replaying
native hooks. A pending Source's request can expose its real acknowledged sheet
association while the general snapshot sheet getter remains unavailable until
the continuation completes.

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

Name atoms and index reservations already owned by the continuation survive
that retry. Fresh containing attribute banks and child-custody capsules still
require admission under the name-local policy. A refusal preserves the original
cursor, receipts, spent work and published effects; increasing the relevant
limit resumes that cursor without repeating acknowledged requests or effects.
Committing an already prepared segment requires no further name allocation.

Native completion tokens expose `NativeSource` provenance; original parsed
permission exposes `ParsedOccurrence` and retains its original admission. The
[original queue contract](html-installation.md#original-notification-permission-and-the-synchronous-frontier)
keeps task custody separate from its still-unavailable synchronous dispatch
frontier. An original pending history is neither a fake native Source operation
nor a blanket prohibition on unrelated asynchronous resource waits. The
[ParserPopped successor](html-installation.md#the-parserpopped-successor-and-recreation)
removes the genuine old association, or executes authenticated known-null
removal after a CSP refusal, and reaches a fresh policy request. Its independently
allowed recreation uses the same creation owner and serial lineage without
requiring old tasks to drain or replacing their original qualification.

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

An idle registered root can also release canonical custody at its original
[hosted Document destruction cleanup](hosted-documents.md#activity-children-and-lifecycle-retirement).
`DomHostedDestructionLease::prepare_style_root_withdrawal` authenticates that
cleanup and issuer, checks the actual locked producer and the Document's
canonical associations and obligations, and prepares the custody change.
Another registered root for the same Document still prevents this withdrawal.
The composing adapter must separately establish that its own tasks, resources
and publications no longer need the selected mutable capability.

Preparation retains the DOM and issuer loans and producer lock. Dropping it
leaves custody unchanged. `publish()` removes canonical custody; `finish()`
releases the lock after the adapter's preformed, infallible resource move.
Withdrawal changes no tree or CSSOM revision and acknowledges no destruction:
the same cleanup publication must still complete normally. The result's
snapshot records withdrawal-time history; independently escaped mutable
capabilities retain their own lifetime and may subsequently change that producer.

`set_style_disabled` changes the existing associated sheet's flag through this
source protocol. A null-sheet assignment does nothing and cannot become a flag
for a later-created sheet. The operation preserves sheet identity and association
serial and queues no new load/error notification. Current CSSOM flags must be
read from the actual retained owner, independently of a historical association's
receipt revision; a flag read does not establish resource readiness. Pending
parsed Style processing remains explicit. A complete original no-effect history
can establish a null sheet during installation; unresolved processing cannot.

## Native Meta composition

An association adapter calls `DomSourceLease::begin_meta_runtime` before forming
its native Meta banks. The returned exclusive hold charges the original Source,
every actual enclosing observer operation and the native envelope before the
adapter performs controlled work or retains backing. `step` admits work;
`admit_runtime` declares the whole simultaneous Runtime holder/output union.
Repeated reports retain their high-water rather than adding the same backing
again. Native limits may be raised componentwise on retry; spent work and output
admission remain with the original operation. This front does not replace the
independent Runtime, Source, Meta, inline or CSSOM limits.

The hold's one `prepare` seals further prefix admission before advancing the
original continuation. With no reached native Meta occurrence it returns the
authentic existing Source progress and optional inline publication. Otherwise it
lends `DomMetaSourcePreparation` over a phase already retained by the original
operation. Its ordered occurrences preserve exact UTF16 and genuine historical
subjects. The adapter reads held root snapshots, stages real flag edits and
prepares its own preference history under the same admission. Actual accepted
Style receipt authority is validated under one retained root lock, including
when that root also receives a Meta edit.

`commit` prepares every actual CSSOM owner, optional inline ingestion, original
Source publication and output before publishing any owner. The resulting
`DomMetaSourcePublished` retains the root locks while the adapter installs its
prepared history. Consuming `into_parts` unlocks and returns the genuine Source
progress, actual root commits, optional inline evidence and retirement backing.
`DomMetaSourceCommit::stamp` identifies the actual resulting DOM publication even
when its next progress is a request. A receipt-only root receives no synthetic
commit. Existing event, parent, Select, hosted and other mandatory tails retain
their original permissions and continuation.

Dropping a preparation publishes nothing and leaves its original pending phase
available for a raised-budget retry. A changed producer basis refuses that retry;
it does not silently rebuild the invocation. A context-free advance cannot skip
the pending native phase. Forgetting a hold prevents reacquisition; forgetting a
preparation seals all further prefix/preparation calls and retains its actual
graph, source debit and work dependencies together. `request_cancel` can still
mark the original operation. Runtime backing must be disposed before its hold
releases the corresponding admission. The capability does not make an external
processor available, resolve pending parsed associations or implement Meta CSP.

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
