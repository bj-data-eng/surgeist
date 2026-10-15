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
