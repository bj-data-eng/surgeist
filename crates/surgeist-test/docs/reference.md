# Fixture metadata reference

## Package and public entry points

[Cargo.toml](../Cargo.toml) declares package `surgeist-test` version `0.1.0`,
Rust edition 2024, and library target `surgeist_test`. It declares no dependencies
or feature table. [src/lib.rs](../src/lib.rs) forbids unsafe code and exposes
`fixtures`.

The types below are available through both `surgeist_test::fixtures` and
`surgeist_test::fixtures::layout_ready`. The
[implementation](../src/fixtures/layout_ready.rs) owns their current definitions.

## Types and defaults

| Type or constant | Construction and meaning |
| --- | --- |
| `LayoutReadyFixtureMetadata` | `new(identity, artifacts, provenance)` uses the current schema, no expectation, and ready status. `with_expectation` and `with_status` consume and return the record. Accessors expose each field by reference, except the copied schema version. |
| `SchemaVersion` | A `u16` wrapper with `new`, `current`, and `get`. `new` accepts any `u16`; it does not check whether a version is supported. |
| `CURRENT_SCHEMA_VERSION` | `SchemaVersion::new(1)`. |
| `FixtureIdentity` | `new` validates an identity string; `as_str` borrows it. |
| `FixturePath` | `new` validates a path string; `as_str` borrows the retained string. |
| `ArtifactPaths` | `new(source, generated)` pairs two validated `FixturePath` values. `source` and `generated` borrow those paths. |
| `GeneratorProvenance` | `new(generator)` starts without an adapter. `with_adapter(adapter)` validates and attaches a label. `generator` and `adapter` borrow the labels. |
| `FixtureExpectation` | `new` validates an expectation label; `as_str` borrows it. |
| `FixtureStatusKind` | `Ready`, `Ignored`, or `KnownFailure`. |
| `FixtureStatus` | `ready()` has no reason; `ignored(reason)` and `known_failure(reason)` require valid text. `kind`, `reason`, and `is_ready` inspect the status. |
| `SchemaError` | Implements `std::error::Error` and `Display`; `field()` identifies the invalid metadata field. |

The record's accessors are `schema_version`, `identity`, `artifacts`,
`provenance`, `expectation`, and `status`. Its fields are private.

## Validation

Text constructors reject strings whose `trim()` result is empty. Accepted strings
retain their original content, including surrounding whitespace.

`FixturePath::new` additionally rejects paths for which the host's
`std::path::Path::is_absolute()` is true, and rejects any yielded component other
than `Component::Normal`. Components come from `Path::components`, so parsing and
component normalization follow the host platform. The stored string is not
rewritten. The constructor does not check file existence, resolve symlinks, or
establish filesystem containment.

Metadata construction does not load artifacts, run a generator, evaluate an
expectation, or enforce status handling. These are records for test consumers.

## Verification sources

[src/fixtures/mod.rs](../src/fixtures/mod.rs) contains four unit tests:

- `layout_ready_metadata_constructor_sets_schema_and_keeps_paths_relative`
- `fixture_paths_are_fixture_relative`
- `status_reasons_must_explain_non_ready_fixtures`
- `expectation_labels_must_be_non_empty`

The [repository guide](../AGENTS.md) owns the local command inventory. The
[getting-started guide](getting-started.md) selects one focused test as the first
success check.
