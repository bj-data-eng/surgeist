# surgeist-html

HTML language values, local interpretation and immutable native construction.
This crate preserves exact UTF-16 in `DocumentString`, names and ordered
attributes, including isolated surrogates. It owns native document/fragment
builders and the controlled `bounded` construction operations used by parser.
Both producers publish the same HTML result and result-local node IDs.

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
the exact expanded name through one bounded inventory. `HtmlCommonState` holds
the shared closed current values; constructing a value does not qualify any
element's state. Native creation obtains fresh common and nonce evidence through
`native_element_initialization_with`. Parsed installation must instead consume
the parser's completed-report and subject-qualified common-state readback, with
the retained cryptographic nonce read independently. A profile permitting a
common-state transition must extend its current-state model before admission.

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

`HtmlFrameNullContent` represents the actual initially null child-content
relationship. Its src/removal and actual load-delay decisions apply only while
that relationship remains null. `frame_post_connection` preserves the exact
browsing-context guard, including an unavailable fact; the consuming live owner
retains child identity, navigation capability and all three mutation hooks.

`HtmlIframeNativeState` represents the actual fresh never-browsing-context,
unprocessed/null-child phase, with initially false current-navigation-lazy and
null timing/resumption values. Its srcdoc/src/sandbox/removal decisions stop at
the actual null-content guards; loading changes use the shared semantic loading
states and return when native resumption is null. `iframe_post_connection`
checks the current node-document browsing context before sandbox parsing or
child creation. Exact absent, present-empty and token-valued sandbox attributes
remain distinct; independent local directive flags are not live sandbox policy.
Current null content alone cannot reinitialize processed/imported history.
Clone-as-new and retained-state installation are distinct causes; adoption
preserves established state. This type grants no numeric sandbox readback,
resource-capable transition, navigation or default-allow policy. Future child or
context creation needs its actual initialization before policy consumption.

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
