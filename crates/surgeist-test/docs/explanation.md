# Fixture schemas and integration boundaries

`surgeist-test` owns reusable test schemas, harnesses, fixtures, and integration
verification support. Its current source implements layout-ready fixture
metadata. A record describes a fixture's identity, source and generated artifact
paths, generator provenance, optional expectation, and readiness status.

## Independent schemas

Schema types remain independent of implementation crates such as
`surgeist-layout`. This lets a fixture describe layout-ready metadata without
importing layout internals. The [manifest](../Cargo.toml) currently declares no
dependencies, and the [public schema](../src/fixtures/layout_ready.rs) uses
standard-library types.

The crate is test-facing. It should not become a runtime dependency of production
Surgeist crates unless the top-level coordinator explicitly approves a boundary
change. That boundary is stated in the [public front door](../src/lib.rs) and the
[repository guide](../AGENTS.md).

## Composition belongs to the root repository

The root `surgeist` repository owns the public facade, cross-crate adapters,
integration tests and tools, workspace wiring, and the API generator and its
generated audit artifacts. This workspace crate owns its focused source, tests,
and documentation; it carries no copies of the root API audit artifacts.

When a fixture needs multiple production crates, root is responsible for
generating adapter-composed metadata. The generated metadata should depend on
stable schema types from this crate, not on root adapters at runtime.

`surgeist-layout` consumes layout-ready metadata as a test-facing source of
fixture records and harness support. It should not use this crate as a route to
root adapters or sibling private module paths. The
[repository guide](../AGENTS.md) is the ownership discovery entry point.

## Metadata has a limited role

Artifact paths describe locations relative to a fixture corpus; the record does
not choose that corpus's filesystem root. Path validation examines the host
platform's path components and performs no filesystem access. Consumers still
own artifact lookup and any filesystem guarantees their harness requires.

Generator, adapter, and expectation values are descriptive labels. They do not
resolve implementations or run checks. Likewise, readiness status records
whether a fixture is ready, ignored, or known to fail, with a reason required for
the latter two states. The consuming harness decides how that information affects
test execution.

See the [reference](reference.md) for exact defaults and validation behavior.
