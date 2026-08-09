# surgeist

Surgeist is a reusable Rust UI framework built from focused crates for
animation, CSS, style, layout, retained state, runtime orchestration, text,
rendering, native windows, dialogs, template contracts, task scheduling, shared
testing, and geometry.

This repository is Surgeist's permanent product-integration facade and
coordination workspace. The implementation crates live under `crates/` as Git
submodules so each boundary can be developed and reviewed independently while
the root crate owns compatible composition and whole-product verification.

The root package is the only member of this repository's Cargo workspace. Its
13 production leaves remain exact path dependencies, and all 14 pinned leaves
remain source inputs for the root-owned public API audits. `surgeist-test` is an
API-audit input rather than a production dependency or facade reexport.

The current reset baseline directly reexports the production crates. It does
not yet implement cross-crate adapters, root integration tests, examples, a
native development harness, or root-owned integration tools. Those integration
surfaces remain root responsibilities when they are designed and implemented.

## Adapter Boundaries

The root `surgeist` crate owns future Surgeist-to-Surgeist adapters: conversions
that compose public models from one Surgeist crate into public inputs for
another. Leaf crates own their domain models, algorithms, and backend-local
adapters, such as rendering, text shaping, window host, or platform integrations
that are implementation details of that crate. Reusable fixture schemas and
harnesses belong in `surgeist-test` so production crates can share test
contracts without depending on the root facade.

## Crates

- `surgeist-animation`
- `surgeist-css`
- `surgeist-dialog`
- `surgeist-layout`
- `surgeist-render`
- `surgeist-retained`
- `surgeist-runtime`
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

Run the root-only Cargo workspace check, which compiles the production path
dependencies:

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
