# Surgeist CSSOM

`surgeist-cssom` owns mutable authored CSS identity, containment, atomic edits and
immutable captures. It depends only on `surgeist-css` and the standard library.
CSS owns grammar, checked syntax, expansion and formatting. CSSOM owns its CSS
integration adapter under the repository's authorized ownership exception.

Standard-named Rust objects such as `CssRule`, `CssStyleSheet` and
`CssStyleProperties` are private-field live handles. Checked store factories
recover them from product IDs; their equality and hash preserve owner-qualified
identity across edits. Reads receive `&CssomStore` explicitly. A handle retains
identity without retaining dropped owner data; `handle.capture(&snapshot)` creates
an owning, read-only `CssomCapture<T>`. Captured children and conversions share
that exact snapshot, so a parent chain remains stable after edits or store drop.
Infallible upcasts preserve identity; checked downcasts return `None` for a
nonmatching interface and explicit errors for foreign or absent objects.
The public `id()` and `snapshot()` accessors support trusted provenance and host
integration separately from standard object operations. Standard rule text uses
the native bounded graph serializer. `css::escape` and `css::escape_with_limits`
forward identifier escaping to CSS; CSS support evaluation remains outside this
authored-state crate. [Object foundation tests](tests/object_foundation.rs) cover
the live/captured distinction, checked conversions and retained ancestry.

Concrete interfaces inherit standard members through private dispatch. Keyframes
rules have their own collection operations rather than grouping-rule inheritance.
Style and nested-declaration rules expose `CssStyleProperties`; Page and Margin
rules expose `CssPageDescriptors` and `CssMarginDescriptors`; FontFace exposes
`CssFontFaceDescriptors`. Keyframe rules expose the common `CssStyleDeclaration`
base, and CounterStyle has its specialized descriptor operations. Descriptor
objects have no ordinary property reflections. Captures keep the same interface
membership and add no setters.

An immediate object mutation stages one native batch and constructs its result
before publication. `CssomOperationOutcome<T>` owns the standard result,
publication, ordered recovery diagnostics and single-use owner effects. Read all
four through accessors or consume `into_parts`; the binding handles effects after
publication. Declaration setters take a borrowed `FnOnce` admission resolver,
called once only when actual checked candidates need decisions. It returns those
decisions with the independently sampled current input inventory. CSSOM stores
no resolver or host callback. An error or unwind aborts staging; an advanced
declaration editor lets a caller stage multiple operations in its own batch.

Inline declaration blocks can carry an immutable, independently current
`CssomInlineDeclarationContext`. Its packet requires explicit Owner, ParserMode
and Support inputs; an empty support profile is a valid unsupported selection.
`create_inline_declarations` starts an empty ordinary block with null parent and
all flags unset. `update_inline_declaration_context` keeps that block's identity
and owner identity while replacing current versions and facts. Existing checked
declarations and their parse-time inputs remain unchanged. Other declarations
continue to use the store context, including its existing `update_context` flow.

Inline preparation and usability requests use the block packet. Advanced
`apply_declaration_with_inputs` calls supply every independently current packet
input/linked version, plus any separate Support decision versions. Context changes
stale prepared work for that block; unrelated block updates within the same batch
do not. Live getters read the current block, captures read their original block,
and manifests distinguish current packets from historical admission versions.
Context rebinding emits no owner synchronization effect. Existing owner effects
still require the exact whole-store revision and manifest, so even an unrelated
inline commit can stale a pending effect. These producer APIs do not implement a
DOM binding or change the existing post-publication owner-write contract.

`CssStyleProperties` provides generated Rust getters/setters for CSS-owned
canonical spellings, vendor aliases and distinct legacy grammars. Captures have
only getters. Reflections delegate the common declaration operations with empty
priority; their existence does not grant composed-owner support. Custom properties
use the string API, and `float` uses `css_float`. The root-owned
[reflection tool](../../tools/cssom-reflections/README.md) records metadata origins,
reproducible generation and checked naming rules. Production consumes checked-in
Rust without depending on that tool.

Create a `CssomStore`, capture a snapshot and acquire an edit guard from that
capture. Stage parsed or checked typed sheets in a batch; commit resolves private
creation tickets to owner-qualified live IDs. One successful publication advances
one revision. Failed or stale batches publish nothing. Source no-ops and batches
that restore the original observable state preserve the revision. IDs are never
reused within an owner. Dropping a staged ticket/batch cannot publish an identity.
Constructed initial ingestion and recovered replacement filter import roots;
external ingestion retains them. Original checked occurrences stay separately owned.
The standard `CssStyleSheet::new` starts empty with new rule-list/media identities
through the existing construction owner. Its explicit Document capture supplies
fixed location and constructor identity; nullable base URL remains separate.
MediaList constructor input is copied through its actual CSSOM text and reparsed,
including recovered `not all` text. Construction does not assert membership in a
host's styleSheets collection: `set_sheet_list_membership` publishes its supplied
selection/order. Its `CssomSheetListTarget::Existing` identities and `Created`
creation tickets can be mixed in one batch, so a newly parsed sheet reaches its
correct caller-selected position in the same publication. Tickets are validated
against that exact batch and never expose tentative live IDs. Length/item inspect
that collection and return null out of range.

The supplied parser context applies to subsequent parsed creation/replacement;
context updates preserve existing checked values and their original parser facts.
Typed ingress preserves the context carried by each checked CSS occurrence.

Sheet titles retain exact UTF16 as `Arc<Vec<u16>>`; applicable owner edits borrow
`[u16]` and copy fallibly within finite admission. Live getters borrow the current
store and captured getters borrow their snapshot; both return null for empty
titles and otherwise exact units, including lone surrogates. The host supplies
the title already selected by HTML applicability. An owner node alone does not
establish a live title reference; absent/inapplicable attributes supply empty.
Staged record copies share immutable backing. The existing string-byte bank
charges each canonical sheet title's checked actual Vec capacity times two,
including spare capacity. This per-record admission does not inspect Arc counts
or deduplicate titles by text or pointer. Separately retained snapshots do not
increase the current State's quota; the distinct CSS source bank deduplicates
actual `CssSourceSnapshot` identities.

`CssomOwnerMediaInput` distinguishes real UTF8 input from an ordinary qualified
CSS media report. The report preserves complete input/context and diagnostics;
context mismatch, full-input overflow and parser resource recovery refuse the
ordinary batch. Owner version/title changes retain the actual parsed media root
even when its query list is semantically unchanged or empty. A true unchanged
version/title/media operation preserves the previous root, revision and effects.
CSS owns UTF16 preprocessing and media grammar; owner adapters do not normalize
title units or fabricate stylesheet wrappers around media attributes.

`create_parsed_sheet_with_media_reports` is the initial two-report ingress. It
validates stylesheet and media source/context/full-input/resource facts together
and returns the existing `CssomSheetConstruction`: body diagnostics stay on the
sheet and media diagnostics retain their separate input coordinates. Its
`inputs.media` must be constructed and empty; a second parsed or nonempty value
is refused rather than ignored. The media report supplies the single actual
initial value, including a retained root for an empty list. Existing typed media
inputs remain already checked values: parser input byte limits apply at actual
parse/report ingress, while retained source backing admission applies to every
retained parsed source regardless of the ingress.

`CssomBatch::create_parsed_sheet_report` consumes an ordinary CSS stylesheet
report, preserving ordered recovery diagnostics. Its owner-issued complete root
source/context must match the staged parser context, including the independent
glyph-definition choice; the complete root parser input obeys `max_input_bytes`
even when no rules survive. Constructed syntax has no parse attestation and uses
typed ingress.

`CssomLimits::max_retained_source_bytes` defaults to 64 MiB and is an intentional
additional storage contract. CSS supplies checked actual backing usage and source
traversal; CSSOM admits all source identities retained in its canonical/staged
sheet, rule, declaration, descriptor, media and map records, including detached
objects and edited values. Repeated references to the same snapshot count once;
distinct parse snapshots count separately even when their text is equal. This
does not redefine logical input/string limits. UTF16 preparation has separate CSS
input-unit/output/work admission, and its shared original/map backing still needs
CSSOM storage admission. Caller-owned historical snapshots do not count again
against the current state's logical quota; this is not process RSS accounting.
Linked snapshots actively retained by the current context or selector-input
contexts do count, including their transitive complete retained states. A link's
selected sheet does not narrow the snapshot's owned data.
`max_source_traversal_steps` (16 Mi steps) charges payload/state/table visits and
identity comparisons into one cumulative bank; `max_source_traversal_depth`
(2048) bounds CSS representation walking independently of CSS nesting. These
finite defaults are storage-admission policy, not CSS grammar limits.

Retained construction and owner-attribute results expose
`retained_backing_bytes_with`: actual private diagnostic Vec capacity plus each
CSS-owned diagnostic's out-of-line backing, under caller admission. It excludes
the result's inline Self and separately shared source graph. This observation is
recomputed without allocation; diagnostics and source identity remain inspectable.
A consuming owner counts its enclosing payload once and uses one
`CssSourceRetention` domain across its retained results and pending publication.
These result receipts do not add the CSSOM sheet graph or redefine its backing-only
source quota; separate owner domains are not a unique process-memory total.

Snapshots own checked current payloads, original occurrences, ordered membership,
namespace facts and every independently revised supplied/linked input. They remain
readable after edits and after dropping the store. Removal/replacement detaches
only the removed root: descendants retain `parentRule` ancestry and resolve a null
`parentStyleSheet` through that root. Detached records count toward product quotas
and remain readable until their capture/owner is dropped; there is no destruction
or reclamation operation in this foundation.

Current selected property entries are independent of raw authored occurrences.
Direct terminal removal rebuilds through the CSS checked selected-entry boundary,
preserving each survivor's source occurrence, priority and order. It never expands
the old shorthand again. Ordinary, keyframe, margin, mixed Page, counter and font
descriptor blocks have explicit domains. Checked grammar recognition and usable composed-owner support are separate.
Valid symbolic inputs can retain a typed unavailable projection rather than becoming
an empty or partial block. Storage does not imply
that every authored property or whole-rule format is supported.

Live counter names are raw strings. `CssCounterStyleRule::name` applies the CSS
identifier writer; its ten descriptor getters read the specified effective
values and return empty text for an unspecified descriptor. These partial reads
remain available for intrinsically undefined rules, independently of the selected
whole-rule format's unavailability. Each descriptor setter parses a complete raw
value and validates the prospective current collection through CSS. Grammar or
undefined-definition rejection and a changed system algorithm are source no-ops;
fixed-system first values can change. Undefined existing rules can be repaired.
Definition-relevant environment substitution or calculated additive weights return
`CounterStylePreparationUnavailable`; resource errors retain their native cause.
Irrelevant pending descriptors remain symbolic checked values. The
[`CounterStyle interface`](../../references/css-counter-styles-3--editor-capture-20261009--ef29d0a06a13.md#the-csscounterstylerule-interface)
supplies these operations, consumed through the existing CSS prospective provider.

`CssomBlock::counter_descriptors` supplies the bounded current checked collection
for intrinsic inspection. Untouched named duplicates preserve genuine parsed
occurrences and last-occurrence reads. An edited kind replaces its current
occurrences with one honest raw value origin, while original syntax remains
separately retained. Edits never invent descriptor-name occurrences. Each retained
edit owns one quota entry and its complete raw UTF-8 input; replacement releases
the previous current edit's quota. Detached objects count globally, while older
immutable captures do not count again against current storage. Request work uses
disjoint cumulative provider allowances and bounded entry/input scans; it does not
evaluate counters, substitute environment values or resolve calculated weights.
The [counter consumer](examples/counter_consumer.rs) demonstrates independent old
and new partial readouts and origin inspection.

Seven ordered feature maps accept raw keys and
unsigned-long sequences, including present-empty values. Exact authored indices
and symbolic values remain separate; unrepresentable authored initialization gives
an explicit conversion failure. Empty nested declaration children retain their
IDs, declaration objects, order and ancestry. CSS formatting views filter text,
without deleting those objects or weakening authored CSS construction.

The foundation edit surface provides parsed/typed creation, constructed recovered
replacement, guarded sheet/group deletion, direct selected-terminal removal,
disabled/media/context updates, raw counter-name/map edits and explicit import
association. MediaList snapshots expose canonical text, length and nullable
indexed items. Text replacement, single-query append and delete use CSS-owned
recovery and canonical byte equality; delete removes all matches. A null single
parse is a no-op and an absent non-null delete is `NotFound`. Mutation preserves
the associated collection's identity and earlier snapshots, without applying
sheet guards or writing owner attributes. Recovered parser diagnostics and typed
resource failures remain observable. Append/delete charge the candidate and
all current members through one cumulative CSS input, projection and output
budget.

`CssCustomMediaRule::name` and `query` expose readonly facets of the checked
current rule. The query is an authored boolean or a typed `MediaList`, including
a genuine empty list; it never defaults to false
or evaluates an alias. Existing MediaList operations edit that associated
collection while old snapshots and original rule occurrences remain immutable.
The readonly query slot and name have no replacement setters. Ordinary and
scoped rule ingress use the same association owner, and retained media objects,
entries and live-name bytes obey product quotas.

`CssColorProfileRule::name`, `src`, `rendering_intent` and `components` read
checked current ordinary/scoped profile data.
They use the CSS identifier and effective descriptor-value writers with explicit
output limits. Absent descriptors return empty text before emission; a present
empty URL emits `url("")`, and an absent intent does not fabricate its initial
value. Symbolic pending descriptors remain symbolic. Getter availability is
independent of the native `FormatUnavailable` whole CustomMedia/ColorProfile
capabilities; no wrapper text, resource loading or channel computation is added.
Wrong-kind, foreign-owner, missing-capture and native writer resource errors
remain typed. [Readonly extension tests](tests/readonly_extensions.rs) cover
these facets, current collection identity, recovery, detached objects, retained
source coordinates and atomic quota failure.

Source sheet getters and the guarded deprecated `rules` alias retain
the same live identities. Owner attribute captures update title/media (absence
clears), without changing fixed location or emitting a host attribute write.
Disabled mutations do not inherit cssRules security/modification guards.
`CssomBatch::remove_sheet` removes selected host-list membership and clears the
sheet's owner node, owner rule and derived parent sheet. The stylesheet, rules,
media, live handles and old captures remain available; removal does not reclaim
their quotas or cancel pending replacement work. An importing rule retains its
forward `styleSheet` reference. A subsequent explicit `associate_import` with
the same inputs restores the removed sheet's owner-rule relationship. Repeating
removal after membership and those owner links are already absent is a no-op.
`insert_sheet_rule` uses the shared rule insertion owner after stylesheet security,
modification and preliminary syntax/constructed-import checks. That owner checks
index, contextual admission, hierarchy and whole-list namespace state, then adopts
the same classified source occurrence. `add_sheet_rule` supplies the standard
legacy defaults, builds its actual bounded input and delegates insertion, returning
`-1`; absent index means the current end. `remove_sheet_rule` delegates deletion,
defaulting its index to zero. These are CSS-standard aliases, not crate shims.
The CSSRule cssText setter is implemented as its standards-defined no-op.

Rule list IDs remain the same while their current membership changes. Store
length/item reads acquire a fresh capture; snapshot reads retain the older order.
Selector replacement uses actual nesting/scope ancestry and current namespace
bindings, preserves declaration/child identities, and retains the successful
input occurrence, captured inputs and recovery diagnostics. Invalid syntax is
the operation's source no-op or Syntax error; resource exhaustion remains a typed
failure. Keyframes append retains duplicate selectors with distinct identities.
Find/delete compare complete ordered selector sequences through CSS's selected
specified-phase normalizer, searching from the last current child. Endpoint and
literal normalization, calculation simplification and unresolved context-bearing
trees remain CSS-owned; authored equality and serialized text are not match keys.
One request-local matcher prepares the query once and retains its native work
budget across every candidate. Invalid/unmatched queries return absence and
delete nothing. A successful deletion detaches only the matched root, preserving
its ID, declaration object and earlier captures. Resource failure aborts a batch
without publishing earlier edits; a read can retry unchanged with larger limits.

`CssRule::css_text` and its inherited concrete members read current selectors,
selected declaration survivors, ordered
membership, keyframe names, Page margins, FontFace descriptors and Import media.
It never rebuilds a selected block from historical shorthand occurrences.
Request-local borrowed child buffers feed one CSS-owned graph traversal. Each
native input constructor does constant work without serialization or grammar
reparse; the buffer assembly and traversal work are separately metered.
`CssomRuleFormatLimits` separately bounds actual preparation steps, view nodes,
ancestry/traversal depth and the final cumulative native CSS request. Pending
sibling buffers count toward the node limit before allocation. Preparation
failure or CSS failure returns no text, changes no capture, and allows unchanged
retry with larger read limits. These are work/buffer bounds, not process-memory
guarantees or copies of private CSS node tariffs.

Whole wrappers whose selected source remains unavailable or undefined preserve
their native typed failure and actual child path; their descendant buffers are
not prepared. A genuine original occurrence is used only as that failure carrier,
never to serialize stale mutable content. Standalone keyframe and margin reads
use their single owning native writer. Defined partial reads stay independent of
unavailable whole wrappers. Import layer names distinguish null, anonymous empty
and native serialized names; block names are local, and statement names return
new owned arrays without ancestor qualification. Their explicit byte allowance
is shared across emitted names.

Conditional matches consume an explicitly supplied evaluator result. Call
`capture_query_result` with the actual rule, required independently revised input
versions and evaluator boolean, then pass that immutable transport to
the captured Media or Supports object's `matches`. The transport retains its
owning immutable snapshot rather than copying the complete captured input
inventories. Media requires an
associated sheet, a Document input and a Window input carrying that Document's
identity. A constructed sheet's supplied
constructor-document fact must agree. Supports requires the actual captured
support-profile input, using the published concrete declaration-usability facts.
CSSOM validates owner, concrete Media/Supports role, rule, complete captured
context and owner-qualified revision before returning the supplied boolean.
Missing or foreign results, stale state/context and mismatched facts are typed
failures; there is no default false, host callback or query evaluator here.
An older capture can still validate its result after the store changes or drops.

Container reads produce new owned name/query dictionaries in authored order,
including duplicates and empty queries for name-only entries. Singular getters
return empty strings for multiple entries. Their explicit byte allowance covers
aggregate emitted fields or final composed text; existing checked CSS payload
constraints remain independent. CSS owns identifier escaping and lexical query
serialization. The specified plural composition uses its public list joiners
without evaluation or logical simplification.

Named-support definitions use an explicitly selected **Surgeist product policy**
where the connecting CSSOM algorithms remain unspecified. Their decoded name is
readonly and their stable grouping list contains one inert leaf per declaration,
qualified candidate or at-rule candidate. Duplicate and unsupported tests remain
ordered. `named_supports_test_child` exposes the three CSS-owned checked variants;
nested candidate bodies are immutable inspectable payloads, with no descendant
CSSOM identities, declaration objects, fetching, registration or evaluation.
Recognized at-rule spellings are equally inert within this test context.

Inherited group insertion admits the complete genuine input through CSS's
strict one-test-child front door, after handle/security/modification/index checks.
Trivia and empty semicolon statements are ignored recursively; multiple children,
discarded syntax and recovered input return Syntax. Deletion detaches exactly one
leaf. CSS recomposes maximal adjacent declaration runs without reparsing unchanged
payloads, preserving individual declaration identities across run splits/merges.
The new list and checked current body publish atomically, with membership/order,
ancestry/provenance and owning-rule `Modification` facts. A failed operation or
publication preserves live contents, identities, captures, revision and effects.
The inherited outer/leaf cssText setters remain no-ops.

`named_supports_body` exposes current CSS recovery metadata. Original occurrences
and sheet diagnostics remain historical evidence. Recomposition preserves the
outer body's local recovery and recomputes the surviving child union; deleting a
candidate removes its descendant recovery. The summary origin is the first local
origin, otherwise the first recovered surviving child. This does not promise
source-byte or recovery round-tripping.

The product cssText policy selects CSS's canonical specified named-support bytes:
`@supports-condition --x { color: red; & { } @future x; }`, including an explicit
space before the closing brace of an empty body. A single native graph operation
serializes current outer/leaf payloads and ordinary ancestors, including opaque
nested candidates, components, work and output. CSSOM preparation separately
counts actual live views/edges and ancestry. Native root depth is zero; CSSOM
counts a live root as one and passes the actual remaining ancestor allowance.
`CssomBatch::rule_css_text` reads bounded staged text for existing committed IDs
without publication or tentative-ID exposure; poisoned batches return BatchAborted.
The staged `named_supports_body` view likewise exposes checked payloads only.
The ordinary parsed CSS SupportsCondition CSSOM writer retains its standards-
qualified unavailable result; the product path explicitly opts into this policy.
This does not implement named-condition evaluation or certify other whole-rule
wrappers or root Window/IDL bindings.

`CssStyleSheet::replace_sync` delegates the recovered replacement algorithm.
Asynchronous replacement is a finite core lifecycle: `replace` publishes the
exclusive modification lock and returns a must-use `CssomStylesheetReplacement`.
Consuming `parse` produces detached, Send-capable bounded CSS parse work;
`finish` publishes recovered replacement and unlocks together.
The exact owner/sheet/operation token is checked independently of publication
revision. A matching completion with stale guard/parser facts, parse resource
failure or adoption failure preserves old rules and returns `Failed(error)` with
the cleanup publication. Foreign/reused work returns an error and cannot release
an active operation. Consuming `cancel` releases only the matching operation.
Pending records consume object/string quotas; begin reserves a terminal revision
and actual history capacity, and unrelated commits preserve those reservations.
No-op publications consume none of that capacity. Successful replacement can
require more resources; its fallback cleanup requires no new identity or rules.
Dropping detached work leaves an explicit owner-side record visible through
`pending_sheet_replacement`; its host must cancel that record. No automatic Store
access from drop, Promise, scheduler or resource loader is implied. Owner drop
removes the live operation; already captured snapshots remain historical values.

`CssStyleDeclaration` and its inherited descriptor/property interfaces provide
`length`, `item`, `parent_rule`, `css_text` and property value/priority reads,
together with source `set_css_text`, `set_property` and `remove_property`. Only
`CssStyleProperties` provides `css_float`. These operations are
separate from the foundation's direct selected-terminal product primitive.
Ordinary/keyframe/margin properties, mixed Page descriptors/properties and FontFace
descriptors use their existing CSS owners. Page `size` is a descriptor; ordinary
width/height cannot supply its missing value. FontFace retains raw occurrences and
an independent unique current selection; descriptor aliases share the CSS kind,
important values are inadmissible, and `src` member recovery remains CSS-owned.
CounterStyle's specialized reflections belong to its separate descriptor owner.

`CssomInputData::Support` contains a concrete `CssomDeclarationSupport` supplied by
the composed owner. It selects complete grammar/descriptor identities, including
the independent SVG terminal separately from its finite legacy alias. Parser
recognition or a conformance catalog never grants usable support. A profile can
promise all checked values or require a decision for each actual whole candidate.
Custom properties bypass usable membership and preserve the exact supplied
semantic name, including punctuation, backslashes, case and NUL. CSS output
escaping does not change live identity.

Prepare, inspect the checked candidates and resolve conditional decisions with
`apply_declaration`; independently revised decision owners use
`apply_declaration_with_inputs`. Tickets and decisions are correlated to their
batch, candidate and captured context. Wrong/stale decisions and unresolved
preparations cannot publish. Dropping a preparation abandons it. Bulk recovery
keeps genuine raw replacement input, effective parser context and diagnostics,
including when all declarations are discarded. Source setters use actual value-only parsers and retain the provenance those
CSS-owned values expose, with no invented parsed name position; retained survivors
keep their original checked occurrences. Parsed/typed ingress remains
honest about its original parser context.

Owner style-attribute notifications accept borrowed UTF8 or shared original UTF16
through `CssomStyleAttributeInput`. Both enter CSS's lossless preprocessing and
raw declaration-block grammar in the block's current context. Preparation takes
explicit UTF16 input limits, declaration-request limits and an interruption poll;
input failures retain their typed cause and poison the batch. UTF8 encoding is
bounded by original units/source backing and polls per scalar. CSS preparation
work and cumulative component/depth/byte admission remain separate from the
opaque general grammar between boundary polls. This is not a whole-operation
live-memory receipt. Unrelated/computed/updating notifications do no input work;
absence keeps the existing parsed empty-replacement branch.

`prepare_created_style_attribute_change` resolves an exact batch-local block
ticket for initialization before the same commit exposes its live ID. Notifications
request no echo owner write. Effect classification compares raw UTF16 units to
the serialized effect without preprocessing; absent and present-empty differ.
`CssomDeclarationInput` readbacks expose text, parser context, diagnostics and an
optional complete parse source. Notification inputs retain that mandatory source
even when empty or entirely discarded, under the canonical source quota, without
an extra String copy. Existing cssText/property setters keep their own UTF8
semantics. Later selected-entry edits preserve the original replacement source
without reconstructing declarations from historical shorthands.

Readonly errors precede source mutation branches. Computed remains independent:
reads/no-ops follow their selected source branches, while a reached owner-update
branch returns `ComputedStyleUpdatePrecondition` before testing for a null owner.
A successful `cssText` setter requires the owner update even for equal/empty text;
absent removal and setter `updated=false` require none. Equal serialized text can
still publish changed occurrence provenance without creating an owner effect.

Bounded nonnull owner facts and nullable owners are supported. Commit reserves
non-clone, single-use `CssomOwnerEffect` tokens correlated with the exact block,
owner version, publication revision and owning input manifest. A binding calls
`begin_owner_application` before its synchronous host write. Its exclusive lease
sets actual transient updating, exposed by `declaration_flags`/lease flags; owning
snapshots preserve their captured flags. Only a notification with that token,
owner, exact style local name, null namespace and exact value is a matching echo.
Unrelated notifications remain owned/queued by the binding and can later enter
`prepare_style_attribute_change`. Qualifying external attribute changes bypass
readonly, replace from real raw contents, and never echo another host write.

The binding reports success or post-publication host failure. Dropping/unwinding
the lease always clears updating and releases the reservation; canceling/dropping
an unused token releases it too. Host failure never rolls back committed CSS.
CSSOM runs no host callback and constructs no Window, Document or runtime adapter.

Trusted snapshot inspection is Rust transport for downstream styling.
`CssStyleSheet::css_rules` and its `rules` alias enforce the origin-clean security
guard. A binding must use this guarded operation when exposing the collection.
Computed and readonly flags are independent; computed specified-block text is
empty, while readonly independently guards edits. Null base URL differs from an
explicit empty base. Host adapters supply facts, not fabricated document objects.

Change summaries own owner, `(from, to]` coverage, category union, affected IDs and
all input endpoints.
Selector admissions correlate each edited rule with its retained input versions,
including endpoints captured before a later context change. Declaration and font
family admission inventories remain independent. Incremental consumers check
complete contiguous matching coverage and inputs. Missing history, foreign
ownership, input drift and conservative
changes request full recomputation from one owning snapshot. CSSOM does not perform
selector matching, cascade, layout, scheduling or resource loading.

`CssomLimits` controls retained objects, entries, host/live UTF-8 string bytes,
per-parse and retained property-input bytes, structural/linked-input depth, identity/revision capacity,
history, pending owner-effect count/bytes/identity and CSS projection work.
`CssomDeclarationRequestLimits` composes input, scans, schema-derived new expansion,
projection, output and final effect formatting. Every CSS provider receives a
disjoint reserved allowance because it exposes no mutable work counter; unused
reservations are conservatively retained. Zero is valid. These are admission/storage/work
quotas; they do not promise an allocator/process-memory bound or recovery from
system OOM. Checked CSS payloads retain their existing provider constraints.

Run the [source declaration consumer](examples/declaration_consumer.rs) for supplied
usable support, source edits and owning old/new captures. Run the
[external snapshot consumer](examples/snapshot_consumer.rs) for direct inspection
and contiguous incremental coverage. [Public tests](tests/foundation.rs) exercise
the owning state and exceptional live cases;
[declaration tests](tests/declaration_operations.rs) exercise source ordering,
domain dispatch, supplied support, atomic failures, effects and provenance.
[Stylesheet tests](tests/sheet_operations.rs) cover source construction,
replacement reservations and bounded terminal cleanup.
[Rule tests](tests/rule_operations.rs) cover lists, guards, contextual selectors,
admission, recovery and associated object identity;
[current graph tests](tests/rule_serialization.rs) cover recursive native output,
selected survivors, membership, lazy capability boundaries and bounded retry;
[condition reads](tests/condition_reads.rs) cover plural/local-name reads and
supplied evaluator-result correlation independently of query evaluation.
[Keyframes tests](tests/keyframe_operations.rs) cover normalized reverse-last
matching, current keyText, invalid no-ops and cumulative search failure/rollback.

`CssFontFaceRule::style` exposes the font-face rule's stable declaration identity
and its named descriptor reflections, including the font_stretch/font_width
alias. Reflected
setters and style's PutForwards cssText prepare through the same checked declaration
owner, support profile, source guards, candidate decisions and atomic publication.
Font loading and host IDL conversion remain separate binding work.

`CssFontFeatureValuesRule` exposes all seven typed maps, including historical
forms. Map setters reuse the existing ordered live map owner.
`CssomFeatureValues` converts a scalar
to one unsigned-long value; empty sequences remain present entries, and map
cardinality checks do not apply the authored descriptor's integer grammar.
The maps retain raw decoded keys, including empty strings and punctuation. Stable
map identities are a product guarantee; the selected source IDL has no SameObject
annotation for these attributes. Clear/delete retain original authored definitions
and prior owning captures. Exact authored values outside u32 remain present and
report a typed conversion limitation instead of narrowing or disappearing.
`keys`, `values` and `entries` return owning ordered iterators; entry conversion
failures remain explicit. Trusted raw records expose `kind`, `raw_entries` and
`authored` separately from projected map operations.

The feature rule's `font_family` getter formats the actual current literal list
with one cumulative CSS writer. `set_font_family` consumes the native
nongeneric family prelude using current staged parser facts: invalid syntax is a
successful no-op, valid EOF recovery remains admitted, and typed resource failure
aborts the batch. `set_font_feature_families` accepts the CSS checked list carrier
without reparsing its values. Equal decoded lists preserve the existing input,
identity and revision. Successful changes retain the carrier's actual whole/member
origins, admission parser context, input versions and recovery diagnostics in
`family_input`; initial authored rules retain their original occurrence separately.
The admission context records current supplied facts, independently of any origin
already carried by typed input. Family admission versions also appear in snapshot
and publication input manifests. Current/detached family entries, decoded UTF-8
bytes and retained new admission data share the store's global product allowance.

`CssFontPaletteValuesRule` exposes readonly name and specified descriptor
strings through its CSS owner. Pending values retain their literal substitution
text; missing
optional descriptors return empty. Complete feature-rule wrapper formatting is
not fabricated when the selected source leaves historical-forms or font-display
placement unresolved.

The [font consumer](examples/font_consumer.rs), [font tests](tests/font_operations.rs)
and [family attribute tests](tests/font_family_operations.rs) exercise these public
owners, including stable identities, raw keys, independent map cardinality,
exact authored conversion limitations, source no-ops/recovery, cumulative budgets,
support decisions, atomic failures and owning captures after store drop.
