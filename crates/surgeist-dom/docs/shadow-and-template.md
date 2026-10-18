# Shadow and Template ownership

ShadowRoot and Template contents are distinct owned relations alongside ordinary parent links. [Slot assignment](slots-and-checkpoints.md) and [semantic capture](semantic-captures.md) consume those actual relations.

## Shadow attachment and tree relations

`attach_shadow` creates an actual eighth node kind through the same batch tickets,
node admission, owner/slot/generation identity and immutable snapshots. It requires
an admitted HTML-namespace Element with a selected DOM valid-shadow-host name,
using HTML's actual custom-name grammar. `DomShadowRootInit::new(mode)` requires
open or closed mode and selects named assignment plus false delegates-focus,
clonable and serializable defaults. Actual `HtmlShadowRootProperties` can select
all five values. The native root stores false declarative/available-to-internals/
keep-registry-null flags and retains the host's actual canonical registry binding:
null or a qualified Global registry, including after hosted teardown. Duplicate
native attachment rejects. Custom definitions and execution, declarative reuse,
focus execution and serialization remain unavailable.

A root has a non-null reciprocal host relation and no ordinary parent. Its
children are the ordinary children of its own separate tree. `hosted_shadow_root`
reads canonical ownership, including closed roots; `shadow_root` implements
open-only Element exposure. `DomSnapshot::shadow_root` and `as_shadow_root`
provide checked root views with actual host and payload. Ordinary root/order,
shadow-including root/order/connectedness, tree scope and host-including ancestry
are separate APIs. Iteration is inclusive preorder without allocation or native
recursion. Shadow-including order visits a root immediately after its host and
before the host's light children. Open/closed never changes these owner relations.
Host-including cycle checks prevent inserting a host into its own hosted tree.

As a source DocumentFragment subtype, a ShadowRoot supplied to insertion transfers
its children and stays parentless/hosted. Its own node document is unchanged by
insertion. Child replacement first internally adopts the supplied fragment root,
so cross-document replacement can leave an emptied root's node document different
from its still-linked host's. Both relationships remain actual and independently
checked; direct public root adoption remains rejected. Host adoption preserves
all identities and updates shadow-including node documents. Detaching a host
preserves its owned roots; destroying a detached host retires all owned shadow
identities. Old snapshots retain the previous complete state.

## Shadow change coverage

`ShadowRoot(host)` changes report a hosted link on an existing Element. Creation
and destruction cover their full new/retired state. `ShadowRelations` explicitly
requests conservative recomputation of all derived shadow/tree-scope/connectedness
relations after a structural change in a store with a shadow tree. Ordinary
Parent/Children remain precise ordinary-tree changes; ordinary-only stores keep
their existing change contract. History gaps still require full recomputation.

## Template contents and inert documents

Native HTML Template owns a separate real DocumentFragment. Its checked
`DomNode::as_template()` view exposes contents and the HTML-owned known-null native
insertion state. Its structural `contents()` read returns `Result<NodeId, DomError>`;
use the contents NodeId in ordinary edits. Ordinary Template
children remain a different, legal native DOM child list. A connected Template
does not connect its contents. `fragment_host` and host-including cycle checks
cross its ownership edge; shadow ancestry, tree scopes and CSS flat queries do not.

Documents retain their actual Template role and associated inert-document cache.
First use creates a fresh default Document (31 metadata string units), marks it
HTML when its source is HTML, preserves the establishing native environment,
and caches it. It keeps application/xml and a fresh
opaque origin; it does not inherit createHTMLDocument's text/html, origin or
skeleton. Siblings share that cached owner; Templates inside an algorithm-created
owner reuse it. Destroying one Template preserves the shared cached Document.
Fixed identity fields count through managed-node capacity; all auxiliary nodes
and strings count toward the ordinary finite limits.

Deep Template import clones contents before parent append/ordinary children, into
the copy's actual appropriate owner. Shallow copies still own fresh empty contents.
Adoption invokes the HTML Template steps after updating its node-document. Direct
contents adoption/replacement can leave a different actual document while keeping
its host; same-document Template adoption does not normalize that history. Captures
preserve the real role/cache/contents and related documents. `TemplateRelations`
covers derived ownership/cache facts separately from shadow relations.

Direct explicit destroy of genuinely owned contents returns InvalidDestruction,
even when empty, parentless or adopted elsewhere. Destroy the detached owning
Template to retire its owned contents/nested/shadow closure atomically; old captures
and shared inert Documents survive. A retained declarative-shadow reference is
non-owning: its target is an actual ShadowRoot with its own host. Detaching,
adopting or destroying the referring Template preserves that root. Retiring the
actual root clears each matching incoming reference before identity reuse, after
which `contents()` returns `DomError::TemplateContentsUnavailable`. Old snapshots
retain their original reference. Cloning reads the source contents' children into
the copy's fresh owned fragment; it does not clone the referenced root as owned.
Captures preserve the distinct shadow-reference relation. Native creation and
for/shadow attributes do not fabricate that parser state or start patching. Declarative/retained
HTML installation uses its [qualified adapters](html-installation.md); nonempty/scoped registry
objects and HTML callbacks are not supplied by this native ownership profile.

[Back to surgeist-dom](../README.md).
