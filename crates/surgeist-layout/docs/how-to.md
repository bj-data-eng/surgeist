# How-to

These procedures assume you have completed [getting started](getting-started.md).
They apply to the `surgeist-layout` workspace crate. The [manifest](../Cargo.toml),
[public crate root](../src/lib.rs), and [Justfile](../Justfile) define the APIs,
features, and commands used here.

## Use higher coordinate precision

For an application that needs `f64`, use the scalar-generic `*Of<S>` forms from
the [API map](reference.md#public-api-map) in place of the default aliases.
Choose one scalar for the complete tree, cache, traversal, and layout run; keep
all values in that lane. The default aliases remain `f32`.

To check an existing `f64` leaf case, run:

```sh
cargo test --locked --offline -p surgeist-layout --lib leaf_tests::leaf_uses_containing_flow_for_percentage_edges_in_f64 -- --exact
```

Expected: one passing test. The [test](../src/leaf_tests.rs) checks content-space
measurement constraints after resolving percentage edges in the containing flow.
Browser XML exercises the default `f32` boundary; use separate crate-local tests
for behavior that needs `f64` evidence.

## Verify the default library configuration

With `just`, Rustfmt, Clippy, and the default dependency cache already available,
run:

```sh
CARGO_NET_OFFLINE=true just verify
```

The [verification script](../scripts/run-verification.sh) runs formatting,
check, test, and strict lint checks. Success is a zero exit after all four stages;
a missing prerequisite or a failed stage leaves verification incomplete.

## Verify the optional generator configuration

With the generator feature's pinned dependencies available locally and Node.js
on `PATH` for the [helper protocol tests](../tests/bin/surgeist-layout-generate/helper_protocol_tests.rs),
run:

```sh
CARGO_NET_OFFLINE=true just verify-generator
```

Expected: all-targets compilation, tests, and strict Clippy pass with
`layout-golden-generate`. This verifies the optional tooling configuration; it
does not execute a browser generation run.

## Validate the checked-in corpus provenance

With `just` and the generator feature dependencies available, run:

```sh
CARGO_NET_OFFLINE=true just corpus-check
```

Expected: `check-corpus` exits successfully after validating persisted source
attestation, hashes, paths, inventory, and accounting. It requires no browser or
source checkout. The recipe clears corpus and browser overrides so the check
uses the checked-in corpus. See the [runtime reference](reference.md#browser-parity-runtime)
for the schema and ownership boundary.

## Compare all checked-in browser expectations

With the default test dependencies available, run:

```sh
CARGO_NET_OFFLINE=true just parity-all
```

This explicitly selects the otherwise ignored full XML parity test and clears
`SURGEIST_PARITY_FILTER`. The expected successful result is the selected test
passing after comparing the full checked-in corpus. Inspect any reported
geometry mismatches in the test output. The command uses existing XML and does
not launch a browser.

## Work with browser generation or the source cache

Use the [browser parity guide](../tests/layout/browser_parity/README.md) when
your goal is to generate XML or maintain imported sources. Its command and cache
sections describe `generate`, `generate-existing`, `import-taffy`, and
`check-taffy-corpus`, along with their required paths and verification behavior.
The [runtime reference](reference.md#browser-parity-runtime) distinguishes
managed acquisition from existing-cache operations, full from filtered
publication, and provenance checks from semantic comparisons.
