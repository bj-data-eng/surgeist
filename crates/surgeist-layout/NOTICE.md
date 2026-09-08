# surgeist-layout

## Dependencies

This notice covers third-party material incorporated into the `surgeist-layout`
0.1.0 source package and its direct optional and development dependencies, as
declared in [Cargo.toml](Cargo.toml) and resolved in [the product Cargo.lock](../../Cargo.lock).
Surgeist's own license remains in [LICENSE](LICENSE).

The default library has no external Cargo dependencies. The optional generator
and test dependencies below are part of the shared product resolution; their
external source is not vendored here. Their entries describe use and preserve
license alternatives.
This is not a transitive license inventory for a compiled generator, test binary,
or downstream application. The separate dependency inventory of
`tools/surgeist-layout-audits`, excluded from the package by `Cargo.toml`, is
outside this notice's scope. Browser executables and temporary reference
checkouts are also outside the source package.

### Ahem

This product includes the Ahem test font, created by Todd Fahrner and updated by
Paul Nelson, distributed through Kozea/Ahem:

* License: [Public domain / Creative Commons Zero declaration](licenses/ahem/COPYING)
* Homepage: [Ahem](https://github.com/Kozea/Ahem)

The font is embedded as WOFF2 in
[test_base_style.css](tests/layout/browser_parity/scripts/gentest/test_base_style.css).
Its embedded bytes are retained from Taffy's pinned browser-test support.
The included `COPYING` is the original declaration from
[Kozea/Ahem revision `7f4a69f546fd7bb7ba7031536d8470fbe87f4e26`](https://github.com/Kozea/Ahem/tree/7f4a69f546fd7bb7ba7031536d8470fbe87f4e26).

### Proptest

This product depends on Proptest 1.11.0, authored by Jason Lingle:

* License: [MIT](licenses/proptest/LICENSE-MIT) OR [Apache-2.0](licenses/proptest/LICENSE-APACHE)
* Homepage: [Proptest](https://proptest-rs.github.io/proptest/proptest/index.html)

The `proptest` crate is a development dependency for property tests, with default
features disabled and `std` enabled. Its license retains the copyright notice
for FullContact, Inc.

### roxmltree

This product depends on roxmltree 0.21.1, authored by Yevhenii Reizner:

* License: [MIT](licenses/roxmltree/LICENSE-MIT) OR [Apache-2.0](licenses/roxmltree/LICENSE-APACHE)
* Homepage: [roxmltree](https://github.com/RazrFalcon/roxmltree)

The `roxmltree` crate is a development dependency used to read XML fixtures.

### Serde

This product depends on Serde 1.0.228 and Serde JSON 1.0.145, authored by Erick
Tryzelaar and David Tolnay:

* License: [MIT](licenses/serde/LICENSE-MIT) OR [Apache-2.0](licenses/serde/LICENSE-APACHE)
* Homepages: [Serde](https://serde.rs) and [Serde JSON](https://github.com/serde-rs/json)

The `serde` crate, with `derive`, and `serde_json`, with `raw_value`, are optional
dependencies enabled by `layout-golden-generate`. The `serde_json` crate is also
a development dependency. The two selected releases carry identical license
text and share the included license files.

### serde_path_to_error

This product depends on serde_path_to_error 0.1.20, authored by David Tolnay:

* License: [MIT](licenses/serde-path-to-error/LICENSE-MIT) OR [Apache-2.0](licenses/serde-path-to-error/LICENSE-APACHE)
* Homepage: [serde_path_to_error](https://github.com/dtolnay/path-to-error)

The crate is an optional dependency enabled by `layout-golden-generate` for
reporting paths in deserialization errors.

### surgeist-generator

This product depends on surgeist-generator 0.2.0, distributed by bj-data-eng:

* License: [MIT](licenses/surgeist-generator/LICENSE)
* Homepage: [surgeist-generator](https://github.com/bj-data-eng/surgeist-generator)

Before the snapshot import, the dependency was pinned to
[`1e0927a299b405792897a9bf5909e38bb8c58053`](https://github.com/bj-data-eng/surgeist-generator/tree/1e0927a299b405792897a9bf5909e38bb8c58053),
which remains historical dependency provenance. The current dependency uses the
local [surgeist-generator crate](../surgeist-generator/Cargo.toml), with default
features disabled and `browser-corpus` enabled. It supplies shared browser-corpus
tooling when `layout-golden-generate` is enabled.

### Taffy

This product includes code and test material ported and adapted from Taffy
0.10.1, authored by Alice Cecile, Johnathan Kelley, Nico Burns, and the Taffy
contributors:

* License: [MIT](licenses/taffy/LICENSE.md)
* Homepage: [Taffy](https://github.com/DioxusLabs/taffy)

The layout implementation has since diverged substantially, with Surgeist's own
public data model, traversal contracts, algorithm phases, and test oracle.
Taffy is incorporated source, rather than a current Cargo dependency.

Supplemental browser fixtures are copied from Taffy's `test_fixtures` at
[`d1ff7e339b9ee35b33858779f8d7653197e93d92`](https://github.com/DioxusLabs/taffy/tree/d1ff7e339b9ee35b33858779f8d7653197e93d92).
The [corpus manifest](tests/layout/browser_parity/corpus.toml) and
[import attestation](tests/layout/browser_parity/html/.surgeist-source.json)
record their source pin, import rules, and file hashes. The browser
[helper JavaScript](tests/layout/browser_parity/scripts/gentest/test_helper.js)
and [base stylesheet](tests/layout/browser_parity/scripts/gentest/test_base_style.css)
also contain adapted Taffy test support. The generated XML corpus includes
expectations derived from these fixtures.

The included upstream license preserves Taffy's original-author notice and its
acknowledgment of the Stretch crate, copyright 2018 Visly Inc.

### Web Platform Tests

This product includes browser fixtures adapted from Web Platform Tests (WPT),
distributed by the web-platform-tests contributors:

* License: [BSD-3-Clause](licenses/web-platform-tests/LICENSE.md), with [CC0-1.0](licenses/web-platform-tests/legalcode.txt) for the individually dedicated source material identified in the [upstream notices](licenses/web-platform-tests/NOTICE.md)
* Homepage: [Web Platform Tests](https://web-platform-tests.org)

The adapted cases live in
[subgrid](tests/layout/browser_parity/html/subgrid/) and
[grid-lanes](tests/layout/browser_parity/html/grid-lanes/), with corresponding
generated XML expectations. Their `Derived from WPT` comments retain source
paths, original assertions where recorded, and the narrower Surgeist fixture
contracts. Pinned source URLs identify revision
[`f01d00b6963a8374784e6aadc67e608b100069d0`](https://github.com/web-platform-tests/wpt/tree/f01d00b6963a8374784e6aadc67e608b100069d0).
Some derived fixtures record only an upstream path; their exact adaptation
revision is not recorded. The included license and individual notices were
checked against the named upstream files at the pinned revision.

## Implementation references

The implementation has also been informed by CSS layout specifications and
browser behavior observed through local parity fixtures. WebKit and Blink
algorithms were studied for grid, subgrid, grid-lanes, inline layout, and baseline
behavior. This acknowledgment records those implementation references; the
incorporated code and fixture material is identified above.
