# surgeist-shape

## Dependencies

This dependency inventory covers `surgeist-shape` 0.1.0 with the repository's
default Cargo features for `aarch64-apple-darwin`. The enabled normal and build
dependency graph was checked with
`cargo tree --offline --locked -p surgeist-shape -e normal,build --target aarch64-apple-darwin` against the
pre-import local `Cargo.lock`. The versions below describe that inspected
resolution; the current product resolution is committed in
[../../Cargo.lock](../../Cargo.lock). A changed resolution or downstream
configuration can require different attribution. No build-only dependencies are
enabled in this configuration.

The inventory records dependency relationships and does not establish that
upstream source code or binaries are redistributed with this repository. It
excludes optional packages present only in the lockfile, upstream development
dependencies, and other repositories. The project's own license remains in
[LICENSE](LICENSE).

License texts were copied unchanged from the exact-version Cargo registry
packages. The accompanying Kurbo author list comes from its recorded upstream
source revision. Each component retains its upstream choice of MIT or Apache
License 2.0; inclusion here does not select one alternative.

### arrayvec 0.7.7

This product depends transitively on arrayvec, distributed by bluss (Ulrik
Sverdrup), through kurbo and polycool:

* License: [MIT](licenses/arrayvec/LICENSE-MIT) OR [Apache License 2.0](licenses/arrayvec/LICENSE-APACHE)
* Homepage: [arrayvec](https://github.com/bluss/arrayvec)

### kurbo 0.13.1 and polycool 0.4.0

This product depends directly on kurbo and transitively on polycool through
kurbo. Both components come from the Kurbo upstream project and carry identical
license texts, including Raph Levien's copyright notice:

* License: [Apache License 2.0](licenses/kurbo/LICENSE-APACHE) OR [MIT](licenses/kurbo/LICENSE-MIT)
* Homepage: [Kurbo](https://github.com/linebender/kurbo)
* Authors: [Kurbo copyright authors](licenses/kurbo/AUTHORS)

Kurbo's upstream documentation credits code adapted from `lyon_geom` to Nicolas
Silva. The included author list preserves that attribution and was retrieved
from the [source revision recorded by kurbo 0.13.1](https://github.com/linebender/kurbo/blob/838b69ef35f3381cefc007d86aa8364f5a6dc673/AUTHORS).

### smallvec 1.15.2

This product depends transitively on smallvec, distributed by The Servo Project
Developers, through kurbo:

* License: [MIT](licenses/smallvec/LICENSE-MIT) OR [Apache License 2.0](licenses/smallvec/LICENSE-APACHE)
* Homepage: [smallvec](https://github.com/servo/rust-smallvec)
