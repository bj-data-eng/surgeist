# surgeist-text

`surgeist-text` is the Rust library for text shaping, measurement, font-facing
abstractions, and text layout in Surgeist surfaces. It turns text, resolved styles,
and layout options into glyph runs, lines, inline boxes, and interaction geometry
through its owned analysis, shaping and line-finalization pipeline. The current package is version `0.1.0`, using Rust edition 2024.

Standalone layout retains raw Preserve and automatic direction. Explicit composed
paragraphs execute source processing and capture their actual base direction;
inline-box alignment requests do not imply CSS vertical-alignment execution.
See the [support reference](docs/reference.md) for the current contract.

`process_source_whitespace` executes source-local whitespace processing before
fonts. `process_contextual_source` additionally consumes captured box, paragraph
and trim facts in original order. Its issued paragraph views retain their styles,
source pieces and occurrences through analysis, preparation and placement;
internal child text remains outside the parent view. Both fronts share whitespace
correspondence and partial plaintext without inserting presentation characters.
See [contextual processing](docs/reference.md#contextual-source-processing).

`Text` retains exact UTF-16 code units, and public source positions and ranges
count those units. Edits can split or join surrogate pairs. `AnalyzedSource`
retains original source and pre-font semantic analysis. Contextual casing,
full-width mapping and the complete pinned full-size-kana transform execute in
that order after Phase I and before analysis, shaping and wrapping. Use
`TextTransform::Set` combines operations built with `TextTransformSet`:
`.with_case(CaseTransform::Uppercase)`, `.with_full_width(true)` and
`.with_full_size_kana(true)`; lowercase and capitalize are also supported.
Full-width uses the raw Unicode wide/narrow relation and
maps SPACE only when the executed processing preserved it.
Original UTF-16 and plaintext remain unchanged through expansions and removals.
The exclusive `TextTransform::MathAuto` requires complete original Text-node
content captured on native paragraph origins. It maps eligible surviving text
through the pinned MathML Core relation; raw standalone layout refuses the
missing context. See [transformation inputs](docs/reference.md#defaults-and-validation).
Layout uses a separate glyph projection
for isolated units. Source-only accessibility needs no placement and
omits unsupported geometry. Positioned accessibility, interaction and paint
readback share the committed layout. Shaping boundaries inside an original pair
still return a precise error.

`System` discovers system fonts by default or uses explicitly registered font
bytes. Registration and refresh publish a new issuer-qualified catalogue identity
and invalidate cached layouts atomically; retained analyses and layouts keep their
actual source and selected resources. `System::layout_analyzed` enriches the same
immutable analysis, while `System::layout` performs the same phases as a convenience.
See [font registration](docs/how-to.md#register-fonts-without-system-discovery).
`System::prepare` retains the same analysis and actual resources. Its issued
handles admit a complete `LineSelectionRequest`; selected spacing allocation and
finalization produce immutable line-local output. `Layout::from_finalized_lines`
validates one explicitly anchored line set without wrapping again. See the
[composed protocol guide](docs/composed-lines.md).

Automatic hyphenation has bounded en-US support through the pinned
[resource and exceptions](docs/hyphenation-resources.md). Prepared word and cut
views retain actual source custody; selected indication text remains presentation
and does not enter plaintext. LogicalAfter continuations pair exact selected
choices across lines. BothSides emits the same chosen text once on each side of
one ending, with the same exact continuation identity. Before and After use their
own authenticated local bidi attachments and fresh joint shaping without changing
original paragraph resolution. Local order or retained font-query evidence that
cannot support the combined presentation returns a precise refusal. Independently
chosen strings for each side and a language-convention table are not provided.

`System::registered_font_faces` lends actual registered resources without shaping
a sample character or resolving a Style. Retained `FontData` can select an explicit
unsynthesized instance and query same-instance f64 font/strikeout metrics under
its captured catalogue publication. Qualified MVAR uses exact canonical coordinates,
actual selected field tags and checked integer delta accumulation; malformed
variation data refuses even at default/inactive coordinates. Actual run instances
retain canonical coordinates, executable x-shear and the selected translated-outline
bold displacement; line metrics and advances receive no bold-spacing adjustment.

`EffectiveFontInstance::glyph_outline` reads conservative same-instance outline
bounds, including synthesis, or proven empty geometry. It qualifies simple and
dense-variable TrueType outlines and static translation-only composites; other
outline formats refuse explicitly. These bounds do not select color or bitmap paint.

Borrowed run, face and effective-instance views expose actual positioned output
without copying glyph arrays. Logical origins and pens are `f64`; raw canonical
glyph offsets/advances and font sizes remain `f32`. The optional standalone paint adapter belongs to
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
coverage are recorded in [NOTICE.md](NOTICE.md) and
[the owned-engine source provenance](docs/owned-engine-provenance.md).
