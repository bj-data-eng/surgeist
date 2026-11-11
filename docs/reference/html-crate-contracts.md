# HTML crate contracts

This reference records the boundary selected for [#1174](https://github.com/bj-data-eng/surgeist/issues/1174)
under [#1173](https://github.com/bj-data-eng/surgeist/issues/1173). It is an
implementation contract for the three-package extraction. The current manifests
and source implement those package boundaries; capability claims remain limited
to the owning contracts and executable evidence below. The source basis is
`11fea87cca1674b93dc2bef57bb138ff55570cfe`; subsequent fixes in the same
contribution must preserve the boundaries below.

`surgeist-html` owns language values, local algorithms, native construction and
immutable construction results. `surgeist-html-semantics` owns qualified captured
facts and semantic query sessions. `surgeist-html-parser` consumes both to ingest
decoded Unicode and publish those same construction results with parser evidence.
The [selected language edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
and the existing capability restrictions remain authoritative.

## Dependency direction

The selected production graph is:

```text
surgeist-html-parser -> surgeist-html-semantics -> surgeist-html
surgeist-html-parser --------------------------> surgeist-html
surgeist-html-parser -> surgeist-css -> surgeist-color
surgeist-html-parser -----------------> surgeist-color
surgeist-html --------------------------------------> surgeist-color
```

HTML has no dependency on parser, semantics, CSS, DOM, or runtime. Its only
dependency is `surgeist-color`, used solely for the opaque named-color table of
the HTML legacy-color recovery; it does not give HTML ownership of CSS color
resolution or numerical color operations. Semantics has no parser or CSS/Color
dependency. No fourth support crate or external dependency is needed.

The parser-to-semantics edge follows concrete production use:

- `request.rs` retains `HtmlCapture`, checked document identities and
  `HtmlFragmentContext`; document request selection reads captured scripting and
  image policy.
- `construction/parser_session/context.rs` uses captured documents, registry
  scopes, definition lookup, template-document caches, host associations, and
  attachment eligibility while constructing real parser records.
- `parser/tree_builder/foreign.rs` reads the actual fragment context element's
  name and attributes, including integration-point classification.

Two alternatives were rejected. Moving the capture into HTML would give the
value/native-construction owner the qualified host/tree model and its query
machinery. Replacing it with a parser-specific frozen context would duplicate
capture qualification and either eagerly read unreached facts or recreate the
same registry, template-cache and host model. Keeping one capture and depending
on its owner preserves lazy reads and typed missing-fact errors.

## Source and public-surface ownership

Each package has one root `src/lib.rs` front door. Existing public symbol names
remain with their semantic owner; old-package reexports are removed. A package
exposes only the types it owns: parser and semantics never reexport or alias an
HTML-owned type, even when it appears in their signatures, and consumers import
it from `surgeist_html` alongside the sibling operation owner. The
controlled primitive surface below is an intentional additional `bounded`
module in HTML, not publication of every private source module.

| Source at the basis | Destination and owned contract |
| --- | --- |
| `values.rs` | HTML: `DocumentString`, `Namespace`, `QualifiedName`, `Attribute`, `OrderedAttributes`, exact UTF-16 and first duplicate expanded-name validation |
| `vocabulary.rs`, `attributes.rs`, `attributes/`, `numeric.rs`, `numeric/`, `temporal.rs`, `tokens.rs`, `interpretation.rs` | HTML: vocabulary, local author/recovery interpretation and bounded numeric/temporal/token primitives |
| `input.rs`, `control_values.rs`, `details_rules.rs`, `nonce.rs`, `custom_names.rs`, `script_type.rs`, `rel.rs`, `media.rs`, `color.rs`, `pi_pseudo_attributes.rs`, `template_markers.rs` | HTML: state/value rules given explicit operands; no captured-tree traversal, parser records or semantic session |
| `construction.rs` | HTML: document mode, native builders, node kinds, opaque result-local IDs, generic tree/association invariants and immutable results |
| `capture.rs` except local fragment decisions | Semantics: capture IDs, builders, validated facts, registry/definition identities, fragment selection and capture-qualified queries |
| `semantics.rs`, `semantics/` | Semantics: all semantic inputs, external producer revisions, context qualification, requests, reports, errors, limits, cancellation and sessions |
| `bidi.rs` and Unicode data/generator | Semantics: directionality's pinned Unicode interpretation; no parser dependency |
| `base.rs` | Semantics: URL/CSP observations, `HtmlParseBaseFacts`, captured base facts and their qualification; parser consumes these existing explicit observations directly |
| `source.rs`, `request.rs`, `settings.rs` | Parser: decoded source and coordinates, parse requests, parse limits and cancellation |
| `parser/` including tokenizer, references, tree builder and control | Parser: source preprocessing, ingestion, recovery, terminal state and accounting |
| `construction/parser_session.rs`, `construction/parser_session/` | Parser: controlled mutation orchestration, HTML construction effects, metadata, diagnostics, provenance, history and report assembly |

The name `HtmlParseBaseFacts` does not require moving its validation into parser:
the object attests external URL/encoding/CSP facts, and `semantics/base.rs` also
consumes the corresponding base model. Shared exact strings and value-domain
types always come directly from HTML, including those mentioned in parser and
semantics signatures.

### Shared exact strings and ordered attributes

`DocumentString` is a 16-byte handle: a shared `TextBlock` of UTF-16 code units and the range
of the block it covers. Cloning copies the handle and shares the block. Equality and hashing
compare the code units, so equal content in different blocks or at different positions is equal.
Isolated surrogates, nulls and whitespace remain exact. There is no parser or semantic
replacement string type and no conversion through UTF-8 at an inter-crate boundary.

`DocumentString::from_utf16` and the `From<&str>` and `From<String>` conversions build a private
block holding exactly the string, one allocation for an authored string. `TextBlock::string`
returns a string over a range of a block and rejects a reversed range, a range past the block and
a range that does not fit the handle. A string over a whole block has no range limit.

Every sequence is valid in this value domain, so adopting backing needs no textual validation.
The accessors that let the parser's allocation ledger and the DOM's accounting qualify the actual
allocation without a second copy or a fabricated capacity are named for their role:

- `shares_storage_with` reports that two strings are the same range of the same block;
- `retained_units` reports the capacity of the block a string keeps alive, spare capacity
  included;
- `block` lends the `TextBlock`, which reports its holders, hands out a weak reference and an
  address, and indexes its units by range;
- `unique_units_mut` lends the units of a string that covers its whole block when nothing else
  holds the block, for the construction owner to grow or truncate in place;
- `to_shared_utf16` hands a string over its whole block as the block's own `Arc<Vec<u16>>`
  without a copy and copies a partial range, and `from_shared_utf16` adopts such a vector without
  a copy, for interfaces that still take `Arc<Vec<u16>>`.

A string never exposes mutable access while another holder can observe it.

`OrderedAttributes::shared_attributes(&self) -> &Arc<Vec<Attribute>>` lends the
actual immutable backing and retained capacity without a clone or validation
pass. Consuming `into_shared_attributes(self) -> Arc<Vec<Attribute>>` returns
that same backing. These observations let the consuming parser qualify its
existing allocation tickets; mutable construction backing remains private.

`OrderedAttributes::from_parser_storage`, `parser_backing_mut` and
`into_parser_storage` must not expose `ParseControl` or mutable shared storage in
HTML. Replace them with HTML-owned `bounded::values::HtmlAttributeStorage` and
`bounded::values::HtmlAttributeValidation`. Storage retains the actual `Arc<Vec<Attribute>>`;
its public view is immutable. A validation cursor compares expanded names in
the existing duplicate-major/first-minor order and compares exact namespace URI
before exact local name. Prefix and value never affect duplicate identity.

`HtmlAttributeStorage::validate` compares every pair of names, which is quadratic in the
attribute count and suits a bounded count. `validate_indexed(HtmlAttributeNameIndex)` reaches
the same verdict, the first attribute whose expanded name repeats an earlier one with that
earlier attribute's index, in work linear in the count and the total length of the names: see
[name index](#attribute-name-index). The cursor's layouts then also report the index slots a step
writes and bytes it copies (`record_writes()`, `copied_bytes()`). `take_index()` gives the index
back before `finish`, which consumes the cursor.

The cursor exposes `next_action()` and an action's `layout()`/`advance()` pair.
The parser admits each record/string comparison before advancing it.
`HtmlAttributeValidation::finish(self) ->
Result<HtmlValidatedAttributes, HtmlFinishFailure<Self, HtmlAttributeValidationError>>` consumes the
cursor only when validation has completed successfully. `NotReady` and the first
`Duplicate { first, duplicate }` refusal hand the same cursor/storage back in the
`HtmlFinishFailure`; cancellation/admission failure likewise retains that owner.
`abort(self) -> HtmlAttributeStorage` returns its actual storage without further
validation, allocation or successful attribute publication.
`HtmlValidatedAttributes::into_attributes` is infallible and shares the original
backing. Source-order insertion, merge and replacement are checked operations on
`HtmlAttributeStorage`; they invalidate an earlier validation state. No mutable
`Arc` or validation flag is supplied by the caller.

Native `OrderedAttributes::new` retains its current first-duplicate behavior.
Native and controlled construction use the same expanded-name invariant. A
private proof or a public constructor accepting arbitrary parser assertions is
not an acceptable substitute.

### Shared immutable construction publication

The important crossing is `ParserArenaSession::freeze`: today it destructures
HTML's private `Identity`, `Record`, `Arena` and directly constructs
`HtmlDocument`/`HtmlFragment`. Moving only this method or making those fields
public would leave HTML's immutable-result invariant unenforced.

HTML therefore owns `bounded::construction::HtmlConstructionStorage` and
`bounded::construction::HtmlConstructionOperation`. They wrap the existing record vector,
identity and generic usage, with private fields. Both native builders and the
parser's session use this same storage implementation; parser-only payloads,
source evidence, allocation tickets and effects stay in parser. No second final
arena is built by walking or copying the parser tree.

Storage issues the existing opaque `HtmlBuilderNode` handles qualified by its storage
serial. Parser-private `ParserNode` becomes a wrapper around that checked handle rather
than duplicating the index/serial invariant. A handle is a `Copy` pair of the
storage's process-wide serial and a 32-bit record index; it holds no reference to
the storage, and a handle of another storage is a serial mismatch.
It provides read-only `node`, `kind`, `parent`, `children`, `template_contents`,
`shadow_root` and `usage` views. Its operation enum provides exactly these
generic transitions: append/insert a node, detach/reparent an ordinary node,
replace content, replace validated attributes, edit an element's attributes
through the complete controlled group below, attach template contents, attach
a shadow fragment, and select the output document/fragment root.
Parser-specific select, form, radio, input, nonce, script and template-patching
decisions remain parser operations that compose these generic transitions.
For the parser's dense retained-record joins, add checked
`node_at(position: usize) -> Option<HtmlConstructionNode<'_>>`, a view's `handle()`,
and `position(&HtmlBuilderNode) -> Result<usize, HtmlConstructionError>`. The view
borrows storage; operations still accept only owner-issued handles. Positions
are local read indices, not result identities or allocation-ID constructors.
Out-of-range and foreign inputs cannot mint valid handles. The parser admits
the actual fixed header/identity reads before these calls, retaining its current
ordinal metadata banks without a second identity map or hidden arena scan.
`node_by_result_id(&HtmlNodeId) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult>`
resolves only an ID previously issued by this storage. It rejects foreign and
out-of-range IDs; it neither constructs a result ID nor publishes an immutable
result. This checked read follows existing owning-association IDs without
exposing their private identity/index or adding another mapping bank. Borrowed
node views provide kind, position, parent, child iteration, contents kind/target,
`fragment_owner() -> Option<&HtmlFragmentOwner>`, shadow properties and issued
handle access. The published result's `HtmlNode::fragment_owner()` returns
`Option<HtmlFragmentOwner>` by value. Following one read edge copies a 16-byte id
and clones nothing; parser admission covers its actual described observation.

Each operation consumes one exclusive storage owner. A prepared
action exposes its actual next inspection, copy, initialized write, reservation
and fixed-transfer layout; the caller admits it before `perform`. Performing it
cannot be replayed or applied to another storage. An action requiring allocation
uses `try_reserve` and reports requested and actual capacities. The native
builder drives the same operations with `HtmlConstructionLimits`; the parser
drives them with its existing `ParseControl` and resources. HTML has no parser
controller, cancellation handle or second execution ledger.

HTML checks foreign handles, permitted parent kinds, ordinary versus association
edges, correct template/shadow endpoints, single owning associations, root
restrictions, cycles and reciprocal parent/child links. Ordinary template contents
and local shadow attachment are owning associations. A declarative template's
contents reference to a shadow fragment is non-owning and independently checked;
it is not an ordinary child or a second owner. The reference is also independent
of the template's ordinary position: a template whose contents became a declarative
host's root is never inserted by the parser (parser template start tag steps, lines
4026 to 4042 of the [retained syntax reference](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md)),
and the DOM lets a retained template be inserted under any element, so its parent
may be the host, another element or nothing, and neither the construction import
nor the capture builder relates it to the root's host. A transition either commits a coherent
generic state or leaves the prior logical state unchanged. Each direct attribute edit
is such a transition; the appends of a merge do not roll back their already committed
prefix on a later stop. Any reserved capacity
remains observable to the caller's actual allocation owner on failure. Temporary
parser algorithm state belongs in its session, not in a published invalid HTML
record. HTML never applies parser placement, recovery or language conformance
rules to native authoring.

The native builders' `finish(self)` drives this publication and returns
`Result<HtmlDocument | HtmlFragment, HtmlConstructionError>`; a refusal ends the session. The
builder keeps its storage through every refused step of an append.

A native append that finds the record bank or the parent's child order full grows it to the
amortized capacity of the bank, from a minimum, so `n` appends change each capacity `O(log n)`
times. The target never exceeds the node limit for records or the edge limit for a child order, and
never falls below what the append needs. Unlike the amortized capacity of the bounded consumers,
which never depends on a limit, the cap is the admission: the native builder has no
capacity-bytes limit and admits nothing beyond its node and edge limits, so spare room above them
could never be used. The record bank grows through a `ReserveRecords` command and the child order
through `perform_with_capacity`. An allocator refusal of the spare room retries the least capacity
that suffices, so an append fails only where growing by one slot fails.

`HtmlConstructionStorage::prepare_finish(self, root: HtmlBuilderNode) ->
HtmlConstructionOperation` is the `Publish` command's consuming convenience
entry point. It starts a publication operation and cannot fail, because it
inspects nothing. It checks that the selected root is the proper document/fragment
kind, that all retained owning links are reciprocal and in range, that
association roots have the right owner, that each non-owning contents reference
targets a live result-local shadow-contents fragment, and that no pending generic operation
exists. Its bounded actions use the original controller. The ready operation
retains the original storage until every required check and arena-sharing
allocation is admitted. The consuming `try_ready` transition below produces the
distinct ready owner. Its `commit` consumes that owner and returns
the published `HtmlConstructionResult::{Document(HtmlDocument), Fragment(HtmlFragment)}`
and recovery lease with no later fallible work. Invalid structure yields
`HtmlConstructionError` and the
same owned storage. A stopped operation exposes no successful document or
fragment.

Result identity is the storage's process-wide serial, issued once with storage
creation, and survives publication,
cloning, associated fragment views and parser report assembly. It is never
replaced by source identity, capture identity or a canonical live DOM ID.
`HtmlNodeId` compares and hashes by serial and index and has no order, so no
charged amount, output or ordering may depend on the value of a serial. Exhausting
the serial counter, or a record count that a 32-bit index cannot name, is
`RepresentationExhausted`.
Published metadata and readback use the same result-qualified `HtmlNodeId`;
private construction handles cannot be confused with immutable IDs. Parser-private
metadata may retain dense bank ordinals qualified through the actual shared
storage views. Those ordinals cannot construct identities or authorize mutation. Add
`HtmlConstructionStorage::result_id(&HtmlBuilderNode)` for checked final-ID
qualification; it does not publish a node view or a partial result. All existing
`HtmlDocument`/`HtmlFragment` read APIs retain their behavior. In particular,
`template_contents()` returns the actual shadow fragment for a declarative
template exposed through parser metadata; it must not substitute empty ordinary
contents. Add `HtmlTemplateContentsKind::{Owned, ShadowReference}` and
`template_contents_kind()` beside that target view. `HtmlFragmentOwner` continues
to describe only owning `Template` and `ShadowHost` relationships. Add
`is_shadow_contents()` to distinguish an explicitly created shadow-role fragment,
including the locally unowned output for a captured-host attachment. Its
`shadow_properties() -> Option<HtmlShadowRootProperties>` reads the actual five
properties stored once with that fragment role. The host's private shadow link
retains its existing qualified endpoint IDs, without another properties field.

Node content is read through views returned by value. `HtmlNode::kind()` returns
the output-only `HtmlNodeKind<'a>` (`Document(mode)`, `Fragment`,
`Element(HtmlElementView<'a>)`, `Text`, `Comment`, `ProcessingInstruction`,
`Doctype`), whose text payloads share the node's stored strings.
`HtmlElementView` reports `name()`, `attributes()` and `link_flags()`.
`HtmlAttributes<'a>` reports `len`, `is_empty`, `get(index)`, `iter`,
`find(namespace, local_name)` and `to_ordered()`, in producer order.
`HtmlConstructionNode::kind()` returns `ConstructionNodeKind<'a>`, whose text is
`BuildStr<'a>` (borrowed code units with `code_units`, `len` and `is_empty`); the view hands
out no shared string of a non-element record.
`HtmlDeclarativeShadowRoot<'a>` becomes a read-only borrowed view over those IDs
and the target's properties; `host()` and `contents()` return the `Copy` IDs by
value, and `properties()` copies the same value without allocation. This
pre-v1 shape change has no compatibility wrapper. Captured-host shadow properties
therefore survive in the same HTML result even when no local host is present.

The alternative of replaying parser output through native append-only builders
was rejected because reparents, foster parenting, detached records and
association state would need a second representation, copy and identity map.
An unchecked parts constructor was rejected because a public immutable result
would cease to certify its own invariants. Controlled shared storage keeps
generic invariants with their owner while parser effects remain in parser.

#### Direct attribute edits

An attribute edit is one method of `HtmlConstructionStorage`, reached through
`bounded::construction`. Each method qualifies its element, reads and compares first,
charges its consumer for the work before it runs, asks the consumer once to admit the
growth of the list, and only then changes the list. A refusal leaves the records, the
attributes and the usage as they were, with two residues. The name index is a cache of the
names of one element: an edit may rebuild it before a later refusal, and the index then
describes the element as it is, which a refused edit did not change. When the allocator
refuses the room that was admitted and the consumer then refuses to record the capacity the
list kept, the list holds the room the fallback reserved; no attribute changed, and the
consumer's refusal is its stop. No
mutable record, `OrderedAttributes`, `Arc` or `Vec` is exposed, and a list that another
owner holds is refused before anything changes. HTML owns expanded-name validity; the
consumer owns HTML-language selection, recovery, provenance, effects and the policy of
its allocation tickets.

| Method | Edit |
| --- | --- |
| `reserve_attributes` | Makes room for further attributes; admits the growth once when the list is full and reports the dimensions afterwards. |
| `append_attribute` | Appends one attribute whose expanded name repeats none of the element's; a duplicate is refused with the duplicate-attribute cause before the attribute is taken. |
| `append_planned_attribute` | Appends the attribute a merge plan kept for a candidate. The name is looked up through the index like any other, so a name the list already holds is refused as a duplicate. When the lookup finds no attribute with the name, the attribute is the candidate's own backing, and the entry the plan left for the position is in the index, that entry stays and no new one is entered; otherwise the index no longer describes the element and the next edit rebuilds it. |
| `replace_attribute` | Replaces the attribute at an index and returns the old one; the new name must repeat no other attribute. |
| `remove_attribute` | Removes the attribute at an index, shifts the later ones down and returns it; the shift is charged in quanta before it runs. |
| `plan_attribute_merge` | Decides which candidates a merge keeps, the first of equal names winning, reports each kept candidate to the consumer in order and changes nothing. |

The consumer of a growing edit implements `HtmlAttributeAdmission`: a `Charge` that also
admits the growth of one list (`admit_growth` receives the dimensions before and the least
capacity that holds the edit, and returns the capacity to reserve, which cannot be smaller)
and records its capacity (`record_growth`). The capacity chosen is recorded before the list
is reserved, so a refusal there leaves the list as it was. It is recorded again after the
reservation only when the capacity reached differs from the one chosen: when the allocator
refused the spare room and the list holds the room that is needed, when it overshot, or when
it refused everything and the capacity is unchanged. The consumer of a merge plan implements
`HtmlAttributeMerge`: the candidate count, each candidate's null-namespace local name, and
where the kept decisions go. The parser grows a list to the greater of the length the edit
needs and double its capacity, so the capacity depends on the list alone and never on a
limit. A merge reserves once for all its kept candidates before the first append, so a
single merge into an empty list leaves no spare room, and merges of one attribute each
double the list and cost copies in proportion to their number.

An append moves one attribute and reserves only when the list is full, so the bytes it
copies do not depend on the list length. A merge of `m` candidates into a list of `n`
attributes plans in work linear in `n + m` and the lengths of the names, then appends the
kept candidates one by one.

After each committed append the parser performs its location/redirect progress and the
form or initial-attribute effects before the next append. Every committed prefix is a valid
ordered collection; a stop between appends does not roll back the prefix already appended.
Available capacity is never evidence that a name is valid.

Checked installation validates the final effective expanded name against the actual
current prefix after all preceding semantic effects and name adjustments. A duplicate
produces the typed refusal without appending, consuming the incoming attribute or running
its effects; the generic owner does not repair or choose a lexical recovery. The normal
parser retains its source-defined duplicate recovery before installation. Native,
foreign/adjusted and generated producers must each either supply actual HTML-owned
evidence for those exact operands and current prefix or execute these checks. A tokenizer
assertion or an earlier check before a name-changing effect is insufficient. Without a
name index, checking successive prefixes costs quadratic name-pair work, measured from the
real input size and exact-name comparison; with one, the check is a probe of the few names
that share a hash: see [name index](#attribute-name-index). Do not hide a batch scan or
raise a blanket limit to accommodate the change.

<a id="attribute-name-index"></a>

##### Name index

`bounded::values::HtmlAttributeNameIndex` holds the attribute-name table of an edit or a
validation: the shared `HtmlNameIndex` (an insert-only open-addressed index hashed under one
random key for the whole process, a power-of-two slot count and at most half the slots in use) over slot storage
(`Vec<HtmlNameIndexSlot>`) that the consumer reserves and accounts. The hash is that of the
exact expanded name (namespace URI units, then local-name units; prefix and value never count).
Each entry records the position of a stored attribute, or of an attribute a merge plan means
to append, and also keeps 32 bits of the name's hash, so a probe reads a name only where those
bits equal the sought name's. The table never decides equality: a probe hands the occupant back
and the edit compares the two names, charging each comparison before it reads. Every clearing
chunk, hash, probe step and comparison is charged before it runs.

The consumer's slot storage has at least `HtmlAttributeNameIndex::slots_required(entries)` slots
of capacity; it adopts them with `new`, passes the index to an edit as `names` or to a validation
(`validate_indexed`), and takes it back with `into_storage` to release it, or with
`HtmlAttributeValidation::take_index`, `abort_with_index` or `finish_with_index`. The owner never
allocates the table.

The index describes the first `len` attributes of one element of one storage as they were at one
revision, followed by `planned` entries for attributes a merge plan has not appended yet. The
storage keeps a serial of its own and a revision that every committed change to an attribute list
advances (an append, replacement or removal, with an index or without, and the
`ReplaceAttributes` operation, and the rollback of appended records). An edit that finds the
serial, element, revision, length or planned count different rebuilds the index before it looks a
name up, so an edit that held no index, or held it for another element or storage, cannot leave a
stale index trusted. The revision is advanced where an element's attribute list is taken for
change, not by each caller: in-place changes go through
`OrderedAttributes::construction_backing_mut`, which takes the revision to advance, and
whole-list replacement through `replace_in_construction`; reserving capacity changes no
attribute and does not advance it. Without enough capacity an edit compares every name; the
verdict is the same and the work quadratic. Verdicts equal the pairwise ones, including the first
reached failure: a repeat is reported against the single earlier attribute that owns the name, at
the attribute that repeats it. A table that cannot serve a step refuses with the
`IndexUnavailable` cause: `HtmlConstructionCause::IndexUnavailable` for an edit and
`HtmlAttributeValidationError::IndexUnavailable` for a validation.

Three paths use the index:

| Path | Use |
| --- | --- |
| `append_attribute` | Looks the appended name up and enters the new position in the empty slot the probe reached. The index keeps describing the element across appends. Replacement and removal make it describe nothing, and the next edit clears and refills it from the stored attributes. An edit of another element does the same. |
| `plan_attribute_merge` | Clears and fills the index with the installed null-namespace names, then probes each candidate: a match with an installed name or an earlier kept candidate marks it not kept, and a candidate with no match is entered at the position it will take. The appends of the plan then find the index already describing them. |
| `HtmlAttributeStorage::validate_indexed` | Enters each name in order after probing for it. |

Clearing, hashing, probing and inserting are charged as `Inspect`, `Compare` and
`RecordWrite` work (slots written and bytes copied per 256-slot clearing chunk, record visits and
inspected bytes of the lengths and units hashed, one slot read and one write per probe slot).

A merge compares the same names in the same order as the pairwise comparison: for each candidate
in source order, the installed attributes up to the first null-namespace/local-name match, then
the earlier kept candidates. Exact UTF-16 comparison and short-circuit order are those of the
pinned algorithm. Prefixes and values do not determine duplicate identity. With a name index the
plan probes instead of inspecting every installed attribute and earlier candidate; the decisions
are identical. The plan's decisions belong to the consumer, which keeps them as
`HtmlAttributeMerge::keep` receives them. `append_planned_attribute` skips the comparison only
when the attribute's name is the candidate's own backing and the index still describes the
element as the plan left it; any other attribute is appended as `append_attribute` appends it,
so a decision is never proof for an attribute it was not made for.

A failed edit hands nothing back: the incoming `Option<Attribute>` keeps its value, and the
element's list, capacity and usage are as before. Stopping between the appends of a merge
leaves the consumer to release the staged attributes it still holds.

#### Direct text appends

`HtmlConstructionStorage::append_text(&mut self, node, units: &[u16], admission) ->
Result<usize, HtmlConstructionTextError<A::Stop>>` appends the units to the data of the text
record `node` and returns the length the text had, the offset of the first appended unit. It takes
no `DocumentString` and no record: the input is the units and the node handle, so it survives any
storage layout. `HtmlTextAdmission` is the consumer: a `Charge` for the work, `admit_append`
(called once for every append that adds a unit), `record_growth` and `record_units`.

The append qualifies the record (a text record whose block no other string holds, or the
shared-allocation cause refuses it), charges the work of the copy in quanta of 256 bytes, then
admits itself once. `HtmlConstructionTextGrowth` reports the text's length and capacity, the units
added and the least capacity that holds the result; `grows()` is true when the text is full. A
text with room reserves nothing and records no capacity. A full text reserves the capacity the
consumer chose, which must be at least the minimum (the parser chooses the greater of the minimum
and double the capacity, so the units copied by any number of appends are linear in the units
appended), records that capacity before it reserves, and records the capacity reached when the
allocator refused the spare room and the reservation fell back to the room that is needed, or
overshot. `record_units` follows the reservation and precedes the first written unit; the units are
then written without a further fallible step.

A refusal before the units are written leaves the text, its length and the storage's usage as
they were. Two residues remain and nothing else: when the allocator reserves more room than was
admitted, or refuses the admitted room and the fallback reserves the room that is needed, and the
consumer then refuses to record the capacity the text reached, the text keeps the capacity it
reached while the consumer holds the capacity it recorded before; and when the consumer refuses to
record the units, the text keeps the room it reserved. No unit of the text changed.

The parser's admission acquires the appended units as `ManagedUtf16Units` and holds them, with
the bytes of every non-element string, in two counters of its construction owner, released once
with the owner. The parser appends to the text of its record in place, so a text holds the units
of its data and nothing else: the tokens it absorbed are not stored strings, and the history of
the text refers to their units as ranges of the text. A refusal of the append releases the
token's string and leaves the text, the usage and the units the parser holds as they were. Three
residues remain: the text keeps the capacity that a growth reserved and recorded before the
refusal, and the bytes counter and the ledger hold the capacity last recorded, which is the
text's capacity unless the second record of the growth was refused (then the text holds the
capacity it reached and they hold the capacity that was chosen); a text that had no group has the
group of no source origins that the merge made for it; and the evidence graph keeps the groups
the merge made and the retirement of the replaced group, which no field names. The leaf of the
token's group in the merged group names units the text does not hold, because the refused append
would have written them; the history readers refuse such a leaf as the wrong token. A parse that
refuses an append stops and publishes none of them.

#### Origins of record strings

A parse that records the origin graph keeps the origin group of each string field of a record in
one table of the parser: one sorted run of entries for each record that has a group, found
through a slot for each record position, so that an edit of one record reads and moves that
record's entries alone. The fields are the name, value, namespace and prefix of an attribute, and
the strings of the non-element records: the data of a text or comment, the target and data of a
processing instruction, and the name, public identifier and system identifier of a doctype. A
parse that records no graph allocates no table. The string of a non-element field is the record's
own: no registry row holds it. A leaf of a group in such a field names the units `offset..` of the
record's string, which is valid for as long as the record holds the string, because a text only
grows at its end and the other strings never change; the parser issues no command that replaces
the content of a record. The binding of the groups of a created record follows its creation, so a
refusal of a binding leaves the record without the group; the parse stops and publishes no record.

#### Construction operation payloads and failures

The public operation signature is
`HtmlConstructionStorage::prepare(self, HtmlConstructionCommand) ->
HtmlConstructionOperation`. Preparation inspects and allocates nothing, so it
cannot fail; the operation's first action qualifies the operands, and a refusal
surfaces from the action, after which `HtmlConstructionOperation::abort`
returns the unchanged storage and the unconsumed command without allocation.
Operations consume storage rather than borrowing a vector or returning mutable
record references.
`HtmlConstructionStorage::begin() -> HtmlConstructionInitialization` creates an
allocation-free initialization owner. It uses the same action/observation
sequence to create the zero-payload shared identity.
`HtmlConstructionInitialization::try_ready(self) ->
Result<HtmlConstructionReadyInitialization,
HtmlConstructionRefusal<HtmlConstructionInitialization>>` consumes
the initialization owner only after all initialization actions and its
post-provider observation are complete. It issues the storage's process-wide
serial; when no serial is left it refuses with `RepresentationExhausted` and
hands the initialization back unchanged. The ready owner's infallible
`commit(self) -> HtmlConstructionStorage` returns empty storage. Pending or
failed initialization cannot call that method. Native document/fragment builders then create their
explicit root through the commands below. Failed or cancelled initialization
retains its owner. `HtmlConstructionInitialization::abort(self) ->
Option<HtmlConstructionRetired>` drops any actually created, unescaped identity
owner and returns its zero-payload receipt; a not-yet-created identity returns
`None`. A ready initialization always holds an identity, so
`HtmlConstructionReadyInitialization::abort(self)` returns the receipt itself.
Abort cannot manufacture usable construction storage.

`HtmlConstructionCommand` is a closed, non-exhaustive enum with the following
payloads. Every handle belongs to the storage; all payload fields below are
command operands, never mutable arena fields.

| Variant | Payload and result |
| --- | --- |
| `Create` | `leaf: HtmlConstructionLeaf` (`Text`, `Comment`, `ProcessingInstruction` or `Doctype`); creates a detached leaf record and returns its `HtmlBuilderNode`. Elements, Documents and fragments have their own commands, so a leaf cannot name them. |
| `CreateDocument` | `mode: HtmlDocumentMode`; creates a detached document root and returns its handle. |
| `CreateFragment` | No payload; creates an unassociated fragment root and returns its handle. |
| `CreateShadowContents` | `properties: HtmlShadowRootProperties`; creates a fragment explicitly marked as shadow contents, retaining its actual properties once, and returns its handle. This role is owner-controlled, not a caller-supplied validation flag. |
| `Insert` | `node`, `parent`, `before: Option<HtmlBuilderNode>`; inserts or reparents an ordinary record; `None` means append. A validated `before == node` request is a structural no-op. Other same-position requests perform ordinary insertion processing; unchanged logical order does not guarantee unchanged storage or successful admission. Parser-owned hooks are independent of the structural shortcut. |
| `Detach` | `node`; removes an ordinary edge, retaining the record and its descendants in storage. Detaching an already-detached ordinary record is a no-op. |
| `ReplaceContent` | `node`, `content: HtmlConstructionContent`; replaces only text/comment/PI/doctype payloads or an element's qualified name. The closed content enum preserves node kind; roots, topology and associations cannot be replaced by this operation. An element's new name must keep its template-ness, an HTML `template` expanded name exactly when the record has template contents, or the operation is refused with `InvalidTemplateAssociation` (the `ReplaceContent` arm of `command_phase` in `operations.rs`, from the pinned HTML rule that only a `template` element has template contents). Link history survives only while the new name is an HTML `link` expanded name and is otherwise cleared (the `ReplaceContent` arm of `HtmlConstructionReadyOperation::commit`, from the `CreateElement` rule that flags need an HTML link name). |
| `ReplaceAttributes` | `element`, `attributes: HtmlValidatedAttributes`; replaces one checked ordered collection without bypassing duplicate validation. |
| `AttachShadow` | `host`, `contents`; associates an unowned shadow-contents fragment with an element, using that fragment's retained properties. A shadow root stays attached to its host: neither the DOM nor the HTML parser removes one, and no command does (DOM [shadow tree](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md) line 249, a shadow root is always attached through its host, and [shadow root](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md) line 2731, its host is never null; parser template start tag steps 7.11.2 to 7.11.6, lines 4026 to 4042 of the [retained syntax reference](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md), which only attach). |
| `AttachTemplateShadow` | `template`, `host`, `contents`; atomically releases the template's old owning association, installs local host shadow ownership of the new shadow contents, and installs the template's non-owning reference, without a transient extra owned edge. |
| `SetTemplateShadowContents` | `template`, `contents`; validates an HTML template with ordinary owned contents and a live unowned shadow-contents fragment, atomically releases that owning template association and installs a typed non-owning contents reference. |
| `SetDocumentMode` | `document`, `mode: HtmlDocumentMode`; parser prologue selection changes only the actual document mode. |
| `SetLinkFlags` | `element`, `flags: HtmlLinkFlags`; installs the local rule's actual link flags only on an HTML link element. Source-qualified history remains parser metadata. |
| `Publish` | `root`; selects a document or fragment view and freezes the complete retained storage. |

Insert validates storage-qualified handles, parent kind, movable node and
reference-child membership before its structural shortcut. An already-last
append or insertion immediately before the current successor is ordinary
insertion: it edits the parent's actual child order like any other placement,
so unchanged logical order does not skip depth and edge preflight or the
commit's described writes. Native and controlled callers use that same
operation; parser form/nonce hooks remain parser-owned.
This follows the [unchanged-order determination in #1174](https://github.com/bj-data-eng/surgeist/issues/1174).
It does not change Detach or other explicitly specified no-op cases.

Insert, move and Detach edit the parents' actual `Children` banks in place at
the atomic commit, and nothing is written to a child order before it. A refused
or aborted operation therefore leaves every order, parent link and logical fact
unchanged, and no pending child-order bank exists. Planning finds each place by
walking the order from both ends in turn, starting at the last entry, one entry
per `Inspect` action: the moving node's place in its old parent, then the
reference sibling's place in the new parent (`before: None` needs no walk). A
walk costs about twice the entries between the target and the nearer end of the
order. Each `Children` bank is a deque, so the commit shifts only the entries
between the place it closes or opens and the nearer end: for a removal at
position `p` of `n` entries the lesser of `p` and `n - 1 - p`, and for an
insertion at `p` among `n` entries the lesser of `p` and `n - p`. When a node
moves within one parent, `n` for the insertion counts the entries left after
its removal. Its `Commit`
description counts the parent link, the removed and inserted entries and every
shifted entry as `record_writes` of one word each in `copied_bytes`, besides
the depth work.
A change near either end of an order therefore describes work independent of
the parent's child count, whatever siblings lie between it and the far end, and
appending, which shifts nothing and needs no walk, does too. Moving every child
from the front of one parent to the end of another costs a constant for each
child, so work linear in their number.

Only a push into a full order allocates. When the new parent's `Children` bank
has no spare capacity, a `PrepareAllocationReport` and a `Reserve` action on it
(`growth: Geometric`) follow depth and edge preflight; otherwise neither is
described. A move within one parent needs no growth, because the node leaves the
order before it returns, and Detach never grows an order. A refused or
interrupted reserve leaves the order unchanged, and the grown bank, with a fresh
allocation incarnation, is retained on abort like the record bank. The `Commit`
action refuses with `ParentChildConflict` any edit whose planned place no longer
holds the entry planning found, and with `CapacityMismatch` a push that would
reallocate or name a missing node or parent; the ready commit then applies
nothing, so a parent link and its list cannot diverge.

Publish checks that every ordinary child appears exactly once in its parent's
child list with work linear in the records and child entries, not in each
parent's fanout. Three passes of `Inspect` actions, each describing two record
visits per step, reset a private one-byte count in every record, add one count
for each list entry naming its list's owner as parent, and then validate the
records in index order, where each record with a parent must have a count of
exactly one (`ParentChildConflict` otherwise). The count lives in the already
admitted record layout, so it needs no allocation, and a stale count is reset
before it is read. Failures keep the order of the earlier per-record scan.

Template association creation is owned by `CreateElement`; lookup returns its issued
contents handle. Cloning a template clones its element through `CreateElement` and
fills that new contents fragment. Moving a whole contents association uses
`ReassignTemplateContents { template, contents }`, which atomically detaches the
old owning association or removes the old non-owning reference, then associates
an unowned ordinary fragment. A shadow-role fragment or a fragment already owned
by another template or shadow host is rejected. Replacing `Owned` leaves its
previous fragment retained and unowned (net zero owned edges); replacing
`ShadowReference` preserves that target's existing shadow owner and role and adds
one template-owned edge. An identical current owned target is a no-op; an
identical shadow-reference target is rejected as `InvalidTemplateAssociation`.
These operations cover parser adoption,
foster-parenting, clone and patch paths without treating association roots as
ordinary children.

The declarative-shadow exception follows pinned HTML877d in-head template
step 7.11.6 ([retained syntax reference](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md))
and the accepted clarification in [#1174](https://github.com/bj-data-eng/surgeist/issues/1174).
Keep a private distinction between owned template contents and a shadow reference
in the existing record; do not create a second mutable target or tree. For a
local host, the referenced fragment's sole owner remains `ShadowHost`. For a
captured host, its explicit shadow role permits a locally unowned result fragment;
the actual capture identity, attachment effect and qualification remain parser
evidence. HTML never imports capture IDs to certify that external attachment.
The old ordinary contents record remains retained and unowned after the atomic
reference transition, preserving its result identity and source associations.
The local `AttachTemplateShadow` transition releases one template-owned edge and
installs one host-owned edge atomically (net zero); the captured-host
`SetTemplateShadowContents` transition releases the template edge (net minus
one). Do not compose independently committing attach/release operations that
introduce a transient extra-edge allowance or a new refusal point. Fixed write,
depth and allocation descriptions reflect the actual coherent transition.
Both declarative transitions require the template initially to have its ordinary
owned contents with reciprocal `Template` owner. Their new target must be a
distinct, unowned shadow-contents fragment. The local command additionally
requires an element host with no existing shadow association. An initial
`ShadowReference`, conflicting host association or already-owned target is
rejected before mutation; this includes resubmitting a previously committed
declarative command. Refusal leaves the same storage/command available for
recovery. Pending action retry is separate from submitting a new command after
commit. No idempotent second ownership transfer or silent host detachment occurs.

`ReassignTemplateContents` installs only an owning ordinary contents association;
it does not turn a shadow reference into ownership of its target. Publication
checks the two relation kinds separately. Structural traversal, cycle/depth and
descendant accounting follow ordinary children and owning associations once;
a contents reference incurs its actual record/read/write cost without another
owned edge or duplicate subtree charge. Native ordinary template authoring and
fallback retain reciprocal contents ownership. Parser stacks and effects use the
same referenced fragment and remain parser-owned. No new native declarative-shadow
convenience or DOM implementation is implied.

`CreateElement { name, attributes, link_flags }`
accepts the actual owned name, validated ordered attributes and optional producer
history as one creation input. It validates supplied flags against its existing
admitted HTML link classification, then forms the private element only at the
same Ready record commit. Invalid supplied flags retain the exact command and
inputs. `None` preserves unknown history. `Create { leaf }` consumes
completed checked payloads and creates no element, so it has no flag input. Parser initial and
clone producers keep their original source qualification before this operation;
no later SetLinkFlags phase or duplicate name scan is introduced. Raw
`HtmlProcessingInstruction` payload construction remains exact-value storage;
native authoring still requires its existing separately checked
`HtmlAuthoredProcessingInstruction`. Those are different phase contracts, not
permission for native syntax checks to reject recovered parser data.

`HtmlConstructionError::cause` returns the exhaustive enum
`HtmlConstructionCause`, so a consumer that classifies every cause fails to
compile when a cause is added. Its variants are `ForeignBuilder`,
`ParentCannotHaveChildren`, `TemplateContentsRequired`,
`ShadowHostMustBeElement`, `ShadowAlreadyAttached`, `UnknownNode`,
`WrongNodeKind`, `RootMove`, `AssociationMove`, `WrongSibling`, `Cycle`,
`AssociationAlreadyOwned`, `InvalidTemplateAssociation`,
`DuplicateAttribute { first, duplicate }`, `AttributeNameMismatch`,
`AttributeOrderMismatch`, `InvalidShadowAssociation`,
`InvalidContentsReference`, `ParentChildConflict`, `InvalidOutputRoot`,
`OperationNotReady`, `ForeignAllocation`, `CapacityMismatch`,
`IndexUnavailable`, `SharedAllocationAliased`, `ReleaseUnderflow`,
`LimitExceeded { resource, limit, attempted }`, `RepresentationExhausted`,
`AllocationFailed` and `Accounting(fault)`. `accounting_fault` returns the
payload of the `Accounting` cause. The parser
maps each structural refusal to its corresponding `SessionError`; admission/cancellation
continues to originate in `ParseControl` and is never disguised as a structural
error. `ReplaceContent` cannot silently change a parent-capable node into a leaf.

`HtmlConstructionOperation::next_action(&mut self) ->
Result<Option<HtmlConstructionAction<'_>>, HtmlConstructionError>` returns an
action borrowing that operation exclusively, `None` only after all actions are
complete, or the latched error while leaving ownership untouched. The action has
`description()` and
`perform(self) -> Result<HtmlConstructionObservation<'_>, HtmlConstructionActionFailure>`.
An action failure contains its typed construction reason and the actual
post-provider allocation snapshot, when a provider was attempted; storage and
the uncommitted command remain in the operation.
An observation continues the action's exclusive borrow and exposes the actual
post-action snapshot. `accept(self)` advances the private phase only after the
consumer has recorded actual capacity and completed its post-action checkpoint.
Dropping it without acceptance leaves the operation awaiting observation;
`try_ready` then returns `OperationNotReady`, and the operation can only be
recovered/aborted. Thus a provider call cannot silently advance straight to a
publication-ready owner. Initialization uses the same observation guard. A
consumer whose external controller stops records the observation, drops the
guard and aborts the retained owner; it never acknowledges that action as ready.
Descriptions are closed variants for inspection, comparison, reserve, fixed
sharing and commit; they state record visits, inspected/copied/initialized bytes,
logical node/edge/attribute/unit deltas and the exact allocation descriptors
below. `next_action` does not perform the described read or mutation.
`HtmlConstructionOperation::try_ready(self) ->
Result<HtmlConstructionReadyOperation,
HtmlConstructionRefusal<HtmlConstructionOperation>>` checks that
every validation, provider action and allocation observation has completed, and
that the complete fixed final-write description has been issued and admitted
through its action. It performs no additional useful work, allocation,
reserve or clone. Pending returns `OperationNotReady`; a failed operation
returns its latched construction reason. An operation whose planned order edits no longer find
their places in the storage (`CapacityMismatch` or `ParentChildConflict`) is refused the same way, so
the commit never receives a state it would have to skip. Every refusal hands the original
owner back unchanged inside `HtmlConstructionRefusal`, with its command, actual
allocations and capacities untouched; a consumed operation cannot be presented
again because the check takes it by value. Only success issues the
private-field ready owner. Initialization and lease recovery use the same refusal
rules: `HtmlConstructionRefusal<T>` carries the owner or owners, `reason`,
`owner` and `into_parts`.

Only `HtmlConstructionReadyOperation` exposes
`commit(self) -> HtmlConstructionCommit`; the commit enum is
`Changed { storage, created: Option<HtmlBuilderNode> }` or
`Published { result: HtmlConstructionResult, lease: HtmlConstructionLease }`.
Both pending and ready owners expose `abort(self) -> HtmlConstructionAbort`,
which returns storage, command and an optional
wrapper-retirement receipt, restoring the previous logical state while
preserving all actual grown capacities. Every
fallible check/reservation and write admission precedes the final coherent
commit. Commit applies only the already-described fixed writes and owning moves;
it cannot inspect an unvalidated edge or run an allocation/provider operation.
Abort never needs a useful-work scan or a new allocation. A ready owner
has no public constructor, `Default`, unchecked conversion, deserializer or
field mutation; a pending/failed operation cannot construct one. Parser calls
the guarded ready transition only after its corresponding admission and
post-provider cancellation checks have succeeded. On an external `HtmlParseStop`
it aborts the retained owner instead. HTML does not encode or override that
independent controller's first-stop policy.

#### Allocation descriptors and stopped-owner recovery

The parser must not name HTML's private `Record`, `Arena` or `Identity` in a
ticket type. Replace only those private-payload tickets with parser-private
`HtmlConstructionTicket`; ordinary public-payload `OwnedVec` and string tickets
retain their existing implementation. This is a specialized adaptation of
`ParseControl`, not a new shared allocation framework.

HTML issues `HtmlConstructionAllocationId` with private fields identifying one
storage identity, role and generation. Roles are `Identity`, `Records`,
`Children` and `ArenaWrapper`. Equality is identity
equality; IDs have no public numeric or
pointer conversion, constructor or arithmetic. Reallocation advances the
generation with checked arithmetic.

Installed attribute vectors and their shared wrappers retain the parser's
existing `VectorCapacity<Attribute>` and `SharedAllocation<Vec<Attribute>>`
tickets over the actual public payload. `HtmlConstructionAttributeCapacity`
reports their actual initialized length, capacity and public element layout, and
`HtmlAttributeAdmission` receives the growth of a list before it is reserved and the
capacity reached afterwards. They need no second opaque attribute-vector/wrapper ticket
or another ledger.

Allocation reports never issue construction node handles. A pending creation
can allocate its child bank before committing a record; aborting it leaves that
record position available for a later creation. Issuing a handle from the bank's
position would let it escape before commit and then identify the later node.
`Children` therefore classifies only the allocation role. Its read-only
`child_position()` is an owner-local accounting join; it cannot mint an
identity or authorize a mutation or retirement.
Checked committed-node views remain the authority for issuing node handles.
Each newly created bank also receives a fresh checked allocation incarnation,
including when it reuses a position left by an aborted operation. A retained
report from the retired bank cannot identify the replacement or authorize its
retirement. Reallocation preserves this distinction.
`HtmlConstructionAllocationId::same_owner_role` compares the issuing storage and
bank role while deliberately ignoring generation. It supports observing a
provider's updated report only after the old exact ID has been qualified against
its live ticket. It does not certify the same allocation incarnation or permit
stale retirement; those require exact issued-ID qualification.

`HtmlConstructionAllocationSnapshot` has read-only accessors for `id`, `role`,
`initialized_len`, `capacity`, `element_size`, `element_alignment`,
`payload_bytes` and `wrapper_payload_bytes`. For a vector, payload bytes equal
capacity times its actual private element size; the wrapper descriptor separately
reports the actual `Vec<T>` layout. For the arena shared wrapper, payload bytes
are exactly `size_of::<Arena>()`; identity has zero known payload bytes. Counts
and products are checked before a provider call. Arc headers and allocator
metadata remain excluded, matching the existing provider limitation.

A reserve description contains the pre-action snapshot, required initialized
length, requested minimum capacity and selected exact/geometric growth policy.
`Exact` means `perform` reserves exactly that minimum. `Geometric` means the
minimum is only the least capacity that suffices: the consumer admits its own
amortizing capacity, which cannot be smaller, and requests it with
`HtmlConstructionAction::perform_with_capacity`; a smaller capacity, or use on
any other action, latches `CapacityMismatch` before a provider call, and
`perform` still reserves the minimum.
The parser computes and admits its current resource/peak/atomic-layout and
reserve/copy/provider work in the same precedence as `reserve_vector`; it cannot
substitute an assumed `Record` layout. `perform` calls `try_reserve` on the actual
hidden vector, then returns its actual post-action snapshot even if subsequent
observation causes a stop. Allocation refusal returns the unchanged snapshot
with `AllocationFailed`. Parser's
`ParseControl::observe_construction_allocation` updates the specialized ticket,
actual capacity and high-water values before its post-provider cancellation or
actual-capacity limit check. It latches the first stop and retains the grown
owner on any refusal. No ordinary record write follows such a stop.

`HtmlConstructionTicket` stores the issuing `ControlIdentity`, last observed
opaque allocation ID/generation, actual capacity and known byte layout. It is
not cloneable. `ParseControl::admit_construction_action` and
`observe_construction_allocation` compare the issued snapshots with that ticket;
foreign controls, wrong roles, stale generations, mismatched capacities and
underflows produce the existing invariant stop. Ticket-vector capacity remains
an ordinary parser `VectorCapacity<HtmlConstructionTicket>` and is charged using
its real public-to-parser layout. HTML stores no resource totals or parser
controller identity.

`Publish` performs the real `Arc<Arena>` construction only after the wrapper's
known payload layout is admitted. The operation retains the sole strong arena
owner until the post-share checkpoint succeeds. If that checkpoint stops,
`abort` unwraps that unescaped Arc, reports retirement of exactly its admitted
wrapper bytes, and restores the same records, capacities and identity. This is
the existing `share_owned` post-allocation recovery behavior, not a new
fallible-Arc-allocation promise. Only successful commit returns the result and
`HtmlConstructionLease`. The lease is non-cloneable and holds HTML's actual weak
arena reference and issued wrapper descriptor; parser tickets no longer need
`SharedAllocation<Arena>` or `SharedAllocation<Identity>`.

`HtmlConstructionLease::try_recover(result: HtmlConstructionResult, lease: Self)
-> Result<HtmlConstructionRecovery,
HtmlConstructionRefusal<(HtmlConstructionResult, HtmlConstructionLease)>>` checks
exact arena/lease identity first. A result clone, associated fragment view or
other strong arena alias returns `SharedAllocationAliased`, handing back both the
result and the lease with all tickets intact.
After aliases disappear the same cleanup can be retried, including after a
sticky parser stop. Successful recovery returns the exact unwrapped storage
together with its non-forgeable wrapper-retirement receipt. Identity handles may
outlive cleanup: its zero-sized payload follows the existing zero-refund rule.

Storage's `retire(self) -> HtmlConstructionRetirement` exposes deterministic
cleanup actions, not mutable records. Reverse node order retires child-vector
buffers, then drops record payload references and the record vector, then the
zero-payload identity owner. Each action supplies its exact live snapshot for
ticket/underflow preflight; `perform` frees that allocation and returns a
non-forgeable `HtmlConstructionRetired` receipt. Only that receipt permits the
parser to refund corresponding capacity. Cleanup does not poll for useful work,
allocate, infer a capacity or silently discard a refused owner. A preflight
refusal leaves the retirement cursor at the same allocation for retry. Parser
then releases its independent payload/effect/evidence ticket banks in their
existing order. Strings and attributes are dropped from records before their
retained parser payload tickets are reclaimed.

These snapshots/actions replace private `VectorCapacity<Record>` accesses in
session reserve/release, private `SharedAllocation<Arena>` in freeze/recovery and
private identity sharing. Read-only storage/result node access replaces
`arena.records` reaches in metadata/history and alias preflight. Private dense
joins remain local; exposed result identities come from the checked storage or
result view, never from constructing an ID from an ordinal.

### Captures, fragment context and lazy facts

Semantics owns `HtmlCapture`, all `HtmlCapture*Id` and builder-handle types,
`HtmlCaptured*` records, `HtmlCaptureBuilder`, limits/usage/errors,
`HtmlFragmentContext` and `HtmlFragmentTarget`. Host facts retain their explicit
unavailable/absent/present states; an immutable construction result supplies
neither a host revision nor current form/custom-element/registry truth.

Do not replace private capture indexes with public integer conversions.
`registry_index`/`registry_at` and `document_index`/`document_at` are removed from
the cross-crate contract. Parser stores checked `HtmlCaptureRegistryId` and
`HtmlCaptureDocumentId` directly in its `RegistryRef`/`DocumentRef`, including
template-cache keys. Equality already includes capture identity. Public
`HtmlCapture::node`, `document_id`, `definitions` and `definition` are the lookup
front doors. Cloning a retained ID is a described action and is charged before
the clone; a compact integer is never treated as external identity.

Replace `template_cache_index` and `CapturedTemplateCacheKey` with
`HtmlCapture::template_document_cache(&HtmlCaptureDocumentId) ->
Result<HtmlCapturedTemplateCache, HtmlCaptureError>`, preserving
`Absent`, `Unavailable`, and `Document(HtmlCaptureDocumentId)` distinctly.
Replace `HtmlCapturedNode::lookup_registry_index` with its existing typed
`lookup_registry` accessor. The immutable observations
`HtmlCapturedNode::lookup_registry_clones_identity` and
`HtmlCapture::template_document_cache_clones_identity` describe whether their
corresponding getter transfers one cloned identity. Parser admits the header
observation, then any actual identity clone and compiled result layout before
calling the getter; absent and unavailable facts require no clone.
Promote `shadow_presence` as
`HtmlCapture::shadow_presence`, retaining `Result<Option<bool>, HtmlCaptureError>`:
unknown must remain distinct from an absent shadow root.

Expose the current definition lookup as `HtmlDefinitionLookupCursor` and
`HtmlDefinitionLookupStep::{Searching, Ready(Option<HtmlCaptureDefinitionId>)}`
from semantics. `HtmlCapture::definition_lookup_cursor` borrows registry/local/is
operands. `next_cost`, `next_clones_registry` and `step` retain the exact existing
ordered lookup behavior. The parser charges the next cost, any ID transfer and
output record before `step`; the semantic session does the same through its own
ledger. This cursor has no internal cancellation or accounting owner. The eager
`lookup_definition` convenience method remains for already-selected unbounded
capture configuration, not for parser execution.

`HtmlFragmentContext::element` and `hosted_fragment` still select one immutable
capture, context element, target/context documents, nearest ordinary-ancestor
form and root registry in their existing order. The fragment context remains
capture-qualified; it is not a local fragment value in HTML. Parser requests
retain it and resolve supported scripting before source admission. Actual
parser construction reads additional facts only when reached. No global capture
prevalidation may turn an unreached unavailable fact into an earlier failure.

HTML owns only the local decision inputs and outputs:
`HtmlParserScriptingMode`, `HtmlFragmentTokenizerState`,
`HtmlFragmentInsertionMode`. The shared private policy selected by #1177 lives
in HTML's private `fragment_initialization.rs`. The bounded front door exposes
`bounded::parsing::fragment_initial_modes<E>(effective: HtmlParserScriptingMode,
compare: impl FnMut(HtmlFragmentNameTest<'_>) -> Result<bool, E>) ->
Result<(HtmlFragmentTokenizerState, HtmlFragmentInsertionMode), E>` wrapper.
`HtmlFragmentNameTest<'a>` has `HtmlNamespace` and `Local(&'a [&'static str])`
variants; the namespace and ordered local-name group are separate observations.
Parser's callback delegates its existing charged namespace/local comparisons;
semantics' capture convenience methods use exact UTF-16 comparisons. Callback
refusal returns the original `E` immediately, before another comparison or a
successful pair. The policy allocates no storage and owns no controller.

The ordinary HTML front door also exposes
`fragment_tokenizer_state(&QualifiedName, HtmlParserScriptingMode) ->
HtmlFragmentTokenizerState` and
`fragment_insertion_mode(&QualifiedName) -> HtmlFragmentInsertionMode`.
`HtmlFragmentContext::initial_tokenizer_state` resolves the actual scripting
mode before calling the former. `initial_insertion_mode` calls the latter
without reading scripting; it cannot invent a mode and discard half of a
combined result. Both ordinary helpers share the same underlying policy and
retain their existing exact UTF-16 comparison order.

The effective scripting mode is already resolved. The function never reads
captured scripting status or claims execution support. Parser maps the returned
value enums to its private tokenizer/tree-builder states. The fixed choices
remain title/textarea Rcdata, the raw-text names and effective-mode noscript
condition, script ScriptData, plaintext, and the context-local insertion table.
Non-HTML context selects Data/InBody. Captured ancestor templates or tables are
not inserted into the parser's open-elements stack.

Parser stack reset consumes a second narrow HTML wrapper,
`bounded::parsing::stack_reset_insertion_mode<E>(compare: impl FnMut(&'static str) ->
Result<bool, E>) -> Result<Option<HtmlFragmentInsertionMode>, E>`. It delegates
the same private fixed table under `FixedInsertionScope::StackReset`; fragment
initialization uses `FixedInsertionScope::Fragment`. The public wrapper does not
expose that private scope type. State-sensitive template/td/th/head/html reset
branches remain parser-owned; the shared table cannot claim a complete parser
reset state. No speculative initialization result/cursor type is introduced.

The capture owner supplies qualified operands for integration-point queries;
the parser supplies its adjusted-current-node operands. Shared local
classification includes MathML text and HTML integration points. The
annotation-xml encoding comparison and prefix policy are defined once by the
#1175 correction; consumer adapters preserve their existing admission before
each comparison. Captured identity and host validation remain in semantics.
The bounded front door exposes `HtmlIntegrationPointComparison::{Exact,
AsciiCaseInsensitive}` and `bounded::parsing::HtmlIntegrationPointRule::{Never, Always,
EncodingAttribute}`, matching the private `integration_points.rs` policy.
`bounded::parsing::html_integration_point_rule` takes optional namespace URI, local name
and the fallible comparator
`FnMut(&DocumentString, &'static str, HtmlIntegrationPointComparison) ->
Result<bool, E>`, returning `Result<HtmlIntegrationPointRule, E>`.
`bounded::parsing::html_integration_point_encoding_attribute` additionally takes the
attribute value and returns `Result<Option<bool>, E>`: `None` is an unrelated
expanded name; `Some(false)` is the actual encoding attribute with a rejected
MIME value and stops further attribute reads. Semantics' ordinary borrowed
adapter and parser's charged field/iteration adapter use those same operations.
Only these consumed operations/types cross the package boundary; the private
module itself remains private.

MathML text integration points use the ordinary
`is_mathml_text_integration_point(&QualifiedName) -> bool` front door and
`bounded::parsing::mathml_text_integration_point<E>(compare:
impl FnMut(HtmlMathmlTextPointTest<'_>) -> Result<bool, E>) -> Result<bool, E>`.
`HtmlMathmlTextPointTest` emits `Namespace(&'static str)` for the exact MathML
URI and, only after success, one `Local(&[&'static str])` group containing
`mi`, `mo`, `mn`, `ms`, `mtext` in that order. Capture uses the ordinary exact
name adapter. Parser's actual-node and adjusted-current-node adapters delegate
the two groups to their existing namespace/local-group comparisons, retaining
group-header, per-choice and unit-read admission. Callback refusal propagates
without another observation. Foreign-dispatch token-kind, `mglyph`/`malignmark`
exceptions and the separate annotation-xml/SVG branch remain parser-owned.

### Local rules and semantic execution

`bounded::script::HtmlScriptTypeCursor` owns the existing script classification actions
for current content and preparation. It borrows original operands, including
the virtual language prefix, and exposes `advance` and the completed optional
kind. `advance` charges the record visits and actual inspected bytes of its
action through the caller's `surgeist_ledger::Charge` before it runs; callers
retain their controller, admission and cancellation. Preparation-only JavaScript whitespace handling is not
applied to current-content classification or module/import-map keywords. The
literal matcher remains private, and ordinary reports use this same classifier.

`HtmlInputType::missing_value_default` selects the authoritative missing-value
row without an attribute read. It is distinct from inspecting an optional
attribute through `HtmlInputTypeCursor`, so parser callers retain their original
fixed admission order. `HtmlInputImageCoordinate::initial` supplies the selected
fresh Image-state zero coordinates; it accepts no arbitrary host activation
coordinates.

`HtmlInputNumericContent`, `HtmlInputNumericStep` and
`select_input_numeric_constraints` expose the one selector over already-converted
finite content, without reparsing raw attributes. It returns `None` for states
other than Number or Range. Missing, failed, unrestricted and recovered step
facts remain distinct. `HtmlInputNumericStep::ANY_KEYWORD` owns the sole `any`
spelling; `any_keyword_unit_matches` tests one reached position with the existing
ASCII kernel, not whole-value validity. The parser checks actual field length
and admits each reached unit before calling it. Range consumers construct actual
headers and inspect issued cells through the existing bounded public accessors;
no mutable field promotion or replacement numerical algorithm is needed.

The semantics-owned base model exposes borrowed document `fallback`, retained
parse `observations`, their exact `raw` and `outcome`, environment
`inputs_match`, and parse-fact `same_owner`. Matching uses URL-record/encoding
identity and producer revision without scanning serialization. Same owner means
the exact immutable facts backing, not equal contents or revisions.
`select_frozen_base` owns the existing historical frozen-slot decision and
returns the actual borrowed fallback/parsed record plus whether base-response
steps are required. It does not qualify observation provenance or mutate parser
state. Data/JavaScript and blocked outcomes select fallback before requiring
unavailable CSP truth. Current semantic href lookup remains its separate
outcome-consuming operation; it is not routed through frozen-slot policy.

Local rules consume explicit values rather than locating them in a tree. HTML
therefore retains input flag transitions/sanitization, numeric and temporal
grammar, token classification, nonce effects, details-name decisions, PI
pseudo-attributes, template marker selection and rel-token effects. Semantics
locates and qualifies operands; parser supplies construction records and source
provenance. Neither consumer copies those algorithms.

The following private reaches need deliberate public bounded operations. Names
in this table identify the existing source units and actual consuming operations;
their producer operations and checked operand/result/error types belong in
`surgeist_html::bounded`.
Private representation fields become getters or checked constructors, not public
fields. No operation accepts a parser or semantic controller.

| Existing unit and actual consumer | Selected HTML boundary |
| --- | --- |
| `HtmlValueInterpreter::admit_attribute_raw`; semantic attribute helpers throughout `semantics/` | Public `admit_attribute_raw`; checks optional UTF-16 length without cloning/allocating. Reports are assembled only through owner operations. `time_element_report_from_attempts` stays private: a public caller cannot assert that arbitrary attempts belong to arbitrary raw text. |
| `numeric::{try_parse_integer, IntegerUnitStep, IntegerUnitStepResult, IntegerUnitStepError, try_floating_point_number, FloatingPointParseMode, FloatingPointParseResult, FloatingPointAllocationError}`; semantic ordinals/range/time/dimensions | Export bounded parse/materialization/step operations and checked result/error accessors, including integer-prefix scanning, long-integer interpretation and capped nonnegative recovery used by image dimensions, tabindex, select and inheritance. Preserve requested/actual capacity accounting on both success and failure. |
| `NumericMaterialized`, `NumericMaterializationFailure`, `HtmlNumericMaterializationError` and current bounded methods on integer/decimal/rational values; semantic time/dimensions | Export the bounded materialization result/error domain and the exact existing `try_*` operations with their work-bound methods; values retain private canonical digits. |
| `numeric::natural_ratio::{NaturalDigits, NaturalProduct, NaturalComparison, NaturalDifference}`; `semantics/natural_dimensions.rs` | Export immutable borrowed operands and bounded arithmetic cursor operations; semantic resource/host qualification stays in semantics. |
| `numeric::range` and its ingress/best/lattice cursors; `construction/parser_session/range.rs` | Export the numerical request/reply/cursor boundary, scratch roles and exact layout descriptions; scratch allocation stays with parser, not HTML. |
| `temporal::{TemporalAlternative, TemporalPhase, BorrowedTemporalPlan, BorrowedYear, BorrowedDate, BorrowedDurationPlan, BorrowedGlobalDateTime, recognize_time_alternative, temporal_recognition_work_bound}`; `semantics/time_element.rs` | HTML owns `HtmlTimeElementInterpretation`, retaining one raw operand and its recognition/materialization phases. Its bounded actions describe recognition, numeric allocation and owner assembly before execution; only its completed operation creates the report. The listed borrowed plans remain private to HTML. Temporal input-coordinate conversion belongs in a distinct HTML module under #1181, retaining its current API. |
| `tokens::{SpaceTokenCursor, SpaceTokenStep, ascii_whitespace}`; semantic rel/microdata/meta | Export the allocation-free space-token cursor and read-only ranges, preserving source order and repeated tokens. Comma-token primitives stay private to HTML's input/value operations. Include the local hash-name scanner and its missing-value result used by image-map interpretation. |
| `attributes::{TargetRecovery, input_readonly_applies, stylesheet_link_context_work_bound, WritingSuggestionsProcessing}`; semantic base/control/text/attribute context | Export the local recovery/classification operations with immutable checked results, including embedded-dimension applicability. Structural selection remains semantics. Existing public form-name, id, map-name and is-attribute interpretation owns report assembly; expose its bounded work metadata rather than private arbitrary report assemblers. |
| `attributes::autocomplete`; semantic autocomplete | Export the actual controlled materialization operation, its work bound, limits and capacity-bearing success/failure observations. Field recognition and valid report assembly remain one HTML-owned operation. |
| `attributes::microdata`; semantic microdata | Provide bounded token-report production retaining its actual raw operand and token-list kind. Preserve source ranges, duplicate selection, dot/colon classification and their admission order. Keep token/report fields private; no caller-asserted raw/range constructor certifies a report. |
| `attributes::token_declarations::{IconSizeCursor, IconSizeShape}` and `attributes::media::media_field`; semantic relationship selectors/media facts | Export local cursor and field selection only; resource/CSS producer attestations remain semantics. |
| `media::ThemeColorNameCursor` and media attribute reports; semantic meta-author/media checks | Export the local allocation-free name recognition cursor. Report production retains the actual value/name operands and completes their interpretation under admitted actions; `HtmlMediaAttributeReport::observed` stays private and cannot certify arbitrary raw/role/value/placement tuples. |
| `rel::{RelMeter, RelInputs, interpret, classify}`; semantic relationships | Export bounded interpretation with private-field `RelInputs` constructed from explicit operands. Include role/comparison operations and owner-checked occurrence updates and aggregate refresh over the actual report. Keep the small meter callback boundary because both native interpretation and semantic sessions implement its actual admission behavior. `RelData` and unchecked occurrence mutation remain private; HTML creates the valid report. |
| `custom_names::{validate_custom_element_name_with, CustomNameScanAction}`; parser creation and semantic definition lookup | Export the local bounded validation action API; capture/registry policy remains in semantics. |
| `pi_pseudo_attributes::{Cursor, Action, Reply, Field, Name, Error}` and `template_markers::{Target, First, Siblings, first}`; parser PI/template and semantic template selection | Export bounded grammar/marker transitions, with checked reply order and no parser metadata. |
| `nonce::{NONCE_INSPECTION_BOUND, nonce_element}`; semantic nonce and parser nonce effects | Export local inspection bound and element applicability; exact capture history and parser source provenance stay with their consumers. |

Parser-only helper code inside tests of `input.rs`, `request.rs` or
`custom_names.rs` moves to parser tests or uses public owner operations; a dev
dependency cycle is not introduced to preserve private test access.

`HtmlTimeElementInterpretation::finish(self) ->
Result<HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
HtmlFinishFailure<Self, HtmlTimeElementInterpretationError>>` follows the same
consuming rule as attribute validation. `NotReady` and a latched processing failure hand the
exact interpretation, raw operand and any actual materialization allocations
back in the `HtmlFinishFailure`; only completed recognition/materialization and admitted report
assembly can consume it and produce a report. `abort(self)` drops unpublished
partial materializations and returns the original borrowed raw operand, without
recognition or allocation. Their already-observed capacities remain charged in
the semantic phase's existing cumulative-allocation model. The public error
distinguishes `NotReady`, processing representation exhaustion and
actual allocation failure; consumer cancellation/admission remains the original
consumer's error. Completed source-defined invalid syntax or unrepresentable
value outcomes are the report's author/recovery states, not a failed publication
transition.
No public report constructor accepts caller-asserted raw/attempt pairs.

All consuming fallible boundaries in this contract follow one of three concrete
forms. The construction owners (initialization, tree mutation and publication,
and lease recovery) consume the owner and return `HtmlConstructionRefusal<Owner>`, which
holds the complete unconsumed owner and the cause, so a refusal loses nothing and
a consumed owner cannot be presented again. The other owner families
(attribute validation and interpretation) keep a guarded
`&mut Option<Owner>` transition that leaves `Some` unchanged on refusal. The
remaining boundaries return an error value containing the complete unconsumed
owner/input. Borrowing action failure leaves ownership in its enclosing
operation. There is no infallible consuming `commit` or `finish` on a
pending/failed owner, and no ready-proof constructor outside the owner. This
applies to initialization, attribute validation, complete attribute edits and
slot commits, tree mutation/publication, interpretation, lease recovery and
semantic import/finalization; it does not add a generic public typestate
framework.

The input-type semantic adapter required by #1173 performs one HTML-owned rule
scan after finite semantic attribute admission. Full semantic reports use
`HtmlValueInterpreter::input_type` under the existing admitted work bound and
post-operation poll, because the state-only `HtmlInputTypeCursor` cannot certify
raw spelling, recognition or author-state report fields. Parser state lowering
uses that cursor. The adapter must not interpret once and then rescan merely to
invent an accounting total, or construct a full report from a processed state.
Allocation-free attribute-report setup from #1176 and
semantic test/context admission consolidation from #1178 remain owner-local
changes under this map.

#### Controlled local report completion

The following protocols select the new bounded APIs for rel, media and
microdata. They are extraction requirements, not claims that these APIs are
already implemented. Each ordinary interpreter and its bounded adapter use the
same HTML-owned rule kernel. Semantics continues to own tree traversal, external
fact qualification and its existing admission controller.

`HtmlRelInterpretation<'raw>`, `HtmlMediaInterpretation<'raw>` and
`HtmlMicrodataTokenInterpretation<'raw>` are concrete retained owners with private
state, original borrowed operands and any actual output allocations. They are
not cloneable. Allocation-free initialization retains the operands before the
first fallible action. Their borrowing actions return the original consumer admission or
cancellation error unchanged. Once such an error occurs, the owner becomes
`Stopped`: its allocations and operands remain retained, no report or aggregate
readback is available, and subsequent operations perform no useful work. HTML
does not clone or replace the consumer's first failure. Protocol refusals before
execution, such as a wrong phase or invalid refinement, leave the owner unchanged.
An explicit borrowing `stop` operation latches the same stopped state without
work when the consumer fails between HTML actions, such as during external-fact
lookup; it does not accept, copy or replace that consumer error.

Each owner has `finish(self) -> Result<Report, HtmlFinishFailure<Self, Error>>`, with its
own existing report and concrete error type. `NotReady` means required actions remain; `Stopped`
means execution failed. These refusals hand the owner back unchanged with its exact backing, so a
caller keeps no `Option` slot and no owner is dropped on an error path. Only an owner whose report
assembly has already succeeded is consumed. Finishing moves that
report without scanning, charging, cloning or allocating.

The standalone operations (`microdata_tokens`, `media`, `media_for_meta`, `rel_with_attributes`,
`control_autocomplete` and `control_autocomplete_for_input`) drive these owners with a meter that
admits all work, and return `HtmlValueInterpretationError`: the per-value admission refusal, a
refused allocation, an unrepresentable size, or a breach of the owner's own protocol by that driver.
They reserve with the fallible allocator and never abort the caller. `abort(self)` drops
unpublished backing and returns the original borrowed inputs, without further
observations or admission. Actual capacities already observed by semantics keep
their existing accounting treatment; abort does not refund cumulative nested
output or allocated-byte usage.
Domain action errors also distinguish wrong phase, checked operand/refinement
refusal and the existing raw-admission error. Actual allocation failure remains
observable by the reservation controller and is mapped to its original failure;
all such execution failures retain the owner and prevent publication.

#### Rel interpretation and refinement

The rel owner retains the actual `HtmlRelInputs<'raw>` used by initial
interpretation: role, rel, rev, href, imagesrcset, itemprop, title, type hint and
as-destination. These are borrowed operands or fixed values, not reconstructed
from `RelData`, which does not retain all of them. `HtmlRelInputs` has private fields
and a constructor taking these explicit operands. Initial interpretation keeps
the existing `interpret` work, reservation and raw-sharing order. The owner
holds its occurrence vector and single data-record vector; it introduces no
second report, raw clone or accounting ledger.

After initial interpretation, readbacks lend the retained role, original raw
operands and occurrences, including spelling, source, ranges, keyword, effect
and registry-resolution status. They do not expose aggregate fields. The caller
can finish this initial result directly, as ordinary interpretation and noopener
processing currently do. Alternatively it explicitly begins refinement. This
choice is irreversible: a refining owner cannot publish the initial aggregates.

A borrowing refinement operation checks the actual retained occurrence index
before mutation. The occurrence must come from `rel`, have original `Unknown`
spelling, and have an index strictly greater than every previously applied
refinement. Out-of-range, wrong-source, wrong-spelling, repeated and out-of-order
updates are distinct checked refusals. Validation uses the owner's actual
occurrence; there is no transferable array-index proof or unchecked public
mutation. Each accepted update executes after the existing four-unit semantic
admission and takes one checked `HtmlRelRefinement`:

| Refinement | HTML-owned change |
| --- | --- |
| `Unavailable` | No keyword or effect; registry unresolved. |
| `NoEffect` | No keyword or effect; registry resolved. |
| `Standard(keyword)` | Retain that keyword, derive its standard effect from the retained role, and mark registry resolved. |
| `Extension(effect)` | No standard keyword; use the explicitly supplied nonstandard effect and mark registry resolved. Reject `InternalResource` and `StylesheetModifier`, and reject `Annotation` for the Link role, matching the existing extension policy. |

Registry lookup, synonym/canonical qualification and supported-token facts stay
in semantics. The refinement domain does not certify those external facts and
does not accept an arbitrary keyword/effect/resolved triple. Standard effects
are derived by the existing role-dependent HTML rule, including effects that
are forbidden for nonstandard extensions.

`refresh` takes the owner and its small admission meter, with no independent
`RelInputs` argument. It runs the existing `summarize`, five-unit aggregate
admission and `image_source_preload` sequence against the retained original
inputs. Mutations during this sequence remain private. Only complete success
makes the coherent report ready; failure leaves a stopped owner with no stale
aggregate view. This avoids rollback copies even when `summarize` has already
changed occurrence effects or preload admission fails after aggregate writes.

The semantic adapter preserves its present sequence: select and admit all nine
attribute operands; derive the role and as-destination; run initial HTML
interpretation; inspect occurrences and obtain external facts; begin refinement;
apply source-ordered updates at their existing four-unit checkpoints; refresh
even when there were no updates; then finish and derive downstream meanings.
Noopener's scratch-accounted initial-only path finishes without another summary.
The completed report supplies a read-only registry-resolution accessor; its data
and mutation helpers remain private.

These requirements follow the actual
[rel kernel](../../crates/surgeist-html/src/internal/elements/rel.rs),
[relationship adapter](../../crates/surgeist-html-semantics/src/semantics/link_relationships.rs)
and [noopener adapter](../../crates/surgeist-html-semantics/src/semantics/link_relationships/noopener.rs).

#### Media interpretation with deferred name recognition

The media owner retains the actual element name and supplied media/name
attribute references. Name supply has three distinct states: `NotSupplied`,
`SuppliedAbsent` and `Present(attribute)`. Explicit absence records the
consumer's supplied operand; it does not certify complete capture traversal.
Traversal completeness and external fact qualification remain semantic policy.

Field matching is performed during the original attribute lookup, not by
rescanning its selected result. A narrow HTML-owned media-field matcher checks
the actual immutable candidate's namespace and local name for `Media` or `Name`.
Its private result retains that same borrowed candidate and expected field;
neither arbitrary matching flags nor a result paired with a different attribute
can be supplied to the owner. The semantic adapter still selects the first
matching candidate, admitting the existing candidate and per-unit reads before
the matcher executes them. It reports absence explicitly when traversal finds
none. Ordinary supplied-attribute interpretation uses the same matching kernel
and retains its existing `DifferentAttribute` or `DifferentContext` outcomes.

The owner's actions separately admit the raw media value, inspect the element
namespace, derive the role from the actual element name, and, for Meta, supply
and admit the selected name operand. Namespace and role classification use the
existing HTML kernels at the semantic adapter's current namespace/known-name
checkpoints; they accept no caller-asserted role. Partial readbacks expose these
completed local decisions so semantics can take its existing foreign-namespace
and inapplicable-element exits. The ordinary driver can instead continue to
produce its existing inapplicable attribute report. Supplying a name twice or
in the wrong phase is a refusal; an ordinary supplied name on a non-Meta element
retains the existing `DifferentContext` result.

Value actions use `HtmlMediaValueCursor`: one admitted unit per `InspectUnit`
and four for `Finish`, including its fixed keyword comparison. Completed value
classification is readable before any theme-color name scan or final report.
Semantics then obtains external media facts and makes its existing grammar,
condition and role-specific decisions. For Meta it explicitly starts the
retained name cursor at the current one-unit start checkpoint and admits each
advance by one unit. This action is required for semantic theme-color processing
even when media is absent. `SuppliedAbsent` produces false; `NotSupplied` refuses
an explicitly requested theme-color scan. No name scan is hidden in value
classification or repeated during report assembly.

Placement is derived by HTML from completed applicability, role, media presence
and name state/result. It is unapplied for inapplicable/non-Meta inputs,
`NotSpecified` for absent media, unchecked when a needed name was not supplied,
and otherwise allowed or forbidden by the theme-color result. The ordinary
`media` driver may finish with an unchecked name; ordinary `media_for_meta` may
skip name recognition when absent media already determines placement. The
semantic driver still requests recognition then, because its processing result
also depends on the name.

Ordinary `media_for_meta` retains its existing raw-length admission priority:
it checks both supplied media and name operands before namespace/role
classification, including for a foreign or inapplicable element. This ordinary
front-door schedule differs from the semantic schedule above; both reuse the
same local recognition and placement kernels without repeating those scans.

Report assembly is a distinct action after the semantic adapter's existing
outer-record reservation and one-unit write admission. It shares each actual
supplied attribute once and derives the report from retained decisions; it
accepts no caller-supplied role/value/placement tuple. The owner itself needs no
variable allocation. External grammar, uniqueness and processing readbacks
remain semantics-owned. `HtmlMediaAttributeReport::observed` stays private.

This order is required by the existing
[media semantic adapter](../../crates/surgeist-html-semantics/src/semantics/media.rs),
[ordinary report kernel](../../crates/surgeist-html/src/internal/attributes/media.rs)
and [media cursors](../../crates/surgeist-html/src/internal/elements/media.rs).

#### Microdata token production

The microdata owner retains the original optional raw string and
`HtmlMicrodataTokenKind`, its space-token cursor, private token vector, current
range, prior-token comparison position, first duplicate result and punctuation
scan position/result. Source ranges come only from that cursor over that raw
operand. No public constructor accepts independently supplied tokens or ranges.
Missing/empty inputs, duplicates and dot/colon presence are normal report
outcomes, not protocol or processing failures.

After the semantic adapter's existing attribute lookup and raw admission, the
owner exposes the following ordered actions. Each admission precedes the named
read or mutation; a refusal prevents that action and stops the owner.

1. For present raw input, admit `SpaceTokenCursor::BEGIN_WORK` before cursor
   creation. Each loop then admits one unit for its header, followed by
   `STEP_WORK` when the cursor inspects a unit or two units otherwise, before
   stepping. Preserve both admissions, including the final `Done` iteration.
2. For each emitted token, visit prior tokens in order: admit one unit per
   candidate, one for the length comparison, and two per compared unit pair when
   lengths match. Stop comparison at the first mismatch and select the first
   equal prior token as `duplicate_of`; comparison is case-sensitive.
3. Admit one unit before reading each current-token unit for dot/colon
   classification. Visit every unit even after a match, preserving the current
   semantic charge sequence.
4. Request the fixed one-token reservation described below, then admit one unit
   before committing that token. A failed reservation or write admission cannot
   publish the token or a report; any capacity already obtained remains owned.
5. After `Done`, or directly for absent raw input, admit one unit before sharing
   the original raw string and assembling the report. Only then is finish ready.

The ordinary interpreter drives the same token-production kernel with ordinary
admission. The existing report's kind-specific processing-token and local-author
rules stay authoritative; in particular ItemProp suppresses duplicate processing
tokens. This protocol preserves the actual checkpoints in
[`md_tokens`](../../crates/surgeist-html-semantics/src/semantics/microdata.rs) while moving
token/report assembly into the
[HTML microdata owner](../../crates/surgeist-html/src/internal/attributes/microdata.rs).

#### Opaque reservations and failure accounting

The small rel meter retains its work callback. Its reservation callback receives
`HtmlRelReservation<'_>`, an opaque borrowing guard over the owner's actual
occurrence or single data-record vector, instead of `&mut Vec<T>`. Microdata
uses a concrete `HtmlMicrodataTokenReservation<'_>` over its actual token vector.
Each guard describes only the current fixed request for room for one additional
record: actual length, capacity and record byte size, the capacity the reservation
will reach (`target_capacity`) and one checked reservation operation. When one more
record fits, nothing grows. When a vector that takes records one at a time is full
(the microdata tokens and rel occurrences), the target is the greater of one more
record and double its capacity (at least 4), so `n` records request `O(log n)`
reallocations. The target depends only on the vector, never on a limit, so the
allocation path and the usage it needs are fixed by the input: a consumer that
admits the target against a limit refuses the reservation when the limit cannot
hold it, and a query that grows only such lists succeeds under a limit exactly when
the limit is at least that usage. The report-data vector of a rel interpretation takes one record and is
reserved exactly. The meter admits the target capacity before the operation
allocates it and charges the capacity the vector actually reaches. The operation
uses `try_reserve_exact` at most once for the target; only if the allocator refuses
a target larger than the one record that is needed does it request that one record
instead. It runs at most once, and when existing capacity suffices it completes
without allocation after the existing one-unit admission.
The guard exposes neither record mutation nor
a vector/slice escape, and accepts no arbitrary additional count. Dropping an
unexecuted guard performs no reservation; successful callback return without a
completed required reservation is a protocol refusal, never permission to push.

Semantics factors its existing private `reserve_records` implementation to
accept this fixed reservation operation; ordinary Vec reservations delegate to
the same implementation. This is an owner-local helper, not a new public
allocation framework. Preserve its exact order: initial poll; checked needed
length; existing-capacity one-unit admission; otherwise requested growth/resource
and byte checks; one-unit allocation admission; actual reservation; actual
capacity/resource/byte accounting; actual-limit checks; final poll. Neither HTML
nor a second semantic helper copies that ledger formula.

On a post-reservation limit or cancellation error, the original consumer error
is returned while the stopped interpretation retains the enlarged actual
vector and all original borrowed operands. The guard permits observing actual
capacity after either reservation success or failure so the existing consumer
records growth at the same checkpoint. No new record is committed after that
failure. Noopener's scratch adapter records capacity changes even on error and
releases its actual scratch retention when the owner is dropped; the existing
cumulative byte accounting remains unchanged. Nested output capacities retain
their existing cumulative treatment. Finishing cannot hide a late poll or
allocation, and neither abort nor refusal can replace the consumer's first stop.

The authoritative accounting body is
[`HtmlSemanticSession::reserve_records`](../../crates/surgeist-html-semantics/src/semantics.rs);
the concrete guards protect private HTML storage while allowing that existing
controller to remain authoritative.

### Native and parsed values entering semantics

`HtmlDocument`, `HtmlFragment`, `HtmlNodeKind<'a>`, names and strings have exactly one
definition in HTML. A parse report lends that same HTML result. Semantics owns
the public `HtmlConstructionCaptureAdapter`; callers supply external facts,
while the adapter owns traversal, sharing, capture construction, admission and
the result-to-capture mapping. It does not depend on parser or consume a parse
report type. Parsed results enter through `report.document()` or
`report.fragment()` and exactly the same document/fragment adapter methods.

The consuming entry points are:

```rust,ignore
HtmlConstructionCaptureAdapter::document(
    builder: HtmlCaptureBuilder,
    source: HtmlDocument,
    facts: HtmlConstructionCaptureFacts,
    limits: HtmlConstructionCaptureLimits,
    cancellation: HtmlSemanticCancellation,
) -> Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure>;

HtmlConstructionCaptureAdapter::fragment(
    builder: HtmlCaptureBuilder,
    source: HtmlFragment,
    facts: HtmlConstructionCaptureFacts,
    limits: HtmlConstructionCaptureLimits,
    cancellation: HtmlSemanticCancellation,
) -> Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure>;
```

These consuming methods are implemented by the capture owner.
The supplied builder determines capture identity and `HtmlHostFactsRevision`.
`HtmlConstructionCaptureFacts::new` takes that expected host revision and a
finite vector of private-field `HtmlConstructionCaptureNodeFacts`. A node-fact
constructor takes its exact `HtmlNodeId`, explicit document assignment and
optional role-specific facts. `HtmlConstructionCaptureDocument` is
`ResultDocument` or `Existing(HtmlCaptureBuilderSubject)`. `ResultDocument` is
valid only for a document import and explicitly attests that node's actual
document; a fragment import must use an existing checked capture document.
Every imported non-document record requires one assignment, including template
contents and associated shadow roots. The importer does not infer template-owner
documents or synthesize a live Document for a fragment.

Document facts supply `HtmlScriptingStatus` and `HtmlCaptureRegistryInput`,
including their explicit `Unavailable` states. Element facts supply
`HtmlCapturedIsValue` and `HtmlCaptureRegistryInput`, also explicitly unavailable
when unobserved. Each imported shadow root requires its own explicit
`HtmlCaptureRegistryInput`, including `Unavailable` when unobserved, and the three
additional observed booleans in `HtmlCapturedShadowProperties`: declarative,
available-to-internals and keep-registry-null. A host registry or
keep-registry-null value cannot attest that root's registry. Its ordinary five
shadow properties come from the actual
construction result and cannot be overwritten by fact input. Absent additional
shadow facts are a typed missing-fact failure, not three invented false values.
An ordinary fragment's absent host is structural evidence only when the selected
source fragment has neither an owner nor a shadow-contents role. A locally
unowned shadow-contents fragment has an unavailable external host, rather than
evidence of absence. Source association fragments retain their
actual host relation. For an associated fragment view, the adapter includes its
source owner element as a supplemental imported record and requires that
element's document/registry facts. It establishes that one template/shadow
association, while unrelated owner ancestry, ordinary children and other
associations remain unavailable unless included by the selected closure.
Input facts cannot erase or replace ordinary topology,
names, attributes, text or associations from the result.
An absent local shadow association remains `Unavailable`, including when ordinary
children were completely traversed: construction topology does not attest absence
of an external or imperative shadow root. Actual local roots and properties are
always imported. A qualified producer may establish absence explicitly through
the existing builder and imported mapping; no additional absence fact is inferred.

Semantics extends its checked capture surface for these relation kinds:

- `HtmlCaptureBuilder::add_shadow_fragment(document, properties, registry)`
  validates the supplied builder document and registry, creates one
  `HtmlCapturedFragmentKind::ShadowRoot` with those explicit
  `HtmlCapturedShadowProperties`, and retains `HtmlCapturedHost::Unavailable`.
  It creates no local host or owning edge. The adapter takes the five properties
  from the source fragment's `shadow_properties()` and requires the same three
  additional producer booleans and explicit root-registry input as for a locally
  hosted shadow root; absence is
  `HtmlConstructionCaptureError::MissingShadowFacts`. It never supplies false
  defaults. The existing absent-host establishment operation rejects this
  shadow kind; unavailable external host is not known absence.
- `HtmlCaptureBuilder::establish_template_shadow_reference(template, contents)`
  checks capture identity, HTML-template/fragment kinds, an unset template
  relation and a shadow-root target. It records a non-owning reference without
  changing that target's host, parent or structural dependents. An invalid
  target yields `HtmlCaptureError::InvalidTemplateContentsReference`; ordinary
  foreign/kind/already-established errors retain their existing precedence.
  `HtmlCapturedNode::template_contents_kind()` reports the shared HTML
  `HtmlTemplateContentsKind` only for an established target. The existing
  `template_contents()` still returns that actual target. Ordinary
  `establish_template_contents` continues to establish reciprocal ownership and
  does not accept a shadow-root target.

Both operations use the capture owner's existing admission and immutable
publication. They admit actual node/reference/registry storage and work before
mutation. The non-owning reference consumes its relationship record allowance,
but no second owned edge, depth propagation or dependent-subtree registration.
Capture validation checks reference endpoints separately from owning reciprocity.
No parser/capture identity enters HTML's construction representation.

Before traversal, reject host-revision mismatch, foreign result IDs, duplicate
node-fact records, wrong role/kind, and existing builder handles from another
builder. During source-order traversal, reject a missing required document or
shadow fact only when that node/association is reached. Unknown current control
state, custom-element definitions, resource facts, nonce environment, base facts
and other unestablished host history remain unavailable under the existing
capture builder's rules. Parser metadata may supply its actual initialized
state as explicit producer evidence to those existing fact-establishment
methods; importing a parsed result does not certify unrelated host facts.

The adapter walks ordinary children in order and owning template/shadow
associations for structural topology. It records non-owning contents references
separately, retaining their source kind and target. A referenced target outside
that structural closure is imported as a supplemental fragment and its owning
subtree, with its actual local owning host included as a supplemental record
when present. That does not import unrelated host ancestry, children or other
associations. A locally unowned shadow-role target uses `add_shadow_fragment`
with unavailable host. Every newly reached record still requires its explicit
document and role-specific facts. A visited source-ID mapping ensures each
record and owning subtree is imported once, including references to an already
imported shadow root or cycles consisting only of references. References are
established after their endpoints through the checked reference operation;
they never manufacture reciprocal Template hosts or another structural walk.
The selected source root retains its existing root/parent qualification. The
adapter establishes complete ordered children only for each actually traversed
ordinary/owning closure; supplemental owner records retain unavailable unrelated
relations. Detached records outside this closure and its required supplemental
records are excluded; requesting their mapping returns
`OutsideImportedClosure`. A fact entry that names such a node is refused with the
same error after the traversal and before any record is built, because the
producer keyed it to a node the import never reaches; it is never ignored. The
import clones immutable name/attribute/string backing without UTF-8 conversion,
re-normalization or serialization. A mapping entry retains the original `HtmlNodeId`; source identity is never equated to capture
subject identity.

`HtmlConstructionCaptureLimits::new(mapping_entries, scratch_entries, work)`
selects finite limits for adapter-owned mapping/scratch lengths and work;
the supplied builder's existing `HtmlCaptureLimits` remain the sole capture
logical-allocation/relationship/depth limits. Work is a conservative upper
bound admitted before each source-node, attribute/unit, relationship and
capture-builder operation, with a cancellation check before and after each
operation. The depth bookkeeping of `establish_children` is charged per record
visited, rather than hidden as one visit. All adapter and capture scratch reservations are finite and fallible;
scratch-entry admission precedes them. This is the capture phase's existing
logical capacity model, not a parser-style claim about allocator metadata or
physical RSS. Query-session accounting begins only after a successful capture
and context have been built.

Importing is linear in the imported nodes and supplied facts. Node and fact
lookups use hash indexes keyed by source node identity (the
standard randomly keyed hash map: `HtmlNodeId` provides equality and hashing,
but neither an order nor a public dense index). Each probe is charged one work
credit, expected constant, and an index is only probed, never iterated, so no
outcome depends on hash seeds or bucket order. A repeated fact node is refused
as `DuplicateNodeFacts` at its first repetition, in fact order, after the
foreign-result check of the same fact. Retained arrays and indexes are charged
for moving their existing entries only when growth actually moves them, so those
charges sum to a constant number of credits per entry. Establishing a container's
children or attaching template contents links each attached root below the root
of the parent's component and charges the records visited finding it, so the
cost of an edge does not depend on the order in which a producer attaches edges
or on the size of the subtree below the attached child. The builder keeps each
owning edge's depth as a weighted union-find (path halving) and the greatest
depth below each component root; no record's depth is materialized or revised.
For an arbitrary attachment order an edge costs amortized logarithmic work in
the records. The adapter attaches each container's children before those of its
descendants and creates a shadow root at its host, so the root of a parent's
component is found within a few steps and the work per imported node is
constant: `deeply_nested_sources_charge_constant_work_per_node` measures chains,
nested templates and nested shadow roots, and requires the charge per node at
1,600 nodes to stay within 5% of the charge at 200.

An edge is refused as a cycle exactly when the root of the parent's component is
an attached child, and as over the depth limit exactly when the parent's depth
plus one plus the tallest attached subtree exceeds it; both are decided from
those roots and heights. Only a refusal walks the attached subtrees breadth
first, charged per record, to report the failure reached first: the new edge's
own depth, the parent's depth plus one, is checked against the limit first, and
a cycle is the parent being found at or below an attached child, reported at the
first such record the walk reaches (before that record's depth is checked
against the limit). The one remaining whole-capture pass is `finish`, which runs
once. No `HtmlConstructionCaptureAdapter` charge scales with the number of other
nodes, so a work allowance sized as a fixed amount per node, attribute, name or
value unit and relationship admits every adapter import of that size. The
crate-private controlled-storage builder mode, whose duplicate-child detection
compares every pair, is not reachable through the adapter, which always uses a
builder in ordinary storage mode.

Scratch entries are counted per live logical entry: one per supplied fact and
one per mapped node while the import runs, plus the traversal worklist and, for
one container at a time, its child lists and duplicate-detection set (and, only
while a refusal is named, the list of records its walk visits). All of these are
bounded by a constant number of entries per node: a fragment of `n` childless
siblings peaks at `7n + 2` scratch entries, measured by
`a_fragment_of_childless_siblings_peaks_at_seven_scratch_entries_per_sibling`. The fact index is released when the import ends, whatever the outcome.
The node index is released on refusal; after success it is moved, not copied,
into the returned `HtmlConstructionCaptureMap`, so its entries stop counting as
live scratch (the reported peak still includes them) and become retained memory
of the map.

The map therefore retains, for every mapped node, one entry (the source
`HtmlNodeId` and its checked handle) and one slot of a hash index from a second
clone of that identity to the entry's position, plus whatever spare capacity the
entry list and the index grew. Their logical sizes are bounded by the import's
mapping-entry limit.
`mapping.builder_subject` and `mapping.subject` probe that index, taking
expected constant time rather than scanning; a node outside the imported closure
is one failed probe and still returns `OutsideImportedClosure`, and `subject`
still checks the capture identity first and returns `ForeignBuilder`. The probes
are not charged to any work allowance; they belong to the caller, not to the
import. The index is only probed, never iterated, so no outcome depends on hash
seeds or bucket order. The map has no equality and its `Debug` output lists the
entries, not the index. `HtmlNodeId` was not given a public dense index for
this; that would change the `surgeist-html` public contract.

`HtmlConstructionCaptureImport` owns the builder, the same source result and
`HtmlConstructionCaptureMap`, plus adapter usage. `mapping.builder_subject(id)`
returns the checked handle for a source ID; `into_parts` returns those owners
without allocation. The caller may establish additional actual host facts
through the existing capture builder methods. Alternatively, consuming
`import.finish()` returns `HtmlConstructionCapturedResult { capture, source,
mapping }`. The mapping retains its actual checked builder handles;
`mapping.subject` obtains immutable subjects through `HtmlCapture::subject_id`,
which checks the published originating identity. No duplicate map allocation or
conversion cost is introduced for the identical retained identity/index pair.
Finalization uses a consuming
`HtmlConstructionCaptureImport::finish(self) ->
Result<HtmlConstructionCapturedResult, HtmlConstructionCaptureFinishFailure>`;
its failure retains the complete current import, including any facts the caller
established after traversal. The capture owner shares its existing full-forest depth
validation through a borrowed preflight, so validation/allocation/cancellation
refusal cannot destroy that builder. After all fallible preflight,
the exact builder data moves into its shared capture wrapper and publication
performs no later fallible operation. The existing standard-library Arc setup
limitation remains explicit. `mapping.subject(id, &capture)` checks capture
identity as well as source membership and cannot be applied to another capture
or result. It carries the originating result root and host revision; no integer
or canonical DOM-ID conversion is exposed.

The adapter mutates only newly imported records; existing documents and
registries are referenced without changing their facts or edges. It keeps the
builder's pre-import record/usage checkpoint and does not expose imported
handles while traversal is incomplete. On any admission, allocation,
cancellation, missing-fact or structural failure, it drops all new records,
restores that original logical builder state and returns
`HtmlConstructionCaptureFailure { reason, builder, source, facts, usage }` through
read-only accessors/`into_parts`. Any retained spare capacity remains with that
same builder. Thus failure returns the original usable builder and exact source,
not an immutable partial capture or a partial mapping. Capture finalization
allocates/validates before publication; a refusal preserves
the same import owner for cleanup or an explicitly fresh retry.

`HtmlConstructionCaptureError` is non-exhaustive and distinguishes
`ForeignResult`, `HostRevisionMismatch`, `ForeignBuilder`, `DuplicateNodeFacts`,
`WrongFactRole`, `MissingNodeDocument`, `MissingShadowFacts`,
`OutsideImportedClosure`, `LimitExceeded`, `RepresentationExhausted`,
`AllocationFailed`, `Cancelled` and `Capture(HtmlCaptureError)`. The first reached
failure and admitted usage are retained; rollback does not run useful import
work or replace that reason. No implicit `From<HtmlDocument>` or
`From<HtmlParseReport>` fabricates external facts.

`HtmlSemanticContext` continues to bind one capture and all actual producer
revisions. `HtmlSemanticSession::new` rejects a different capture before query
work. Each session has one sticky first failure and one semantic usage owner.
Context creation admission and qualification setup remain explicit as required
by #1178. Reports retain reached dependencies, context/capture/host identities,
input limits, selected Unicode basis and influencing external revisions.

### Parser evidence, errors, allocation and terminal behavior

All `HtmlSource*`, `HtmlDocumentParseRequest`, `HtmlFragmentParseRequest`,
`HtmlParse*`, tokenizer and insertion-mode diagnostics currently reexported in
`lib.rs` move to parser, except shared local decision enums listed above and the
external base-fact model assigned to semantics. Preprocessing machinery
remains parser-private. UTF-8 source ranges use byte
coordinates; UTF-16 ranges use code-unit coordinates, with source identity and
coordinate basis preserved together. Native construction has no invented source
identity or provenance.

Parser owns `ParseControl`, `OwnedVec`, capacity/shared-allocation tickets,
`ControlIdentity`, work/resource categories, cancellation and first-stop
recording. Its controlled HTML, semantic-capture and CSS/Color adapters describe
and admit actual actions through that same controller. Local cursors do not
open another phase ledger. Semantic query sessions separately own query usage;
capture building separately owns capture admission. These are distinct real
phases, not a way to hide parser execution work.

Admission precedes the corresponding read, reserve, copy, initialized write,
provider action or transfer. Existing cancellation checkpoints and the first
reached missing-fact/syntax/unsupported/resource failure order are preserved.
An allocation that succeeds with excess actual capacity is recorded before
reporting a capacity stop. A stopped owner retains all live allocations and the
original request until explicit cleanup completes; stopped cleanup does not
resume useful parsing. Cleanup failure remains distinguishable from the first
execution failure. Successful partial trees, effects or report fragments are
not returned on error.

`OrdinaryInitializationOwner`, EOF completion, `SealedOrdinaryConstruction`,
`FrozenStandaloneParts` and `HtmlParseReport` remain parser-private.
Construction publication proves an HTML tree; parser publication additionally
requires completed metadata, associations, lexical/source evidence and terminal
state. Native freeze alone cannot stand in for a complete parse report.
#1179 consolidates EOF/terminal ownership and separates general construction
effects from select metadata without changing these proof obligations.

The HTML input color adapter belongs to parser:
[`construction/input_color.rs`](../../crates/surgeist-html-parser/src/construction/input_color.rs),
with its CSS action servicing in
[`construction/css_color.rs`](../../crates/surgeist-html-parser/src/construction/css_color.rs).
Its source path drives `surgeist_css::color::controlled::CssColorParse`: it
starts the parse over its own `CssColorOperationOwner`, owns the three slot banks,
and admits every described `CssColorParseAction` through the original
`ParseControl` before servicing it. It does not call the ordinary
`surgeist_css::color::parse_used_color` helper, which owns local scratch and
hides those actions from the parser ledger. A semantic failure
(`CssColorError`) ends that parse; dropping a pending action or parse abandons
it, leaving the caller-owned banks to the parser. A standard failure is a
`CssColorFailureCheckpoint` whose `fallback` supplies the HTML opaque-black used
color. Numerical transforms lend the separately held Color owner to
`CssUsedColorCheckpoint::transform`, which yields a total `CssColorTransform`
that the adapter admits and computes. CSS does not wrap serialization: the adapter
drives `surgeist_color::Serialization` over the checkpoint's value with the
separate actual `DigitSlot` bank required by the color contract, and the CSS
owner is never borrowed as its own nested numerical owner. The adapter consumes
`surgeist_color::{Color, ColorSpace, PredefinedColorSpace, Transformation,
SerializationRequest, Serialization}` directly for numerical work.
The [color contract](color-crate-contracts.md) owns that numerical API. CSS
grammar, contextual facts and source identities remain CSS-owned; HTML-specific
input branches, source operands, recovery diagnostics and
`HtmlParseColorComputation` remain parser-owned. The adapter maps failures to the
existing `HtmlParseStop::ColorProcessing` domain and retains the original
controller, order of provider calls and provenance. HTML legacy color grammar
and `HtmlLegacyColor` remain HTML values; no numerical matrix, gamut or
serialization implementation is copied into HTML or parser.

## Error conventions and enum evolution

The crate documentation in [lib.rs](../../crates/surgeist-html/src/lib.rs) owns
this policy. A reader predicts any error's shape by asking, in order, whether
the refusal hands back owners, whether it misuses an owner or cursor protocol or
merges many operations, whether a consumer matches every variant exhaustively,
and otherwise that it names a rule or vocabulary.

| Family | Shape | Evolution |
| --- | --- | --- |
| `HtmlConstructionRefusal`, `HtmlConstructionActionFailure` | Carrier struct with the reason and the recovered owners | Fields private |
| the rel, media, microdata-token and time-element interpretation errors, `HtmlCspProtocolError`, and the cursor and reply errors such as `HtmlFocusingError`, `HtmlDialogCursorError`, `HtmlSelectCursorError`, `HtmlSlotAssignCursorError`, `HtmlFormOwnerResetCursorError`, `HtmlCloseWatcherRequestError` | Struct with a private cause, `is_*` predicates and fact accessors | New causes are non-breaking |
| `HtmlConstructionError` with its cause enum `HtmlConstructionCause`, which the parser maps variant by variant | Struct whose `cause()` returns an exhaustive enum with the payload of the cause, and `accounting_fault` | Exhaustive: a new cause fails to compile in the parser |
| Resource accounting that a consumer matches exhaustively: `HtmlIntegerUnitStepError` and `HtmlNumericMaterializationError` (semantics `time_element.rs`, `natural_dimensions.rs`, `image_dimensions.rs`, `semantics.rs`), `HtmlFloatingPointAllocationError` (semantics `range.rs`), `HtmlIntegerRecoveryError` (semantics `semantics.rs`, `meta_pragma.rs`), `HtmlAutocompleteMaterializationError` (semantics `autocomplete.rs`), `HtmlPiPseudoAttributeError` (parser `pi.rs`, semantics `template.rs`) | Enum of the charged resource or integrity failure | Exhaustive: a driver must charge every resource |
| Two-source wrappers `HtmlCspError` (DOM `store/csp.rs`) and the rel, media and microdata-token action errors (parser, semantics) | Enum separating the caller's refusal from this crate's | Exhaustive |
| Rule, syntax, range and unavailability verdicts such as `HtmlTokenError`, `HtmlAttributeConformanceError`, `HtmlTemporalError`, `HtmlSrcsetAuthorViolation`, `HtmlRangeNumericError`, `HtmlSrcsetInterpretationError`, and vocabularies that drivers construct such as `HtmlAttributeValidationError` | Enum naming one rule, syntax class or reason | `#[non_exhaustive]` |

Keyword and state sets that the pinned standard enumerates, and the action,
reply, outcome and step enums of the controlled operations, are exhaustive. Every
error implements `Display` and `Error`; a wrapper's text describes its own
operation and exposes its cause through `source`. Errors capture no backtrace,
because most are `Clone` and `Eq` values returned under exact allocation
accounting. The `From` conversion of an owned cause replaces repeated
`map_err`; the generic caller-error wrappers keep `map_err` because a blanket
`From` would overlap the protocol conversion.

## Consumer, generated-data and attribution changes

This extraction is a breaking public move under the repository's pre-v1 policy.
There are no old-path reexports, compatibility aliases, forwarding crates or
migration machinery. Consumers import the actual owner directly.

| Surface | Required extraction effect |
| --- | --- |
| Root `Cargo.toml`, `Cargo.lock`, `src/lib.rs` | Add both packages and facade dependencies; keep `html` for HTML values and add `html_parser`/`html_semantics` modules. Root `default-members` remains facade-only. |
| `crates/surgeist-html/tests/` | Keep value/native tests in HTML; move `semantics*.rs`, capture and qualified-query cases to semantics; move public parse/source/request/color-input/history/recovery/conformance cases to parser. Mixed cases import HTML and the actual operation owner explicitly. |
| Inline parser/semantic tests | Move with their owner; shared fixture builders remain test-only. Preserve independent expected values and qualified failure cases rather than replacing them with implementation-derived expectations. |
| `crates/surgeist-html/fuzz/` | Preserve the existing package, workers, descriptor IDs, seeds, codecs, observations and regression semantics. Change manifest/imports to depend directly on HTML, parser and semantics. Both existing document/fragment harnesses remain buildable; #1172 owns further adoption/campaign work. |
| Vocabulary generator and `INVENTORY.md` | Stay HTML-owned with the checked vocabulary output. |
| Character-reference generator and data, `PARSER-ORIGINS.md`, html5lib fixtures and conformance matrix | Move to parser with updated owner-relative paths and generator tests; preserve exact pinned normative inputs and fixture licenses. |
| Bidi generator, `tools/data/DerivedBidiClass-18.0.0.txt`, generated `bidi.rs` | Move together to semantics; retain Unicode version, revision and digest qualification. |
| Root API generator | Remains in `api/generator`; its manifest-directory discovery finds both new crates. Refresh `api/crates/surgeist-html.txt`, add parser/semantics audits and refresh `api/public-api.txt` through the generator. No hand editing generated audits. |
| Root and crate notices | Relocate incorporated html5ever notices/origin links and bundled html5lib fixtures to parser, Unicode data notices to semantics; update root `NOTICE.md` and `licenses/{html5ever,whatwg,unicode,w3c}/NOTICE.md` paths where affected. Preserve legal text and upstream source identities. |
| Documentation and policy discovery | Update root `AGENTS.md`, affected crate guides/READMEs, `docs/explanation.md`, `docs/html-parser-suitability.md`, `references/SOURCE-EDITIONS.md` and cross-links to moved artifacts; distinguish language authority from implementation provenance. |
| Other production callers and examples | At the source basis no non-facade production crate imports HTML. There is no HTML examples directory. Re-run the repository import/path inventory after integration; do not assume that remains true after concurrent work. |

## Acceptance evidence

Extraction acceptance requires focused tests for HTML, parser and semantics,
their warning-denied Clippy checks, facade check/lib tests/Clippy, formatting,
the existing fuzz contract suite and supported existing harness feature builds.
Use the root/crate command inventories and bounded `pisct run`; do not substitute
a blanket workspace suite. Run the moved vocabulary, reference and bidi
generators in check mode and root-owned API audit tests/refresh/check.

Boundary tests must demonstrate exact isolated-surrogate sharing, first
duplicate rejection, result/capture/source foreign-ID rejection, native and
parsed construction under the same read API, association edges and fragment
views, controlled attribute validation and construction publication interruption,
retained allocation ownership on refused transfer, no partial successful result,
reached versus unreached unavailable facts, bounded definition lookup, fragment
initialization and integration-point parity, and sticky first failure through
cleanup. Existing numerical/color and semantic qualification witnesses remain
with their owners.

Attribute-edit evidence additionally covers a real source merge with installed
and earlier lexical collisions, namespace-distinct names and first-value/source
order retention; an independently owned duplicate token exercises the path a
normal tokenizer deduplicates. Refusals at each borrowed preflight and new plan,
witness, provider and ready boundary retain the exact caller input. One-short
capacity/work cases use the actual new plan-cell and slot layouts, separately
accounting for old work and added witness work and their changed live peaks.
Foreign target/group use, witness replay, wrong namespace, out-of-order slots,
equal text in different backing and expired/reallocated names cannot authorize
an append. Target mutation and publication are excluded for the entire active
group. Tests retain the actual prefix/stage/redirect after cancellation, observe
actual reserve capacity, drain ID/initial effects before the next append, and
retire all witness/plan/string/attribute tickets to zero. Existing wide-merge
and duplicate-only poll-span witnesses remain applicable. Checked initial and
single append, replacement, adjacent swap and pop need separate valid, duplicate,
refusal and partial-prefix evidence appropriate to their actual transitions.
Exercise final adjusted/foreign and generated names after preceding effects,
native uniqueness, normal parsed duplicate recovery, no effects for a refused
slot, and the measured current-prefix comparison/poll bound.

The final integration review checks the complete production graph, public API
inventory, actual `use` paths and absence of private reach-through, unchecked
construction, copied rule tables and compatibility shims. A child extraction is
ready to integrate only when its owner-controlled constructors and adapters
implement these contracts and its focused evidence passes. No exception to
these boundaries is selected by this reference.
