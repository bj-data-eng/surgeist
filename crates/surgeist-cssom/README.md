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
This convenience product ingress is distinct from the source stylesheet constructor,
which starts empty; `CssomSheetInputs::constructed` supplies its explicit defaults
and constructor-document facts without inventing a document object.

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
association. Complete #1049 text/selector/property/MediaList operations and async
replacement are separate implementation work. No stub methods stand in for them.
The CSSRule cssText setter is implemented as its standards-defined no-op.

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
and contiguous incremental coverage. [Public tests](tests/foundation.rs) exercise
the owning state and exceptional live cases.
