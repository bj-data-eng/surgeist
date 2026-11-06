# JavaScript engine source

The source basis is [Boa v0.22 at 337a3668](https://github.com/boa-dev/boa/tree/337a3668a0dc86dd401ea20906e782249a64a228).
Surgeist owns the engine, its supporting crates, and the qualification tools.

| Boa source | Surgeist package |
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
| `utils/tag_ptr` | `surgeist-js-tagged-pointer`, retired with symbol tagging |
| `core/runtime` | `surgeist-js-test-host` |
| `core/wintertc` | `surgeist-js-test-web-api` |
| `benches` | `surgeist-js-benchmarks` |
| `tests/tester` | `surgeist-js-test262` |
| `tests/macros` | `surgeist-js-macro-tests` |
| `tests/insta-bytecode` | `surgeist-js-bytecode-tests` |

Engine packages sit under `crates/`. The optional ICU provider is selected by
`intl_bundled`. The six qualification packages are under `tools/` and outside
the engine's dependency graph. The test host supplies the upstream harness
and benchmark APIs. Surgeist's browser bindings own the browser-facing
objects separately.

The import maps package and Rust crate names, including generated macro
paths. The bytecode snapshot filenames follow the package name and keep
their expected contents. The engine uses the workspace dependency
resolution in [Cargo.lock](../../Cargo.lock). The pristine reference runs use
the upstream lock. Attribution for the JavaScript workloads, ICU4X data
and Test262 suite is in [NOTICE.md](NOTICE.md).

## Safety boundary

The engine uses a runtime-owned tracing heap. Host handles root live
allocations, and stored references identify allocations by heap, slot and
generation. The collector and its trace derive use safe Rust. The owned
source boundary includes libraries, tests, benchmarks, build scripts and
generated proc-macro code. Every JavaScript package inherits the workspace's
`unsafe_code = "forbid"` lint.

The JavaScript packages call registry dependencies through safe APIs.
Dependencies such as `bytemuck` and `portable-atomic` use unsafe
internally. Their selected versions and checksums are recorded in
[Cargo.lock](../../Cargo.lock).

See [benchmark and conformance inputs](../../tools/surgeist-js-test262/baseline/README.md) for the evidence inputs and
benchmark configuration.
