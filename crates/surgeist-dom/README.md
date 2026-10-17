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
execution consumes its qualified Source observation, described below.
Internals/popover fields retain their closed values.
Special built-ins lacking their actual family adapter reject atomically with
`NativeInitializationRequired` carrying the exact HTML-owned requirement. SVG
lowercase script has its actual native state; uppercase Script is ordinary SVG.

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
capability.

Native HTML Iframe retains HTML's actual fresh never-browsing-context,
unprocessed/null-child state, read through `iframe_native_state()`. Its lazy
navigation flag is false; pending timing fields and lazy-load resumption are
null. It potentially delays load but has no actual child load delay. Frame and
Iframe share the canonical connected-subtree and removing hooks, with each
family consuming its own HTML decision. The actual current node document's
absent browsing context returns before sandbox parsing and child creation.
Exact src/srcdoc/sandbox changes preserve the null-child phase; loading changes
consume the shared language interpreter and eager-transition guard before the
actual null resumption returns. Raw sandbox absence, empty content and tokens
remain distinct attributes, not a numeric live sandbox policy. This admits no
navigation, resource handle, active policy or context-creation transition.
Clone/import-as-clone genuinely creates a fresh element and applies copied
attributes; adoption and immutable copying preserve existing family state.
Current null content alone never justifies resetting processed retained history.
Semantic captures retain this state in their source snapshot; the semantic
schema has no child-navigable field to synthesize.

Native HTML Output has a checked `DomNode::as_output()` view over its actual
nullable default-value override and canonical Form owner/custom-error state.
`None`, `Some(empty)` and `Some(text)` remain distinct. Read current and default
values through the existing bounded HTML semantic `OutputValue` and
`OutputDefaultValue` queries on `html_capture`; no second text getter or cached
current value is retained. Output text is exact ordinary descendant UTF-16,
including nested control default Text, without Textarea newline normalization
or traversal into owned Template contents or ShadowRoot associations.

`set_output_value` stores the current default as an override before replacing
children. `set_output_default_value` replaces children only while the override
is null; otherwise it changes only that override. `reset_output` reads the
default, replaces children, then clears the override. The shared HTML cursor
owns this order. Output and Textarea use one real DOM string-replace-all helper:
new Text admission precedes old-child removal, detached nodes stay managed, and
every actual remove/insert children hook still runs. Equal text does not skip
identity-changing replacement. No extra final children hook is invented.

Mutation-time descendant reading walks ordinary parent/sibling links without a
traversal buffer. Two cancellable passes count and materialize one exact UTF-16
buffer; its actual capacity is bounded by the canonical string-unit ceiling.
This finite tree/string work does not consume the numeric/CSS ledger. Override,
custom message, new Text and detached old Text are independently retained and
accounted. Clone/import creates a fresh null override and custom error before
copying descendants; adoption preserves established state. Both initial and
Slot-supplemented captures carry actual Output defaults, with aggregate override
units bounded by the initial capture's UTF-16 ceiling. Native installation uses
the same family and tree owner. Per-control reset does not dispatch a whole-Form
cancelable reset event.

Native HTML Textarea has a checked `DomNode::as_textarea()` view over one exact
raw UTF-16 value, shared dirty/user-validity flags, actual selection and native
association/custom-error state. `state().raw_value()` is borrowed raw storage;
`value(DomTextareaReadLimits, cancel)` materializes the API value through HTML's
newline normalizer, and `text_length` counts that normalized value. `default_value`
reads actual immediate child Text, excluding nested text. These finite reads
have explicit output-unit and cumulative-work ceilings. Read work charges each
raw normalization read, output write, child visit and output reservation; counting
and materialization are separate passes. Actual output capacity is admitted and
failure returns a typed error. No hidden getter cache or mutable API-value mirror
is retained, and this API does not claim submitted hard wrapping.

`set_textarea_value` retains exact raw input, marks dirty, clamps selection to
normalized API length and consumes HTML's additional setter-finish decision.
Changed API values unselect and move to end even when a range survives clamping;
equivalent CRLF/LF raw spellings preserve selection. The actual platform's two
independent support facts apply to source-required writes, while adoption retains
established state. Temporary normalized comparison buffers are each bounded by
their admitted raw operand, at most twice the existing canonical unit ceiling;
all scans/copies poll cancellation. No string work spends numeric/CSS allowance.

`set_textarea_default_value` string-replaces actual children through normal
creation/removal/insertion hooks; detached old children remain addressable.
Clean children-changed steps copy immediate Text in order, including reached
intermediates during same-parent moves, Fragment transfer and CharacterData edits.
Dirty raw stays independent. `reset_textarea` clears dirty/user-validity and reads
the actual default children while preserving custom error and Form owner. The
shared children dispatcher also preserves Script's connected preparation order.
Clone/import copies only raw and dirty into fresh native state before descendants;
ordinary descendant hooks still run. Adoption preserves current state, and
retirement uses the existing canonical owner cleanup. Native fragment installation
uses these same actual family/tree hooks. Parser-pop reset, user-edit events,
selection events, whole-Form reset dispatch and submitted wrapping are separate
boundaries; none is fabricated by native creation.

Native HTML Input has a checked `DomNode::as_input()` view over actual flags,
current value, selection/direction, empty selected-file state, Image history and
custom error. The view's `value()` applies HTML's current/default/default-on/file
getter mode; `state().current_value()` reads the separately retained internal
string. Every shared type state is admitted. Type/default/checked/name/multiple,
Range constraints and Color alpha/colorspace edits execute their real source
effects rather than reconstructing current state during reads. String/temporal,
Range and Color grammar and normalization remain with the shared producers.

`set_input_value`, `set_input_checked`, `set_input_indeterminate` and `reset_input`
are atomic native operations in `DomBatch`. Checkedness and dirty checkedness
are separate from checked content. Value setters compare the final sanitized
value, clamp real selection and apply the source's additional cursor effect.
Reset follows the shared per-control order; it does not dispatch a cancelable
whole-Form reset event or clear unspecified state. Custom error storage is shared
with Button/Fieldset and consumes its own normalized retained units.

Radio normalization uses the actual ordinary root, exact nonempty name and
current same/null Form owner. Owner reset applies radio effects immediately at
each clear/association, so A-to-null-to-A is observable in peers even when the
final owner is unchanged. Connected insertion, actual name/type changes and
true checkedness invoke their own source triggers. Shadow roots partition
ordinary groups; peer uncheck preserves peer dirty/default/indeterminate flags.
No second group cache or end-of-batch repair is maintained.

Input and Button share actual reflected-target writes and retirement. Input has
one popover target and no commandFor slot. `set_popover_target_element` accepts
either real family, including same-batch, self, detached and other-document
Elements. Its raw weak slot and HTML's scoped semantic getter remain distinct.
Generic popovertarget content changes clear the explicit slot even on redundant
sets. Clone initializes fresh weak/message/files/selection/history state, copies
attributes through actual hooks and applies only the Input cloning overlay for
current value and source-selected flags. Color cloning does not resolve its
default again before copying the actual current value. Adoption preserves live
state except actual owner/tree hooks; explicit retirement clears exact weak
references and releases all retained strings without following reused IDs.

`Dom::with_native_input_platform` selects the two independent existing HTML
empty-selection and none-direction support enums. `Dom::new` chooses the named
`DomNativeInputPlatform::headless()` Supported/Supported profile. These immutable
facts are explicit native policy, not consequences of absent browsing context.
Native Documents and their Template inert owner Documents retain the establishing
environment. Adoption does not reset selection merely because it reads a Document.
Native image acquisition is explicitly disabled, with no enabled option. Image
entry/re-entry and src hooks retain exact absent/empty/nonempty source history and
selected zero coordinates; no URL validation, fetch/error/event or resource object
is fabricated. Native File has an actual empty selection; its nonempty value
setter is a typed refusal, not a synthetic file/path installation.

Reached variable Range/CSS/Color processing requires caller-selected
`DomLimits::with_input_processing(scratch_bytes, work)`. The seven-argument
constructor leaves both capacities zero, so fresh empty Text uses no numeric/CSS
service or invented numerical allocation. Other types remain real Input states;
insufficient processing allowance rejects the whole batch atomically. Scratch
uses `usize` bytes for the peak logical live adapter layouts, heterogeneous banks,
digits and output, including old/new allocation overlap and actual capacities.
It is not allocator bookkeeping/RSS or all Rust stack storage. Work is a separate
`u64` cumulative credit ceiling across every reached service in the private batch,
including nested hooks, with a typed `InputProcessingWorkLimit` stop. Logical work
can exceed address size without allocating that many bytes.

The private services charge reached protocol/fixed actions and described
observations/writes, register/action envelopes, actual retained-prefix movement
and output transfer. Fees are DOM's documented service policy, not parser credit
identity. Reservations are fallible, sizes checked and cancellation polled before
actions/copies; actual owner-borrowing banks are dropped before phase/owner reuse.
Standard-invalid Color parsing alone chooses HTML's black fallback. Missing CSS
facts, numerical failures, resource exhaustion and cancellation stay distinct
typed refusals. Current/default/message/Image-history strings independently
consume canonical retained UTF-16 admission. Lowering future processing limits
does not erase admitted values. No new pickers, file/image acquisition, numeric
IDL stepping, selection events or general validation UI is implied.

Native HTML Script has a checked `DomNode::as_html_script()` view over its real
canonical flags, optional type, parser/preparation Document identities and shared
Uninitialized result/null-ready state. Fresh native creation has null documents,
force-async true and other flags false. Actual native node-document browsing
context proves scripting Disabled, independently of XML/HTML class. Preparation
still stores recognized type, already-started and actual preparation document
before that source return. No fetch, compilation, error result, ready callback
or JavaScript execution is manufactured.

Script source hooks run at each actual operation: async Added, connected nonnull
src writes (including redundant values), parent children-changed after insertion/
removal and CharacterData replacement, then source static descendant
post-connection order. Only immediate nonempty Text contributes source content;
whitespace and isolated UTF-16 units are preserved. Type/language interpretation
uses HTML's admitted borrowed cursor. Type edits alone do not prepare; equal data
edits or empty-fragment insertion can subsequently trigger a state-only change.
Template contents remain disconnected through their host. Shadow connectivity
does not replace actual node-document identity when a root differs from its host.

Clone/import-as-clone applies actual copied attributes, then copies only source
already-started; type/documents/result and other flags start fresh. Adoption,
detach and immutable snapshot copying preserve established preparation history.
Historic preparation documents stay in capture association closure, without
declaring their unrelated trees complete or adding a fake semantic Script fact.
Native APIs admit no arbitrary parser/processed-history installation; that cause
must preserve/remap real metadata separately. Resource-capable preparation
remains outside this native HTML contribution.

Native SVG Script has a separate checked `DomNode::as_svg_script()` view over
HTML's `SvgScriptFlags` and its actual `DomSvgScriptOrigin::{Native, Clone}`.
Fresh native creation is unprocessed. The selected Tiny four-step operation uses
DOM3071 shadow-including connectedness and the existing static post-connection
list: hosted roots precede a host's ordinary descendants, and inserted roots keep
source order. An eligible false bit becomes true even for empty content, unusual
type, absent/empty/data/same-document/external href or inert handler attributes.
No HTML empty/type/async/preparation guards or restart hooks are imported.
Ordinary Template contents and detached hosts remain disconnected; Slot assignment
alone does not connect them. Removal and later href/content edits do not restart.

Native constructors establish a private immutable closed SVG processing witness:
all network/data/same-document script acquisition and inline/external execution
are disabled, and no admitted SVG load/error recipient, executable handler,
callback, retained task, event or resource output can observe omitted notification.
This follows the actual constructor/command/host/output boundary, independently
of null browsing context or HTML Disabled. Authored handler strings are inert
UTF-16 data; existing Slot intentions remain real and deliverable. The processing
step consumes that witness from the script's actual node Document, which can
differ from its shadow-connected host's Document. No URL is classified, fetched
or decoded, no success/error/event/promise is fabricated, and no execution occurs.
A future resource/recipient or retained-history installation front must implement
its actual qualified transition before using this closed premise.

Clone/import copies the actual true or false bit through the shared clone rule,
sets Clone cause after copied attributes and before parent insertion, and uses
the target native environment. Adoption preserves state/cause. An insert followed
by detach can therefore leave a required state-only ElementState change despite
unchanged final topology. Snapshots/captures retain their original state. Native
fragment installation uses fresh native cause and these same insertion hooks;
it does not certify parsed/resource history. Scalar state adds no retained string
cost. Processing reuses the finite canonical-node static traversal and fallible
buffers with per-node cancellation; it introduces no task queue or numeric quota.
SVG use-instance trees and resource-capable processing remain separate domains.

Native HTML Fieldset retains its actual form-owner relation, native unset parser
flag and initially empty custom-validity message. `fieldset_state()` reads the
intrinsic parser/message state; `DomNode::as_fieldset()` provides the checked
`DomFieldset::form_owner()` relation at the captured revision. Exact form/ID attributes, associated subtree insertion/removal,
fragment transfer, replacement and adoption drive HTML's shared form-owner reset
cursor against the canonical staged tree. Ordinary form lookup does not cross a
shadow host; connectedness uses the actual shadow-including root. First-ID lookup
observes the first exact nonempty DOM ID, including a non-Form blocker. A present
empty form attribute is distinct from absence. ID-bearing removal uses the actual
post-removal non-Document-tree predicate, without an old-connected prerequisite.
Native HTML Form retains HTML's three fresh false scalar values and an actual
past names map. `DomNode::as_form()` reads its scalars and retained exact-name
aliases. Native creation/clone starts the map empty. The current public fronts
cannot insert past-name history or perform named-property lookup; map cleanup
handles actual retained aliases without treating the map as permanently empty.
Every owner change removes all aliases for the control from its former Form at
the intermediate owner state, before later lookup or reassociation. Thus even an
A→null→A reset can change the Form map. Ordinary nearest-owner retention and
unrelated name edits preserve aliases. Retained name units count independently
of attributes; a future growing history front requires its own finite capacity.

Form owners live once in the canonical Node/Draft relation, using real NodeIds
and staged Keys respectively. Same-batch/private imported Forms resolve through
the existing final allocation map. `DomButton::form_owner()` and
`DomFieldset::form_owner()` read that relation; scalar state values carry no
identity mirror. Native Form clone/import starts fresh scalars/map, while deep
copied controls acquire their new actual Form. Adoption preserves retained state
and runs actual split-tree resets. Explicit retirement clears surviving owners
and aliases for exact retired identities without following generation reuse.
Snapshots preserve the old relations/maps and ElementState covers their changes.

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

Semantic capture publishes actual established-current Fieldset and Button associations,
closing actual Form owners and retained map targets. Both initial capture and
Slot supplementation establish negative custom-control classifications only from
actual HTML elements with an actual null definition. The finite subject budget
bounds these records; adapter work admits their context validation and duplicate
comparison envelope. No custom spelling or registry emptiness stands in for a
definition fact.
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
execution is introduced.


Native Select, Option, Optgroup and Selectedcontent compose the same canonical
owner. Select retains shared native control/message and user-validity flags;
Option retains selectedness/dirtiness and its real cached nearest Select identity.
Optgroup disabled changes consume the same bounded nearest-Select ancestry rule,
including HTML barriers. Selectedcontent retains actual insertion-derived disabledness.
`set_option_selected`, `set_select_value`, `set_select_selected_index` and
`reset_select` execute the shared HTML decisions against the current staged tree.
Value/index setters preserve the first exact match and intentionally leave no
selection after no match. Both ordinary and Slot-supplemented semantic captures
retain actual Option flags and canonical Select form association.

Selectedcontent copies actual Option children, with fresh canonical identities,
through the shared iterative clone executor and real family/Template/Shadow hooks.
Each child finishes its deep clone before append to an actual temporary Fragment;
replace-all detaches old children without retiring them. The proven private empty
wrapper is reclaimed before publication. Source insertion, attribute, removal and
static postconnection order remains observable; updates run no host callbacks.

`DomLimits::with_select_processing` explicitly bounds pending Select updates,
DOM adapter/effect scratch bytes and cumulative DOM work. Unconfigured variable
work has zero capacity; native constructors need no variable work. Reached value
setters additionally require `with_select_value_queries(HtmlCaptureLimits,
HtmlSemanticLimits)`. One sequential producer capture has its own finite retention
limits. Producer semantic work and allocated bytes are cumulative over every
Option query in a private batch; other semantic limits apply per query. These
producer domains remain separate from DOM temporary/effect banks. The adapter
passes the caller's cancellation token through the original capture and query
algorithms, and preserves producer usage on both success and refusal. During a
composed Source edit, its remaining live-byte, UTF-16 and work allowance also
covers the synchronous producer graph, interleaved DOM mapping banks and the
returned value occurrence. The producer graph remains admitted until the
returned value has its own storage admission; resuming preserves spent work.

Real Option removal queues duplicate-preserving UpdateSelect intentions in the
same FIFO as one coalesced Slot marker. Checkpoint handoff leaves the Select head
pending and returns an immutable `DomSelectTask` with shared once-only authority.
`complete_select_task` executes current state under `DomNativeCheckpointLimits`;
failure preserves the head and permission, and dropping all handles permits
reacquisition. Foreign owners and replay reject. Exact Select retirement runs
cleanup first, cancels every captured occurrence with TargetDestroyed evidence,
and allows one historical SkippedTargetDestroyed completion of an issued handle.
Detach/adopt preserve intentions. No browser event loop, picker or rendering
resources are implied by these intrinsic checkpoints.

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
vector is bounded by twice the explicit subject bound (two per Button, one per
Input), with
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
steps, then copies the captured nonce slot. Standalone native Document environment stores
real null browsing-context and never-navigable proof independently of XML class.
Insertion runs common steps over inserted shadow-including descendants; this native environment
proves nonce content remains visible without inventing CSP facts. Hosted creation
uses a separate controlled source protocol; it does not convert a closed Document
or advance its common tuple into interactive state.

Native standalone Img retains HTML's actual fresh request tuple: Unavailable,
empty current URL, density1, null preferred dimensions/pending request/last source/
last auto-sizes width/lazy resumption, and no acquired image data. The canonical
dimension-attribute source is the Img's own exact identity. `DomNode::as_img`
reads that tuple, its nullable Form owner and parser flag independently of
constraint-validation controls. Img is nonlisted; its `form` attribute does not
select a listed-control owner, and `set_custom_validity` remains inapplicable.
The existing shared reset cursor operates on its actual nearest Form/tree state.
Both initial and Slot semantic captures include the real established-current
association, without inventing loaded resources or external geometry.

Creation and every relevant mutation synchronously start HTML's update-image-data
algorithm. Its first step waits for the actual node Document to become fully
active. Closed native Documents and genuinely retired hosted Documents establish
inactivity through the canonical activity owner, so the wait occurs before the
later images-disabled abort/reset. Unavailable live authority does not establish
inactivity. The current request stays unchanged; no load/error event, request
reset, promise or continuation task is reported as completed. Native activation
and resource-history installation are
unadmitted phases, with `ImageProcessingUnavailable` at a reached unsupported
processing boundary.

`DomImg::suspended_update` reads the latest owned wholly inactive invocation.
Its opaque `DomImageUpdateId` contains the exact target generation and checked
nonwrapping invocation order; the readback transfers no execution authority or
work ownership. Every admitted start advances the high-water mark. A later start
precludes an older wait's post-activation continuation even when the later one
has already aborted. Under this native prefix, superseded waits have no resource,
callback, promise, queued continuation or independent task identity, so the owner
retains only the latest wait and its exact flags. Flags are replaced per start,
never ORed. Equal-value src sets request restart-animation; other admitted causes
use their actual shared decisions. Semantic crossorigin/referrerpolicy states
are compared through HTML's interpreter. Eager loading checks the actual null
resumption without manufacturing work.

Source srcset/sizes/media/type/width/height changes update only following direct
Img siblings of an actual Picture. Source insertion and root removal visit all
actual direct Img children of the current/old Picture in source order. Img
insertion into Picture and root removal from Picture start their own updates;
removal then clears the already-null auto-sizes width. Descendant hooks preserve
the subtree-root distinction. No temporary fanout list or resource selection is
fabricated. Audio/Video remain unadmitted native owners; future Source media
processing requires actual src/network state and a real selection operation.

Adoption into a different Document preserves the image request state and starts
a new update against that Document. Its actual removal and Form owner reset
hooks can change the Form association. Detachment preserves waiting ownership, while
executing any real removal mutation. Clone/import-as-clone and native immutable
installation start fresh requests/self dimension sources and run their actual
creation, copied-attribute and tree hooks. Parser history cannot enter through
that native cause. Old snapshots retain exact prior waits and associations.

The latest wait and high-water are fixed per-node storage admitted by existing
canonical node capacity; constructors require no new default-zero quota. Ordered
scans and comparisons use finite canonical node/string/command bounds and poll
cancellation per step. When nested in Select effects they consume that actual
work ledger too. Explicit destruction performs removal hooks first, then cancels
the latest still-owned exact-generation wait. `ImageUpdateCancelled` evidence
includes its operation, exact flags and `TargetDestroyed` reason, even if the
latest invocation started earlier in the same batch. Its fallible evidence bank
is bounded by managed-node capacity and joins the live Select scratch ledger;
all change output is built before atomic publication. Entirely private
create/destroy closures fabricate no public target or cancellation identity.
Detach/adopt do not cancel, pin or rebind old generations. No handoff or scheduler
API is introduced. Any future real handoff must validate target/invocation
liveness against its actual owner. ElementState, NodeDocument and ordinary tree/
attribute changes retain their distinct coverage; rejection restores all state,
waits, revision and history together.

Native HTML Base retains a nullable historical frozen URL record, its actual
freezing Document, full external influencing revision and response disposition.
Documents retain their actual first ordinary-tree Base-with-href and separately
qualified base facts. Becoming first or changing the current first's exact
null-namespace href freezes immediately, including selection revealed by removal.
Shadow roots and Template contents are separate trees. Losing first position,
removing href, detaching and adoption preserve old frozen history; ownerDocument
and freezing Document remain distinct. Clone/import starts unfrozen, then copied
attributes and actual insertion run fresh hooks. Native immutable installation
has the same fresh cause, without replaying parser history.

`DomBatch::bind_base_facts` consumes an exact `DomDocumentStamp` and the
producer's immutable `HtmlParseBaseFacts`. This explicitly binds external
observations to that actual native about:blank/UTF8 environment. The supplied
opaque encoding identity is bound to the known native encoding, rather than
interpreted as a charset label. The operation changes no URL/encoding metadata
and accepts no arbitrary parsed environment: srcdoc/about-base/nonblank or
contradictory known URL observations reject. Existing bindings reject foreign
producers and equal-revision fallback/encoding identity substitutions. A changed
same-producer full revision is an explicit newly qualified observation binding;
it does not itself refreeze historical state. Every supplied observation must
match the actual fallback record identity, encoding identity and full revision;
reached freezing additionally matches exact raw UTF16 href. Missing observations
or CSP permission refuse atomically. No URL parser or CSP interpreter is copied.

`DomLimits::with_base_processing` bounds aggregate actual retained observation
capacity and cumulative DOM qualification/scan/response work per batch, default0.
Canonical string admission counts fact-formation UTF16 extent and each retained
frozen URL serialization. Input facts are formed under the semantic producer's
own finite `HtmlBaseFactsLimits`; DOM does not claim that its work budget covers
work already performed by that external constructor. Scans are stackless and
poll every visit/comparison; no traversal bank is allocated. Failed/cancelled
binding or freezing preserves the old facts, first binding, attributes and
history together. Constructor-only paths need no configured Base processing.

`DomHtmlCaptureLimits::with_base_facts(records, utf16_units, validation_work)`
selects the separate producer fact-formation limits explicitly. Adapter vectors
obey the existing subject/work domains; record/unit limits apply to each retained
formation, and producer validation work is cumulative across initial capture and
Slot reconstruction. A newly supplied Slot ceiling includes already consumed
work. Semantic query sessions retain their own caller-selected limits. Canonical
Link history joins the actual capture. Frozen rows bind the current nodeDocument
and href as required by the producer, while the canonical historical freezing
Document stays separate. The producer accepts one full revision for document
rows; captures containing differently qualified bound Documents reject with
`RevisionMismatch` rather than inventing a shared revision. The independent
Base href getter still uses declared parse results even when CSP blocked the
historical frozen URL.

HTML's shared `select_frozen_base` owns failure/data/javascript/CSP-block fallback
and successful selection. Successful assignment precedes actual native response
qualification. Current Scripts have real `Uninitialized` results, including
speculationrules-typed Scripts: native Disabled preparation supplies no result
callback. The actual shadow-inclusive response walk reads those states. Native
Documents have never been active in a navigable, so node navigable is null and
HTML877d76574's top-level guard returns before flag/task access. Current batch
commands and host observations admit no UI recipient or selector/style executor.
DOM publishes actual `DocumentBase` invalidation and frozen `ElementState` changes;
`FactsPublished` certifies that bounded canonical publication, not UI,
CSSOM or task delivery. Identical qualified refreezes whose actual state/effects
remain unchanged are canonical no-ops, without fabricated history counters.

Native Link retains shared `HtmlLinkFlags` and its nullable creating-parser
Document. Only actual null-namespace disabled removal enables the historical flag;
readdition/adoption preserve it and clone/native installation start fresh. Native
creation also establishes an actual null associated stylesheet, and Documents
start with the source-initial empty render-blocking set (HTML877d7480–7481).
No admitted native command installs a Link sheet or enters that set; structural
View Transition CSSOM identities establish no Link association. The actual
href/rel/disabled/insertion guards read the current native no-BC environment:
stylesheet, modulepreload, preload, prefetch, preconnect, dns-prefetch and expect
processing triggers require browsing-context connectedness. Disabled-set has no
associated sheet to disable; id/anchor-name processing iterates the actual empty
render-blocking set. This proves no general render-blocked Boolean. Actual HTML
A name changes now execute that hook; other Elements' ordinary name attributes
remain exclusion evidence.

Icon/manifest/alternate-feed/search UI, discovery and service uses have no admitted
native recipient; this owner advertises no supported processing-token registry.
Pingback's external processing remains unsupported, without a blanket no-BC
claim. Resource-bearing sheet/set/environment state refuses precisely through
`LinkProcessingUnavailable`. Future retained parsed installation must transfer
actual facts, frozen history, flags and remapped Document references under its
own cause before installation hooks; native fresh initialization cannot erase
unknown resources/history. Current public Document destruction remains
unadmitted; this unit neither widens it nor pins historical references.

Native A/Area start with a null associated URL and execute HTML's real set-URL
creation steps, returning on absent href without external facts or extra query
admission. A changed null-namespace href first clears the cache, then encoding-
parses the exact UTF16 against the **current Document base URL** and actual
encoding. The shared Document-base selection uses the current first Base's
frozen record when present; it does not substitute the fallback or rerun Base's
CSP selection. Exact equal optional href values and namespaced attributes return
before parsing, including after adoption. Present-empty href still parses.

`DomBatch::bind_encoding_url_facts` binds immutable producer observations to
an exact initial `DomDocumentStamp`. The existing native Base-facts binding
establishes opaque URL/encoding identities against the real UTF8/about:blank
Document environment; identity binding never infers a charset from an arbitrary
label. The hyperlink bank validates the complete current parse revision, selected
base identity and encoding identity. A frozen base may retain an older revision.
A missing, stale, foreign or mismatched context refuses atomically; an actual
qualified failed parse instead retains its provenance with a null URL.
`DomNode::as_hyperlink` reads the stored URL and historical parse Document without
reinitializing an href IDL getter. Observation updates do not silently reparse.

`DomLimits::with_encoding_url_processing` bounds aggregate installed observation
capacity and cumulative batch work, separately from producer formation limits.
Both allowances default to zero. Canonical unit admission charges actual retained
raw/base/result string backing capacity for each bank and chosen cache, including
transient parses later discarded. Scans and raw comparisons poll cancellation;
Select composition also charges its actual enclosing work. All admission and
cancellation precede the single publication. Whole final semantic equality ignores
only derived aggregate unit accounting: a capacity-only candidate retains the
old canonical backing, usage and revision, including at exhausted revision.
Other real semantic changes publish the candidate's actual admitted usage and
complete changes; no per-node allocation restoration or synthetic revision occurs.

Adoption preserves URL history; a later changed href parses its actual new
Document context. Clone/import-as-clone and native immutable installation create
fresh null state, then run real copied-attribute/tree hooks in the destination.
Snapshots and captures preserve selected opaque parse identities and history.
Captures map the historical parse Document independently from current nodeDocument,
with producer string/relation retention and cancellable adapter work admission;
Slot supplements preserve those same observations. Explicit destruction retires
ordinary state without pinning Documents or following a reused generation.

A/Area href/rel/referrerpolicy and insertion/removal hooks consider speculative
loads using actual current/old-ancestor facts. The native Document's exact null
node-navigable guard returns before task/service processing. A name/id hooks use
the existing actual empty render-blocking-set proof. These steps provide no
navigation, download, loading service, Blob-store truth, origin/components or
href-IDL reinitialization. Future parsed installation must transfer its actual
history under its own creation cause.

Native HTML Details derives open state solely from exact null-namespace `open`
presence. Generic name/open changes and per-node insertion consume HTML's shared
Details rules. Groups use equal nonempty exact UTF16 names in the same ordinary
tree, including detached trees and Template contents; distinct shadow trees stay
separate. Opening first records the initiating transition, then removes the first
open peer's attribute through the real mutation engine. Name changes and insertion
instead close the given Details. Detach/adopt preserve identities and pending
intentions; clone/import and native immutable installation run fresh copied
attribute/insertion hooks in their actual order.

`DomLimits::with_details_toggle_intentions` admits a concrete number of raw,
ordered, duplicate-preserving transition records. The default is zero; constructor
and closed-state edits need no intention capacity. Current-batch retirement
evidence shares that count so destroying a recipient cannot reset admission.
Group scans use the finite canonical node/attribute/string domains, poll each
visit/comparison, and allocate no traversal bank. Canonical strings retain their
existing unit accounting; fixed intention records and transfer output use their
separate count limits and fallible exact reservations.

`take_details_toggle_intentions(max_output, cancel)` admits and builds the whole
output before atomically relinquishing DOM ownership. Its opaque owned transfer
and intention records cannot be cloned or publicly reconstructed; borrowed
inspection permits a future consumer to plan admission before consuming the
transfer. Every published transition has an opaque owner-qualified checked
operation identity, preserved through handoff and cancellation. Private creations
destroyed before publication consume no identity. Queue-only transitions and
draining have explicit `DetailsToggleIntentions` change coverage and revisions;
failed or cancelled staging preserves attributes, order and identities together.
Explicit destruction cancels only still-owned intentions for exact retired
targets, with finite `TargetDestroyed` evidence carrying the target, operation
identity and states. The transfer batch retains its historical source snapshot;
consuming iteration yields records retaining their operation, target and states.
A consumer must validate each exact target against a fresh current snapshot before
execution. DOM does not coalesce tasks or fire ToggleEvents. Runtime's shared
`DomSession` owns the shared DOM manipulation task bank, concrete Details tracker,
coalescing and native event dispatch. Its controlled ingestion admits the actual
pending bank before taking custody; a session-bound caller cannot bypass that
handoff with a raw transfer.
Tasks retain their original queued Document and hosted Global across adoption,
while dispatch uses the current tree's parent path. Standalone DOM consumers can
still use the raw transfer API; prior nonempty external transfers prevent later
session bootstrap because that external custody cannot be recovered.

This common contribution does not complete native HTML. Remaining exact producer
requirements include other native form/control relations and values,
resource-capable HTML preparation, resource-capable
iframe navigables, resource-capable link/style associations, modal dialog
operations, and resource/media
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
`Dom::native_checkpoint_step` executes the oldest native FIFO turn. A Slot
marker transfers the then-current signal set as historical slotchange intentions (bubbles=true, composed=false,
cancelable=false), without dispatching callbacks or fabricating observer objects.
A failed/cancelled drain preserves the set. Native destroy removes exact retired
pending recipients with `SlotSignalCancelled { reason: Destroyed }` evidence;
detach/adopt/disconnect do not cancel. Drained batches retain their historical
capture; `revalidate` checks the live owner and reports `SkippedDestroyedTarget`
without following generation reuse. Later signals belong to the next checkpoint.
New pending work advances the revision even when final node values are restored.

Native HTML Style retains its creation history and nullable CSSOM association.
Ordinary atomic edits refuse at an external processing boundary without
publishing a partial edit. `begin_source_edit` runs one `DomSourceEdit` through
the same canonical source engine and returns a lease for actual external
requests. Constructors take existing identities; callers sequence separate
operations explicitly. Native installation and Select completion retain their
own feature roots in that engine. Once a source phase publishes, dropping the
lease parks its mandatory continuation in the DOM; `resume_source_edit`
reacquires that exact operation. An acknowledged CSSOM publication survives a
failed later segment and is not reissued. Cancellation preserves an already
published phase and its remaining cleanup obligations.

`DomSourceLimits` bounds represented live bytes, UTF-16 backing capacity,
cumulative work, phase publications, output records, published native
identities and simultaneously outstanding nested operations. Storage includes
simultaneous private candidates, checkpoints, temporary banks and retained outputs.
Shared immutable payloads are charged per
retained occurrence; allocator metadata and Arc control blocks are excluded.
Growth admits requested replacement storage while the old bank is live and
checks actual capacity afterwards. These are representation limits, not a
physical-memory ceiling. CSS and CSSOM keep their independent producer limits.

Before exposing an external phase, the source owner reserves checked revision,
invocation and association headroom for its mandatory current-command tail.
Finite work or storage refusal can preserve that tail for a raised-budget retry;
retry retains spent work and acknowledged effects. Independent Base, Meta,
Select and numerical limits remain separate and can be raised through
`set_limits` without resetting their retained usage. Changes to cumulative
OptionValue work and allocated-byte budgets must cover the amounts already
spent by the retained operation.

Style processing retains the actual mutable CSSOM store through
`DomStyleCssomRoot`, independently of an execution lease. Binding admits the
canonical root bank before external publication. `style_cssom_root` reissues
access to that exact live root's retained owner, returns `None` for a missing
binding and refuses an incomplete source tail or invalid root identity. Busy or
poisoned access preserves custody and any unattempted commit envelope. Root
retirement releases canonical custody; independently held access capabilities
may outlive it without pinning the tree. Snapshots preserve historical identities
but do not keep a mutable CSSOM owner alive.

`set_style_disabled` changes the existing associated sheet's flag through this
source protocol. A null-sheet assignment does nothing and cannot become a flag
for a later-created sheet. The operation preserves sheet identity and association
serial and queues no new load/error notification. Current CSSOM flags must be
read from the actual retained owner, independently of a historical association's
receipt revision; a flag read does not establish resource readiness. Pending
parsed Style processing remains explicit and cannot certify a null sheet.

The exclusive standalone `Dom` is `Send`, including a parked published source
tail, its acknowledged receipt and mutable CSSOM custody. Transfer moves the
whole owner; it does not reconstruct identity from a snapshot or authorize
concurrent writers. Private numerical accounting uses shared atomics while
owner-local cells remain, so this contract does not provide `Sync`. Runtime owns
its separate session, resource/task lifecycle and controlled event reentry.

Native HTML Dialog exposes retained `returnValue` through `DomNode::as_dialog`.
`set_dialog_return_value` preserves exact UTF-16 and old snapshots; equal values
are no-ops. Native clone/import starts with the source-defined fresh return value,
while adoption preserves the existing value. Ordinary `open` attribute and tree
hooks consume HTML's rules against the actual Document environment and canonical
Document lists. `DomDialog::close_watcher` observes the stored optional watcher
identity; historical snapshots retain their captured association. Runtime owns
the actual Window manager and its watcher objects. Ordinary atomic edits refuse
a reached watcher service with `NeedsEventfulOperation` before publication.
Tracker and modal state retain their own source-defined rules. Request-close
readbacks retain the actual nullable request result/source and force-enable flag
at the snapshot's revision; they grant no current invocation authority.

`DomSourceEdit::show_dialog` starts the shared HTML Show cursor and can expose
its actual cancelable beforetoggle request. A bound event executor acknowledges
only a completed dispatch. Its scoped permission permits callback edits and
nested Source operations; after dispatch the original cursor resumes against
fresh canonical state, preserving those edits. The initial event observation
has no unpublished Draft and does not mark the snapshot incomplete. Dropping
its lease retains the exposed request even before the first tree publication.
Each outstanding ancestor constrains real nested work and retained storage.
Publication, identity and output counters remain owned by each original
operation; callback edits preserve its reserved closing revision without
pretending that it published those edits.
If a pending child exhausts an observing ancestor's allowance, retain that
ancestor's issued event request and use `raise_source_event_limits` to raise
its policy monotonically. This does not execute the parent or replace the top
child. Spent work and accepted receipts survive the update; ordinary top resume
still performs full admission for policy reductions and preserves the required
closing phase. Canonical `set_limits` refuses while an event observation or its
retained ancestor is outstanding.
Task transfers, notification completion and other atomic controllers whose
outputs have not joined this admission retain their strict Source-idle guard.
They preserve pending obligations for later service.

Canceling beforetoggle is a normal HTML return, distinct from canceling the
Source operation. If the callback retires the receiver, genuine event
cancellation completes without reading it again. An uncanceled post-event read
instead reports the original receiver's retirement. QueueToggle publishes the
real task and canonical tracker through the original bound DOM-manipulation
owner. Its exact publication is accepted once; refusal retains it for retry
without queueing again. Show adds `open=""` through the ordinary attribute hooks
and publishes the reached prefix. An inactive or disconnected receiver then
reaches RememberFocusedElement. A connected, fully active receiver enters its
Document's open-dialog list and requests EstablishCloseWatcher. Runtime's Source
driver registers the actual watcher and accepts its exact result through the
original Source receipt; Show then requests RememberFocusedElement. Direct
Source attribute and tree operations use the same hook and retained suffix.
Without the actual service, the request remains pending. Registration uses the
Dialog's original relevant Global, independently of its current node-Document;
adoption does not substitute a new Window.

Each Source operation retains its actual entry scope outside the mutable Draft.
Hosted entries preserve their original Global and optional Host association;
they cannot fall back to native binding when that association is unavailable.
A native entry may receive one explicit Host binding from the original trusted
issuer before its first advance. Advancing without one seals it unbound, and
resume cannot change that choice. Nested roots select independently. The
read-only `DomSourceExecutionEntryScope` observation grants no execution or
publication authority.

At RememberFocusedElement, the original trusted Host producer prepares an
operation/action/occurrence-qualified observation against the actual canonical
basis and original live Host identity. Runtime's sealed headless Host can attest known
absence; missing or unsupported observations leave the request pending. Accepted
absence writes null to the shared common HTML state and acknowledges the cursor
once. Retained publication custody survives
refusal and retry without sampling again. An already-null slot remains a
semantic no-op. No current positive focused-Element producer or arbitrary DOM
focus setter is exposed. Inactivity, no Global and surface selection alone
cannot establish absence. The headless proof covers the complete admitted Host
lifetime, so unrelated issuer progress does not invalidate it. Actual Host
retirement/reuse, disposed authority or a stale canonical basis still refuses.
Future mutable focus adapters require their actual source and scope revisions.

Show then captures the receiver's current node-Document in its original
continuation. The ancestry query reads that Document's qualified raw top layer
and the members' actual popover visibility. The admitted common state proves
hidden, so both Auto and Hint showing lists are empty and the endpoint is null.
Hide rechecks the captured Document and that proof before advancing to
RunDialogFocusingSteps. Nonpopover top-layer entries do not prevent it.
Unavailable history or unqualified nonhidden state leaves the corresponding
action pending. These checks use
canonical Document state independently of the Host's headless focus proof;
they perform no popover dispatch or focus movement.

At RunDialogFocusingSteps, the allow-focus decision reads the receiver's
then-current Document. An explicitly retired hosted Document has no browsing
context, so feature permission is denied. If its actual original Window remains
in Runtime's watcher custody, that Window's constructor-owned never-activated
state supplies the second negative input. The shared HTML rule then returns
before focus-target selection or focus effects. The original affine receipt
acknowledges this proven denial and lets Show reach its actual Complete branch.
The receiver's original Global still owns any previously queued task; adoption
does not substitute it for the current Document's Window.

A missing or pruned Window supplies no activation observation. Native Documents
without a Global and branches requiring an unknown permission or actual focus
movement retain their service wait. This negative branch provides no arbitrary
focus setter or positive focus implementation.

Open removal and source-required tree cleanup destroy the actual watcher before
clearing the canonical association. This cleanup remains available after hosted
execution retirement, which leaves the surviving Dialog and its original Window
custody intact. Retries retain the original receipt without repeating manager
effects. Queued delivery cannot bypass an unfinished synchronous tail.
Show completion requires the actual producer Complete branch, including the
proven negative allow-focus path above.

`DomSourceEdit::close_dialog(receiver, result)` represents the standard
close method with its actual null source. An omitted result preserves returnValue;
a supplied empty string writes empty. The original HTML cursor orders authentic
noncancelable beforetoggle, the postcallback open read, toggle publication,
complete open-removal/watcher hooks and result/request-state updates. A surviving
callback that removes open supplies the genuine early return. Known previous-focus
absence reaches actual final Close task publication. Modal Close requests deferred
top-layer removal and clears the modal flag; unsupported focus restoration
retains its precise action wait. Method Complete requires the
real QueueClose acknowledgment.

`DomSourceEdit::request_close_dialog(receiver, result)` represents standard
requestClose with its null source. Closed, disconnected or inactive receivers
return at their actual source guards. Otherwise the original Source continuation
captures the real watcher, enables it for the request and writes the nullable
request operands before invoking the watcher. The watcher fires an actual
cancelable plain `cancel` event. A normal watcher result always precedes the
literal final force=false write, including genuine cancellation. Cancellation
does not clear the request slots or undo callback edits.

Source retains both HTML cursors and invokes the existing Close frame when the
watcher's current guards permit it. At that call it captures the then-current
request operands, so reentrant requests can affect the supplied Close result.
A reentrant request which observes running cancel returns without running another
cancel action, then performs its own literal force=false. The outer request's
fresh enabled check can consequently skip Close. Omitted and supplied-empty
request values remain distinct throughout.

Runtime retains each started invocation's exact original watcher record after
manager membership removal. Guard reset, reached activation consumption and
invocation settlement complete through action-qualified affine receipts. Lease
drop, cancellation or admission refusal preserves the original mandatory tail.
A retired receiver cannot receive the final force write: the owner retains its
authentic stopped action and any real cleanup, exposes `PendingCleanup` with a
typed cause while settlement remains, then clears the canonical incomplete-source
marker and returns the native failure. A surviving receiver receives final false.
Neither path fabricates a normal nested Close return or event receipt.

`DomSourceEdit::show_modal_dialog(receiver)` runs the standard null-source
ShowModal prefix. Initial invalid-state checks precede real cancelable
beforetoggle; after an uncanceled event it checks current open, connection and
popover-showing state without repeating the activity guard. Genuine canceled
and post-event early-return branches can complete. The positive prefix queues
the actual toggle and adds open through its complete ordinary hooks. After the
valid watcher assertion, one callback-free canonical phase sets the modal flag,
establishes Document blocking through raw membership, and adds the Dialog to the
top layer if it is absent. That phase publishes before the remembered-focus
observation. An already-raw Dialog takes the source's literal skip-Add branch,
including when pending removal. Supported remembered-focus and empty-popover
branches reach the genuine `RunDialogFocusingSteps` wait. The operation remains
incomplete there; modal publication does not certify focusing or full ShowModal
completion.

`DomDocument::top_layer()` borrows immutable raw membership, pending removals
and the Document stamp. Its allocation-free `ua_rules()` projection emits the
universal UA-important `overlay:none` baseline, followed by exact-Element
UA-important `overlay:auto` inputs in raw order for members not pending removal.
These are canonical producer declarations for downstream Style collection and
cascade. They do not establish computed overlay values or rendering. The getter
requires a complete Source basis and qualified Document history; an unfinished
ShowModal therefore cannot use it or semantic capture to bypass that guard.
An earlier complete snapshot retains its historical lists and inputs.

Raw membership and ordered pending removals share the canonical Document owner.
Requesting removal retains raw membership and removes the dynamic auto input;
re-adding a pending member removes both entries before appending it at the top.
Immediate source removal clears both lists and the input. Ordinary removal,
adoption and destruction use the same cleanup against the old Document; fresh
clone/import initializes independent nonmodal state. Manually removing `open`
does not stand in for Close and can leave modal/raw blocking state. Actual
rendering-checkpoint removal still requires separately qualified rendering and
computed-style evidence, which this surface does not admit.

The watcher assertion observes the published canonical prefix. If a callback
adopted the receiver into a connected inactive Document, its completed open hook
can leave the watcher null. Proven null yields `SourceInvariantFailed`, carrying
the exact receiver, ShowModal action, observed stamp and
`MissingDialogCloseWatcher` reason. It preserves real open, queued work and
callback effects. Incurred cleanup retains the same failure through
`PendingCleanup`; terminal release returns that failure rather than retrying
the assertion after later adoption. A new invocation observes retained open
nonmodal state and follows the initial InvalidState branch. Missing or unfinished
setup, stale facts, inconsistent watcher identity and receiver retirement remain
distinct from this proven invariant failure.

Started ShowModal beforetoggle dispatch retains its real path and receipt through
receiver retirement. Actual cancellation can complete normally; an uncanceled
event instead fails at the required ReadOpen. Later prefix failures preserve
published effects and settle only their incurred original-owner work.

The `dispatch_relevant_global`, `dispatch_global_document` and
`dispatch_document_fully_active` fronts qualify current hosted reads during an
actual active Source event scope. They validate the original executor and
published observer chain, charge its Source work and preserve the incomplete
marker. Outside that scope they require a completed Source basis; an unfinished
nonobserving operation cannot use them to bypass ordinary completeness guards.

Dialog toggles use the DOM-manipulation task source and its tracker. Close
notifications are separate user-interaction tasks, with no toggle tracker or
coalescing. Runtime's one element-task owner selects the actual source explicitly;
both preserve the queued target and original Global/Document association.
Successful Toggle dispatch clears the surviving Dialog's current tracker;
Close dispatch does not clear it or repeat method effects. Real receiver
retirement during beforetoggle preserves the started dispatcher and authentic
receipt, then yields `SourceReceiverRetired` at ReadOpen rather than a synthetic
absent-open return. Internal Close callers with nonnull source, complete modal execution,
positive focusing, platform close requests and general popover
processing remain separate contributions.

A started dispatcher registers its actual Node invocation targets through
`register_dispatch_custody`. Configure aggregate
`DomDispatchCustodyLimits` with `DomLimits::with_dispatch_custody`; the default
has zero capacity. Custody retains exact target generations and records their
actual staged node-document only when retirement successfully publishes,
including adoption and destruction in one edit. It retains no mutable node or
tree. Live targets use their current association; a retirement stamp grants
neither mutation nor current Document/Global execution authority. Dropping the
affine scope releases its rows, while the bounded weak index charges remaining
headers until reclamation. This mechanism is independent of snapshots and
change history and preserves the standalone owner's `Send` contract.

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
HTML installation, nonempty/scoped registry objects and HTML callbacks still require their real
adapters, as do #1124's remaining language/control/pseudo/host capabilities.

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

Native hosted creation is a separate, retained source operation driven by
Runtime's `HostedRuntime` through the same `DomSession`. The DOM source issues
owner-qualified, generation-checked Host, browsing-context, Global, history-entry
and navigable identities. Its controlled phases establish the initial Document,
Permissions Policy, actual Global association, loaded time and initial entry.
Operation and phase checks reject foreign, stale and replayed publications while
preserving the valid continuation. Dropping a permission does not finish its
operation. Explicit record, byte and work limits admit the actual state.

The initial no-opener constructor establishes one immutable COOP policy and
URL/base/encoding payload before handing execution to Runtime. The published
Document borrows these actual values through `opener_policy()` and
`base_document_facts()`; other native creation profiles return `None`. Its
`document_base_record()` selects the actual current base through the same Base
owner as `document_base_url()`. Reads and retries preserve the constructor's URL,
encoding and provenance identities, including in historical snapshots after
destruction. These facts do not qualify an external URL parser or its responses.

The initial hosted Document has a real non-null, complete empty Global registry.
Its canonical registry view consumes the original HTML-semantics lookup over the
retained producer capture. A known definition miss is distinct from a null or
unavailable registry. This bounded factory supplies no definition mutation or
scoped registry implementation.

`DomDocument::is_fully_active()` reads the actual captured hosted association;
closed Documents return false. Snapshot `global_document` and `relevant_global`
lookups also retain historical identity. The corresponding methods on the current
`Dom` owner check the live issuer, identity and association before execution.
Native Frame/Iframe Source hooks create initial children and retain their actual
parent/container relationships and queued history work in this same owner.
Runtime computes the selected initial fullscreen Permissions Policy; required
qualified URL observations remain explicit external inputs. Child entry target
names preserve absent, supplied-empty and exact supplied values. The fixed
top-level target name remains supplied-empty. Iframe creation consumes its
retained sandbox flags; declared-origin policy processing separately consumes
the source-selected raw sandbox attribute. Later attribute edits do not rewrite
an already completed Document policy.

Destruction retires hosted participation without deleting the managed Document
or its tree. Canonical creation, clone/import, adoption, shadow attachment and
native fragment installation can still consume its actual metadata, original
Global identity and retained registry. Input and Textarea keep using its fixed
platform policy. XML creation copies the associated origin and policy into a
fresh closed Document. None of these facts restores execution permission or
supplies unavailable family/resource services.

Current staged activity is false
for actual retired participation; expired or unknown live authority remains an
error. Dialog hooks and standalone Img use that same activity decision.
An older snapshot retains its captured activity state. Live-child removal is
distinct from retained history and cleanup custody. Queued initial history
attachment remains obligatory after removal, and completed Document destruction
clears the actual entry's nullable
Document field without deleting the managed Document. These initial-entry
operations do not establish general navigation, replacement, resource loading, modal services or
JavaScript execution. The [Runtime explanation](../surgeist-runtime/docs/explanation.md#dom-orchestration)
describes native task custody and checkpoint limitations.

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
registry-object setter; qualified completed-report transfer is described below.
Native environment proof also survives captures.

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
Actual Input/Textarea flags and established-current Form associations use separate
records, bounded together by twice the subject ceiling. Applicable mode-correct
current values consume the HTML capture UTF-16 ceiling and language validation
work; Input applicability comes from the shared HTML predicate, while Textarea
supplies actual API-normalized current text. Directionality and value-missing
observations do not invent submitted wrapping. Slot supplements
rebuild these same values/flags with assignments under their explicit language
limits, without replacing known current state with an empty fact vector.
Qualified names, ordered attributes, current nonce, saved creation `is`, registry
and custom-element state come from actual canonical records. It creates no
intermediate HTML construction tree. Common slots not represented by HTML's
capture schema remain available through the retained `source()` snapshot.

`DomDocumentMetadata::design_mode_enabled` reads canonical Document state.
Native creators establish false; completed-report installation transfers each
owned ordinary, auxiliary and inert Document's original HTML-qualified state.
Captured existing Documents preserve their current state. Initial and Slot
semantic contexts feed these actual Document rows to the shared editability
kernel. The explicit subject/work limits also bound this fact formation and its
producer validation independently per formation. No cumulative producer budget
or external spellcheck default is inferred. Explicit spellcheck attributes can
answer; policy-dependent defaults remain missing until supplied by their owner.
This readback does not execute the designMode setter, range reset or focusing.

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
It rebuilds the other canonical supplements alongside those flat facts.

Initial captures and Slot reconstruction both qualify each represented Document's
actual managed raw top layer. The last actual HTML Dialog is the blocking subject,
independently of its `open`, modal flag or pending-removal state. A qualified layer
without a Dialog supplies known unblocked state, including for supplemental
Documents whose ordinary trees are only partially captured. Unavailable history
or a blocker absent from the capture's subject map retains unavailable blocking
facts. No absence is inferred from captured children or Dialog attributes.
Initial captures supply these Document facts without certifying any
flat-node relation; Slot supplementation supplies the actual flat records needed
for connected inherited inertness. Own inert can succeed independently. Detached
subjects remain known flat-excluded, with inherited inert ancestry unavailable
under the selected contract.

This native shadow/Slot/control/Template semantic profile does not complete #1124. Other live controls,
registry objects,
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

`Dom::install_native_html_fragment` consumes an immutable `HtmlFragment` as an
explicit fresh-native authoring input. It checks an exact `DomDocumentStamp`,
the destination parent and optional insertion reference before source work,
then stages the whole installation through the same Draft/publication owner.
Raw namespace/prefix/local components, attribute order and UTF-16 are preserved;
DOM's component grammar applies without reconstructing a QName or applying the
namespace-aware invocation's reserved-prefix rules. Actual native attribute and
tree hooks run in source order. Native Template owned contents use real cached
inert Documents; Script uses fresh native state and actual Disabled preparation.

The returned `DomNativeHtmlInstallation` owns the installed immutable snapshot,
change report and full source-qualified `HtmlNodeId` to `NodeId` bindings. The
source fragment maps to a real detached empty canonical fragment after transfer;
owned Template contents are mapped, private auxiliary Documents are not. Bindings
follow source preorder with owned contents before ordinary children. Source drop,
later edits and retirement do not rewrite this historical result. Every fallible
map/output allocation happens before publication; refusal preserves the entire
live tree, identities, revision and history.

`DomNativeHtmlInstallLimits` separately admits mapped nodes, adapter work and the
simultaneous retained capacities of traversal/map/output buffers. Work charges
visited nodes, five association observations per node, edges, attributes, prior
map comparisons, map writes, final remapping/bindings and raw field UTF-16 extents.
An empty ordinary source costs nine work units and creates its mapped wrapper.
Canonical staged tree/name/family work and publication storage retain their
existing `DomLimits` admission and cancellation checks. The adapter quota does
not claim to meter those independent operations.

Declarative shadow relationships and hosted source fragment roots receive typed
association refusals: native attachment cannot supply retained declarative/root
document facts. Unsupported native families retain their actual requirement
errors. A bare fragment does not carry complete parse-report context, provenance,
effects or remaps; passing one explicitly requests native creation.

`Dom::install_parsed_html_document` separately transfers an admitted completed
offline parser report into a fresh canonical Document. The caller supplies the
current store stamp and explicit URL, opaque encoding, origin and destination
input-platform facts through `DomParsedDocumentEnvironment`. The operation
preserves actual construction state and publishes fresh canonical identities,
without replaying native attribute or insertion hooks.

`Dom::install_parsed_html_fragment` checks the original `DomHtmlCapture` against
the actual current target and materializes a detached fragment in that Document.
It applies the report's ordered Template-cache and shadow-attachment effects;
it does not implicitly append the fragment to its context. Retained auxiliary
parser Documents remain distinct from the captured target and template owners.
`DomParsedHtmlInstallation` keeps the report, installed snapshot, changes and
separate construction-subject and captured-subject bindings as historical data.
Subsequent edits cannot extend or rewrite those bindings.

The current profile requires the report's supported closed scripting, registry,
image and Document-policy qualifications. Missing required family or historical
state, a stale or foreign capture, incompatible environment, cancellation or
admission failure refuses the complete installation. The adapter separately
bounds mapped subjects, work, historical backing and simultaneous scratch
capacities, including old and replacement vectors during growth. Canonical
Draft and publication retain their own limits. Downstream associations are
explicitly pending; installation does not execute resources, CSSOM or callbacks.
For captured historical hyperlink observations, installation checks the actual
parse Document against its current canonical Base selection, opaque encoding,
full independently bound revision and exact raw-input outcome before and after
the ordered construction effects. Missing or changed required facts receive
`CurrentPolicyUnavailable`. Auxiliary parser and inert-owner history keep their
original roles. An optional unused whole captured Base bank remains fully charged
historical request data; it does not override the current canonical bindings.
Completed contextual SVG Script transfer uses the original closed fragment
producer and independently checked current canonical Document environments.
It preserves the actual processed bit and Parser/Clone cause, including false
state in detached fragment or Template contents. Installation does not replay
processing; later explicit canonical insertion uses the actual connection rule.
Captured target, auxiliary parser and inert-template Documents keep their
distinct identities and historical bindings. This grants no acquisition,
execution or load/error notification capability.

Parsed Dialog transfer consumes the original controller's retained current
return value and closed-service qualification, independently of content
attributes. Newly owned Documents transfer their own qualified interaction
history; captured target Documents retain their existing lists. An unfulfilled
original Dialog operation refuses installation with its subject and action,
while its Document and cause remain in the immutable report. Materialization
does not replay hooks. Later canonical open changes, removal and insertion run
the ordinary hooks against the then-current node Document. This does not grant
hosted show/close, focus, watcher or top-layer services. Other report profiles and
missing family state remain under
[#1135](https://github.com/bj-data-eng/surgeist/issues/1135).

Remaining native special families also remain
unadmitted capabilities. This crate does not claim complete DOM or
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
