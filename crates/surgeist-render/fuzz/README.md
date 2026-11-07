# surgeist-render-fuzz

This package owns bounded public headless Render cases. The shared
[Fuzz framework](../../surgeist-fuzz/README.md) supplies engines, process custody,
canonical capture, replay and evidence. Render production code does not depend
on this package. No target creates a GPU device, renderer, surface or window.

## Checked boundaries

| Target | Properties |
| --- | --- |
| `surgeist-render::resource-admission` | Typed font/image/numeric admission; source-qualified Ahem glyph ink and empty glyphs; preparation limits, immutable prior results and retry. |
| `surgeist-render::scene-inspection` | Authored scene structure, exact image-content observations, concrete transform coefficients, public clip normalization and image-mask placement; accepted-prefix refusal and retry. |

`Scene::inspect` reports authored records. These cases do not certify whole-scene
normalization, emitted ordering or coordinates, image-fit execution,
clipped/transformed pixels, or GPU/driver behavior. CSS background percentages,
cover/contain/repetition and box-decoration interpretation belong to the semantic
paint work tracked in [#1081](https://github.com/bj-data-eng/surgeist/issues/1081).
The public concrete clip operation retains geometry plus its transform; it does
not return newly transformed geometry.

Each case has a strict versioned binary encoding, with separate target magic,
exact IEEE float bits, product limits, resource bytes and bounded recipes.
Unknown tags/revisions, trailing bytes, truncation, oversized counts and invalid
fixture references are rejected before allocation. Generation entropy is separate
from a materialized case. Captured canonical resource bytes reproduce the input
without relying on a seed or a fixture name.

Ceilings are 128 KiB per canonical case,32 KiB per font (2 KiB for arbitrary-font
generation),four images of at most16x16 with4 KiB aggregate RGBA bytes,64 commands,
depth8,128 authored clip-path elements,16 glyphs and8 variation coordinates.
Glyph preparation has separately captured producer limits, with a maximum of1024
output path elements. Harness construction
bounds are separate from product work/resource limits and process RSS.

Expected product rejections name the checked properties. A refusal may retain
accepted work in its attempt report; it does not imply zero work or an unchanged
report. Cases check the actual atomicity contract, prior immutable outputs,
released scratch and fresh retry. Unsupported capabilities are exclusions, not
successful coverage of the missing behavior.

Unexpected producer rejection on a validated positive case is a declared
property failure with a stable discriminator. Codec, schema and harness
allocation failures remain separate errors. Test-only controls check a changed
real command-count observation and a real public inspection refusal through
canonical capture, property execution and exact direct replay; neither is a
reported product regression.

## Replay and finite runs

Use the registered descriptor [fuzz.toml](fuzz.toml) and the framework's
[authoring and campaign guide](../../surgeist-fuzz/README.md). Both default worker
binaries serve the framed capture/replay protocol. The optional `property` and
`coverage` features use the same target evaluator. Ordinary builds do not link
the coverage engine.

```rust
use surgeist_fuzz::{CaseTarget, engine};
use surgeist_render_fuzz::SceneTarget;

let target = SceneTarget::new();
let case = target.literal(0).unwrap();
let mut captured = Vec::new();
let observed = engine::evaluate_case(&target, &case, &mut |bytes| {
    captured = bytes.to_vec();
    Ok(())
}).unwrap();
let replayed = engine::replay(&target, &captured, &mut |_| Ok(())).unwrap();
assert_eq!(observed, replayed);
assert!(!observed.outcome.checked_properties().is_empty());
```

Run focused package checks from the repository root under its process policy:

```sh
cargo test --offline --locked -j 1 -p surgeist-render-fuzz
cargo test --offline --locked -j 1 -p surgeist-render-fuzz --features property
cargo test --offline --locked -j 1 -p surgeist-render-fuzz --features coverage
cargo clippy --offline --locked -j 1 -p surgeist-render-fuzz --all-targets --all-features -- -F unsafe-code -D warnings
cargo fmt -p surgeist-render-fuzz -- --check
```

Finite runs report evaluated, generation-rejected, unsupported and failed cases
separately, with exact source/lockfile/executable/provider identity and owned
cleanup. A finite successful run covers only its declared properties and inputs.

The fixed canonical seeds are generated from their source-qualified literal case
factories. Regenerate them through the owning codec, then inspect the resulting
diff; never hand-edit encoded seeds:

```sh
cargo run --offline --locked -j 1 -p surgeist-render-fuzz --example materialize_seeds -- crates/surgeist-render/fuzz/seeds
```

## Provenance

The glyph oracle lane covers the source-qualified X and space glyphs plus explicit
out-of-range requests; other valid glyph contours are excluded from that oracle.
Arbitrary fonts use a separate admission lane.

Literal font cases use the existing licensed
[Ahem fixture](../tests/fixtures/fonts/ahem/PROVENANCE.md), SHA-256
`f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc`.
Its exact bytes are retained in materialized cases; the independent em-square
expectations come from the fixture's source, as in the public
[preparation tests](../tests/prepared_outline.rs). General accepted font bytes
receive structural observations, without an invented glyph-geometry oracle.

Authored case code is covered by the repository [MIT license](../../../LICENSE).
Retained font attribution is owned by the existing
[Render notice](../NOTICE.md) and fixture provenance above.
