# Persistent storage and controlled retention

DOM keeps one immutable canonical State and one exclusive mutation owner.
Snapshots share persistent storage; transactions retain changed records and
reversible mutations. Publication prepares the candidate, admission, identity
mapping and change records before replacing the canonical root. Source uses
the same transaction machinery across its admitted phases.

The private [bank implementation](../src/store/bank.rs) stores indexed records
and vacant-slot summaries in persistent pages. Copying a root handle does not
copy its records. A local replacement allocates the affected index paths and
the replacement record. Selected child sequences still require work for the
sequence actually changed; unrelated sequences remain shared. Range and other
dependency indices identify the affected participants without enumerating every
registered object.

## What the admission budget counts

The retained-byte budget is a conservative representation budget, rather than
a count of unique allocator bytes. Each materialized payload field incurs its
established capacity-based charge, even when two fields share backing. UTF-16
backing capacity also contributes retained units. Allocator metadata and Arc
control blocks remain excluded.

Canonical sequences with zero capacity have no shared backing or out-of-line
charge. A reserved empty sequence retains its actual buffer and Vec header;
clones keep that backing. Sequence contents are borrowed as slices, and the
first write prepares its admitted replacement before changing the selected field.

The [custody primitives](../src/store/custody.rs) attach an admitted debit to
controlled ownership. Shared receipts represent aliases of the same materialized
record or page. Their last controlled owner releases that debit. Immutable
semantic values carry private materialization identities and cached descriptors;
they carry no DOM admission lease. Identities do not confer public DOM identity
or execution authority. Hosted producers separately retain their original
allocation receipts for the lifetime of their semantic allocations, including
pages shared by snapshots. These receipts grant no host execution authority.

| Retained object | Charge and lifetime |
| --- | --- |
| Canonical State | Its body and retained queue-field backing occurrences. Absent queues add no out-of-line storage. Bank records, pages and auxiliary graphs have their own custody. |
| Persistent page or record | Actual new page storage, or record body and its payload-field occurrences, admitted before controlled retention. Shared descendants keep their existing debit. |
| Canonical names | Owner-qualified immutable records with weak index entries. Strong tokens retain text independently of the owner; admitted objects retain the corresponding child custody. |
| Ordered attributes | One immutable DOM-owned typed payload and actual attribute-array capacity. Scalar-only record changes share that payload; text occurrences and name-record child custody remain distinct charges. Fresh empty attributes have no bank allocation. |
| Retained Form history | One typed payload body, actual entry capacity and conservative name-field occurrences. Known-empty history has no payload. Slot controls own its child custody; another Slot or staged alias of that payload does not charge its contents again. |
| Additional bank-root or savepoint alias | Actual holder and control storage. Sharing a root or copying record handles does not materialize its reachable records again. |
| Changed transaction field or selected sequence | The new materialized field/backing, while the previous occurrence remains charged for as long as canonical state or rollback still retains it. |
| Candidate publication | The union of candidate, base, rollback and output ownership. Overlapping persistent roots do not justify adding their complete standalone summaries. |
| Moved output field | Its existing payload debit transfers to the receiving owner. A separately retained cloned output field has its own occurrence charge. |
| Hosted auxiliary graph | Its actual outer storage and changed records, with shared child custody. Hosted producer retention and DOM admission are independently bounded domains. |
| Retained revision history | Native chunk bodies and actual deque capacity share the DOM custody ledger. Each retained entry owns the payload charge for its cloned changes field. The public history-byte quota counts used entry headers and exact changes payload, excluding spare chunk capacity. |
| Discarded intermediate | Its debit releases after the represented data. Rollback restores reversible state and storage, while spent work remains monotonic. |
| External snapshot | Excluded from DOM custody until it enters an admitted operation as a retained input. It does not keep a former DOM owner's debit alive. Hosted producer receipts still follow their actual allocations. |

The custody registry uses checked, nonreused identities across bank types and
owners. It holds weak accounting sidecars, rather than weak semantic records.
Registry paths, sidecar targets, release records and retained capacity have their
own admitted storage. Cleanup releases only objects whose controlled ownership
has actually ended.

`DomNameLimits` bounds directly owned name records, text, index capacity and
operation receipts, plus canonical attribute-array storage and the directly
owned name-custody capsules. Attribute-value text keeps its existing payload
admission. Pending and dead entries count until removal; a live record
is never evicted to escape admission. Name work and scratch have separate finite
operation allowances. Shared custody-registry administration keeps its existing
global and Source admission rather than acquiring a second name debit. The name
ceiling constrains registered records even when external tokens keep them alive.
It does not bound the whole DOM, caller-owned cloned handles or values surviving
owner destruction, when the registry policy ends. Physical memory qualification
includes these separately accounted resources.

The [typed payload owner](../src/store/payload.rs) reserves a fresh identity
before retaining or allocating a new body. Its pending holder keeps incoming
Formation credits beside the data through refusal, then transfers them into the
existing registry. Form history uses this owner; its receipt is excluded from
the Slot's shallow descriptor. Projecting changed identity-bearing entries
creates a new payload while the staged and previous versions remain controlled.
Historical adoption borrows that same immutable payload and its cached receipt;
the external semantic handle itself carries no custody.

Optional owner-local pools reuse equal ordered attribute banks and immutable
UTF-16 strings during creation, import, installation and actual edits. Whole-bank
lookup follows required name resolution and precedes optional component sharing.
Text replacement offers the completed value. Namespace/local-name atoms keep
their required identity; sharing prefixes, values or Text never merges node
identity, native state or parser provenance. A semantic no-op keeps its original
backing and history, including deliberately reserved capacity.

The pools live outside snapshots and retain bounded weak entries. Their bucket,
inverse and known weak-target storage is admitted separately. A strong candidate
upgrade admits its full retained graph before selection; sharing does not reduce
the established per-field occurrence charges. Fresh zero-capacity attribute banks
have no target to pool. String reuse can avoid duplicate empty UTF-16 targets;
reserved empty backing still contributes its actual capacity. Installation keeps
the producer report and receipt intact while forming canonical DOM arrays, so
retained producer and DOM arrays remain separate storage.

The session attachment retains its admitted weak-owner storage, but pool roots
remain dormant until eligible value work starts. Value-free edits pay no pool
root, mark or seal work. A Source checkpoint predating activation restores by
aborting that session's late root; an existing active root uses paired marks to
undo the selected suffix. Both preserve spent work. Resumed formation uses the
current admission envelope after external retained data has been reconciled.

Publication prepares all closing work before accepting marks and freezing the
paired pools; the final seal is infallible. Refusal, cancellation and
whole-operation no-ops abort their session. Discarding a Draft for reentrant
observation ends its uncommitted session before child edits use the current
owner. Bounded eviction and optional qualifier exhaustion affect reuse, never
semantic identity.

Revision history uses closed chunks of 32 entries with direct oldest and newest
access. Publication prepares its insertion, actual oldest-entry evictions and
storage peak before committing. Unchanged entries are neither copied nor
recounted. Empty oldest chunks release their backing after their entries, while
the complete old and pending history remain charged until the publication
decision commits or is discarded. Source consumes that same prepared decision;
it adds no separate external history envelope. History queries copy the requested
chronological interval into a caller-owned report, whose storage is independent
of retained history.

## Historical inputs and Source checkpoints

Import retains its supplied snapshot. Admission borrows that input while
reconciling its missing persistent frontier with current controlled storage.
An identical controlled root requires holder bookkeeping; shared subtrees prune
reconciliation. A disjoint historical input can require work and storage
proportional to that input. Its successful capture is shared across the original
continuation, rather than re-admitted for each imported node or checkpoint.

Source checkpoints retain a journal mark and the interpreter state needed to
restore the original continuation. Mutation barriers prepare inverse storage
before changing a field. Actual external acknowledgments and cumulative work
are distinct from reversible state: retry cannot undo already spent work or
invent a new external receipt. The [Source contract](source-and-style.md)
describes public continuation and admission behavior.

Private tests compare sparse publication against independent full-state and
change calculations. Scale tests measure actual post-setup operations and
report setup separately. Private producer counters observe actual bank visits,
record materialization, administrative allocation targets and backing capacity.
Sequence counters count initialized, moved or copied elements, including a
completed prefix before refusal. A separate relation-sequence subtotal keeps
child/relation copies visible independently of journal and interpreter storage.
These counters exclude fixed in-place field assignments, stack-local moves,
standard-library sorting internals and opaque allocations inside lower-level
crates; they are not a global allocator measurement.

Savepoint measurements distinguish newly retained administration/backing from
the complete controlled Source union; they do not recursively sum the storage
reachable through each saved root. Retained-budget observations and process
memory complement the producer counters. Resource limits and successful small
fixtures do not establish million-node qualification.

[Back to surgeist-dom](../README.md).
