# surgeist-css-parser

This crate is authored in the Surgeist repository under the repository's MIT
License. This notice records third-party attribution only.

## Dependencies

This notice covers the third-party material that ships with this crate: the
`encoding_rs` runtime dependency, which is compiled into the library, and the
Web Platform Tests files that ship as test fixtures. Cargo development
dependencies build and run the tests, are not distributed with the crate, and
are not part of this inventory. This is an attribution set for the package, not
a complete transitive inventory of a binary release.

This product depends on `encoding_rs` 0.8.35 for CSS stylesheet byte decoding,
copyright Mozilla Foundation, authored by Henri Sivonen. It is a runtime
dependency compiled into the library. Its included WHATWG Encoding Standard
data carries the additional BSD license:

* License: ([Apache-2.0](licenses/encoding-rs/LICENSE-APACHE) OR [MIT](licenses/encoding-rs/LICENSE-MIT)) AND [BSD-3-Clause](licenses/encoding-rs/LICENSE-WHATWG); [upstream copyright and declaration](licenses/encoding-rs/COPYRIGHT)
* Homepage: [encoding_rs](https://github.com/hsivonen/encoding_rs)

This product includes Web Platform Tests (WPT) `css/css-syntax` test files as
packaged test fixtures, distributed by the web-platform-tests contributors. The
22 files are test data under `tests/corpus/wpt/source/`, imported at WPT commit
`ddcca5943fd41232d42149aaa19d9a04c5651b18` with the upstream license text; they
are not compiled into the library and nothing executes them. The
[acquisition record](tests/corpus/wpt/acquisition.json) binds each file to its
immutable URL, SHA-256 digest, license and any explicit author declaration. The
[corpus guide](tests/corpus/wpt/README.md) describes the reviewed vectors adapted
from them.

* License: [BSD-3-Clause](tests/corpus/wpt/source/LICENSE.md)
* Homepage: [Web Platform Tests](https://web-platform-tests.org)
