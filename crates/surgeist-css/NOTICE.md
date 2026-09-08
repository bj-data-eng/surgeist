# surgeist-css

The project's own license is in [LICENSE](LICENSE).

## Dependencies

This notice covers the bundled CSSTree fixtures and the 23 registry crates in
the local Cargo resolution of `surgeist-css` 0.1.0 with default features,
including development and build dependencies. `app-strict` adds no dependencies.
The Cargo dependencies are not vendored in this repository. The roles below
describe their use by this crate; a downstream executable's contents depend on
its build. Transitive versions can change with dependency resolution. Upstream
license alternatives are preserved without selecting one.

This product depends on `cssparser` 0.37.0 and `cssparser-color` 0.5.0 as direct
runtime dependencies, and `cssparser-macros` 0.7.0 for procedural macros,
distributed by Simon Sapin (`cssparser` and `cssparser-macros`) and Emilio Cobos
Álvarez (`cssparser-color`):

* License: [MPL-2.0](licenses/rust-cssparser/LICENSE)
* Homepage: [rust-cssparser](https://github.com/servo/rust-cssparser)

This product includes CSSTree test fixtures from commit
`88e3d965c0b1628642a30a841745b410d6835052`, distributed by Roman Dvornov. The
imported fixtures and generated neutral expectations are under
`tests/corpus/csstree/`; their provenance and transformations are recorded in
the [corpus README](tests/corpus/csstree/README.md):

* License: [MIT](tests/corpus/csstree/LICENSE)
* Homepage: [CSSTree](https://github.com/csstree/csstree)

This product depends on `dtoa` 1.0.11 as a transitive runtime dependency,
distributed by David Tolnay:

* License: [MIT](licenses/dtoa/LICENSE-MIT) OR [Apache-2.0](licenses/dtoa/LICENSE-APACHE)
* Homepage: [dtoa](https://github.com/dtolnay/dtoa)

This product depends on `dtoa-short` 0.3.5 as a transitive runtime dependency,
distributed by Xidorn Quan:

* License: [MPL-2.0](licenses/dtoa-short/LICENSE)
* Homepage: [dtoa-short](https://github.com/upsuper/dtoa-short)

This product depends on `fastrand` 2.4.1 for PHF macro generation at build time,
distributed by Stjepan Glavina:

* License: [Apache-2.0](licenses/fastrand/LICENSE-APACHE) OR [MIT](licenses/fastrand/LICENSE-MIT)
* Homepage: [fastrand](https://github.com/smol-rs/fastrand)

This product depends on `itoa` 1.0.18 as a transitive runtime and test dependency,
distributed by David Tolnay:

* License: [MIT](licenses/itoa/LICENSE-MIT) OR [Apache-2.0](licenses/itoa/LICENSE-APACHE)
* Homepage: [itoa](https://github.com/dtolnay/itoa)

This product depends on `memchr` 2.8.3 through its JSON test support, distributed
by Andrew Gallant and bluss:

* License: [Unlicense](licenses/memchr/UNLICENSE) OR [MIT](licenses/memchr/LICENSE-MIT); [upstream declaration](licenses/memchr/COPYING)
* Homepage: [memchr](https://github.com/BurntSushi/memchr)

This product depends on `phf`, `phf_shared`, `phf_generator`, and `phf_macros`
0.13.1, distributed by Steven Fackler and Yuki Okushi. `phf` and `phf_shared`
support runtime lookup; `phf_generator`, `phf_macros`, and `phf_shared` also
support macro generation at build time:

* License: [MIT](licenses/rust-phf/LICENSE)
* Homepage: [rust-phf](https://github.com/rust-phf/rust-phf)

This product depends on `proc-macro2` 1.0.106 for procedural macro compilation,
distributed by David Tolnay and Alex Crichton:

* License: [MIT](licenses/proc-macro2/LICENSE-MIT) OR [Apache-2.0](licenses/proc-macro2/LICENSE-APACHE)
* Homepage: [proc-macro2](https://github.com/dtolnay/proc-macro2)

This product depends on `quote` 1.0.46 for procedural macro compilation,
distributed by David Tolnay:

* License: [MIT](licenses/quote/LICENSE-MIT) OR [Apache-2.0](licenses/quote/LICENSE-APACHE)
* Homepage: [quote](https://github.com/dtolnay/quote)

This product depends on `ryu` 1.0.23 through its JSON test support, distributed
by David Tolnay:

* License: [Apache-2.0](licenses/ryu/LICENSE-APACHE) OR [BSL-1.0](licenses/ryu/LICENSE-BOOST)
* Homepage: [ryu](https://github.com/dtolnay/ryu)

This product depends on Serde, distributed by Erick Tryzelaar and David Tolnay.
`serde` 1.0.228 and `serde_json` 1.0.145 are direct development dependencies;
`serde_core` 1.0.228 and the `serde_derive` 1.0.228 procedural macro provide
their supporting serialization and derive facilities:

* License: [MIT](licenses/serde/LICENSE-MIT) OR [Apache-2.0](licenses/serde/LICENSE-APACHE)
* Homepage: [Serde](https://serde.rs/) and [Serde JSON](https://github.com/serde-rs/json)

This product depends on `siphasher` 1.0.3 through PHF's runtime and macro
generation support, distributed by Frank Denis and the Rust Project Developers:

* License: [MIT](licenses/siphasher/MIT-LICENSE-TEXT) OR [Apache-2.0](licenses/siphasher/APACHE-2.0-LICENSE-TEXT); [upstream copyright and declaration](licenses/siphasher/COPYING)
* Homepage: [rust-siphash](https://github.com/jedisct1/rust-siphash)
* License-text provenance: [siphasher note](licenses/siphasher/NOTICE.md)

This product depends on `smallvec` 1.15.2 as a transitive runtime dependency,
distributed by the Servo Project Developers:

* License: [MIT](licenses/smallvec/LICENSE-MIT) OR [Apache-2.0](licenses/smallvec/LICENSE-APACHE)
* Homepage: [rust-smallvec](https://github.com/servo/rust-smallvec)

This product depends on `syn` 2.0.118 for procedural macro compilation,
distributed by David Tolnay:

* License: [MIT](licenses/syn/LICENSE-MIT) OR [Apache-2.0](licenses/syn/LICENSE-APACHE)
* Homepage: [syn](https://github.com/dtolnay/syn)

This product depends on `unicode-ident` 1.0.24 for procedural macro compilation,
distributed by David Tolnay, with Unicode data from Unicode, Inc.:

* License: ([MIT](licenses/unicode-ident/LICENSE-MIT) OR [Apache-2.0](licenses/unicode-ident/LICENSE-APACHE)) AND [Unicode-3.0](licenses/unicode-ident/LICENSE-UNICODE)
* Homepage: [unicode-ident](https://github.com/dtolnay/unicode-ident)
