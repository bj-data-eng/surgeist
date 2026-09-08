# Construct and annotate fixture records

These procedures assume the [first metadata test](getting-started.md) passes.
The Rust example belongs in a test target that already has access to
`surgeist_test`; it does not configure a consuming crate's dependencies.

## Construct a layout-ready record

Choose a fixture identity, paths relative to the fixture corpus, and a generator
label. The paths below are illustrative metadata values; constructing a record
does not require those files to exist.

```rust
use surgeist_test::fixtures::{
    ArtifactPaths, FixtureIdentity, FixturePath, GeneratorProvenance,
    LayoutReadyFixtureMetadata, SchemaError, SchemaVersion,
};

fn fixture_metadata() -> Result<LayoutReadyFixtureMetadata, SchemaError> {
    let metadata = LayoutReadyFixtureMetadata::new(
        FixtureIdentity::new("block_basic")?,
        ArtifactPaths::new(
            FixturePath::new("html/block/basic.html")?,
            FixturePath::new("xml/block/basic.xml")?,
        ),
        GeneratorProvenance::new("root-layout-fixture-generator")?,
    );

    assert_eq!(metadata.schema_version(), SchemaVersion::current());
    assert_eq!(metadata.artifacts().source().as_str(), "html/block/basic.html");
    assert!(metadata.status().is_ready());
    assert!(metadata.expectation().is_none());
    Ok(metadata)
}
```

Call `fixture_metadata()` from the consuming test and propagate or inspect its
result. The assertions verify the selected source path and constructor defaults.
The constructors return `SchemaError` for invalid text or paths; inspect
`field()` and the displayed error to identify the rejected input.

## Record an adapter or expectation

When an existing fixture was produced through an adapter, call
`GeneratorProvenance::with_adapter` before passing its provenance into
`LayoutReadyFixtureMetadata::new`. Supply a nonblank adapter label and propagate
the returned `Result`. Verify it through `metadata.provenance().adapter()`.

When a fixture has a named expected outcome, create a `FixtureExpectation` with
`FixtureExpectation::new`, then retain the record returned by
`metadata.with_expectation`. Verify the label through `metadata.expectation()`
and `FixtureExpectation::as_str`.
The [construction test](../src/fixtures/mod.rs) demonstrates both annotations.
These labels record information; they do not execute an adapter or an assertion.

## Explain a fixture that is not ready

For a record that should be ignored or is known to fail, construct
`FixtureStatus::ignored(reason)` or `FixtureStatus::known_failure(reason)` and
pass the successful result to `metadata.with_status`, retaining the returned
record. A blank reason is rejected. Verify the resulting
`metadata.status().kind()` and `reason()`; `is_ready()` is false for either status.
To mark a record ready, use `FixtureStatus::ready()`; its reason is absent.

See the [reference](reference.md) for the complete type inventory and validation
limits, and the [explanation](explanation.md) for fixture ownership.
