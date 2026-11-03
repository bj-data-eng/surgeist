# surgeist-css

The project's own license is in [LICENSE](LICENSE).

## Dependencies

This notice covers the bundled CSSTree and Web Platform Tests fixtures and two
registry crates, `cfg-if` and `unicode-segmentation`, of the three in the normal
and build dependency graph of `surgeist-css` 0.1.0 with default features. The
third, `encoding_rs`, is compiled into `surgeist-css-parser`, whose
[notice](../surgeist-css-parser/NOTICE.md) records it. Cargo development
dependencies build and run the tests, are not distributed with the crate, and
are not part of this inventory.
The Cargo dependencies are not vendored in this repository. The roles below
describe their use by this crate; a downstream executable's contents depend on
its build. Transitive versions can change with dependency resolution. Upstream
license alternatives are preserved without selecting one.

This product depends on `cfg-if` 1.0.4 through the stylesheet byte decoding of
`surgeist-css-parser`, distributed by Alex Crichton:

* License: [MIT](licenses/cfg-if/LICENSE-MIT) OR [Apache-2.0](licenses/cfg-if/LICENSE-APACHE)
* Homepage: [cfg-if](https://github.com/rust-lang/cfg-if)

This product includes CSSTree test fixtures from commit
`88e3d965c0b1628642a30a841745b410d6835052`, distributed by Roman Dvornov. The
imported fixtures and generated neutral expectations are under
`tests/corpus/csstree/`; their provenance and transformations are recorded in
the [corpus README](tests/corpus/csstree/README.md):

* License: [MIT](tests/corpus/csstree/LICENSE)
* Homepage: [CSSTree](https://github.com/csstree/csstree)

This product depends on `unicode-segmentation` 1.13.3 for authored text-alignment
string validation. Its package lists kwantam and Manish Goregaokar as authors;
the included MIT text credits the Rust Project Developers:

* License: [MIT](licenses/unicode-segmentation/LICENSE-MIT) OR [Apache-2.0](licenses/unicode-segmentation/LICENSE-APACHE); [upstream copyright and declaration](licenses/unicode-segmentation/COPYRIGHT)
* Homepage: [unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation)

This product includes a selected Web Platform Tests CSS source bundle from commit
`ddcca5943fd41232d42149aaa19d9a04c5651b18`, distributed by the web-platform-tests
contributors. Preserved upstream source and reviewed parsing/serialization data
adaptations live in `tests/corpus/wpt/`. The [acquisition record](tests/corpus/wpt/acquisition.json)
binds each source to its immutable URL, SHA-256 digest, license, and any explicit
author declarations. The [corpus guide](tests/corpus/wpt/README.md) describes the
adaptations and their distinction from browser execution.

* License: [BSD-3-Clause](tests/corpus/wpt/source/LICENSE.md)
* Homepage: [Web Platform Tests](https://web-platform-tests.org)
