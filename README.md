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

## API Artifact

The committed API coordination artifact lives at `api/public-api.txt`.
Refresh it explicitly when the public API changes:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

API refresh tooling is command-only. Do not wire it into normal `cargo test`
runs.

## License

Licensed under the MIT license. See [LICENSE](LICENSE).
