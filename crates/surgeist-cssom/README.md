# Surgeist CSSOM

`surgeist-cssom` owns mutable authored CSS identity, containment, atomic edits and
immutable captures. It depends only on `surgeist-css` and the standard library.
CSS owns grammar, checked syntax, expansion and formatting. CSSOM owns its CSS
integration adapter under the repository's authorized ownership exception.

Create a `CssomStore`, capture a snapshot and acquire an edit guard from that
capture. Stage parsed or checked typed sheets in a batch; commit resolves private
creation tickets to owner-qualified live IDs. One successful publication advances
one revision. Failed or stale batches publish nothing. Source no-ops and batches
that restore the original observable state preserve the revision. IDs are never
reused within an owner. Dropping a staged ticket/batch cannot publish an identity.
Constructed initial ingestion and recovered replacement filter import roots;
external ingestion retains them. Original checked occurrences stay separately owned.
This convenience product ingress is distinct from `construct_sheet`, which starts
empty with new rule-list/media identities. Its explicit Document capture supplies
fixed location and constructor identity; nullable base URL remains separate.
MediaList constructor input is copied through its actual CSSOM text and reparsed,
including recovered `not all` text. Construction does not assert membership in a
host's styleSheets collection: `set_sheet_list_membership` publishes its supplied
selection/order. Length/item inspect that collection and return null out of range.

The supplied parser context applies to subsequent parsed creation/replacement;
context updates preserve existing checked values and their original parser facts.
Typed ingress preserves the context carried by each checked CSS occurrence.

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

Live counter names are raw strings. `counter_name` applies the CSS identifier
writer; `counter_descriptor` reads any of the ten specified effective descriptor
values and returns empty text for an unspecified descriptor. These partial reads
remain available for intrinsically undefined rules, independently of the selected
whole-rule format's unavailability. `set_counter_descriptor` parses a complete raw
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

Readonly extension facets expose `custom_media_name` and `custom_media_query`
from the checked current rule. The query is an authored boolean or a stable
`CssomMediaListId`, including a genuine empty list; it never defaults to false
or evaluates an alias. Existing MediaList operations edit that associated
collection while old snapshots and original rule occurrences remain immutable.
The readonly query slot and name have no replacement setters. Ordinary and
scoped rule ingress use the same association owner, and retained media objects,
entries and live-name bytes obey product quotas.

`color_profile_name`, `color_profile_src`, `color_profile_rendering_intent` and
`color_profile_components` read checked current ordinary/scoped profile data.
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
`insert_sheet_rule` uses the shared rule insertion owner after stylesheet security,
modification and preliminary syntax/constructed-import checks. That owner checks
index, contextual admission, hierarchy and whole-list namespace state, then adopts
the same classified source occurrence. `add_sheet_rule` supplies the standard
legacy defaults, builds its actual bounded input and delegates insertion, returning
`-1`; absent index means the current end. `remove_sheet_rule` delegates deletion,
defaulting its index to zero. These are CSS-standard aliases, not crate shims.
Complete #1049 rule text/selector operations remain separate work.
The CSSRule cssText setter is implemented as its standards-defined no-op.

`replace_sheet_sync` delegates the recovered replacement algorithm. Asynchronous
replacement is a finite core lifecycle: `begin_replace_sheet` publishes the
exclusive modification lock and returns detached bounded CSS parse work;
`finish_replace_sheet` publishes recovered replacement and unlocks together.
The exact owner/sheet/operation token is checked independently of publication
revision. A matching completion with stale guard/parser facts, parse resource
failure or adoption failure preserves old rules and returns `Failed(error)` with
the cleanup publication. Foreign/reused work returns an error and cannot release
an active operation. `cancel_replace_sheet` releases only the matching operation.
Pending records consume object/string quotas; begin reserves a terminal revision
and actual history capacity, and unrelated commits preserve those reservations.
No-op publications consume none of that capacity. Successful replacement can
require more resources; its fallback cleanup requires no new identity or rules.
Dropping detached work leaves an explicit owner-side record visible through
`pending_sheet_replacement`; its host must cancel that record. No automatic Store
access from drop, Promise, scheduler or resource loader is implied. Owner drop
removes the live operation; already captured snapshots remain historical values.

Common source declaration operations provide `length`, `item`, `parent_rule`,
`css_text`, property value/priority and `css_float` reads, together with prepared
`cssText`/property/float setters and direct source removal. These operations are
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

Trusted snapshot inspection is Rust transport for downstream styling. Source
`sheet_rules` enforces the origin-clean security guard. A binding must call that
operation instead of exposing trusted snapshot membership as its IDL cssRules.
Computed and readonly flags are independent; computed specified-block text is
empty, while readonly independently guards edits. Null base URL differs from an
explicit empty base. Host adapters supply facts, not fabricated document objects.

Change summaries own owner, `(from, to]` coverage, category union, affected IDs and
all input endpoints. Incremental consumers check complete contiguous matching
coverage and inputs. Missing history, foreign ownership, input drift and conservative
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

Fonts views expose the font-face rule's stable declaration identity and its named
descriptor reflections, including the fontStretch/fontWidth alias. Reflected
setters and style's PutForwards cssText prepare through the same checked declaration
owner, support profile, source guards, candidate decisions and atomic publication.
Font loading and host IDL conversion remain separate binding work.

Font feature views expose all seven maps, including historical forms. Map setters
reuse the existing ordered live map owner. `CssomFeatureValues` converts a scalar
to one unsigned-long value; empty sequences remain present entries, and map
cardinality checks do not apply the authored descriptor's integer grammar.
The maps retain raw decoded keys, including empty strings and punctuation. Stable
map identities are a product guarantee; the selected source IDL has no SameObject
annotation for these attributes. Clear/delete retain original authored definitions
and prior owning captures. Exact authored values outside u32 remain present and
report a typed conversion limitation instead of narrowing or disappearing.

The feature rule's `font_family` getter formats the actual current literal list
with one cumulative CSS writer. `set_font_feature_family_text` consumes the native
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

Palette views expose readonly name and specified descriptor strings through their
CSS owner. Pending values retain their literal substitution text; missing
optional descriptors return empty. Complete feature-rule wrapper formatting is
not fabricated when the selected source leaves historical-forms or font-display
placement unresolved.

The [font consumer](examples/font_consumer.rs), [font tests](tests/font_operations.rs)
and [family attribute tests](tests/font_family_operations.rs) exercise these public
owners, including stable identities, raw keys, independent map cardinality,
exact authored conversion limitations, source no-ops/recovery, cumulative budgets,
support decisions, atomic failures and owning captures after store drop.
