# surgeist-animation

CSS animation and transition timing contracts for Surgeist integrators. This
Rust library samples normalized tracks into typed property values, completion
states, and next-frame hints.

Version 0.1.0 supports timing, easing, numeric and percentage interpolation,
premultiplied color interpolation, and discrete switching. Transform and
composite interpolation return typed unsupported outcomes. CSS lowering,
runtime scheduling, and root integration belong to their respective owners.

## Start

From this checkout, run the public transition sampling test:

```sh
cargo test --offline -p surgeist-animation --test public_api delayed_transition_sample_uses_public_reexports -- --exact
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
