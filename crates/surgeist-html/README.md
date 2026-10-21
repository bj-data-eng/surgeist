# surgeist-html

HTML language values, local interpretation and immutable native construction.
This crate preserves exact UTF-16 in `DocumentString`, names and ordered
attributes, including isolated surrogates. It owns native document/fragment
builders and the controlled `bounded` construction operations used by parser.
Both producers publish the same HTML result and result-local node IDs.

`HtmlNode::document_design_mode_enabled` reads the actual owned Document's
source-defined initial false state; non-Documents return `None`. These controlled
constructors admit no design-mode mutation. Existing parser report views lend
the same qualified node, so consumers can transfer its state without inventing
a native default. Captured external Documents retain their own state authority.

`HtmlNode::document_focused_area_designation` reads stored original state.
The actual controlled `CreateDocument` transition issues selected `KnownNull`;
an unqualified Document record remains `Unavailable`, and non-Documents return
`None`. Structural/mode edits and original arena recovery preserve the state.
The current closed construction domain has no positive designation writer.
Immutable copies retain the same evidence; captured external Documents cannot
acquire it from matching metadata. This grants no viewport, focus execution,
system focus or completion of pending autofocus/interaction work.

The [selected WHATWG edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior. [INVENTORY.md](INVENTORY.md) connects the vocabulary and local
operations to their clauses. The [detailed HTML guide](../../docs/reference/html-operations.md)
records author/recovery behavior, external inputs and capability limits.

HTML has no parser, semantic-capture, DOM, runtime, CSS or Color dependency.
Its `cssparser` dependency supplies only the HTML legacy named-color grammar.
Parsing belongs to [surgeist-html-parser](../surgeist-html-parser/README.md);
captured-tree queries belong to [surgeist-html-semantics](../surgeist-html-semantics/README.md).
Consumers depend directly on the owner they use. No HTML result is a live DOM.

`HtmlNameInput`, `HtmlAttributeInput` and `HtmlAttributesInput` lend exact
semantic operands without prescribing their storage. HTML construction values
implement these traits; another owner can supply immutable names and attributes
without constructing an HTML attribute array. Attribute slices preserve their
original order. The traits supply no validation, atom identity, tree facts or
custody, and do not normalize UTF-16, prefixes or raw values.

Name getters remain lazy: controlled classification admits its namespace or
local-name header before requesting that component, and never requests the
unused prefix. Attribute interpretation admits each original raw value before
applicability, including selected values overridden by another attribute.
Reports still retain their selected individual name/attribute evidence by
sharing the original immutable text backing after admission. These handle
retentions are distinct from exporting an owned attribute array; whole-bank
exports and their overlapping storage remain the consuming owner's explicit
admission responsibility. Intrinsic token/number/report allocation retains its
existing contract; borrowed inputs do not make every interpretation allocation
free.

`element_classification_with` selects family and common/nonce applicability from
the exact expanded name through one bounded inventory. `HtmlCommonState<R>` holds
current common values with consumer-owned element references. Its HTML-only state
retains the actual nullable previously focused element; the remaining reaction,
click, internals and popover fields retain their supported closed values.
Constructing a value does not qualify any element's state. Native creation obtains
fresh null-only common state (`R = Infallible`) and nonce evidence through
`native_element_initialization_with`. Parsed installation must instead consume
the parser's completed-report and subject-qualified common-state readback, with
the retained cryptographic nonce read independently. Reference mapping preserves
nullness and applicability; consumers qualify and publish live transitions.

`HtmlCanvasNoneState` describes a native canvas with no bound rendering context,
an origin-clean transparent-black bitmap and exact attribute-derived dimensions.
It moves already-parsed dimensions without allocating pixels; consumers own
attribute lookup, admission and any later context installation. Bitmap dimensions
follow the content-attribute rule rather than the unsigned-long IDL projection.
`HtmlMarqueeState` owns the initially turned-on slot, actual native current loop
index zero, and start/stop writes which preserve that index. The closed native
domain exposes no rendering-layer increment or general animation counter;
copying a stopped value preserves its state, while fresh creation is on/index0.
These values establish neither rendering nor execution capability.

`HtmlChildNavigablePresence` supplies local guards over the consuming owner's
actual null or present child relationship. Null content returns before access,
attribute processing or child destruction and cannot delay the load event.
Present content requires the remaining source steps: it establishes neither an
accessible Document/Window nor a load delay. `frame_post_connection` and
`iframe_post_connection` preserve the actual node-document browsing-context
guard, including unavailable facts. The live owner retains child identities and
executes navigation, access checks and mutation hooks.

`iframe_attribute_branch` selects srcdoc by actual attribute presence, including
an empty value. The bounded shared Frame/Iframe helpers select about:blank for
absent or empty src, lend a nonempty src for the consuming owner's URL parser,
and preserve the ancestor-guard return before later URL/history decisions.
The consumer supplies actual ancestor URL equality; these helpers perform no
URL parsing, comparison or history mutation. Permissions Policy's separate
declared-origin algorithm can require parsing a present-empty src.

`HtmlIframeInitialState` holds the source-initial intrinsic tuple: false
current-navigation-lazy and null timing/resumption values. It is independent of
child presence; creating a child alone does not change those fields. The owner
must leave this initial domain before a reached intrinsic write and cannot
reset processed/imported history from null content. Loading changes use shared
semantic loading states; sandbox parsing preserves absent, present-empty and
token-valued attributes. Local directive flags do not establish live policy.
Clone-as-new initializes fresh fields, while adoption preserves actual state.

`HtmlSandboxFlags::parse_with` produces restrictions directly from one present
value without allocating a report or token strings. Its admission callback
covers reached token and keyword reads and can return cancellation unchanged.
An absent contribution uses `empty`; a present empty value imposes all sandbox
restrictions. `union` combines actual restriction sets without relaxing either
contribution. The live owner selects the actual element and parent Document
contributions before establishing a child policy.

`HtmlOpenerPolicyValue` supplies the five pinned language values, and
`HtmlOpenerPolicy::initial` creates the actual four-field default record. Both
policy values start as unsafe-none and both reporting endpoints are null.
Inheritance copies the source-selected actual parent record; a native no-opener
relationship does not select noopener-allow-popups. These immutable values
perform no header parsing, matching, reporting or browsing-context group change.

`HtmlNewlineNormalizer` applies Infra's CR LF/CR normalization one raw UTF-16
unit at a time, preserving all other units. Keep its state across chunks of one
string; a trailing CR emits LF immediately and needs no finish step. Consumers
own per-unit admission, cancellation and output storage.

`HtmlImgFreshState` retains the closed fresh standalone image request tuple;
it is independent of Input Image Button history. Shared image decisions preserve
the inactive-Document wait before image-support checks, exact per-invocation
flags, latest-started supersession even after a later abort, semantic attribute
changes and Img/Source Picture mutation fanout. Consumers retain canonical
dimension-source/form identities, checked invocation order, waiting work,
retirement, finite admission and atomic publication. These values do not execute
resources, complete a suspended update or replace retained image history.

`HtmlDetailsToggleEvent` retains source-owned old/new states for the actual
queued toggle. It supplies values, not a DOM target, listener registry or event
execution; Runtime owns the task tracker and real dispatch.

`HtmlDesignModeInput` recognizes ASCII-insensitive on/off without trimming.
`HtmlDesignModeCursor` orders enablement, the actual active-range read/reset,
current document-element lookup and the genuine focusing call. A proven null
active range skips only reset, preserving the focusing tail. The adapter owns
Range identity, writes, Source publication and child custody; unavailable facts
leave the exact action pending. The immutable construction owner retains its
initial-false design-mode state.

`HtmlDialogCursor` orders Dialog methods and ordinary lifecycle hooks. Its
standard null-source showModal protocol distinguishes initial InvalidState
guards from normal post-dispatch returns. It requires the completed open hook
and watcher assertion before modal state, Document blocking and conditional
top-layer addition, then the shared remembered-focus, popover and focusing
steps. Each action requires its actual owner; the protocol supplies no native
modal state or service completion.

`HtmlDialogFocusingCursor` owns the complete called focusing protocol: allow-focus,
autofocus or delegate selection, actual focusing, the selected control's top
Document and origin comparison, then the required autofocus-state writes.
The outer method completes only after this child completes. The narrow
`dialog_subject_focusing_decision` can prove a called focusing invocation makes
no change for an actual own-inert, non-root, nondelegating Dialog subject;
it supplies no rendering or focus state and does not skip the remaining tail.

`HtmlFocusDelegateCursor` runs the autofocus delegate first, then a separate
ordinary-descendant traversal if needed. Dialog targets test sequential
focusability; other targets test focusable-area status before called resolution.
`HtmlAutofocusDelegateCursor` checks qualified autofocus presence and actual
direct or resolved areas, with click filtering only for an exact click trigger.
Consumers retain the actual roots, descendants, returned areas and original
trigger. The trigger classification controls a branch and does not replace the
operand forwarded to called resolution. Neither cursor supplies focusability or
performs implicit shadow traversal. Wrong-stage and terminal replies preserve
the cursor.

`HtmlAutofocusInsertionCursor` owns the separate generic insertion protocol:
autofocus and ordinary Document-tree membership, captured target Document,
full activity, sandbox automatic features, allow-focus, captured top Document,
processed flag, then exact candidate removal and append. Consumers retain actual
identities and acknowledge completed reads/effects. Negative guards return in
source order; invalid replies preserve the cursor. The protocol continues past
the standard's optional user-preference return. It performs no focusability
check, attribute-change enqueue or rendering-time autofocus flush.

`HtmlCurrentlyFocusedAreaCursor` resolves the current area of a top-level
traversable in source order: system focus, its active Document, then actual
focused navigable-container children. It preserves the separate loop and final
designation reads and distinguishes null, the candidate Document, and its
designated area. Consumers retain the actual area and DOM anchor independently;
the cursor adds no activity filter, native focus state or host routing.

`HtmlGetFocusableAreaCursor` applies the first matching target-resolution clause,
preserving image-map shape order, flat-tree region order, viewport and active
Document results. Delegating shadow hosts use the actual current area and anchor,
then the complete focus delegate with the original trigger. The actual
current-area result role distinguishes null, a non-area Document and a designated
area. Null and Document results proceed to delegation without an undefined anchor
query; a viewport area keeps its legitimate Document anchor.

`HtmlFocusingCursor` preserves resolution, optional fallback, content-Document
replacement, actual anchor inertness and target equality in source order. It
requires completed old/new chain construction and focus update before returning.
Those called dependencies retain their own implementation and admission needs;
the pure protocol supplies neither native focus effects nor host observations.

`HtmlCurrentFocusChainCursor` composes current-area resolution and chain
construction, preserving known-null versus unavailable observations.
`HtmlFocusChainCursor` orders actual area, corresponding Area Element and anchor
entries, then crosses each child Document through its own navigable container
Element into the enclosing Document. A non-area Document never acquires an
invented self-anchor. The adapter retains and qualifies identities
and list storage; missing container evidence does not complete a shorter chain.

`HtmlFocusUpdateCursor` trims shared chain tails, processes old entries forward,
applies platform conventions, then processes new entries in reverse. It owns
input-change guards, validity-before-change ordering, per-entry Document capture,
changed-only Navigation flag and designation writes, exclusive event-target
selection and exact related-target roles. `HtmlFocusEventCursor` selects the
actual event view and FocusEvent metadata. Both require completed effects and
authentic dispatch before advancing. They provide language protocols; the
consumer owns native state, callbacks, eligibility, work and host routing.

`autofocus_presence` consumes the actual Element name and one
attribute. It admits the exact HTML, SVG and MathML namespace URIs, then
recognizes exact null-namespace `autofocus` without inspecting its value.
Null and custom Element namespaces do not gain HTML autofocus semantics from
a literal attribute. Authored attribute reports retain their own applicability
contract. The consumer admits the actual bounded namespace checks and attribute walk.
Insertion and delegate selection use this same presence contract.

The Dialog method cursor's requestClose protocol requires current open, connection and Document-activity
observations, writes the actual nullable request operands and force-enable
state, and resets that state after either normal watcher-request result.
`HtmlCloseWatcherRequestCursor` owns the watcher's request and Close ordering:
cancel runs within the running-cancel guard, genuine cancellation consumes
history-action activation, and an uncanceled request rechecks current guards
before destruction and the close action. A true result can be an early return;
it does not establish that a Dialog closed. Consumers retain actual identities,
state and invocation custody, dispatch events and complete effects before
acknowledging them. These allocation-free protocols provide no native dispatcher,
Window, activation storage or live DOM operation.

`HtmlStyleUpdateCursor` owns the ordered remove/guard/CSP/create/blocking
steps. Consumers acknowledge actual owner actions after completion; the cursor
executes no CSSOM mutation or CSP interpretation. Exact type eligibility and
source triggers are shared local rules. `HtmlStyleNotificationCursor` requires
real event dispatch before script bookkeeping and rendering unblocking;
queue-time observations cannot replace post-handler reads.

Both cursors require an explicit immutable `HtmlStyleExecutionProfile` at
construction. `SourcePredicates` preserves actual boolean script predicates and
registration/removal actions. `NoExecutableJavaScript` projects out only the
selected executable-script bookkeeping: its `ScriptBookkeepingExcluded` reply
is accepted only at `ReadScriptBlocking`, advancing the update cursor to
`ReadRenderBlocking` or the notification cursor to `UnblockRendering`. It never
supplies a false predicate, waives event dispatch or answers the independent
rendering/media, policy or resource questions. Script boolean replies refuse in
that profile; rendering boolean replies retain their ordinary meaning.
Consumers qualify this projection against their actual operation/execution
owner and preserve it across resume. Missing observations, headlessness and
empty CSS do not select a profile or establish a predicate result. Wrong-phase
and wrong-profile replies leave the cursor unchanged. Constructors have no
implicit/default profile.
`HtmlStyleState` distinguishes fresh or general-clone creation from actual
Document-parser creation. Allocation does not imply a stack push; actual parser
push/pop operations update the stack flag while preserving creation history.
The parser separately owns its creating Document relation. These values certify
neither an associated stylesheet nor completed CSSOM processing.

The bounded Meta kernels share exact pragma classification, language-token and
refresh framing, plus Meta-specific referrer vocabulary and legacy aliases.
Refresh returns original UTF-16 ranges and admits delay materialization before
processing the suffix; it does not truncate a delay or parse a URL. Consumers own
Document history, URL facts, CSS/CSSOM, policy enforcement and affected UI.
`default_document_referrer_policy` supplies the selected initial Document policy,
which is separate from a fetching attribute's empty fallback.

HTML also provides the [neutral Meta CSP protocol](src/csp_policy.rs) for an
explicitly supplied external processor. A consumer session issues qualified
Document associations and exact UTF-16 insertion requests. The external backend
declares supported semantic profiles under its processor authority; the caller
validates the selected service profile before invocation. Actual CSP parsing,
Meta/Enforce directive filtering and complete resulting policy semantics remain
with that backend. HTML supplies neither an engine nor an enforcement service.

An immutable policy state retains the actual native `Arc<A>`. Its narrow native
borrow validates processor and semantic profile before testing the artifact type;
it recovers the retained value without a resolver, typed mirror or general
downcast API. Genuine initial-empty is distinct from unavailable history/service.
Installation and inheritance preserve the artifact and its processing provenance
while issuing new destination Document authority. Later requests use the actual
previous native state, including after the original report is gone.

Every controlled operation describes work, live overlap and closing obligations
before retention or mutation. Cached artifact layouts include complete native
storage and an independent finite disposal-work/atomic bound. The real backend
guarantees allocation-free, nonpanicking, nonblocking transitive last-owner
destruction with no policy work or enforcement. It prepays partial construction
before creating custody. Consuming controllers retain their original receipts,
drop backing before refunding acquired storage, and never refund spent work.
Each new strong occurrence prepays its possible last-owner bound. A moved input
keeps its existing receipt: `HtmlCspLayout::transferred` identifies that custody;
`retained` describes complete returned custody and `peak` includes the action's
overlap. Complete-graph reservations can conservatively count shared graphs;
they do not claim exact physical deduplication or include opaque Arc/allocator
bookkeeping. Ordinary Clone values still require admission when retained by a
controlled consumer; Clone establishes no currentness or publication authority.

Publication preparation validates the supplied association and reserves checked
generation headroom, clones and outputs. Its borrowing guard either aborts with
the actual owned inputs or publishes by moves/swaps, returning the actual receipt,
retired whole association and input custody. No allocation, fallible callback or
last native destructor follows the state change. Every accepted occurrence,
including genuine no-change, advances association generation separately from
immutable policy identity. Latest qualification retains prior identity/revision,
not a predecessor-artifact chain.

The real parser/DOM adapters must bind neutral subjects to their actual
construction/canonical Documents, establish source guards and currentness, and
coordinate publication with their existing owner. A private association Clone may
be a transactional branch or snapshot; this protocol has no current-association
registry. Native/parser integration and exact-policy Base/Style qualification are
not yet connected. Existing reached CSP gaps/refusals therefore remain. Controlled
adapter tests establish this protocol, not CSP engine conformance.

`HtmlHyperlinkSetUrlCursor` orders A/Area cache clearing, href presence,
encoding-parse and successful storage. Shared attribute and tree decisions
preserve the namespace/equality and connection guards. Consumers retain actual
URL history and current node-Document identity; missing external parse evidence
is distinct from URL failure. These steps provide no URL engine or speculative
loading service.

`reflected_element_attribute_change` owns the exact name/namespace clear rule
for caller-established nullable Element reflected properties, including redundant
attribute sets. `HtmlReflectedElementSetter` supplies the asymmetric null/non-null
setter effects in source order. Consumers retain actual weak identities and
complete each effect, including nested attribute changes, before requesting the
next. Qualified target lookup remains with HTML semantics.

Shared strings and attributes expose immutable allocation views. Controlled
validation and edits retain their actual inputs on refusal, while callers admit
work/providers and retain cancellation and allocation accounting. Generic tree
records, allocation identities and mutable backing remain private.

`bounded::HtmlStringPool` and `HtmlAttributePool` provide optional owner-scoped
weak sharing of completed immutable values. Probe a whole validated attribute
bank before processing component strings; offer only the final formed bank.
Exact ordered names, prefix presence and raw UTF-16 decide equality. Public
backing/capacity observations and expanded-name validation are unchanged.
`HtmlValidatedAttributes::from(OrderedAttributes)` transfers the bank's already
established expanded-name uniqueness proof and original backing without another
validation or allocation; it does not validate unchecked raw attribute storage.
Do not offer pending mutable builder storage: retaining a Weak changes whether
that backing can subsequently be mutated uniquely.

Pool actions are owned scalar tokens. Describe under the consumer's owner lock,
release that lock to admit the action's work/storage and poll cancellation, then
perform a pool action under the lock or a read action outside it. No producer
method calls consumer code. Only one operation is current in an owner. Starting
another supersedes unfinished actions; checkpoint changes also invalidate them.
An unlocked immutable read is not independent mutation authority. Destroy a
superseded operation before reconciling its provisional reservation, or retain
that reservation separately until its actual data is destroyed. A completed
immutable candidate remains usable with its own receipt, independently of the
pool's later contents.

Hashing and equality read raw UTF-16 in length-qualified runs of at most 32 units;
component and bank headers are separately admitted. A comparison completes its
prepaid run even after a mismatch within that run. Same-backing hits avoid
content copying, but still fingerprint the input. A completed miss can use
`offer_miss` for the identical still-borrowed immutable operand before consuming
its result: the current qualified operation retains its fingerprint and probe
result, so this path performs neither another hash nor another candidate scan.
A changed formed operand uses a new offer. Supersession or checkpoint changes
prevent miss promotion without changing pool storage.

The private policy has 32 fingerprint buckets, four candidates per bucket and
128 total candidate slots, materialized in pages of eight. Each insertion scans
at most four policy bits: a successful qualified hit grants a second chance;
CLOCK clears skipped hot bits and eventually replaces one candidate. Hints are
written only in an active, unsuspended, unfrozen root; rootless and suspended
probes still return immutable hits without changing policy. New unique
values start cold. This protects a recurring live value among one-use inputs
without promising universal retention, a hit ratio or canonical addresses.
Expired Weak entries are eligible replacement slots. String length and retained
capacity are each limited to 8192 units; banks contain/retain at most 64 attribute
slots and their aggregate string capacity is limited to 8192 units. Independent
actual-payload and pool-metadata ceilings are each 256 KiB. Not every ceiling
binds on every platform: entry/checkpoint geometry can be stricter than the byte
ceiling for the current layouts. The byte checks protect the actual retained
extent, rather than serving as a separately reachable workload claim. Ineligible
values, optional reserve refusal and private qualifier exhaustion bypass sharing
with the original input intact. Mandatory admission/work/cancellation refusal
remains an explicit consumer error, including after earlier work was spent.

A temporary candidate occurrence is admitted before Weak upgrade and keeps its
receipt through comparison and output transfer. Pool layouts report the absolute
preadmitted peak heap extent, including old/new overlap, lazy checkpoint backing
and known Vec target-allocation space for every retained Weak occurrence; opaque
Arc/allocator internals are excluded. Dead Weak entries do not retain element
buffer charges. Reconcile the exact `heap_usage` only after provisional data is
destroyed or a transition releases it. Accepting a mark may release redundant
before-images and reduce that extent. The inline owner layout is separate.

Root sessions and at most 16 nested marks support rollback. Thin checkpoint
headers allocate no directory until their first slot write; only reached pages
then retain first-before slot and cursor images. Repeated writes in a checkpoint
drop superseded current Weak occurrences instead of accumulating inverses. Hit
bits have first-before images too. Accept merges into the nearest older surviving
checkpoint, retaining its earlier image; accepting an older mark preserves the
younger marks. Restore applies the selected checkpoint and younger ones newest
first, recovering entries, generations, cursor/hit policy and actual page presence.
Storage is bounded by pool capacity and active checkpoints, not report length.
No full-pool sweep occurs per offer.

The consumer's root guard aborts on Drop unless moved into publication; its nested
mark guard accepts on Drop or restores explicitly. `prepare_seal` refuses any
outstanding mark and returns the original root without changing authority.
Successfully prepared state is frozen until seal or explicit `abort_seal`;
immutable probes remain possible without changing retention policy. Prepare and
validate both concrete pools' seals before committing either. Failure preparing
the second must abort the first prepared seal and retain the second root's
rollback route.

Admit each concrete pool's fixed `disposal_work()` **before constructing it**,
even an empty one: a refused finalization must still destroy it without another
admission callback. This covers initialized slots, checkpoint directories/cells,
current and saved Weak occurrences and backing release, including sealed owners.
Weak references retain no strong payload. Admit `root_work`, `mark_work` and
`seal_work` before their respective preparations, and each action's
`closing_work` before performing it. Prepare prepays provisional Weak/backing
cleanup and bounded later checkpoint transfers; Upgrade prepays temporary strong
candidate cleanup before upgrading. These payments follow actual storage through
abandonment, staleness, errors and ownership transfer, independently of external
strong-owner liveness. Attribute candidates include initialized records and nested
string handles; primitive UTF-16 elements require no drop traversal. Disposal
bounds describe destruction/release work, with no invented copies, reads or spare
capacity traversal. Later inverse movement has its separate prepaid copy bound.
Closing existing authority allocates nothing and issues no fresh qualifier.
Failure to issue a nested mark suspends mutations for that root; a paired adapter
also suspends its other root after restoring any acquired mark. Pooling never
rewrites an existing semantic no-op or releases immutable report custody. Parser
and DOM integration remain with those consumers.

From the repository root, run focused checks through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-html
cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html/tools/generate_vocabulary.py --check
```

[Attribution and licenses](NOTICE.md) cover the incorporated language material
and dependency graph.
