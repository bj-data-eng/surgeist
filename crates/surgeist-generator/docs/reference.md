# Reference

## Package and public surface

[Cargo.toml](../Cargo.toml) owns package metadata, exact dependency pins, features,
and targets. The package is `surgeist-generator` 0.2.0, library
`surgeist_generator`, edition 2024, MSRV 1.97, licensed MIT.

| Feature | Public surface | Executable / dependencies |
| --- | --- | --- |
| Default (empty) | Shared contracts reexported by `src/lib.rs` | Serde, Serde JSON, SHA-2, TOML; target-specific Rustix |
| `css-corpus` | `surgeist_generator::css` | `surgeist-css-generate`; no additional optional dependency |
| `browser-corpus` | `surgeist_generator::browser` | Caller-owned host; Chromiumoxide, Futures, Tokio, URL |

Mutation support and Rustix are restricted to Apple-Silicon macOS. The default
value/read library is checked for native and `wasm32-unknown-unknown` targets.
Owned Rust forbids unsafe code at the [library front door](../src/lib.rs).

## Interfaces and declarations

| Surface | Authoritative source | Responsibility |
| --- | --- | --- |
| Shared contracts | [src/lib.rs](../src/lib.rs), [core](../src/core/mod.rs) | Checked paths, corpus locations, manifests, sources, digests, dispositions, and reports |
| Errors | [src/error.rs](../src/error.rs) | `GeneratorError`, `GeneratorErrorKind`, and `Result` |
| CSS | [src/css/mod.rs](../src/css/mod.rs) | `CssRequest`, `CssCommand`, `run`, `run_from_env` |
| Browser | [src/browser/mod.rs](../src/browser/mod.rs) | Generation/checking, adapter models, acquisition, import, reports, supervisor entry |
| Browser declarations | [src/browser/model.rs](../src/browser/model.rs) | Corpus, fixture/case, launch/settings, resources, requests, prior ownership, adapter errors |
| Browser reports | [src/browser/report.rs](../src/browser/report.rs) | Schema-4 executable/engine identity, inputs, attestations, settings, artifact hashes, outcome accounting |

CSS accepts `import-csstree`, `import-wpt`, `generate`, and `check-corpus`. All
require explicit `--owner-root` and `--corpus-root`; both import operations
require `--source-root` and forbid `--filter`. Generation permits `--filter`.
`check-corpus` dispatches on `source.kind`: CSSTree checks source attestation,
neutral expectations, and report; WPT checks its imported inventory and receipt.
`generate` rejects WPT because no source transformation is implemented. These
operations do not acquire sources or execute JavaScript.

Manifests and caller declarations own pins, counts, paths, browser settings,
and artifact formats. This crate does not prescribe a fixture-domain serializer.

## WPT manifest and receipt

The CSS-only WPT importer reads `corpus.toml` with exactly these schema-1 fields:

| Declaration | Contract |
| --- | --- |
| `source.kind` | `"wpt"` |
| `source.repository` | `"https://github.com/web-platform-tests/wpt.git"` |
| `source.revision` | Full lowercase Git object ID accepted by `SourceRevision` |
| `source.import_root` | One non-reserved relative component |
| `[[files]]` | Nonempty explicit list of `path`, canonical `sha256`, and nonempty `licenses` path references |
| `[[licenses]]` | Nonempty explicit list of `path` and canonical `sha256` |

Unknown fields, duplicate paths or bindings, case-aliased path components,
file/directory prefix collisions, traversal, and generator-reserved path
components are rejected. Each file's license references must identify entries
in `[[licenses]]`. Source and license bytes must be regular, single-link files
with mode 0644; the shared rooted filesystem rejects symlinks and mount changes.
Only declared files are read and copied. Unlisted bundle files have no effect.

`<import_root>/.surgeist-source.json` is canonical compact JSON with one final
newline. It records `schema_version = 1`, `generator = "surgeist-css-generate"`,
`verification = "manifest-file-digests"`, declared `source` identity, SHA-256 of
the exact manifest bytes, sorted `files` with sorted license references, and
sorted `licenses`. It declares reviewed source identity; it does not attest a
Git tree. Acquisition review must bind every digest to its immutable upstream
URL at the declared revision.

Import permits an empty destination or replaces the exact inventory authenticated
by an existing canonical receipt. It rejects unknown files, unknown directories,
missing owned files, and changed bytes. Source, manifest, namespaces, and prior
inventory are revalidated before atomic publication. `check-corpus` requires the
receipt to match the current manifest and verifies the exact imported inventory
without the original bundle. Consumer-owned adaptation vectors and behavioral
checks are outside this receipt and belong beside the consuming CSS tests.

## Storage locations

| Location | Role |
| --- | --- |
| `tmp/surgeist-sources/<source-id>/<full-revision>/` | Owner-local pinned source cache |
| `tmp/surgeist-browser/` | Owner-local cache with separate browser-version entries |
| `<corpus>/.surgeist-generator/` | Machine-local coordination, transactions, and durable browser-profile journals |

Caches are outside Cargo build output and corpus transactions. Repository `tmp/`
is ignored and survives `cargo clean`. Callers must ignore the corpus coordination
directory. Cache removal is separate maintenance, not profile/publication recovery.

## Verification

[AGENTS.md](../AGENTS.md#command-inventory) owns the command inventory: offline
checks, tests, and warning-denied Clippy across no features, each optional feature,
and all features; the no-feature wasm library check; metadata, formatting,
license checks, and the installed advisory database audit.

Tracked [integration tests](../tests) cover the public API, shared contracts,
package/feature policy, CSS CLI, and browser manifests/imports. Module-local tests
cover transactions, source protection, browser lifecycle, input revalidation,
retry, adapter errors, and mixed generated/unsupported outcomes.

Ignored diagnostics are inventory-only unless explicitly authorized: retain
`--list`. `cargo audit --no-fetch --stale` uses the installed advisory database
and does not establish that it is current online. All checks use already-present
tooling, lock data, and caches; offline/no-fetch flags do not authorize acquisition.

For rationale and trust limits, see [explanation](explanation.md).
