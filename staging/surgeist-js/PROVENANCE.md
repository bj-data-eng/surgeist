# JavaScript engine staging source

The source basis is [Boa v0.22 at 337a3668](https://github.com/boa-dev/boa/tree/337a3668a0dc86dd401ea20906e782249a64a228).
The isolated staging workspace holds the imported implementation while its
unsafe Rust is replaced. Production workspace integration requires the
qualified safe source and a landing branch with safe history.

| Boa source | Staging package |
| --- | --- |
| `core/engine` | `surgeist-js` |
| `core/ast` | `surgeist-js-ast` |
| `core/gc` | `surgeist-js-gc` |
| `core/icu_provider` | `surgeist-js-icu-provider` |
| `core/interner` | `surgeist-js-interner` |
| `core/macros` | `surgeist-js-macros` |
| `core/parser` | `surgeist-js-parser` |
| `core/string` | `surgeist-js-string` |
| `utils/small_btree` | `surgeist-js-small-btree` |
| `utils/tag_ptr` | `surgeist-js-tagged-pointer` |
| `core/runtime` | `surgeist-js-test-host` |
| `core/wintertc` | `surgeist-js-test-web-api` |
| `benches` | `surgeist-js-benchmarks` |
| `tests/tester` | `surgeist-js-test262` |
| `tests/macros` | `surgeist-js-macro-tests` |
| `tests/insta-bytecode` | `surgeist-js-bytecode-tests` |

The first ten packages sit under `crates/`. The optional ICU provider is
selected by `intl_bundled`. The other six are under `tools/` and outside
the engine's dependency graph. The test host supplies the upstream harness
and benchmark APIs. Surgeist's browser bindings own the browser-facing
objects separately.

The import maps package and Rust crate names, including generated macro
paths. The bytecode snapshot filenames follow the package name and keep
their expected contents. The JavaScript fixtures, ICU4X blobs and Test262
configuration are copied byte-for-byte. The staging lock retains the
upstream versions and checksums for the selected external packages.
Attribution and data sources are in [NOTICE.md](NOTICE.md).

The retained source surface for safety review includes libraries, tests,
benches, build scripts, and generated proc-macro code. The initial conversion
covers values, strings, interning and VM access. It also covers the heap, object
handles, native storage, tracing, buffers, typed arrays and Atomics.
Review includes parser, macro, harness and residual utility
sites. Search all retained Rust for `unsafe` and evaluate the selected
features and targets with unsafe code forbidden at final qualification.

See [baseline/README.md](baseline/README.md) for the evidence inputs and
benchmark configuration.
