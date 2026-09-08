# surgeist-style

## Dependencies

This notice covers the 37 external packages in the locally resolved dependency
graph for `surgeist-style` 0.1.0 with its manifest defaults and no target filter.
The versions below identify the inspected sources. The current product
resolution is committed in [../../Cargo.lock](../../Cargo.lock); changed
resolutions may select different compatible versions.

Each entry identifies its use in the library, its build, or the test suite.
Windows-only entries belong to the test dependency graph. A listed dependency is
not a claim that its source or compiled code is included in every distribution;
downstream releases must account for their actual versions, targets, and features.

License and accompanying attribution files are copied verbatim from the listed
versions' published crate sources. Shared upstream projects share local files
only where their legal text matches. Keep this notice and `licenses/` together
when distributing this attribution set. The project's own [MIT license](LICENSE)
is separate from these third-party terms.

### arrayvec 0.7.7

This product depends on arrayvec, distributed by Ulrik Sverdrup (bluss):

* License: [MIT](licenses/arrayvec/LICENSE-MIT) OR [Apache-2.0](licenses/arrayvec/LICENSE-APACHE)
* Homepage: [arrayvec](https://github.com/bluss/arrayvec)
* Use: Transitive library dependency through kurbo and polycool.

### autocfg 1.5.1

This product depends on autocfg, distributed by Josh Stone:

* License: [Apache-2.0](licenses/autocfg/LICENSE-APACHE) OR [MIT](licenses/autocfg/LICENSE-MIT)
* Homepage: [autocfg](https://github.com/cuviper/autocfg)
* Use: Build dependency of num-traits.

### color 0.3.3

This product depends on color, distributed by the Color Authors:

* License: [Apache-2.0](licenses/color/LICENSE-APACHE) OR [MIT](licenses/color/LICENSE-MIT)
* Homepage: [Color](https://github.com/linebender/color)
* Use: Transitive library dependency through peniko.

### equivalent 1.0.2

This product depends on equivalent:

* License: [Apache-2.0](licenses/equivalent/LICENSE-APACHE) OR [MIT](licenses/equivalent/LICENSE-MIT)
* Homepage: [equivalent](https://github.com/indexmap-rs/equivalent)
* Use: Transitive test dependency through indexmap.

### euclid 0.22.14

This product depends on euclid, distributed by the Servo Project Developers:

* License: [MIT](licenses/euclid/LICENSE-MIT) OR [Apache-2.0](licenses/euclid/LICENSE-APACHE)
* Homepage: [euclid](https://github.com/servo/euclid)
* Attribution: [COPYRIGHT](licenses/euclid/COPYRIGHT)
* Use: Transitive library dependency through kurbo.

### glob 0.3.3

This product depends on glob, distributed by the Rust Project Developers:

* License: [MIT](licenses/glob/LICENSE-MIT) OR [Apache-2.0](licenses/glob/LICENSE-APACHE)
* Homepage: [glob](https://github.com/rust-lang/glob)
* Use: Test dependency through trybuild.

### hashbrown 0.17.1

This product depends on hashbrown:

* License: [MIT](licenses/hashbrown/LICENSE-MIT) OR [Apache-2.0](licenses/hashbrown/LICENSE-APACHE)
* Homepage: [hashbrown](https://github.com/rust-lang/hashbrown)
* Use: Transitive test dependency through indexmap.

### indexmap 2.14.0

This product depends on indexmap:

* License: [Apache-2.0](licenses/indexmap/LICENSE-APACHE) OR [MIT](licenses/indexmap/LICENSE-MIT)
* Homepage: [indexmap](https://github.com/indexmap-rs/indexmap)
* Use: Transitive test dependency through toml.

### itoa 1.0.18

This product depends on itoa, distributed by David Tolnay:

* License: [MIT](licenses/itoa/LICENSE-MIT) OR [Apache-2.0](licenses/itoa/LICENSE-APACHE)
* Homepage: [itoa](https://github.com/dtolnay/itoa)
* Use: Transitive test dependency through serde_json.

### kurbo 0.13.1 and polycool 0.4.0

This product depends on kurbo and polycool, distributed by the Kurbo Authors:

* License: [Apache-2.0](licenses/kurbo/LICENSE-APACHE) OR [MIT](licenses/kurbo/LICENSE-MIT)
* Homepage: [Kurbo](https://github.com/linebender/kurbo)
* Use: Transitive library dependencies; peniko uses kurbo, which uses polycool.

### linebender_resource_handle 0.1.1

This product depends on linebender_resource_handle, distributed by the Raw Resource Handle Authors:

* License: [Apache-2.0](licenses/linebender_resource_handle/LICENSE-APACHE) OR [MIT](licenses/linebender_resource_handle/LICENSE-MIT)
* Homepage: [Linebender Resource Handle](https://github.com/linebender/raw_resource_handle)
* Attribution: [AUTHORS](licenses/linebender_resource_handle/AUTHORS)
* Use: Transitive library dependency through peniko.

### memchr 2.8.2

This product depends on memchr, distributed by Andrew Gallant and bluss:

* License: [Unlicense](licenses/memchr/UNLICENSE) OR [MIT](licenses/memchr/LICENSE-MIT)
* Homepage: [memchr](https://github.com/BurntSushi/memchr)
* Attribution: [COPYING](licenses/memchr/COPYING)
* Use: Transitive test dependency through serde_json.

### num-traits 0.2.19

This product depends on num-traits, distributed by the Rust Project Developers:

* License: [MIT](licenses/num-traits/LICENSE-MIT) OR [Apache-2.0](licenses/num-traits/LICENSE-APACHE)
* Homepage: [num-traits](https://github.com/rust-num/num-traits)
* Use: Transitive library dependency through euclid.

### peniko 0.6.1

This product depends on peniko, distributed by the Peniko Authors:

* License: [Apache-2.0](licenses/peniko/LICENSE-APACHE) OR [MIT](licenses/peniko/LICENSE-MIT)
* Homepage: [Peniko](https://github.com/linebender/peniko)
* Use: Direct library dependency for the Color-to-Peniko conversion.

### proc-macro2 1.0.106

This product depends on proc-macro2, distributed by David Tolnay and Alex Crichton:

* License: [MIT](licenses/proc-macro2/LICENSE-MIT) OR [Apache-2.0](licenses/proc-macro2/LICENSE-APACHE)
* Homepage: [proc-macro2](https://github.com/dtolnay/proc-macro2)
* Use: Transitive test dependency for compiling serde_derive.

### quote 1.0.46

This product depends on quote, distributed by David Tolnay:

* License: [MIT](licenses/quote/LICENSE-MIT) OR [Apache-2.0](licenses/quote/LICENSE-APACHE)
* Homepage: [quote](https://github.com/dtolnay/quote)
* Use: Transitive test dependency for compiling serde_derive.

### serde, serde_core, and serde_derive 1.0.228

This product depends on serde, serde_core, and serde_derive, distributed by Erick Tryzelaar and David Tolnay:

* License: [MIT](licenses/serde/LICENSE-MIT) OR [Apache-2.0](licenses/serde/LICENSE-APACHE)
* Homepage: [Serde](https://serde.rs)
* Use: Test dependencies through trybuild and its serialization dependencies;
  serde_derive is a procedural macro used when compiling that test support.

### serde_json 1.0.150

This product depends on serde_json, distributed by Erick Tryzelaar and David Tolnay:

* License: [MIT](licenses/serde_json/LICENSE-MIT) OR [Apache-2.0](licenses/serde_json/LICENSE-APACHE)
* Homepage: [Serde JSON](https://github.com/serde-rs/json)
* Use: Test dependency through trybuild.

### serde_spanned 1.1.1 and the toml crates

This product depends on serde_spanned 1.1.1, toml 1.1.2+spec-1.1.0,
toml_datetime 1.1.1+spec-1.1.0, toml_parser 1.1.2+spec-1.1.0, and
toml_writer 1.1.1+spec-1.1.0:

* License: [MIT](licenses/toml/LICENSE-MIT) OR [Apache-2.0](licenses/toml/LICENSE-APACHE)
* Homepage: [TOML for Rust](https://github.com/toml-rs/toml)
* Use: Test dependencies; trybuild uses toml, which uses the other listed crates.

### smallvec 1.15.2

This product depends on smallvec, distributed by the Servo Project Developers:

* License: [MIT](licenses/smallvec/LICENSE-MIT) OR [Apache-2.0](licenses/smallvec/LICENSE-APACHE)
* Homepage: [smallvec](https://github.com/servo/rust-smallvec)
* Use: Transitive library dependency through peniko and kurbo.

### syn 2.0.118

This product depends on syn, distributed by David Tolnay:

* License: [MIT](licenses/syn/LICENSE-MIT) OR [Apache-2.0](licenses/syn/LICENSE-APACHE)
* Homepage: [Syn](https://github.com/dtolnay/syn)
* Use: Transitive test dependency for compiling serde_derive.

### target-triple 1.0.0

This product depends on target-triple, distributed by David Tolnay:

* License: [MIT](licenses/target-triple/LICENSE-MIT) OR [Apache-2.0](licenses/target-triple/LICENSE-APACHE)
* Homepage: [target-triple](https://github.com/dtolnay/target-triple)
* Use: Test dependency through trybuild.

### termcolor 1.4.1

This product depends on termcolor, distributed by Andrew Gallant:

* License: [Unlicense](licenses/termcolor/UNLICENSE) OR [MIT](licenses/termcolor/LICENSE-MIT)
* Homepage: [termcolor](https://github.com/BurntSushi/termcolor)
* Attribution: [COPYING](licenses/termcolor/COPYING)
* Use: Test dependency through trybuild.

### trybuild 1.0.117

This product depends on trybuild, distributed by David Tolnay:

* License: [MIT](licenses/trybuild/LICENSE-MIT) OR [Apache-2.0](licenses/trybuild/LICENSE-APACHE)
* Homepage: [trybuild](https://github.com/dtolnay/trybuild)
* Use: Direct development dependency for compile-pass and compile-fail tests.

### unicode-ident 1.0.24

This product depends on unicode-ident, distributed by David Tolnay:

* License: ([MIT](licenses/unicode-ident/LICENSE-MIT) OR [Apache-2.0](licenses/unicode-ident/LICENSE-APACHE)) AND [Unicode-3.0](licenses/unicode-ident/LICENSE-UNICODE)
* Homepage: [unicode-ident](https://github.com/dtolnay/unicode-ident)
* Use: Transitive test dependency through proc-macro2 and syn. Its generated
  Unicode character tables also require the included Unicode license.

### winapi-util 0.1.11

This product depends on winapi-util, distributed by Andrew Gallant:

* License: [Unlicense](licenses/winapi-util/UNLICENSE) OR [MIT](licenses/winapi-util/LICENSE-MIT)
* Homepage: [winapi-util](https://github.com/BurntSushi/winapi-util)
* Attribution: [COPYING](licenses/winapi-util/COPYING)
* Use: Windows-only test dependency through termcolor.

### windows-link 0.2.1 and windows-sys 0.61.2

This product depends on windows-link and windows-sys, distributed by Microsoft Corporation:

* License: [MIT](licenses/windows-rs/license-mit) OR [Apache-2.0](licenses/windows-rs/license-apache-2.0)
* Homepage: [Rust for Windows](https://github.com/microsoft/windows-rs)
* Use: Windows-only test dependencies; winapi-util uses windows-sys, which uses
  windows-link.

### winnow 1.0.3

This product depends on winnow:

* License: [MIT](licenses/winnow/LICENSE-MIT)
* Homepage: [Winnow](https://github.com/winnow-rs/winnow)
* Use: Transitive test dependency through toml and toml_parser.

### zmij 1.0.21

This product depends on zmij, distributed by David Tolnay:

* License: [MIT](licenses/zmij/LICENSE-MIT)
* Homepage: [zmij](https://github.com/dtolnay/zmij)
* Use: Transitive test dependency through serde_json.
