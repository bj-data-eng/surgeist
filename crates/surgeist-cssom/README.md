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
descriptor blocks have explicit domains. The implemented property profile is the
CSS provider profile (#22/#471); valid symbolic inputs can retain a typed unavailable
projection rather than becoming an empty or partial block. Storage does not imply
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
Complete #1049 selector/property operations remain separate work.
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

Bounded supplied owner facts are supported, including a declaration block's nullable
owner. The foundation performs no host attribute writes and emits no host effects.
Its `updating` read is false; the future #835/#840 synchronous application seam must
own a transient correlated application lease and binding echo suppression, with
cleanup on success, failure and unwind. A published boolean alone is insufficient.

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
history and CSS projection work. Zero is valid. These are admission/storage/work
quotas; they do not promise an allocator/process-memory bound or recovery from
system OOM. Checked CSS payloads retain their existing provider constraints.

Run the [external consumer](examples/snapshot_consumer.rs) for direct inspection
and contiguous incremental coverage. [Foundation tests](tests/foundation.rs) and
[stylesheet operation tests](tests/sheet_operations.rs) exercise owning state,
source construction/recovery, stable identities and bounded terminal cleanup.
