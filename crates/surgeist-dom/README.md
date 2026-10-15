# surgeist-dom

Canonical native document ownership with atomic edits and owned immutable
snapshots. The native contribution supports Document, ordinary DocumentFragment,
Text, Comment, ProcessingInstruction and DocumentType. It directly consumes
HTML's exact UTF-16 `DocumentString`; it has no parser, runtime, CSSOM or facade
dependency.

`Dom::new` requires explicit finite `DomLimits` for managed nodes, stored string
code units, commands per batch, and retained change-history entries and bytes.
Detached nodes count toward the live limits. Logical string lengths count each
node's stored units even when immutable backing is shared, including PI targets,
data and derived pseudo-attribute map fields, all three doctype strings, and
each document's content type, encoding name and URL serialization (31 units for
the native defaults).
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
order, detach, destroy detached non-Document subtrees, and replace Text/Comment/PI
data using UTF-16 code-unit offsets. Documents reject Text children and accept
at most one doctype. Doctypes cannot be inserted into fragments or have children.
A fragment is emptied by insertion and never becomes an ordinary child.
Reference-child membership and cycles are validated before publication.
An attached doctype must be explicitly detached before reinsertion; direct
pre-insertion validation still sees the document's existing doctype.
Cross-document insertion
is explicitly unsupported until adoption is implemented. No mutation invokes
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
source-defined fixed name, PI target or doctype name for each exposed kind.

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

Each commit reports complete changed domains for this supported surface.
`changes_since` returns chronological records for an exact covered interval or
typed `FullRecomputeRequired`, never a successful incomplete delta. History is
bounded by entry count and entry-record plus exact change-array payload bytes.
Zero or insufficient history budget does not reject an otherwise legal edit;
it drops unavailable coverage. Caller-held reports retain their memory
independently. Cancellation is checked before staging, between commands and
immediately before publication; the final cancellation observation is the
publication boundary.

Element/attribute construction, HTML document creation and native initialization,
shadow trees, adoption/import, HTML installation and semantic capture adapters
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
