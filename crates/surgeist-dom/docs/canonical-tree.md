# Canonical tree, edits and publication

Use this contract when changing tree mutation, exact values, identity, admission or change coverage. Document constructors and import have their own [creation contract](document-creation.md).

## Finite admission and atomic batches

`Dom::new` requires explicit finite `DomLimits` for managed nodes, stored string
code units, commands per batch, Slot references, raw Slot assignment inputs per
batch, and retained change-history entries and bytes. Those seven capacities
are followed by a required `DomNameLimits` policy for retained name bytes,
registry entries, operation work and temporary storage. Profiles with no
assignments explicitly select zero Slot capacities.
Detached nodes count toward the live limits. Logical string lengths count each
node's stored units even when immutable backing is shared, including PI targets,
data and derived pseudo-attribute map fields, all three doctype strings, and
each document's content type, encoding name and URL serialization (31 units for
native XML and inert owners, 25 for createHTMLDocument). Element names and every attribute's namespace URI,
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
assignment and history. Unpublished creation followed by destruction with no
other final semantic change is a no-op. Final no-op batches do not advance `DomRevision`; replacing an
existing node with an equal fresh node does. Revision and owner exhaustion reject
without wrapping; a destroyed slot whose generation is exhausted is retired.

The supported mutations insert/move nodes, transfer a fragment's children in
order, replace a child, adopt within the same store, detach, destroy detached
non-Document/non-ShadowRoot subtrees, including their owned shadow and Template contents trees, and replace Text/Comment/PI data using UTF-16 code-unit
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
whole shadow-including subtree, including an ordinary fragment root. The source's child replacement also
explicitly adopts its supplied fragment before transferring its children.
Adopting a Document is `NotSupported`; directly adopting a ShadowRoot is
`HierarchyRequest`; foreign-store adoption is `ForeignOwner`.
Ordinary batch mutations do not invoke callbacks, run scripts or perform host
effects; reached external requirements use the [Source protocol](source-and-style.md).

## Names and native element initialization

Element names and attribute names use `DomQualifiedName`: owner-qualified
namespace and local-name atoms, with a separately retained exact prefix.
`local_name()` and `namespace_uri()` borrow their exact UTF-16 text;
`local_name_atom()` and `namespace()` expose the strong opaque tokens.
`DomNameAtom` equality and hashing compare identity. `DomQualifiedName` and
`DomAttribute` equality and hashing compare exact values across owners, including
prefixes. `same_expanded_identity` compares namespace/local identity within one
owner and rejects a foreign owner; prefixes do not participate in that comparison.

`Dom::bind_name` resolves exact text without language validation, case folding,
node creation, revision advancement or history. Element and attribute ingress
perform their existing language processing before binding the resulting names.
The owner keeps a bounded weak index; tokens and snapshots retain their values
after owner destruction. Same-owner copies reuse tokens, while cross-owner
imports resolve destination tokens without changing the text. Attribute values,
prefixes and ordinary text are not mandatory atoms.

PI construction validates the target using the DOM-delegated XML Name
production and rejects initial data containing `?>`. It does not apply HTML's
stricter native authoring grammar or reserved-target exclusions. CharacterData
replacement preserves arbitrary exact units, including `?>`, and recomputes the
whole-string XML pseudo-attribute map through HTML's shared grammar cursor.
Invalid pseudo-attributes leave an empty map and preserve raw data. This does
not implement PI attribute setters or style loading. Doctype
construction uses DOM's relaxed name validation, including valid empty names;
public and system identifiers remain exact strings. `node_name()` returns the
source-defined fixed name, qualified Element spelling, PI target or doctype name
for each exposed kind.

`create_element` validates DOM's whole local name before any transformation.
In native XML it preserves exact case and no namespace; a name such as `input`
or `template` has no HTML family semantics there. In HTML Documents it folds
ASCII uppercase and selects HTML namespace, then requires the actual native
family initialization. `INPUT` therefore requires the real Input family; it
cannot evade admission as a generic Element. The whole name `a:B` becomes local
name `a:b` with no prefix. QName splitting belongs to namespace-aware creation.
Registry and definition bindings are known-null, saved creation `is` is absent,
and shared HTML rules establish the appropriate uncustomized/undefined state.
A later `is` attribute does not rewrite saved creation state.

`create_element_ns` validates the selected relaxed Element QName rules and derives
native initialization from the actual expanded name through HTML's shared producer.
It admits generic XML namespaces, source-proved common HTML/SVG/MathML profiles,
and actual native HTML Canvas/Marquee/Frame/Iframe/Slot/Form/Fieldset/Button/Input/
Textarea/Output/Select/Option/Optgroup/Selectedcontent/Details/Base/Link/Img/Picture/
Source/A/Area/Template/Script and SVG Script families,
preserving prefix, case and exact UTF-16. It does not apply HTML content-model
validation or parser aliases. Registry/definition are known-null; actual valid
HTML custom names establish Undefined state through shared rules. Common state
includes the actual empty reaction queue/unset click flag, and HTML-only common
state with a nullable canonical `NodeId` for previous focus. Native and cloned
elements start null; parsed installation maps the report's actual null-only
state. Attribute edits, detach and adoption preserve the value. Remembered-focus
execution consumes its qualified [Source observation](dialogs-and-focus.md#entry-scope-and-remembered-focus).
Internals/popover fields retain their closed values.
Special built-ins lacking their actual family adapter reject atomically with
`NativeInitializationRequired` carrying the exact HTML-owned requirement. SVG
lowercase script has its actual native state; uppercase Script is ordinary SVG.

## Nonce state

HTML/SVG/MathML Elements retain a real initially empty cryptographic nonce slot.
Exact null-namespace `nonce` attribute changes update it, removal clears it, and
`set_nonce` changes only the slot. Case/namespace mismatches do not affect it.
State-only edits report `ElementState`, separately from `Attributes`, and advance
the same DOM revision. Clone/import appends actual attributes through their change
steps, then copies the captured nonce slot. Standalone native Document environment stores
real null browsing-context and never-navigable proof independently of XML class.
Insertion runs common steps over inserted shadow-including descendants; this native environment
proves nonce content remains visible without inventing CSP facts. Hosted creation
uses a separate controlled source protocol; it does not convert a closed Document
or advance its common tuple into interactive state.

## Attributes and contextual reads

Attribute edits use ordered expanded identity `(namespace, local_name)`. Ordinary
name operations lowercase the input in ASCII only for HTML-namespace Elements
whose actual current node-document is HTML, then select the first exact qualified
spelling. An existing matching attribute retains its actual namespace/prefix;
a newly set literal `xml:lang` is an unnamespaced local name. Namespace-aware setting validates and
splits only the first colon, normalizes empty namespace to none and applies the
source's reserved namespace checks. Updating an existing expanded name preserves
its prefix and position. All raw values, including isolated surrogates and CR/LF,
remain exact. Absent removal is a no-op and performs no construction validation.
Snapshots expose Element-only children, siblings and zero-based sibling indices
over the same canonical identities, without counting Text or Comment nodes.
`DomNode::node_name`/`tag_name` uppercase the qualified spelling in ASCII only
under the same HTML namespace/document-class conjunction. Raw local name/prefix
remain exact. `DomNode::attribute` provides the contextual DOM getter;
`DomElement::attribute` is the intrinsic exact-storage lookup. Namespace/local
attribute getters and all NS mutations stay exact. NS-created `ID` and `id` may
coexist, while the contextual HTML getter's `ID` input searches `id`. Adoption
changes these contextual getters and subsequent ordinary writes through actual
node-document state; it does not rewrite stored names/attributes. Captured import
also preserves raw QName/attribute fields while target context controls display.

## Owned snapshots

`DomSnapshot` owns immutable storage for every managed node, including detached
nodes and multiple documents. It survives later edits, destruction and the
store's own destruction. Reads reject foreign/stale identities. Document views
validate membership at their captured owner/document/revision basis. A Document
belongs to itself for membership; its `owner_document()` getter returns `None`.
A detached node keeps its owner document while its ordinary root and
connectedness change.

## Change coverage and cancellation

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

## Source basis

The selected [DOM foundation capture](../../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md)
at revision `3071e5f269448350fdea79bb284eedaf17bc431e` supplies ordinary
tree/pre-insertion, shadow ownership/attachment, host-including cycles,
shadow-inclusive adoption/cloning, fragment transfer, ownerDocument, native document metadata,
nodeName/tagName, HTML document skeleton and API case rules, PI/doctype
construction and CharacterData rules. PI targets and pseudo-attribute maps use
the retained [XML grammar](../../../references/xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md)
and [XML-stylesheet pseudo-attribute rules](../../../references/xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md).
The native ownership, finite admission, atomic publication and snapshot policies
supplement that bounded source surface.

[Back to surgeist-dom](../README.md).
