# surgeist-dom

Canonical native document ownership with atomic edits and owned immutable
snapshots. The native contribution supports Document, native Element,
ordinary DocumentFragment, Text, Comment, ProcessingInstruction and DocumentType. It directly consumes
HTML's exact UTF-16 `DocumentString`, `QualifiedName`, `Attribute` and
`OrderedAttributes`, CSS's checked `CssViewTransitionIdent`, and CSSOM's real
sheet/rule identities and immutable snapshots. Parser, runtime and facade
composition remain outside its dependencies.

`Dom::new` requires explicit finite `DomLimits` for managed nodes, stored string
code units, commands per batch, and retained change-history entries and bytes.
Detached nodes count toward the live limits. Logical string lengths count each
node's stored units even when immutable backing is shared, including PI targets,
data and derived pseudo-attribute map fields, all three doctype strings, and
each document's content type, encoding name and URL serialization (31 units for
the native defaults). Element names and every attribute's namespace URI,
prefix, local name and value count as separate logical stored slots. An applicable
cryptographic nonce slot counts independently of its content attribute.
Limits do not promise a total heap bound: caller-owned batches, snapshot/report
retention, allocator bookkeeping and temporary transaction preparation are
separate. A limit update
cannot lower live capacity below admitted usage; smaller history bounds discard
coverage without mutating document state.

`DomBatch` creates nodes through batch-qualified tickets. `Dom::apply` validates
ordered edits against private staged state and publishes once. Only a successful
commit resolves surviving tickets to owner-qualified, generation-safe `NodeId`s.
A failed or cancelled batch preserves tree state, revisions, identities, slot
assignment and history. Unpublished creation followed by destruction is a
semantic no-op. Final no-op batches do not advance `DomRevision`; replacing an
existing node with an equal fresh node does. Revision and owner exhaustion reject
without wrapping; a destroyed slot whose generation is exhausted is retired.

The supported mutations insert/move nodes, transfer a fragment's children in
order, replace a child, adopt within the same store, detach, destroy detached
non-Document subtrees, and replace Text/Comment/PI data using UTF-16 code-unit
offsets. Documents reject Text children and accept at most one Element and one
doctype, with the doctype preceding the Element. Elements and fragments accept
Elements and CharacterData. Doctypes cannot be inserted into them or have children.
A fragment is emptied by insertion and never becomes an ordinary child.
Reference-child membership and cycles are validated before publication.
Direct pre-insertion uses an empty exclusion list, so an attached Document
Element or doctype must be detached before reinsertion. Replacement excludes
its replaced child instead. Validation checks the original tree and reference
membership before mutation.

Cross-document insertion adopts each inserted node without changing its ID.
A transferred fragment retains its own node document; an empty fragment insert
is a semantic no-op. Explicit `adopt` removes its supplied node and updates its
whole subtree, including a fragment root. The source's child replacement also
explicitly adopts its supplied fragment before transferring its children.
Adopting a Document is `NotSupported`; foreign-store adoption is `ForeignOwner`.
No mutation invokes
callbacks, runs scripts or performs host effects.

PI construction validates the target using the DOM-delegated XML Name
production and rejects initial data containing `?>`. It does not apply HTML's
stricter native authoring grammar or reserved-target exclusions. CharacterData
replacement preserves arbitrary exact units, including `?>`, and recomputes the
whole-string XML pseudo-attribute map through HTML's shared grammar cursor.
Invalid pseudo-attributes leave an empty map and preserve raw data. This does
not implement PI attribute setters, style loading or live ranges. Doctype
construction uses DOM's relaxed name validation, including valid empty names;
public and system identifiers remain exact strings. `node_name()` returns the
source-defined fixed name, qualified Element spelling, PI target or doctype name
for each exposed kind.

`create_element` currently creates no-namespace generic Elements in the native
XML document, preserving case and exact UTF-16 units. It uses the selected relaxed
DOM name validation, not an XML QName regex or HTML authoring validation. A name
such as `input` or `template` has no HTML family semantics in this namespace.
Registry and definition bindings are known-null, saved creation `is` is absent,
and shared HTML rules establish the uncustomized state. A later `is` attribute
does not rewrite saved creation state.

`create_element_ns` validates the selected relaxed Element QName rules and derives
native initialization from the actual expanded name through HTML's shared producer.
It admits generic XML namespaces and source-proved common HTML/SVG/MathML profiles,
preserving prefix, case and exact UTF-16. It does not apply HTML content-model
validation or parser aliases. Registry/definition are known-null; actual valid
HTML custom names establish Undefined state through shared rules. Common state
includes the actual empty reaction queue/unset click flag, and the HTML-only
closed focus/internals/popover tuple. Exposed operations preserve those values.
Special built-ins lacking their actual family adapter reject atomically with
`NativeInitializationRequired` carrying the exact HTML-owned requirement. SVG
lowercase script is one such requirement; uppercase Script is ordinary SVG.

HTML/SVG/MathML Elements retain a real initially empty cryptographic nonce slot.
Exact null-namespace `nonce` attribute changes update it, removal clears it, and
`set_nonce` changes only the slot. Case/namespace mismatches do not affect it.
State-only edits report `ElementState`, separately from `Attributes`, and advance
the same DOM revision. Clone/import appends actual attributes through their change
steps, then copies the captured nonce slot. Native Document environment stores
real null browsing-context and never-navigable proof independently of XML class.
Insertion runs common steps over inserted descendants; this native environment
proves nonce content remains visible without inventing CSP facts. No API attaches
a browsing context or advances the closed common tuple into interactive state.

This common contribution does not complete native HTML. Remaining exact producer
requirements include native form/control relations and values, template hosted
contents/inert documents, HTML/SVG script state and preparation, frame/iframe
navigables, link/style associations, details/dialog state, and resource/media
families. Their real initialization and applicable attribute/tree/adoption hooks
must join before admission; a requirement error is not a permanent HTML dialect.

Attribute edits use ordered expanded identity `(namespace, local_name)`. Ordinary
name operations select the first exact qualified spelling; a newly set literal
`xml:lang` is an unnamespaced local name. Namespace-aware setting validates and
splits only the first colon, normalizes empty namespace to none and applies the
source's reserved namespace checks. Updating an existing expanded name preserves
its prefix and position. All raw values, including isolated surrogates and CR/LF,
remain exact. Absent removal is a no-op and performs no construction validation.
Snapshots expose Element-only children, siblings and zero-based sibling indices
over the same canonical identities, without counting Text or Comment nodes.

`DomSnapshot` owns immutable storage for every managed node, including detached
nodes and multiple documents. It survives later edits, destruction and the
store's own destruction. Reads reject foreign/stale identities. Document views
validate membership at their captured owner/document/revision basis. A Document
belongs to itself for membership; its `owner_document()` getter returns `None`.
A detached node keeps its owner document while its ordinary root and
connectedness change.

`DomHostPublisher` captures complete fixed focus/hover element-selection
observations from an external producer against one exact `DomDocumentStamp`.
Supported absence, unsupported capability and inapplicability remain distinct.
Its immutable source identity includes the selected domain configuration; checked
`HostStateRevision` advances only when observations change. DOM-only revisions
require explicit whole-domain recomputation without advancing equal host facts.
Captures retain exact historical values independently of the publisher. Complete
adjacent coverage is bounded to two domains, with explicit fallback for older
history and whole-domain invalidation for availability changes. This boundary
does not implement HTML focus policy, pointer processing or canonical controls;
Runtime owns its concrete surface-selection association and capture adapter.

Native `create_document()` establishes an XML document with `application/xml`
content type, `UTF-8` encoding, `about:blank` URL, no-quirks mode, declarative
shadow roots disabled and a known-null custom element registry. A checked
document view exposes these canonical values through `metadata()`. Each native
document has a fresh opaque origin, distinct across documents and stores even
when their URLs match. Cloning an origin preserves its identity; URL spellings
and source labels do not establish origins. Metadata and origins are staged
privately with the rest of the batch, and old snapshots retain them. Ordinary
doctype edits do not change document mode. Host/imported metadata and registry
objects have no admission API yet. Native environment proof also survives captures.

`import_node` reads a checked node from an owned immutable `DomSnapshot` and
copies it into a target Document with fresh identities and a detached root.
The source can belong to another store or retain nodes already destroyed in its
live store. Shallow import still copies every Element attribute; deep import
walks ordinary children in order without recursive native calls. Exact attribute
names/prefixes/values are appended through the same admitted attribute writer;
PI cloning copies its current target/data, including `?>` introduced by a later
CharacterData edit, without repeating initial PI construction validation.
Document import is `NotSupported`. Supported common import includes no special
HTML family, shadow-tree or registry-object installation. The returned batch ticket names
only the imported root; descendant IDs become available from the committed
snapshot. Native tickets before or after that root remain independent of its
private descendant allocations. Failure/cancellation publishes none of them,
and destroying the unpublished imported subtree is a semantic no-op. Copied
nodes and all logical string slots consume target limits even when immutable
string backing is shared with the source capture.

Canonical View Transitions state is an explicitly initialized structural
capability in the same atomic owner. Configure finite transition, participant
record and generated-pseudo capacities with `DomLimits::with_view_transitions`;
these capacities start at zero. Participant names count toward stored UTF-16
units. Hidden, inactive records and pseudos remain admitted until explicit
release; retaining caller snapshots does not pin live nodes or consume live
capacity. Node, transition and record creation tickets are separate and resolve
only after publication. Transition/record generations are checked and never
reissued.

Every feature edit requires an exact already-published `DomDocumentStamp` for
its pre-batch basis. Native document creation leaves this feature uninitialized.
`initialize_view_transitions` admits an actual captured CSSOM sheet and establishes
source defaults: absent active transition, rendering suppression false and
show-tree false. An initialized binding always contains a real sheet identity;
uninitialized is not the source's default null stylesheet. This structural
association does not establish the source-required fresh UA stylesheet's actual
cascade-origin placement or document installation. Those effects belong to
[#1058](https://github.com/bj-data-eng/surgeist/issues/1058).

A transition owns an ordered map of case-sensitive checked participant names,
with pending, old-only, new-only or paired structural branches. An old branch
certifies no image. A newly assigned new-element reference must resolve to an
actual same-document Element, and one element cannot occupy two names in the
same transition. Pending records cannot be set up. Setup is one-shot and creates
the separate generated relation: root, groups in record order, one image pair
per group, then old before new when present. This never changes ordinary node
children. Actual Elements and all owned VT pseudos have independent captured
flags, initially false, through a typed subject domain. Other CSS pseudo families
need their future concrete identities under
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124); the selected source's
broader CSS-element capture requirement remains outside the admitted family.

Complete pseudo relation reads use this snapshot's actual document element as
the mandatory ultimate origin. Replacing it changes current origin readback;
removing it produces `ViewTransitionOriginUnavailable` without rejecting the
ordinary tree edit. Identity/captured-flag reads remain available for cleanup.
Old snapshots keep their old actual origins. Each pseudo's originating pseudo
is its parent in the generated tree.

Rule associations retain real CSSOM identities qualified by the snapshot
revision used to admit them. Ordinary slots require actual directly installed
Style rules in the bound sheet; the keyframes slot requires Keyframes. The
image-animation group contains zero, one old/new rule, or two distinct ordinary
rules ordered old then new. DOM validates kinds, membership, owner and known
revision order, while #383/#1058 own their content, installation and removal.
An old retained association is historical evidence, not proof of current CSSOM
state. Clearing association records performs no CSSOM mutation.

`clear_view_transition_state` follows retained new-element identities and clears
still-live captured flags, active state and show-tree atomically. It preserves
suppression, records, generated identities and rule associations. Reset
suppression separately; release an inactive transition separately. Repeated
cleanup is a no-op; cleanup of an old transition rejects if a newer transition
is active, before touching its potentially reused participants.

Detaching or adopting a participant preserves its full `NodeId`, including when
it moves to another Document. Clear follows that identity across documents.
Explicit native destruction retains the existing detached-root/subtree rules
and is not blocked by active or inactive records. The record retains its exact
historical identity; `resolve_new_element` distinguishes never assigned
`Ok(None)` from destroyed `Err(StaleNode)`. Clear skips destroyed generations
and never follows a reused slot. This is the selected native-destruction policy,
not a browser object-lifetime claim. Runtime must requalify current participation
after canonical revisions.

Runtime [#388](https://github.com/bj-data-eng/surgeist/issues/388) owns actual
images, geometry/style snapshots, phase timing and the exact resource join to
these DOM transition/record identities and revisions. DOM stores no image or
resource handle and certifies no usable composed transition, rendering, pointer
hit testing, accessibility behavior or CSSOM effect. The selected
[View Transitions capture](../../references/css-view-transitions-1--CRD-css-view-transitions-1-20240328--7f8e537956b6.md)
and amended [#1201](https://github.com/bj-data-eng/surgeist/issues/1201) define
this bounded contribution; full composition remains in #382/#391.

`DomSnapshot::html_capture` adapts one immutable document directly into
`surgeist-html-semantics`, including every managed same-document detached node.
It retains that snapshot, a checked bidirectional canonical/semantic identity
map, and an optional host capture validated against the exact document stamp.
Qualified names, ordered attributes, current nonce, saved creation `is`, registry
and custom-element state come from actual canonical records. It creates no
intermediate HTML construction tree. Common slots not represented by HTML's
capture schema remain available through the retained `source()` snapshot.

`DomHtmlCapture::query` invokes the real bounded HTML semantic session and
preserves its typed unavailable/inapplicable outcomes, result origins and
conservative recomputation contract. Native documents' actual absent browsing
context establishes disabled scripting under HTML's source rule; it does not
establish image policy, header CSP or language fallback history. Current host
captures retain their actual focus/hover observations and full external stamp;
they do not supply those other domains. Construction limits bound mapped
subjects and adapter work separately from HTML capture retention and query
limits. Failed or cancelled composition returns no partial successful capture
and changes neither DOM nor host revisions. Old captures retain their original
membership after adoption, destruction and import elsewhere.

This ordinary/common semantic profile does not complete #1124. Shadow/slot and
flat-tree relations, live controls, template owner/cache state, registry objects,
parser installation and the remaining qualified external domains still require
their actual owning adapters. A future hosted root needs its real host and
node-document associations, including cross-document host/root relationships;
it must not inherit ordinary fragment absence from this profile.

Each commit reports complete changed domains for this supported surface,
including `Attributes`, `NodeDocument` and distinct View Transition document,
transition/record and captured-subject domains. Captured-only edits do not report
`Attributes`. Ordered `Children` changes also cover
derived Element-sibling/index changes.
`changes_since` returns chronological records for an exact covered interval or
typed `FullRecomputeRequired`, never a successful incomplete delta. History is
bounded by entry count and entry-record plus exact change-array payload bytes.
Zero or insufficient history budget does not reject an otherwise legal edit;
it drops unavailable coverage. Caller-held reports retain their memory
independently. Cancellation is checked before staging, between commands and
immediately before publication; the final cancellation observation is the
publication boundary.

HTML document creation, specialized HTML/SVG/MathML native family initialization,
shadow trees and HTML installation are not exposed yet. This crate does not
claim complete DOM or HTML conformance or replace the retained/facade
integrations. Those remaining
contracts stay with [#1074](https://github.com/bj-data-eng/surgeist/issues/1074),
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124) and
[#1135](https://github.com/bj-data-eng/surgeist/issues/1135).

The selected [DOM foundation capture](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md)
at revision `3071e5f269448350fdea79bb284eedaf17bc431e` supplies ordinary
tree/pre-insertion, fragment transfer, ownerDocument, native document metadata,
nodeName, PI/doctype
construction and CharacterData rules. PI targets and pseudo-attribute maps use
the retained [XML grammar](../../references/xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md)
and [XML-stylesheet pseudo-attribute rules](../../references/xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md).
The issue's selected native ownership, finite admission, atomic publication and
snapshot policies supplement that bounded source surface. Existing retained
atomicity and no-op tests supplied reuse evidence; its ownerless IDs,
borrowed snapshots, projection slots and public API are not reused.

From the repository root, through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-dom
cargo clippy --offline --locked -j 1 -p surgeist-dom --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-dom -- --check
```
