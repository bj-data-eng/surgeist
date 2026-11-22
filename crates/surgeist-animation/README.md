# surgeist-animation

CSS animation and transition timing contracts for Surgeist integrators. This
Rust library samples normalized tracks into typed property values, completion
states, and next-frame hints.

Version 0.1.0 supports snapshot-based CSS animation reconciliation and transition
decisions, typed native keyframe normalization and computed interval sampling,
timing, easing, computed numeric composition, background/mask position and size
layers, repeatable lists, immutable effect-stack folds and replacement decisions,
logarithmic ratio interpolation, normalized positions, standard-space colors,
computed basic shapes and native shape/path command lists, resolved 2D/3D
transforms, box/text shadow lists, computed clip-path branches, and discrete
switching. Basic shapes retain explicit reference boxes and use whole-value
fallback for incompatible pairs.
Contextual calculations retain
source identity in symbolic combination plans. Transform lists support identity
padding, common primitives, suffix matrix interpolation, list addition and
accumulation. Shadow lists preserve inset matching, transparent padding,
ordered addition and component-based accumulation of supplied computed values.
Clip paths retain opaque supplied sources, geometric boxes and complete shape
payloads. Compatible shapes use the shared kernel; other branches switch as a
whole. Masking's discrete property policy and legacy clip rectangles can use
the existing discrete and computed-component families. Shared layer sampling
uses repeatable-list pairing for background and mask positions and sizes.
Filter lists use corresponding identity padding and ordered addition; native
filter images specialize matching used sources or execute supplied generic
image interpolation.
Grid values retain track/function kinds, fixed and automatic repeats, subgrid
name lists and symbolic percentages. Compatible tracks combine numerically;
automatic repeats and incompatible structures select complete values. Native
Grid addition and accumulation share the same component traversal.
Multi-column sampling accepts computed counts, widths, gaps, rule colors and
discrete longhands. Column widths support intrinsic keywords, functional
fit-content and bounded symbolic calc-size interpolation.
[Multi-column sampling](docs/reference.md#computed-multi-column-properties)
describes the supplied operands and consumer responsibilities.
Resolved color pairs use a supplied interpolation method and the Color crate's
finite-progress sampler. The result retains missing components and extended
alpha. [Color sampling](docs/reference.md#standard-space-color-sampling)
describes the endpoint and result types.
Explicit transform/composite diagnostic markers retain typed
unsupported outcomes. CSS lowering, contextual evaluation, final
clamping, runtime scheduling, and root integration belong to their respective
owners. Shape addition and accumulation remain outside the current capability.

## Start

From this checkout, run the public transition sampling test:

```sh
cargo test --offline -p surgeist-animation --test public_api delayed_transition_sample_uses_public_modules -- --exact
```

Expected result: one passing test that checks a delayed transition before its
active interval: an initial fill value and a `MayChange` next-frame hint.
See [Getting started](docs/getting-started.md) for prerequisites and a complete
transition example.

## Documentation

- [Getting started](docs/getting-started.md): sample your first transition.
- [How-to guides](docs/how-to.md): sample keyframes and verify crate-local work.
- [Reference](docs/reference.md): public contracts and capability coverage.
- [Explanation](docs/explanation.md): ownership, normalization, and integration boundaries.

## License and attribution

The project is [MIT licensed](LICENSE). [Third-party attribution](NOTICE.md)
describes the source inventory covered by this repository's notice.
