# Surgeist

Surgeist is a Rust UI framework for developers building retained, CSS-styled
document interfaces. Its crates provide typed contracts for authoring, state,
animation, layout, text, rendering, and host integration. One repository and
Cargo workspace contain the `surgeist` facade, its domain crates, test support,
and shared corpus-generation tooling.

The current facade directly reexports the production crates. Cross-crate
adapters, root integration tests, examples, and a native development harness
remain unimplemented; consolidating the source provides a common development
boundary for that integration work.

## Start

From a checkout with the required Rust toolchain and dependencies already
available locally:

```sh
cargo test --offline --locked -j 1 -p surgeist --lib -- --test-threads=1
```

Expect `tests::exposes_crate_identity` to pass. This compiles the facade and its
default production dependencies and checks that `surgeist::crate_name()` returns
`"surgeist"`. It does not launch a UI or run other crates' test suites.
See [getting started](docs/getting-started.md) for checkout and setup details.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Prepare a checkout and verify the library entry point. |
| [How-to](docs/how-to.md) | Select focused checks and maintain API audits. |
| [Reference](docs/reference.md) | Find packages, facade modules, features, and source paths. |
| [Explanation](docs/explanation.md) | Understand crate boundaries, source consolidation, and integration limits. |

## License And Attribution

Surgeist is licensed under the [MIT License](LICENSE).
[Dependency attribution](NOTICE.md) describes the source-checkout coverage and
links the accompanying upstream license material and crate notices.
