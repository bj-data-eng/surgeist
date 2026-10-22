# surgeist-text

`surgeist-text` is the Rust library for text shaping, measurement, font-facing
abstractions, and text layout in Surgeist surfaces. It turns text, resolved styles,
and layout options into glyph runs, lines, inline boxes, and interaction geometry
through Parley. The current package is version `0.1.0`, using Rust edition 2024.

The API has explicit limits: whitespace collapse and explicit base direction are
unsupported, and inline-box alignment requests do not imply backend placement.
See the [support reference](docs/reference.md) for the current contract.

`process_source_whitespace` executes source-local Discard, Preserve and
PreserveSpaces before fonts, retaining exact original source, document delimiter
facts and UTF-16 correspondence. Explicit affinity projects contraction interiors
and transparent box anchors. Its partial plaintext iterator returns same-result
slices and real box occurrences, without document serialization or opportunity
analysis. Paragraph layout does not yet consume this phase: Discard and
PreserveSpaces still refuse there; standalone Preserve layout remains unchanged.

`Text` retains exact UTF-16 code units, and public source positions and ranges
count those units. Edits can split or join surrogate pairs. Layout currently
accepts scalar-representable source through a lossless private UTF-8 projection;
isolated surrogates and shaping boundaries inside pairs return precise errors
before font/cache work. Interaction and optional accessibility map back to the
same source units. This interim boundary does not provide isolated-surrogate
shaping or the future prepared-text pipeline.

`System` can discover system fonts (the default) or use only explicitly
registered font bytes. Registration and refresh publish a new issuer-qualified
catalogue identity and invalidate current cached layouts atomically; retained
layouts keep their selected font resources. See [font registration](docs/how-to.md#register-fonts-without-system-discovery).
This catalogue foundation retains the current Parley engine; prepared text and
layout-selected line finalization remain separate unfinished work.

`System::registered_font_faces` lends actual registered resources without shaping
a sample character or resolving a Style. Retained `FontData` can select an explicit
unsynthesized instance and query static, same-instance f64 font/strikeout metrics
under its captured catalogue publication. Variable/MVAR metrics and unresolved
synthesis remain unsupported; this does not change current paragraph metrics.

Borrowed run, face and effective-instance views expose actual positioned output
without copying glyph arrays. The optional standalone paint adapter belongs to
[Render](../surgeist-render/README.md) under its `render-text` feature; Text does
not depend on Render.

## Start

From the crate directory, run the focused layout test:

```sh
cargo test --offline -p surgeist-text --lib tests::builds_plain_text_layout -- --exact
```

Expected result: one passing test that shapes `hello world` into one line with
nonempty glyph runs. See [Getting started](docs/getting-started.md) for prerequisites
and the corresponding library call.

## Documentation

- [Getting started](docs/getting-started.md): verify the checkout and build a first layout.
- [How-to](docs/how-to.md): compose styled text, inspect geometry, and apply edits.
- [Reference](docs/reference.md): public interfaces, defaults, features, and support limits.
- [Explanation](docs/explanation.md): validation, caching, source identity, and ownership.

## License and attribution

The project uses the [MIT license](LICENSE). Third-party attribution and its
coverage are recorded in [NOTICE.md](NOTICE.md).
