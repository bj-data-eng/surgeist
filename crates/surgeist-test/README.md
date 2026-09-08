# surgeist-test

`surgeist-test` is a Rust library for Surgeist test authors who need shared
fixture metadata and cross-crate verification support. Its current implementation
provides layout-ready metadata schemas without dependencies on production crates.
It is test-facing; using it as a production runtime dependency requires an
explicit boundary decision from the top-level coordinator.

## Start

From the crate directory, run the focused metadata construction test:

```sh
cargo test --offline -p surgeist-test fixtures::tests::layout_ready_metadata_constructor_sets_schema_and_keeps_paths_relative -- --exact
```

Expect one passing unit test. It constructs a fixture record and checks its
schema version, paths, provenance, expectation, and ready status. See
[getting started](docs/getting-started.md) for prerequisites and verification.

## Documentation

- [Getting started](docs/getting-started.md): run the first metadata check.
- [How-to](docs/how-to.md): construct and annotate fixture records.
- [Reference](docs/reference.md): public types, defaults, and validation behavior.
- [Explanation](docs/explanation.md): schema ownership and integration boundaries.

## License and attribution

- [Project license](LICENSE)
- [Third-party attribution](NOTICE.md)
