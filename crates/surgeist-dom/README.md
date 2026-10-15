# surgeist-dom

Canonical native document ownership with atomic edits and owned immutable
snapshots. The native contribution supports Document, generic XML Element,
ordinary DocumentFragment, Text, Comment, ProcessingInstruction and DocumentType. It directly consumes
HTML's exact UTF-16 `DocumentString`, `QualifiedName`, `Attribute` and
`OrderedAttributes`; it has no parser, runtime, CSSOM or facade
dependency.

`Dom::new` requires explicit finite `DomLimits` for managed nodes, stored string
code units, commands per batch, and retained change-history entries and bytes.
Detached nodes count toward the live limits. Logical string lengths count each
node's stored units even when immutable backing is shared, including PI targets,
data and derived pseudo-attribute map fields, all three doctype strings, and
each document's content type, encoding name and URL serialization (31 units for
the native defaults). Element names and every attribute's namespace URI,
prefix, local name and value count as separate logical stored slots.
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
does not rewrite saved creation state. Namespace-aware Element creation and HTML,
SVG or MathML native initialization have no public constructor yet.

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

Native `create_document()` establishes an XML document with `application/xml`
content type, `UTF-8` encoding, `about:blank` URL, no-quirks mode, declarative
shadow roots disabled and a known-null custom element registry. A checked
document view exposes these canonical values through `metadata()`. Each native
document has a fresh opaque origin, distinct across documents and stores even
when their URLs match. Cloning an origin preserves its identity; URL spellings
and source labels do not establish origins. Metadata and origins are staged
privately with the rest of the batch, and old snapshots retain them. Ordinary
doctype edits do not change document mode. Host/imported metadata and registry
objects have no admission API yet.

`import_node` reads a checked node from an owned immutable `DomSnapshot` and
copies it into a target Document with fresh identities and a detached root.
The source can belong to another store or retain nodes already destroyed in its
live store. Shallow import still copies every Element attribute; deep import
walks ordinary children in order without recursive native calls. Exact attribute
names/prefixes/values are appended through the same admitted attribute writer;
PI cloning copies its current target/data, including `?>` introduced by a later
CharacterData edit, without repeating initial PI construction validation.
Document import is `NotSupported`. Generic XML import includes no HTML family,
shadow-tree or registry-object installation. The returned batch ticket names
only the imported root; descendant IDs become available from the committed
snapshot. Native tickets before or after that root remain independent of its
private descendant allocations. Failure/cancellation publishes none of them,
and destroying the unpublished imported subtree is a semantic no-op. Copied
nodes and all logical string slots consume target limits even when immutable
string backing is shared with the source capture.

Each commit reports complete changed domains for this supported surface,
including `Attributes` and `NodeDocument`. Ordered `Children` changes also cover
derived Element-sibling/index changes.
`changes_since` returns chronological records for an exact covered interval or
typed `FullRecomputeRequired`, never a successful incomplete delta. History is
bounded by entry count and entry-record plus exact change-array payload bytes.
Zero or insufficient history budget does not reject an otherwise legal edit;
it drops unavailable coverage. Caller-held reports retain their memory
independently. Cancellation is checked before staging, between commands and
immediately before publication; the final cancellation observation is the
publication boundary.

HTML document creation and HTML/SVG/MathML native Element initialization,
shadow trees, HTML installation and semantic capture adapters
are not exposed yet. This crate does not claim complete DOM or
HTML conformance or replace the retained/facade integrations. Those remaining
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
