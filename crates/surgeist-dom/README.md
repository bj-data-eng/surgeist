# surgeist-dom

Canonical native document ownership with atomic edits and owned immutable
snapshots. The native contribution supports Document, native Element,
ordinary DocumentFragment, ShadowRoot, Text, Comment, ProcessingInstruction and
DocumentType. It directly consumes
HTML's exact UTF-16 `DocumentString`, `QualifiedName`, `Attribute` and
`OrderedAttributes`, CSS's checked `CssViewTransitionIdent`, and CSSOM's real
sheet/rule identities and immutable snapshots. Parser, runtime and facade
composition remain outside its dependencies.

`Dom::new` requires explicit finite `DomLimits` for managed nodes, stored string
code units, commands per batch, Slot references, raw Slot assignment inputs per
batch, and retained change-history entries and bytes. The seven constructor
arguments select those capacities in that order. Existing caller profiles with
no assignments explicitly select zero Slot capacities.
Detached nodes count toward the live limits. Logical string lengths count each
node's stored units even when immutable backing is shared, including PI targets,
data and derived pseudo-attribute map fields, all three doctype strings, and
each document's content type, encoding name and URL serialization (31 units for
native XML, 25 for native HTML). Element names and every attribute's namespace URI,
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
non-Document/non-ShadowRoot subtrees, including their owned shadow trees, and replace Text/Comment/PI data using UTF-16 code-unit
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
and actual native HTML Canvas/Marquee/Frame/Slot/Fieldset/Button families,
preserving prefix, case and exact UTF-16. It does not apply HTML content-model
validation or parser aliases. Registry/definition are known-null; actual valid
HTML custom names establish Undefined state through shared rules. Common state
includes the actual empty reaction queue/unset click flag, and the HTML-only
closed focus/internals/popover tuple. Exposed operations preserve those values.
Special built-ins lacking their actual family adapter reject atomically with
`NativeInitializationRequired` carrying the exact HTML-owned requirement. SVG
lowercase script is one such requirement; uppercase Script is ordinary SVG.

Native HTML Canvas retains HTML's actual none-context state: no bound context,
origin-clean true and a transparent-black bitmap description. Exact no-namespace
width/height attribute changes use shared HTML prefix recovery and update the
source bitmap dimensions, including zero and integers beyond machine magnitude.
Missing or failed values use the shared300×150 defaults. These are bitmap
dimensions, distinct from unsigned-long reflected IDL values. Redundant sets
have no extra none-mode reset action. No pixels, drawing context, image handle
or width×height allocation is fabricated. `canvas_none_state()` exposes the
current shared value. Each retained non-default canonical dimension's ASCII
digits count as stored UTF-16 independently of the raw attribute; inline numeric
defaults own no digit string. Immutable Arc storage prevents transaction and
snapshot copies from cloning unbounded integer Strings.

In an HTML Document, non-NS `WIDTH`/`HEIGHT` inputs reach Canvas hooks after
ASCII lowercase. NS-created uppercase names remain distinct raw attributes and
do not resize the bitmap. The same case rule precedes applicable nonce hooks.

Native HTML Marquee retains HTML's actual initially-on state and closed current
loop index0. `start_marquee` and `stop_marquee` batch commands change only the
on/off state; repeated assignments are semantic no-ops. `marquee_state()` exposes
that current value. Rendering-layer loop increments and animation scheduling
have no admission API. Snapshots preserve stopped state, while native DOM
clone/import initializes a fresh on/index0 Marquee. Canvas clone/import similarly
initializes fresh none state and derives dimensions from copied attributes through
their actual change steps. Neither family adds tree/adoption/CharacterData hooks
beyond the existing applicable common effects. They remain coherent inside
ordinary and hosted shadow trees. New family state reports `ElementState`
independently of raw `Attributes`; equal recovered dimensions with changed raw
spelling report only Attributes. Failed/cancelled edits roll back both domains.

Native HTML Frame retains the actual initially null child-content relationship,
read through `frame_null_content()`. Its content document/window are null; it
potentially delays load by type but causes no actual child load delay. Frame
post-connection runs after insertion/common steps and only when still
shadow-including connected, using the actual node document's known-null browsing
context. That source guard returns before creating a child or processing src.
Exact null-namespace src changes and removing steps consume HTML's null-content
decisions without URL interpretation, child destruction or navigation effects.
Removal visits the actual ordinary and owned shadow subtree after detachment.
Clone/import creates fresh null family state and copies actual attributes;
adoption preserves state and IDs while changing the real node document. Frame
owns no additional strings and fabricates no Window, child identity or host
capability. Iframe and other resource families still require their own adapters.

Native HTML Fieldset retains its actual form-owner relation, native unset parser
flag and initially empty custom-validity message. `fieldset_state()` reads those
stored values, including an `Option<NodeId>` owner that can represent later real
Form admission. Exact form/ID attributes, associated subtree insertion/removal,
fragment transfer, replacement and adoption drive HTML's shared form-owner reset
cursor against the canonical staged tree. Ordinary form lookup does not cross a
shadow host; connectedness uses the actual shadow-including root. First-ID lookup
observes the first exact nonempty DOM ID, including a non-Form blocker. A present
empty form attribute is distinct from absence. ID-bearing removal uses the actual
post-removal non-Document-tree predicate, without an old-connected prerequisite.
Current constructors cannot admit an actual HTML Form, so completed lookups have
actual null results. Positive ownership requires Form's real state and past-name
map effects before it can become reachable; there is no permanently-null owner
API or fabricated successful association.

`set_custom_validity` applies to native Fieldset and Button. It uses HTML's unit-fed
CRLF/CR-to-LF rule before replacing the stored message, preserving every other
exact UTF-16 unit. Raw batch operands are caller-owned; normalized output scratch
is bounded by the admitted retained length, and the final normalized message
counts independently of attributes. Every raw-unit read and output write polls
cancellation. State-only edits report ElementState, equal final messages are
semantic no-ops, and snapshots/history retain their original values. Native
clone/import initializes a fresh empty message/null association and executes
copied-attribute hooks; adoption preserves the message and runs actual removal
and insertion/reset effects. The Fieldset payload allocates no control value,
selection, cached disabledness or fake form object.

Semantic capture publishes actual established-current Fieldset and Button associations.
Fieldset collections, Legend delegation, first-Legend disabledness and constraint
candidacy remain HTML semantic queries over the current captured tree. Slot
supplementation rebuilds actual control facts into that same context, preserving
Slot directionality/inert facts. `DomHtmlCaptureLimits` bounds each ordinary adapter vector
by subjects (nullable reference facts use twice subjects) and adapter work by work; the same explicit finite work value
independently caps shared control validation including its context join. Checked
move-only facts are rebuilt from the retained snapshot/map, never cloned or
replaced with inferred ownership. This contribution does not execute constraint
validation, report/check validity, expose a complete ValidityState schema or
provide validation UI/events. A nonempty Fieldset custom error does not make it
a constraint-validation candidate.

Native HTML Button composes that same private native-control storage and reset
adapter with its own public `button_state()` / `DomNode::as_button()` view.
`set_custom_validity` also applies to Button. Actual form association, native
parser flag and custom message obey the same fresh clone/import and preserving
adoption rules. Disabledness, Auto submit role, source form settings and constraint
candidacy are current HTML semantic queries; no activation, submission or event
execution is introduced, and Form/Select remain unadmitted native families.

`set_command_for_element` and `set_popover_target_element` drive HTML's actual
nullable Element setter cursor. The two weak slots are independent and initially
null. A non-null write first sets its exact content attribute empty (including its
clearing hook), then installs the supplied Element; a null write clears the slot
before deleting content. Exact null-namespace commandfor/popovertarget changes,
including redundant sets, clear only their corresponding slot. Missing removal
runs no new attribute hook. Namespace/case mismatches remain distinct attributes.
Targets must be actual same-store Elements, and can be same-batch creations,
detached, self, other-document or outside getter scope. Staged references use
actual draft keys and map to final generation-safe identities at publication.
Two fixed slots per Button are covered by managed-node capacity, not dynamic Slot
assignment-reference capacity. No target IDs, attribute data or history records
are invented from ticket indices.

`DomButton::explicit_command_for` / `explicit_popover_target` read raw stored weak
references. Their scoped associated-element getters remain existing HTML semantic
queries, with first-ID content fallback only when the actual explicit slot is
null. An out-of-scope explicit target is retained and queries null without content
retargeting. Capture closes actual weak targets, node documents and ancestors
conservatively, without claiming unrelated supplemental children are empty or
complete, and joins existing checked reference/control facts. Each reference-fact
vector is bounded by twice the explicit subject bound (two per Button), with
finite producer validation work. Slot supplementation rebuilds these actual facts
and preserves its own directionality/inert facts. Native clone/import starts both
slots null and processes copied attributes; adoption preserves actual weak IDs.
Explicit destruction clears surviving references to the exact retired identity,
including same-batch targets, and never follows generation reuse. Old snapshots
retain their actual historical reference and target. State-only changes report
ElementState alongside separate attribute changes; failed/cancelled batches roll
back both domains.

HTML/SVG/MathML Elements retain a real initially empty cryptographic nonce slot.
Exact null-namespace `nonce` attribute changes update it, removal clears it, and
`set_nonce` changes only the slot. Case/namespace mismatches do not affect it.
State-only edits report `ElementState`, separately from `Attributes`, and advance
the same DOM revision. Clone/import appends actual attributes through their change
steps, then copies the captured nonce slot. Native Document environment stores
real null browsing-context and never-navigable proof independently of XML class.
Insertion runs common steps over inserted shadow-including descendants; this native environment
proves nonce content remains visible without inventing CSP facts. No API attaches
a browsing context or advances the closed common tuple into interactive state.

This common contribution does not complete native HTML. Remaining exact producer
requirements include other native form/control relations and values, template hosted
contents/inert documents, HTML/SVG script state and preparation, iframe
navigables, link/style associations, details/dialog state, and resource/media
families. Their real initialization and applicable attribute/tree/adoption hooks
must join before admission; a requirement error is not a permanent HTML dialect.

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

`DomSnapshot` owns immutable storage for every managed node, including detached
nodes and multiple documents. It survives later edits, destruction and the
store's own destruction. Reads reject foreign/stale identities. Document views
validate membership at their captured owner/document/revision basis. A Document
belongs to itself for membership; its `owner_document()` getter returns `None`.
A detached node keeps its owner document while its ordinary root and
connectedness change.

`attach_shadow` creates an actual eighth node kind through the same batch tickets,
node admission, owner/slot/generation identity and immutable snapshots. It requires
an admitted HTML-namespace Element with a selected DOM valid-shadow-host name,
using HTML's actual custom-name grammar. `DomShadowRootInit::new(mode)` requires
open or closed mode and selects named assignment plus false delegates-focus,
clonable and serializable defaults. Actual `HtmlShadowRootProperties` can select
all five values. The native root stores false declarative/available-to-internals/
keep-registry-null flags and a known-null registry. Duplicate native attachment
rejects; there is no fabricated registry object, custom definition, declarative
toggle, focus execution or serialization capability.

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

`ShadowRoot(host)` changes report a hosted link on an existing Element. Creation
and destruction cover their full new/retired state. `ShadowRelations` explicitly
requests conservative recomputation of all derived shadow/tree-scope/connectedness
relations after a structural change in a store with a shadow tree. Ordinary
Parent/Children remain precise ordinary-tree changes; ordinary-only stores keep
their existing change contract. History gaps still require full recomputation.

Native HTML Slot is an Element family initialized through HTML's actual-name
producer. `DomBatch::assign_slot` consumes the real HTML assign cursor over raw
Element/Text operands, including duplicates, with one canonical staged owner.
Manual ordered sets/inverses and stored assigned-node lists/inverses are distinct
state. Manual intent may name detached, other-document or self Slots; current
find algorithms filter by the actual host-child relationship. A manual transfer
reassigns only the target's ordinary root. It preserves another root's historical
stored list and inverse until a later actual source operation visits it.
`DomSlot::assigned_nodes` reads stored source assignedNodes(false);
`DomNode::assigned_slot` freshly finds a Slot with open-root presentation.
`stored_assigned_slot` is privileged algorithm state and can differ. Names are
exact UTF-16 no-namespace attribute values, with missing/empty source guards.

Every existing insert/remove/fragment/replace/adopt/attribute/clone path runs
the selected Slot hooks in source order. Clone/import fresh-initializes manual
state instead of copying foreign references. Explicit native destruction cleans
permitted-weak manual references and exact-generation surviving stored inverses;
ordinary detach/adopt/assignment retain historical inverse fields. This native
cleanup creates no synthetic global assignment or slotchange.

The Slot reference capacity counts each installed manual-list entry, stored-list
entry and pending signal recipient independently; fixed inverse fields are covered
by managed-node capacity. Raw command capacity counts duplicates across the whole
batch before resolution. Replacement candidates use checked temporary storage
bounded by admitted node/input extents and count installed replacements exactly.
These logical limits do not promise physical RSS or budget caller-held captures.
`DomSlotQueryLimits` separately bounds output nodes and scratch references.
Flattening is iterative; retained candidate extents and each traversal frame are
charged until the frame is released. CSS output records include their queue/
parent storage in output admission, with separate scratch for Slot candidates.

`flattened_slottables` uses fresh find results, slottable fallback and recursive
Slot substitution. `DomSnapshot::flat_tree` follows the selected CSS document
algorithm: it retains Slot parents, uses all child-node fallback, replaces a host's
children with its shadow-root children, and preserves explicit outside-document
membership. Its node iterator is source breadth-first construction order.
Both results retain the exact immutable source; neither changes canonical links.

Source signals accumulate an ordered deduplicated set in the same transaction.
`Dom::take_slot_notifications` is the explicit atomic native intrinsic checkpoint,
returning historical slotchange intentions (bubbles=true, composed=false,
cancelable=false), without dispatching callbacks or fabricating observer objects.
A failed/cancelled drain preserves the set. Native destroy removes exact retired
pending recipients with `SlotSignalCancelled { reason: Destroyed }` evidence;
detach/adopt/disconnect do not cancel. Drained batches retain their historical
capture; `revalidate` checks the live owner and reports `SkippedDestroyedTarget`
without following generation reuse. Later signals belong to the next checkpoint.
New pending work advances the revision even when final node values are restored.

Template contents/inert-document caches, declarative installation, registry objects and
HTML callbacks remain separate unadmitted capabilities. Shadow-including and
host-including ancestry coincide for admitted shadow boundaries today; future
template-hosted fragments must extend the host-including relation with their
actual HTML owner hooks. This unit does not complete #1124's remaining
language/control/pseudo or broader host-fact requirements.

`DomHostPublisher` captures complete fixed focus/hover or document-target Element
observations from an external producer against one exact `DomDocumentStamp`.
Supported absence, unsupported capability and inapplicability remain distinct.
Its immutable source identity includes the selected domain configuration; checked
`HostStateRevision` advances only when observations change. DOM-only revisions
require explicit whole-domain recomputation without advancing equal host facts.
Captures retain exact historical values independently of the publisher. Complete
adjacent coverage uses constant storage, with explicit fallback for older history
and whole-domain invalidation for availability changes. The independent
`DocumentTarget` variant retains the host's one selected target Element or known
absence, following HTML's target-elements contract. It does not navigate, decode
fragments or infer a target from IDs or URLs. An unknown current observation is
not a supported absence: a producer must obtain a qualified value or report its
unavailability without publishing. This boundary
does not implement HTML focus policy, pointer processing or canonical controls;
Runtime owns its concrete surface-selection association and capture adapter.
It rejects document-target bindings because it has no target-selection producer.

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

`create_html_document(associated_document, title)` implements the selected DOM
createHTMLDocument tree in the same staged owner: Document, html doctype, HTML
`html` element, `head`, optional `title` with a Text child, then `body`. `None`
omits title; `Some("")` creates both title and empty Text. The actual same-store
associated Document supplies only the origin identity. New metadata is HTML,
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
new Document()'s associated-global origin semantics. DOMImplementation's separate
XML createDocument(namespace, qualifiedName, doctype) algorithm, including its
XHTML/SVG content types, remains unexposed. The source createElement XHTML MIME
branch does not imply that creator or a metadata override capability.

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

Actual open and closed ShadowRoots extend that closure through their reciprocal
host/root association, ordinary ancestors, actual node documents and shadow
subtrees. The root's document remains independent of the host's document,
including after fragment replacement adopts only the root. `subject()` accepts
these actual supplemental members. Supplemental document/ancestor records keep
unavailable children unless their complete ordinary subtree was included; an
unrelated document tree is never reported as empty or complete. Current root
properties, declarative/internals/keep-registry-null flags and registry come
from their canonical state. Ordinary parent/root, tree scope, shadow-including
connectedness and open-only presentation remain distinct in the retained source.

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

`DomHtmlCapture::with_slot_semantics` reuses actual stored lists for HTML
directionality and current document flat relations for the inert producer. It
closes required stored-reference subjects across actual documents, then invokes
the real HTML kernels using explicit producer/mapping/query limits. Flat membership
follows actual shadow-including documents independently of node-document ownership.
It preserves actual Fieldset associations; language fallback, other live-control
values and modal state remain unavailable.
Connected inherited inertness therefore still needs current modal facts; own
inert can succeed independently. Detached subjects are known flat-excluded, with
inherited inert ancestry unavailable under the selected contract.

This native shadow/Slot/Fieldset semantic profile does not complete #1124. Other live controls,
template owner/cache state, registry objects,
parser installation and the remaining qualified external domains still require
their actual owning adapters. Future hosted kinds need their own real association
closure and must not inherit ordinary fragment absence from this profile.

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

Remaining native special families, XML DOMImplementation creation and HTML
installation remain unadmitted capabilities. This crate does not claim complete DOM or
HTML conformance. Those remaining
contracts stay with [#1074](https://github.com/bj-data-eng/surgeist/issues/1074),
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124) and
[#1135](https://github.com/bj-data-eng/surgeist/issues/1135).

The selected [DOM foundation capture](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md)
at revision `3071e5f269448350fdea79bb284eedaf17bc431e` supplies ordinary
tree/pre-insertion, shadow ownership/attachment, host-including cycles,
shadow-inclusive adoption/cloning, fragment transfer, ownerDocument, native document metadata,
nodeName/tagName, HTML document skeleton and API case rules, PI/doctype
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
