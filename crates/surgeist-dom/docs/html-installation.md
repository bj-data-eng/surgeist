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
contribution and binds its finite `DomParsedHtmlAssociationLimits`. The original entry
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

### Original Style inline-policy prefix

`begin_parsed_html_style` selects only the first influencing original Style
operation in a fresh ordinary constructed Document. DOM checks the authentic
current pending entry, exact installed Document generation and source-root
mapping, all capture dispositions, the absence of a canonical CSSOM binding,
and that every earlier association operation is producer-proven no-effect.
This establishes fresh consumer association state with no associated sheet.
The historical `HtmlStyleState` and a native null sheet are not that proof.
A refused earlier Meta classification may already have spent work; it does not
constitute prior association execution or restart the original envelope.

DOM adapts those retained inputs through HTML's real Style cursor, acknowledging
its proved no-op removal and original connection/type guards. The initial prefix stops at
`CheckInlineCsp`. The original insertion runs before the parser pushes the Style
or consumes its raw text, so this first request has empty text even when the
completed tree contains CSS. Later parser-pop text is a different original
operation and cannot replace it. Nonce is the exact retained internal nonce,
including its original UTF16. Current attributes and descendants are never
reconstructed into historical inputs.

The durable entry moves from `Unstarted` to `AwaitingInitialPolicy` and then
`InlinePolicyPending`. The binding command accepts only its exact original
constructor-declared `OfflineUnbound` InitialPolicy slot and an explicitly
supplied immutable `HtmlCspAssociation`. A genuine constructor-empty artifact
must be supplied explicitly too. Successful binding takes the Option input;
refusal preserves it. Known, deferred captured and lost policy facts cannot be
replaced by current or empty defaults. Captured/fragment and earlier influencing
histories outside the fresh profile remain pending. Reached intrinsic Meta CSP
gaps still refuse installation.

Binding and request formation are separate commands. A late formation refusal
retains the accepted artifact, original qualification and spent work. Raising
the original limits permits retry; lowering them or replacing an accepted
binding does not. The same immutable report installed independently has distinct
destination qualifications and can bind distinct explicit policies.

`parsed_html_style_prefix` borrows the actual stored qualification or request
through a nonblocking canonical read guard, after temporary command holds have
ended. It supplies no policy reply or publication authority. Repeated queries
observe the same request. Drop releases the read loan; forgetting it leaves
Busy rather than discarding or recreating the phase. The
[borrow examples](../src/store/pending_associations/style_prefix.rs) show how to
release the query before issuing a binding command. Runtime's Session forwards
this canonical query and orchestrates short commands without retaining another
policy owner or cloning a report graph.

An external supplier can inspect that immutable request after all mutable
Runtime and DOM command loans end. `DomParsedHtmlStyleInlinePolicyObservation`
records its actual allow/block decision; DOM provides no evaluator. An absent
supplier leaves `InlinePolicyPending`, including for an opaque native artifact.
Genuine InitialEmpty is an explicitly supplied policy state, not an inference
from missing policy or browsing context. HTML's Meta CSP processor attests Meta
insertion, not this inline decision.

The observation constructor accepts only the actual stored request. Its fixed
stamp retains the original qualification and every bound CSP identity,
generation, revision and profile; it retains no text, nonce, artifact or report
backing. The one formed request is immutable and never replaced or reset, so
those identities qualify its exact original text and nonce. The observation
does not grant canonical currentness by itself.

`accept_inline_policy` takes an Option input only after validating that stamp
against the still-pending `CheckInlineCsp` request and admitting original work,
cancellation and one cumulative durable decision output. Refusal preserves the
same input, binding and cursor, allowing raised-limit retry without another
evaluation. The same HTML cursor consumes `InlineAllowed` once: allow reaches
`CreateSheetPending`; block reaches `InlinePolicyBlocked` for this occurrence.
The inline-policy query then returns no pending request, and repeated advance
only observes the reached phase. Repeated acceptance is refused. Both branches
retain the same graph, policy, cleanup reservation and debit until normal entry
disposal; no large backing is retired inside the decision transition.

The one original parsed-association control retains finite work, outputs,
limits and peaks. Work includes prepaid native last-owner disposal credits;
those credits are not refunded on retry. The artifact's cached independent
cleanup-frame extent remains reserved in processing peak admission for its
entire retained lifetime. This reservation is separate from actual bank/ledger
storage, which charges the real graph and old/replacement overlap. Later nonce
formation and temporary banks cannot consume that reserved cleanup headroom.
The graph drops before its debit. Canonical pending usage includes the persistent
graph even when an older disposal-only retirement shares its entry; future
installs/removals preserve that accounting.

### First original sheet publication

After an accepted allow decision, `create_sheet_document` identifies the actual
mapped destination under the same original admission. `prepare_create_sheet`
borrows that hold and the genuine CSSOM capability exclusively for the join.
Its closed editor lends the exact original empty text, title, media, parser
Document and mode. It checks original source order and refuses unexplained
existing producer membership. It does not construct native Source facts or use
the later parser-pop text. Captured and otherwise unsupported histories retain
their existing refusal.

The consumer parses these original UTF16 inputs through CSS's public bounded
fronts. Runtime charges every actual public preparation/pre/post grammar poll
to both original and Runtime work. CSS's private grammar work, interruption
latency and transient allocations are outside that consumer envelope. Actual
reports transfer into the private CSSOM batch under CSSOM's logical/source
limits. Returned media diagnostics retain their authentic diagnostic backing
and source union under original admission; that receipt does not measure the
entire CSS AST or grammar scratch.

Staging takes the Option reports only for an actual producer attempt. Invalid
qualification leaves the input intact. Any attempted construction or subsequent
admission failure closes the editor; catching that failure cannot make an
orphan construction publishable. Dropping preparation publishes nothing. Limits,
spent work and the selected original cursor remain available for a raised retry.
Operational producer identities already issued during failed work may remain
spent, while canonical revisions, membership and effects remain unchanged.

The final preparation uses the actual prepared ticket's authenticated sheet ID
and result revision. All DOM formation, original work/output/storage admission,
association serial, canonical binding, and HTML `Applied` successor are formed
before either owner publishes. The infallible suffix publishes the genuine
CSSOM commit and DOM association, then moves the same original cursor to
`ReadScriptBlockingPending`. The published capability retains the root lock
through Runtime's sole add-sheet/preference installation. Only `into_parts`
releases that lock and returns genuine completion plus disposal-only retirement.
Old State and its custody, old root binding, CSSOM retirement and exact impending
history evictions remain retained through that fence; media diagnostics remain
with the completion. No large backing is finally disposed inside the joined
suffix.

This is an intrinsic association publication, not processing readiness. Native
sheet access remains `PendingParsedProcessing`/unavailable while the original
contribution is Pending.

### Explicit exclusion of executable-script bookkeeping

The admitted Rust execution owner has no executable-JavaScript engine. Its
original Style cursor retains HTML's `NoExecutableJavaScript` profile from
selection onward. At the reached `ReadScriptBlocking` action, Runtime supplies
that actual owner profile to `exclude_script_bookkeeping` with the exact retained
contribution/occurrence qualification. DOM checks the mapped current Document,
original report and stored operation profile. A caller's profile declaration
alone grants no authority. `SourcePredicates` belongs to integrations supplying
real normative predicate/membership results and cannot take this exclusion.

The specific HTML `ScriptBookkeepingExcluded` reply projects both the executable
script predicate and its sets out of this supported profile. It does not claim a
false predicate, register a pretend script blocker or infer anything from media,
headlessness or a missing service. One cumulative durable output and original
work/cancellation are admitted before installing the same cursor's successor.
Refusal preserves the binding, request, policy, cleanup reservation and reached
phase; raised-limit retry retains spent work. No sheet, issuer or cursor is
recreated. Drop/resume preserves the exact original occurrence.

Acceptance reaches `ReadRenderBlockingPending`. The canonical view's
`script_bookkeeping_exclusion` returns the accepted profile at this reached
phase and after this invocation completes rendering; selecting the profile
earlier returns no such disposition. Repeated advance observes the stored
phase, while repeated exclusion or a wrong-phase,
profile or qualification refuses. Both unresolved media and a universal media
query can reach this render wait: neither is a reason to keep the excluded
script phase pending, and exclusion supplies no rendering/media observation.

Actual render context, resources, later policy/notification actions, parser-pop
text updates and subsequent occurrences retain their separate authority. Whole
contribution completion is not performed. DOM provides no CSP evaluator. Use the
Session structural-installation route; raw Session admission keeps its existing
influencing-Style refusal.

### Original rendering actions

After the accepted script exclusion, `process_rendering` uses the retained
occurrence's Style inputs, exact parser-Document identity and HTML media cursor.
Absent Style media has its all-media default; empty and ASCII-whitespace-only
values match without an environment. A nontrivial value (including `all`) returns
`Pending(MediaEnvironment)` when the potential-render predicate still needs its
match. A genuinely false potential predicate needs no media observation.
Unavailable original Document facts remain `Pending(OriginalContext)`.

The media/potential predicate is separate from the original Document eligibility
gate. A head Style can add canonical render-blocking membership even though the
installed final Document already has a body. A body Style completes the same
invocation without adding membership. The current mapped element and its actual
Document must still match the original qualification; original body facts do
not authorize an effect in an adopted or different destination.

`Complete` owns the genuine `DomCommit`, narrow publication-coordinate membership
evidence and actual output/retirement backing under the same original admission.
A no-op gate returns the actual empty commit. Draft, change/history backing,
retired State/custody and the cursor successor are prepared before the infallible
publication tail. Refusal/cancellation preserves the reached cursor and canonical
state while retaining spent work; retry may raise the same envelope. Every owned
Pending result and completed publication consumes finite cumulative output.

The new `RenderingComplete` phase completes only this original invocation's
rendering actions. It preserves the accepted script exclusion, original sheet
and whole Pending history. General `is_render_blocking`/sheet readiness queries
continue to refuse `PendingParsedProcessing`; the narrow completion is historical
evidence rather than a general live query. Later parser-pop text updates,
resource and notification processing remain separate work.

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
