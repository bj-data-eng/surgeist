# Surgeist

[Surgeist](https://github.com/bj-data-eng/surgeist) is licensed under its own
[MIT License](LICENSE). This notice records attribution for the root source
checkout: the 13 production path dependencies in [Cargo.toml](Cargo.toml), the
audit-only `surgeist-test` submodule, and the two direct dependencies of the
[API generator](https://github.com/bj-data-eng/surgeist/blob/0b0c6338c20f77f9dfe743c47914afaf6b98792d/api/generator/Cargo.toml).
Leaf versions below refer to the
revisions selected by the root's committed Git submodule links; their source
repositories are recorded in [.gitmodules](.gitmodules).

The checkout also retains the existing notices for adapted source carried by
the layout and render leaves. The copies linked below preserve their upstream
legal text and modification notices. All 14 leaf `LICENSE` files contain
identical text and share one copy here; the root project's own license remains
separate.

This source checkout does not contain a linked application. This inventory
does not enumerate transitive runtime dependencies, optional platform or
feature dependencies, or dependencies of a compiled release. Those inventories
belong to the owning leaf or the particular release distribution and must match
the material, targets, and features actually distributed. API-generator
dependencies are development tooling, not root facade runtime dependencies;
their entries do not imply that their source or compiled libraries are bundled
with this checkout. Distribute this notice and its accompanying `licenses/`
files together, retaining applicable leaf notices with redistributed leaf source.

## Dependencies

### public-api 0.50.0

The API generator depends on public-api, distributed by the contributors to
cargo-public-api.

* License: [MIT](licenses/cargo-public-api/LICENSE)
* Homepage: [public-api](https://github.com/cargo-public-api/cargo-public-api/tree/main/public-api)

The exact package's source metadata identifies commit
`05a842ac79b84f44e8fcabaeebfd351c137391b3`. Its
[upstream license](https://github.com/cargo-public-api/cargo-public-api/blob/05a842ac79b84f44e8fcabaeebfd351c137391b3/LICENSE)
is included verbatim and is identical to the selected rustdoc-json license.

### rustdoc-json 0.9.7

The API generator depends on rustdoc-json, distributed by the contributors to
cargo-public-api.

* License: [MIT](licenses/cargo-public-api/LICENSE)
* Homepage: [rustdoc-json](https://github.com/cargo-public-api/cargo-public-api/tree/main/rustdoc-json)

The exact package's source metadata identifies commit
`5e7a38ed6bcfa2a5a4c19c5911d80e00b28d72a6`. Its
[upstream license](https://github.com/cargo-public-api/cargo-public-api/blob/5e7a38ed6bcfa2a5a4c19c5911d80e00b28d72a6/LICENSE)
is included verbatim in the shared cargo-public-api license file.

### surgeist-animation 0.1.0

The root facade depends on surgeist-animation, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-animation](https://github.com/bj-data-eng/surgeist-animation)

### surgeist-css 0.1.0

The root facade depends on surgeist-css, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-css](https://github.com/bj-data-eng/surgeist-css)

### surgeist-dialog 0.1.0

The root facade depends on surgeist-dialog, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-dialog](https://github.com/bj-data-eng/surgeist-dialog)

### surgeist-layout 0.1.0

The root facade depends on surgeist-layout, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-layout](https://github.com/bj-data-eng/surgeist-layout)

The leaf records adaptation from Taffy 0.10.1 and continued use of Taffy layout
fixtures. Its [original notice](licenses/taffy/NOTICE.md) and
[Taffy MIT license](licenses/taffy/LICENSE-TAFFY.md) are included
verbatim. The latter retains the original authors' copyright statement and the
2018 Visly Inc. attribution for Stretch. Upstream:
[Taffy](https://github.com/DioxusLabs/taffy).

### surgeist-render 0.1.0

The root facade depends on surgeist-render, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-render](https://github.com/bj-data-eng/surgeist-render)

The leaf includes private raster-engine source adapted from Vello 0.9.0,
copyright 2020 the Vello Authors. The adapted material retains the
[Apache-2.0](licenses/vello/LICENSES/Vello-0.9.0-APACHE-2.0.txt) OR
[MIT](licenses/vello/LICENSES/Vello-0.9.0-MIT.txt) license alternatives.
The [original Vello provenance and modification notice](licenses/vello/NOTICE-VELLO.md)
is included verbatim with its relative license links intact. Upstream:
[Vello](https://github.com/linebender/vello).

### surgeist-retained 0.1.0

The root facade depends on surgeist-retained, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-retained](https://github.com/bj-data-eng/surgeist-retained)

### surgeist-runtime 0.1.0

The root facade depends on surgeist-runtime, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-runtime](https://github.com/bj-data-eng/surgeist-runtime)

### surgeist-shape 0.1.0

The root facade depends on surgeist-shape, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-shape](https://github.com/bj-data-eng/surgeist-shape)

### surgeist-style 0.1.0

The root facade depends on surgeist-style, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-style](https://github.com/bj-data-eng/surgeist-style)

### surgeist-task 0.1.0

The root facade depends on surgeist-task, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-task](https://github.com/bj-data-eng/surgeist-task)

### surgeist-template 0.1.0

The root facade depends on surgeist-template, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-template](https://github.com/bj-data-eng/surgeist-template)

### surgeist-test 0.1.0

The source checkout includes the audit-only surgeist-test submodule,
distributed by bj-data-eng. It supplies shared test infrastructure and fixtures;
it is not a dependency of the root facade.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-test](https://github.com/bj-data-eng/surgeist-test)

### surgeist-text 0.1.0

The root facade depends on surgeist-text, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-text](https://github.com/bj-data-eng/surgeist-text)

### surgeist-window 0.1.0

The root facade depends on surgeist-window, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-window](https://github.com/bj-data-eng/surgeist-window)
