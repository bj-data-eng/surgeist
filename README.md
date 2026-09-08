# Surgeist

Surgeist is a Rust UI framework for developers building retained, CSS-styled
document interfaces. Its independent crates provide typed contracts for
authoring, state, animation, layout, text, rendering, and host integration. This
repository composes their public APIs through the `surgeist` library and pins
their source revisions as Git submodules.

The current baseline directly reexports the production crates. Cross-crate
adapters, root integration tests, examples, and a native development harness are
not implemented in this baseline; root retains ownership of those integration
surfaces.

## Start

From an initialized checkout with the required Rust toolchain and dependencies
already available locally:

```sh
cargo test --offline -p surgeist --lib
```

Expect `tests::exposes_crate_identity` to pass. This compiles the facade and its
default production dependencies and checks that `surgeist::crate_name()` returns
`"surgeist"`. It does not launch a UI or run the independent leaf test suites.
See [getting started](docs/getting-started.md) for checkout and setup details.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Prepare a checkout and verify the library entry point. |
| [How-to](docs/how-to.md) | Check features, inspect leaf crates, and maintain API audits. |
| [Reference](docs/reference.md) | Find facade modules, feature forwards, and authoritative source paths. |
| [Explanation](docs/explanation.md) | Understand repository ownership and the reset baseline. |

## License And Attribution

Surgeist is licensed under the [MIT License](LICENSE).
[Dependency attribution](NOTICE.md) describes the source-checkout coverage and
links the accompanying upstream license material.
