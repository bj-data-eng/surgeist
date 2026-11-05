# Getting started

Build a layout for `hello world` and verify that it contains one line and shaped
glyph runs. The executable proof is `builds_plain_text_layout` in
[the crate's tests](../src/tests.rs).

## Prerequisites

- A local Surgeist checkout and Rust/Cargo with edition-2024 support.
- The dependencies declared in [Cargo.toml](../Cargo.toml) already available in
  the local Cargo cache. The command below uses offline mode and will fail if
  required packages are absent.
- The bundled [Ahem fixture](../tests/fixtures/fonts/ahem/PROVENANCE.md).
  The test registers this file and names its family explicitly.

The manifest does not declare a `rust-version` minimum. Its edition is not a
separate compatibility promise.

## Verify the layout path

1. Open a terminal at the crate directory.
2. Run the existing test with default features:

   ```sh
   cargo test --offline --locked -j 1 -p surgeist-text --lib tests::builds_plain_text_layout -- --exact
   ```

3. Confirm the result reports one passing test. The test constructs a `System`,
   supplies a width of `240.0`, and asserts one line and nonempty glyph runs.
   A dependency-resolution error occurs before this proof runs; it is not a
   layout result.

## The library call

The [public entry point](../src/lib.rs) exports the crate as `surgeist_text`.
The same operation in a Rust target that depends on `surgeist-text` and
`surgeist-font` starts with an explicit font file. Run this example from the Text
crate directory.

```rust
use surgeist_font::{DecodeLimits, FontResource};
use surgeist_text::{Font, Options, Style, System};

let limits = DecodeLimits::new(1 << 20, 1 << 20, 1 << 20, 64, 4, 1 << 20, 3 << 20);
let file = std::fs::File::open("tests/fixtures/fonts/ahem/Ahem.ttf")?;
let font = FontResource::read_from(file, limits)?;
let mut system = System::default();
system.register_font_resource(font)?;

let mut builder = system.builder("hello world");
builder.default_style(Style { font: Font::new().family("Ahem"), ..Style::default() });
builder.options(Options { width: Some(240.0), ..Options::default() });
let layout = builder.build()?;

assert_eq!(layout.metrics().line_count(), 1);
assert!(!layout.glyph_runs().is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

`build()` validates inputs before shaping or looking up a cached layout. Keep the
`System` when creating subsequent layouts to reuse its resource/plan scratch and cache. To analyze without fonts, construct
`AnalyzedSource::new(source, style, AnalysisTypography::HorizontalComposition)` and retain it; a later
`System::layout_analyzed(&analyzed, options)` uses that same analysis identity.
Continue with [How-to](how-to.md) to add styles, inspect geometry, or edit text.
