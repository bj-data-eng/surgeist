# Native HTML and parser-report installation

Fresh-native construction and completed parser-report transfer have different causes. Both install through the same canonical Draft/publication owner and return immutable, source-qualified historical bindings.

## Fresh-native HTML fragments

`Dom::install_native_html_fragment` consumes an immutable `HtmlFragment` as an
explicit fresh-native authoring input. It checks an exact `DomDocumentStamp`,
the destination parent and optional insertion reference before source work,
then stages the whole installation through the same Draft/publication owner.
Raw namespace/prefix/local components, attribute order and UTF-16 are preserved;
DOM's component grammar applies without reconstructing a QName or applying the
namespace-aware invocation's reserved-prefix rules. Actual native attribute and
tree hooks run in source order. Native Template owned contents use real cached
inert Documents; Script uses fresh native state and actual Disabled preparation.

The returned `DomNativeHtmlInstallation` owns the installed immutable snapshot,
change report and full source-qualified `HtmlNodeId` to `NodeId` bindings. The
source fragment maps to a real detached empty canonical fragment after transfer;
owned Template contents are mapped, private auxiliary Documents are not. Bindings
follow source preorder with owned contents before ordinary children. Source drop,
later edits and retirement do not rewrite this historical result. Every fallible
map/output allocation happens before publication; refusal preserves the entire
live tree, identities, revision and history.

`DomNativeHtmlInstallLimits` separately admits mapped nodes, adapter work and the
simultaneous retained capacities of traversal/map/output buffers. Work charges
visited nodes, five association observations per node, edges, attributes, prior
map comparisons, map writes, final remapping/bindings and raw field UTF-16 extents.
An empty ordinary source costs nine work units and creates its mapped wrapper.
Canonical staged tree/name/family work and publication storage retain their
existing `DomLimits` admission and cancellation checks. The adapter quota does
not claim to meter those independent operations.

Declarative shadow relationships and hosted source fragment roots receive typed
association refusals: native attachment cannot supply retained declarative/root
document facts. Unsupported native families retain their actual requirement
errors. A bare fragment does not carry complete parse-report context, provenance,
effects or remaps; passing one explicitly requests native creation.

## Completed parser reports

`Dom::install_parsed_html_document` separately transfers an admitted completed
offline parser report into a fresh canonical Document. The caller supplies the
current store stamp and explicit URL, opaque encoding, origin and destination
input-platform facts through `DomParsedDocumentEnvironment`. The operation
preserves actual construction state and publishes fresh canonical identities,
without replaying native attribute or insertion hooks.

`Dom::install_parsed_html_fragment` checks the original `DomHtmlCapture` against
the actual current target and materializes a detached fragment in that Document.
It applies the report's ordered Template-cache and shadow-attachment effects;
it does not implicitly append the fragment to its context. Retained auxiliary
parser Documents remain distinct from the captured target and template owners.
ShadowRoot captures also preserve the root's actual node Document separately
from its host's Document. Installation resolves both through their qualified
bindings and materializes the fragment in the target Document.
`DomParsedHtmlInstallation` keeps the report, installed snapshot, changes and
separate construction-subject and captured-subject bindings as historical data.
Subsequent edits cannot extend or rewrite those bindings.

The current profile requires the report's supported closed scripting, registry,
image and Document-policy qualifications. Missing required family or historical
state, a stale or foreign capture, incompatible environment, cancellation or
admission failure refuses the complete installation. The adapter separately
bounds mapped subjects, work, historical backing and simultaneous scratch
capacities, including old and replacement vectors during growth. The two
mandatory shared output Vec-header payloads are reserved in that same original
envelope before collection and consumed at actual wrapping; this reservation
for future backing is separate from measuring simultaneous allocator occupancy.
Canonical
Draft and publication retain their own limits. Downstream associations are
explicitly pending; installation does not execute resources, CSSOM or callbacks.
For captured historical hyperlink observations, installation checks the actual
parse Document against its current canonical Base selection, opaque encoding,
full independently bound revision and exact raw-input outcome before and after
the ordered construction effects. Missing or changed required facts receive
`CurrentPolicyUnavailable`. Auxiliary parser and inert-owner history keep their
original roles. An optional unused whole captured Base bank remains fully charged
historical request data; it does not override the current canonical bindings.

## Meta Document history

Installation transfers qualified per-Document language, referrer, refresh,
theme-color and authored color-scheme history. The original parser selects CSS
values under its own finite admission; installation retains the sealed result
without parsing CSS or replaying native Meta hooks. Selected color-scheme backing
also consumes the canonical retained-report allowance. Untouched Template-owner
Documents retain their own known absence. Reached unavailable media/CSS facts
or intrinsic CSP processing still refuse the entire installation.

## Pending stylesheet associations

A sealed complete parser association capture can install structurally while
association execution remains pending. `Dom::pending_parsed_html_associations`
lends the canonical owner's retained contributions, including default-style Meta
with no sheets. Each contribution keeps the authentic immutable report,
original result-qualified occurrences/causes/contexts, exact Document bindings,
the original canonical captured-context stamp, and optional subject bindings.
Historical subjects without a current binding
remain historical; installation invents no NodeId for them. The captured-context
stamp is original history, not current freshness or execution authority. A later
context observation must match that original basis while the association join
independently qualifies its actual current owner. An unavailable
capture refuses before publication.

`DomParsedHtmlAssociationId` combines the actual publication stamp and installed
root. Installing the same report twice issues distinct destination identities;
the original occurrence identities stay equal. The canonical bank retains this
history independently of installation packets, owner moves and later permitted
node retirement. Intrinsic-only installs retain no additional report owner.
Removing a Meta, Style or fragment never acknowledges pending work.

For an exact Style whose complete original inventory proves initial creation
and no reached association obligation, installation establishes its null sheet.
Disconnected Template-only and consistently type-ineligible histories can supply
this proof. Another Style in the same Document may remain pending. Final
attributes, disconnectedness or an absent pending entry do not establish it;
any potentially influencing earlier processing stays pending. Later native edits
use the ordinary Style processing protocol and preserve parser creation history.

`DomLimits::with_parsed_associations` selects finite pending byte and UTF-16
allowances; the constructor selects 64 MiB and 16 Mi units. Current backing
accounting includes each actual report graph once, each destination's maps and
metadata, and sparse semantic index backing. The same canonical custody ledger
also admits the actual record/control/path union and replacement overlap during
formation, and later Source operations see that retained union. Per-invocation
HistoricalBytes remains a separate admission. Caller-held immutable output
history and allocator/Arc bookkeeping are outside these allowances; they are
not an RSS claim. Limits cannot fall below current retained pending backing.

Native Style updates, association owner-attribute/disabled operations, adoption
across either affected Document and nonempty default-style Meta processing
refuse with `ParsedAssociationPending` until a real association consumer can
join the original contribution. Unrelated native edits remain usable. Hosted
Document cleanup refuses pending obligations; ordinary public destruction does
not support Document destruction. No positive Document-retirement path is
claimed here.

The read views grant no processing or acknowledgement authority. The explicit
Meta consumer below joins supported original history; external context binding
and influencing Style processing remain separate work. Native hooks are never
replayed as an execution shortcut.

### Whole parsed-Meta consumption

`Dom::begin_parsed_html_meta` exclusively borrows one authentic pending
contribution and binds its finite `DomMetaAtomicLimits`. The original entry
retains cumulative work and output admission across retries; limits can only
increase componentwise. Runtime must charge and admit its actual temporary
banks through the hold before forming them. Repeated whole storage reports
retain their positive high-water deltas, rather than repeatedly charging the
same backing. Disposed temporary storage is released; spent work and reserved
output evidence remain cumulative.

`validate_meta` traverses the complete original parser report without accessing
CSSOM. It accepts original default-style changes and producer-proven no-effect
occurrences. Any influencing Style or another unsupported operation refuses the
whole contribution. No URL, CSP or media binder is requested for this slice.
Validation may inspect a later contribution, while preparation requires that
this contribution is first for every affected canonical Document.

`prepare` receives genuine destination CSSOM capabilities, qualifies actual
canonical bindings and holds each real root lock once. It returns ordered
original occurrence IDs, historical HTML subjects, mapped Documents and exact
UTF16 names. Runtime stages its existing sheet-set preference algorithm and
sheet flags through these held snapshots and batches. All producer preparation,
canonical acknowledgement, output and replacement storage admission completes
before the first producer publishes. `commit` publishes those prepared CSSOM
batches and the canonical acknowledgement without advancing the tree revision.
Its published value retains the root locks while Runtime installs its already
qualified preference names. `into_parts` then unlocks and returns the authentic
completion and retirement; large retired backing is disposed outside that fence.
A first empty CSSOM owner can bind through this same atomic publication.

Dropping a preparation publishes nothing and permits a later hold to retry the
original contribution. Its own hold is sealed after preparation, so further
work requires dropping it and reentering. Forgetting a preparation leaves its
actual private graph, root locks and admission debits retained; it does not
reopen execution or refund live storage. A completed identity is no longer
pending and cannot replay; no registry of all completed identities is kept.
Shared reports remain charged once while any installed contribution retains
them. Removing the first pending contribution repoints each affected Document
to its next surviving contribution instead of clearing another obligation.

The current parser's default-style effects arise in its active Document tree.
Fragment-context Meta and Template contents do not become new influencing Meta
history merely because their node Document names a live target. Fresh-Document
reports and independent installations are supported. Broader existing-Document
and multiple-contribution execution depends on authentic producer history; this
front does not manufacture such reports or infer processing from the final tree.
Runtime owns the corresponding Session admission, preference state and output
orchestration; the lower DOM front does not create another preference owner.

## Control state and form associations

Installation transfers the report's current values, dirty/checked/selected flags,
control state and qualified associations. Result-local Form and Option/Select
references map to their fresh canonical identities. Parser-special nonancestor
Form associations from ordinary document parsing retain their actual owner and
parser-inserted flag through installation.

A captured nearest Form supplies fragment parsing context. It does not itself
establish a parser-special external owner: the pinned
[create-element algorithm](../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md)
requires a null fragment context for that association. Detached installation
preserves the actual report state. Later explicit insertion, clone/import,
removal and adoption apply their ordinary copy/reset rules against the current
canonical tree. Retained reports and installation snapshots preserve their
historical values and associations after those edits.

## Contextual SVG Script and Dialog history

Completed contextual SVG Script transfer uses the original closed fragment
producer and independently checked current canonical Document environments.
It preserves the actual processed bit and Parser/Clone cause, including false
state in detached fragment or Template contents. Installation does not replay
processing; later explicit canonical insertion uses the actual connection rule.
Captured target, auxiliary parser and inert-template Documents keep their
distinct identities and historical bindings. This grants no acquisition,
execution or load/error notification capability.

Parsed Dialog transfer consumes the original controller's retained current
return value and closed-service qualification, independently of content
attributes. Newly owned Documents transfer their own qualified interaction
history; captured target Documents retain their existing lists. An unfulfilled
original Dialog operation refuses installation with its subject and action,
while its Document and cause remain in the immutable report. Materialization
does not replay hooks. Later canonical open changes, removal and insertion run
the ordinary hooks against the then-current node Document. This does not grant
hosted show/close, focus, watcher or top-layer services. Unsupported report
profiles and missing required family state refuse installation.

[Back to surgeist-dom](../README.md).
