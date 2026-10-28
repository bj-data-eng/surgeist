# surgeist-shaper

Safe Rust OpenType and AAT shaping, maintained as a Surgeist workspace crate.
The implementation derives from Harfrust 0.6.2; [provenance](PROVENANCE.md), the
[original README](upstream/README.md) and [notice](NOTICE.md) retain its origins.
This crate is distinct from `surgeist-shape`, which owns geometry.

## Use

The public entry point is [src/lib.rs](src/lib.rs). Build `ShaperData` and a
`Shaper` from a `FontRef`, put Unicode input and its direction/script into a
`UnicodeBuffer`, then shape it. Check the actual returned
`GlyphBuffer::shaping_status()` before accepting glyph output: a refused result
can still contain plausible glyphs. Consumers scale ordinary output from design
units and retain responsibility for fonts, paragraph interpretation and layout.

Shaper owns substitution, positioning, completion/refusal reporting, original
joining correspondence, closed ordinary-symbol presentation, original
default-ignorable custody and supplied vertical positioning. The provenance
explains each extension's admission, typed errors and logical resource limits.
The supplied vertical operation decodes its internal half-unit domain to exact
f64 design-unit positions; ordinary `GlyphPosition` units stay unchanged.

## Features And Verification

Default `std` enables `read-fonts/std`. `--no-default-features` uses `no_std` with
`alloc` and the existing Read Fonts `libm` path. Edition 2021, MSRV 1.85 and the
production dependency requirements are retained from the maintained provider.
All compiled targets forbid executable unsafe Rust.

Run from the repository root with already available dependencies:

```sh
cargo test --offline --locked -j 1 -p surgeist-shaper
cargo check --offline --locked -j 1 -p surgeist-shaper --no-default-features
cargo test --offline --locked -j 1 -p surgeist-shaper --no-default-features
cargo clippy --offline --locked -j 1 -p surgeist-shaper --all-targets -- -F unsafe-code -D warnings
cargo doc --offline --locked -j 1 -p surgeist-shaper --no-deps
cargo fmt -p surgeist-shaper -- --check
```

The inherited lint settings remain in [Cargo.toml](Cargo.toml) and source;
retaining them does not establish a newly classified Clippy baseline. The inherited
`clippy::cargo` group checks metadata for the owning workspace, so strict provider
Clippy also reports missing keywords/categories in other workspace packages. Public API
audits are generated and checked by the [root audit owner](../../docs/how-to.md#check-public-api-audits).

The adjacent handwritten tests qualify maintained extensions using the existing
[Text Ahem fixture](../surgeist-text/tests/fixtures/fonts/ahem/PROVENANCE.md).
They do not establish comprehensive shaping coverage. Generated tables,
machines and Unicode companion data retain their original identities and
notices; source and generation commands are recorded in their headers.
