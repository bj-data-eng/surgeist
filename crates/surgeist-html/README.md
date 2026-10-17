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

The [selected WHATWG edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior. [INVENTORY.md](INVENTORY.md) connects the vocabulary and local
operations to their clauses. The [detailed HTML guide](../../docs/reference/html-operations.md)
records author/recovery behavior, external inputs and capability limits.

HTML has no parser, semantic-capture, DOM, runtime, CSS or Color dependency.
Its `cssparser` dependency supplies only the HTML legacy named-color grammar.
Parsing belongs to [surgeist-html-parser](../surgeist-html-parser/README.md);
captured-tree queries belong to [surgeist-html-semantics](../surgeist-html-semantics/README.md).
Consumers depend directly on the owner they use. No HTML result is a live DOM.

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
real event dispatch before reading current script-blocking predicates and
unblocking; queue-time observations cannot replace post-handler reads.
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

From the repository root, run focused checks through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-html
cargo clippy --offline --locked -j 1 -p surgeist-html --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html/tools/generate_vocabulary.py --check
```

[Attribution and licenses](NOTICE.md) cover the incorporated language material
and dependency graph.
