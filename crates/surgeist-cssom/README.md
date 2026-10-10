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

Live counter names are raw strings. Seven ordered feature maps accept raw keys and
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
