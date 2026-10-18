# Document creation and import

These creation paths use the same [canonical tree and publication owner](canonical-tree.md). They establish different causes and metadata; [completed parser-report installation](html-installation.md#completed-parser-reports) transfers retained history separately.

## Native XML document

Native `create_document()` establishes an XML document with `application/xml`
content type, `UTF-8` encoding, `about:blank` URL, no-quirks mode, declarative
shadow roots disabled and a known-null custom element registry. A checked
document view exposes these canonical values through `metadata()`. Each native
creation through this primitive has a fresh opaque origin, distinct across
documents and stores even
when their URLs match. Cloning an origin preserves its identity; URL spellings
and source labels do not establish origins. Metadata and origins are staged
privately with the rest of the batch, and old snapshots retain them. Ordinary
doctype edits do not change document mode. There is no general host-metadata or
registry-object setter; qualified [completed-report transfer](html-installation.md#completed-parser-reports) has its own contract.
Native environment proof also survives captures.

## HTML document skeleton

`create_html_document(associated_document, title)` implements the selected DOM
createHTMLDocument tree in the same staged owner: Document, html doctype, HTML
`html` element, `head`, optional `title` with a Text child, then `body`. `None`
omits title; `Some("")` creates both title and empty Text. The actual same-store
associated Document supplies the origin identity, while the establishing native
environment retains its explicit selection/image policy. New metadata is HTML,
`text/html`, UTF-8, `about:blank`, no-quirks, declarative shadow permission false
and null registry. Two creations from one associated Document share its origin;
other source metadata is not inherited. This document retains actual absent
browsing context/never-navigable/inactive native proof, independently of class.
It does not attach a Window, execute a parser or authorize external lifecycle.

Only the Document has a caller creation ticket; all skeleton nodes are private
allocations available in the committed snapshot. Omitted-title creation adds
5 nodes and 125 stored UTF-16 units; supplied title adds 7 nodes and
`158 + title.len()` units, including empty Text. These increments join existing
store usage and all failures/cancellation roll back the complete graph. There is
no class/metadata setter or empty-HTML shortcut. The existing zero-argument
native XML creator keeps its fresh opaque origin; it does not claim WebIDL
new Document()'s associated-global origin semantics.

## DOMImplementation XML creator

`create_xml_document(associated_document, namespace, qualified_name, doctype)`
implements the separate DOMImplementation XML creator. The new Document keeps
XML class and default metadata. A nonempty qualified name creates an exact
namespaced root; an empty name creates no root. A supplied actual DocumentType
is moved and adopted through the ordinary canonical operations before the root
is appended. Its identity survives, including when it came from another Document
in this store. Only the new Document receives this operation's caller ticket.

After insertion, the creator copies the associated Document's actual origin and
selects content type from the namespace argument: HTML gives
`application/xhtml+xml`, SVG gives `image/svg+xml`, and all others give
`application/xml`. This selection also applies to an empty qualified name.
The closed native environment uses the associated Document's explicit input
policy; it does not acquire a browsing context or execution service. Final
metadata occupies 37 XHTML, 29 SVG or 31 ordinary XML UTF-16 units, independently
of root and doctype fields. Admission also covers initial default metadata and
overlapping replacement backing. XHTML `create_element` uses HTML namespace
without changing the XML Document's case-preserving behavior.

A root such as HTML Style can reach external processing during insertion.
Ordinary atomic application refuses that boundary without publication. Composed
Source execution retains the actual same-command tail through publication,
parking and reacquisition: insertion effects finish before origin, MIME and
ticket finalization, exactly once. It neither restarts creation nor promises
rollback of an already published phase. Complete change evidence and old
snapshots follow the same canonical owner.

## Captured native import

`import_node` reads a checked node from an owned immutable `DomSnapshot` and
copies it into a target Document with fresh identities and a detached root.
The source can belong to another store or retain nodes already destroyed in its
live store. Shallow import still copies every Element attribute; deep import
walks ordinary children in order without recursive native calls. Exact attribute
names/prefixes/values are appended through the same admitted attribute writer;
PI cloning copies its current target/data, including `?>` introduced by a later
CharacterData edit, without repeating initial PI construction validation.
Direct Document or ShadowRoot import is `NotSupported`. Supported common import
includes no unadmitted special HTML family or registry-object installation. A
clonable hosted shadow root copies deeply after the ordinary-descendant branch,
even for shallow host import; non-clonable roots are omitted. The returned batch ticket names
only the imported root; descendant IDs become available from the committed
snapshot. Native tickets before or after that root remain independent of its
private descendant allocations. Failure/cancellation publishes none of them,
and destroying the unpublished imported subtree is a semantic no-op. Copied
nodes and all logical string slots consume target limits even when immutable
string backing is shared with the source capture.

[Back to surgeist-dom](../README.md).
