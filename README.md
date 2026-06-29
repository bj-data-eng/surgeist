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

## API Artifacts

Root owns API artifact generation for the facade and linked Surgeist crate
submodules. API artifacts are stored in this repo under `api/`; do not copy
`api/generator` or generated API artifacts into crate repos.

Refresh all API artifacts:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Refresh one target:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --crate surgeist-task
```

Root's facade artifact lives at `api/public-api.txt`. Crate artifacts live at
`api/crates/<crate>.txt`.

Check artifacts without rewriting:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

API refresh tooling is command-only. Do not wire it into normal `cargo test`
runs.

## License

Licensed under the MIT license. See [LICENSE](LICENSE).
