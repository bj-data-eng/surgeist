# How-to

These procedures assume the [first layout test](getting-started.md) succeeds and
the calling Rust target already depends on `surgeist-text` and `surgeist-font`.
The composition and edit examples use the registered `system` and the named
`base_style` from [font registration](#register-application-fonts).

## Compose text with styled spans

Use the [composer](../src/composer.rs) when constructing text incrementally so
that ranges follow the UTF-16 code units you append:

```rust
use surgeist_text::{Options, Style, source};

let source = source(|text| {
    text.push("Hello ");
    text.with(Style { size: 20.0, ..base_style.clone() }, |text| {
        text.push("world");
    });
});
assert_eq!(source.text().code_units(), "Hello world".encode_utf16().collect::<Vec<_>>());
assert_eq!(source.spans()[0].range(), surgeist_text::Range::new(6, 11));

let layout = system
    .layout(source, base_style.clone(), Options::default())
    .expect("composed text should build");
assert!(!layout.glyph_runs().is_empty());
```

`with` styles only the text appended inside its closure. For manual ranges, use
`SourceRange::try_new` or `Composer::try_span` to check boundaries against the
current source. A span holds a complete resolved `Style`; overlapping spans are
applied in declaration order, with later spans taking precedence. The composer
places an outer `with` span before its nested spans.

Verify the authored text and span ranges through `Layout::source()`, and inspect
the resulting styles through `Layout::glyph_runs()`. The composer and overlapping
span tests in [src/tests.rs](../src/tests.rs) demonstrate both contracts.

## Query interaction and inline-box geometry

With a built `Layout`, use `hit(Point)` to distinguish text cursors, inline boxes,
and empty space. Use `cursor(Cursor)` for caret geometry, `selection(Selection)`
for selection rectangles, and `move_cursor` or `move_selection` for supported
cluster, word, line, and document movements. These offset-taking methods return
`Result`; source pair interiors project by affinity, while invalid bounds refuse.

For inline content, create an `InlineBox` at a UTF-16 scalar boundary, assign an ID
unique within the source, and supply finite, nonnegative dimensions. Add it with
`Source::inline_box`, `Builder::inline_box`, or `Composer::box_` at the current text
end. Choose `InFlow` when the box should participate in flow, or `OutOfFlow` for a
positioned anchor that does not contribute the same flow metrics.

Verify the box's ID, source index, line, and rectangle with `inline_boxes()`.
Use `inline_metric_facts()` when an adapter also needs requested alignment and
baseline-shift facts. These requests are retained separately from the actual
rectangle; consult the [alignment limits](reference.md) before interpreting them.
Geometry and movement behavior is implemented in [src/layout.rs](../src/layout.rs)
and exercised by the inline-box, selection, hit, and movement tests in
[src/tests.rs](../src/tests.rs).

## Apply an edit and lay out the new source

Use the fallible edit path for ranges that can come from callers:

```rust
use surgeist_text::{Edit, Options, Source};
let layout = system
    .layout(Source::new("hello"), base_style.clone(), Options::default())
    .expect("initial layout should build");
let edited = layout
    .apply(Edit::Insert { index: 2, text: "y".into() })
    .expect("edit boundary should be valid");

assert_eq!(edited.text().code_units(), "heyllo".encode_utf16().collect::<Vec<_>>());
assert_eq!(edited.revision(), 1);
assert_eq!(layout.source().text().code_units(), "hello".encode_utf16().collect::<Vec<_>>());

let updated = system
    .layout(edited, base_style.clone(), Options::default())
    .expect("edited source should build");
assert_eq!(updated.source().text().code_units(), "heyllo".encode_utf16().collect::<Vec<_>>());
```

The edit returns a new `Source`; it does not mutate or reflow the existing layout.
Supply the desired default style and options again when rebuilding. For edits
before layout, use `TextEdit::insert`, `replace`, or `delete`, then `apply_to`.
See [the edit implementation](../src/edit.rs) for span and inline-anchor projection.

## Inspect cache reuse

Reuse one `System` and submit identical source, styles, and options. Compare
`Layout::key()` and the `layout_hits()` / `layout_misses()` counters returned by
`System::stats()`. The `repeated_layout_uses_cache` test in
[src/tests.rs](../src/tests.rs) demonstrates a repeat hit.

`System::refresh_fonts()` replays the admitted application resources and publishes
a new catalogue identity. A successful publication clears current cached layouts,
so the next build is a miss. Old layouts retain their original keys and selected
faces. Registration has the same cache behavior. See [src/system.rs](../src/system.rs).

## Register application fonts

A new `System::default()` has an empty catalogue. Open an explicit font file,
admit it with caller-chosen limits, and name its real family in the style.

```rust
use surgeist_font::{DecodeLimits, FontResource};
use surgeist_text::{Font, Options, Source, Style, System};

let limits = DecodeLimits::new(1 << 20, 1 << 20, 1 << 20, 64, 4, 1 << 20, 3 << 20);
let file = std::fs::File::open("tests/fixtures/fonts/ahem/Ahem.ttf")?;
let font = FontResource::read_from(file, limits)?;
let mut system = System::default();
system.register_font_resource(font)?;
let base_style = Style { font: Font::new().family("Ahem"), size: 20.0, ..Style::default() };
let layout = system.layout(Source::new("AX"), base_style.clone(), Options::default())?;
assert!(!layout.glyph_runs().is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

If the application already holds encoded bytes, use
`system.register_fonts(bytes, limits)`. To transfer a populated Font catalogue
into Text, use `System::new(catalogue)`. Every face must pass admission before the
system publishes the registration. Repeated registrations append in request
order. Supply an ordered list of explicit family names when the text needs
fallback choices. The bundled Ahem fixture is for deterministic tests.
See [the fixture provenance](../tests/fixtures/fonts/ahem/PROVENANCE.md)
and [Font admission](../../surgeist-font/README.md).

## Read metrics without shaping text

Use the registered `system` from the preceding example. Retain its actual face
and publication before reading metrics.

```rust
let face = system.registered_font_faces().next().expect("registered face");
let basis = face.catalogue_identity().clone();
let metrics = face.unsynthesized_instance(20.0, &[])?.metrics(&basis)?;
assert_eq!(metrics.ascent(), 16.0);
assert_eq!(metrics.descent(), -4.0);
# Ok::<(), Box<dyn std::error::Error>>(())
```

This static unsynthesized Ahem example reads the registered resource's own
metrics. Keep the original publication with historical resources; substituting
a new System publication after refresh refuses. Consult the [metric domain and
strike normalization](reference.md#glyph-free-associated-font-metrics) before
using these values for placement.
