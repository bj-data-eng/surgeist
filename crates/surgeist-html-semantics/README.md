# surgeist-html-semantics

Qualified immutable captures and bounded intrinsic HTML semantic queries.
Capture builders retain explicit document, registry, host and external facts;
query sessions preserve typed missing-fact results, foreign-ID rejection,
resource admission and sticky first failure.

This crate depends directly on [HTML values](../surgeist-html/README.md). It has
no parser, CSS, Color, DOM or runtime dependency. The
`HtmlConstructionCaptureAdapter` consumes immutable native or parsed HTML
results through the same public views. Callers supply external facts; the
adapter owns traversal, admission, capture construction and result-to-capture
mapping. A parse report enters through its document or fragment value, without
introducing a parser dependency here.

The [detailed semantic guide](../../docs/reference/html-operations.md#captured-custom-element-author-context)
records query families, qualifications and capability limits. External URL/CSP,
resource, layout and host observations remain supplied facts. Queries do not
execute scripts, load resources or mutate a live tree. The
[selected language edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior.

From the repository root, run focused checks through bounded `pisct run`:

```sh
cargo test --offline --locked -j 1 -p surgeist-html-semantics
cargo clippy --offline --locked -j 1 -p surgeist-html-semantics --all-targets -- -F unsafe-code -D warnings
python3 crates/surgeist-html-semantics/tools/generate_bidi_class.py --check
```

The Unicode first-strong provider is a selected HTML directionality input,
not a full text-layout bidi implementation. Its pinned data and all other
incorporated material retain their [attribution and licenses](NOTICE.md).
