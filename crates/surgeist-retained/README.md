# surgeist-retained

`surgeist-retained` is the Rust library for Surgeist's retained semantic UI model.
It gives integration code stable node handles, canonical trees, retained state,
projected traversal, and reports of mutations and routed commands.

The [manifest](Cargo.toml) currently declares version `0.1.0`. This crate owns the
retained model; layout, style, rendering, platform input, widget implementations,
and application command execution belong outside its
[public boundary](src/lib.rs).

## Start

From a checkout, run the smallest test of inserting a node and reading it through
a snapshot:

```sh
cargo test --offline -p surgeist-retained tests::canonical_patch_inserts_and_snapshots_children -- --exact
```

The named test should pass. It verifies that the insertion report and snapshot
identify the same child and preserve its `main` key. See
[getting started](docs/getting-started.md) for prerequisites and the complete path.

## Documentation

- [Getting started](docs/getting-started.md): establish a working local checkout.
- [How-to guides](docs/how-to.md): mutate trees, resolve projections, and dispatch events.
- [Reference](docs/reference.md): public interfaces, source locations, and local checks.
- [Explanation](docs/explanation.md): identity, topology, state, and ownership boundaries.

## Stewardship

- [MIT license](LICENSE)
- [Third-party attribution](NOTICE.md)
