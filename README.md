# Surgeist

Surgeist is a Rust UI framework for retained, CSS-styled document interfaces.
Its crates provide typed contracts for authoring, state, animation, layout,
text, rendering and host integration in one repository and Cargo workspace.

The facade reexports production crates. `surgeist::dom` exposes the canonical
live tree, native construction, atomic edits and immutable snapshots; consumers
own their direct adapters to the contracts they use. Complete application
integration, examples and a native development harness remain planned.

## Start

With the required Rust toolchain and dependencies already available locally:

```sh
cargo test --offline --locked -j 1 -p surgeist --lib
```

Expect `tests::exposes_crate_identity` to pass. This compiles the default facade
and checks `surgeist::crate_name()`; it does not launch a UI or test every crate.
See [getting started](docs/getting-started.md) for setup and the next step.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Prepare a checkout and verify the library entry point. |
| [How-to](docs/how-to.md) | Select focused checks and maintain API audits. |
| [Reference](docs/reference.md) | Find packages, facade modules, features and source paths. |
| [Explanation](docs/explanation.md) | Understand crate boundaries and integration limits. |

## License And Attribution

Surgeist is licensed under the [MIT License](LICENSE).
[Dependency attribution](NOTICE.md) records source provenance and links the
accompanying upstream license material and crate notices.
