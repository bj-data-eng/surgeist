# surgeist

Surgeist is a reusable Rust UI framework built from focused crates for CSS,
style, layout, retained state, text, rendering, native windows, dialogs,
template contracts, task scheduling, shared testing, and geometry.

This repository is the top-level facade crate and coordination workspace. The
implementation crates live under `crates/` as Git submodules so each boundary
can be developed and reviewed independently while the root crate verifies the
integrated framework shape.

## Adapter Boundaries

The root `surgeist` crate owns Surgeist-to-Surgeist adapters: conversions that
compose public models from one Surgeist crate into public inputs for another.
Leaf crates own their domain models, algorithms, and backend-local adapters,
such as rendering, text shaping, window host, or platform integrations that are
implementation details of that crate.

Adapter-composed fixture generation belongs in root-owned tools, where root
adapters can prepare integrated fixture metadata. Reusable fixture schemas and
harnesses belong in `surgeist-test` so production crates can share test
contracts without depending on the root facade.

## Crates

- `surgeist-css`
- `surgeist-dialog`
- `surgeist-layout`
- `surgeist-render`
- `surgeist-retained`
- `surgeist-shape`
- `surgeist-style`
- `surgeist-task`
- `surgeist-template`
- `surgeist-test`
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
