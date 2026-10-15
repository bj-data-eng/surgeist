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
