# surgeist

Surgeist is a reusable Rust UI framework built from focused crates for CSS,
style, layout, retained state, text, rendering, native windows, dialogs, and
geometry.

This repository is the top-level facade crate and coordination workspace. The
implementation crates live under `crates/` as Git submodules so each boundary
can be developed and reviewed independently while the root crate verifies the
integrated framework shape.

## Crates

- `surgeist-css`
- `surgeist-dialog`
- `surgeist-layout`
- `surgeist-render`
- `surgeist-retained`
- `surgeist-shape`
- `surgeist-style`
- `surgeist-text`
- `surgeist-window`

## Development

Clone with submodules:

```sh
git clone --recurse-submodules https://github.com/bj-data-eng/surgeist.git
```

Update submodules:

```sh
git submodule update --init --recursive
```

Run the integrated workspace check:

```sh
cargo check --workspace
```

## License

Licensed under the MIT license. See [LICENSE](LICENSE).
