# Surgeist

[Surgeist](https://github.com/bj-data-eng/surgeist) is licensed under its own
[MIT License](LICENSE). This notice records attribution for the repository source
checkout: the 13 facade production path dependencies in [Cargo.toml](Cargo.toml),
the `surgeist-test` support crate, the shared `surgeist-generator` corpus tooling,
and the two direct dependencies of the
[API generator](https://github.com/bj-data-eng/surgeist/blob/0b0c6338c20f77f9dfe743c47914afaf6b98792d/api/generator/Cargo.toml).
All 15 crates under `crates/` are source owned in this repository. They were
imported from the 14 revisions selected by root
[`e0303b14ccd6a81cf3d092105201daf7797ef6aa`](https://github.com/bj-data-eng/surgeist/tree/e0303b14ccd6a81cf3d092105201daf7797ef6aa/crates)
and `surgeist-generator`
[`17f6159a4adb18f0d03cab58f81ad635660e2054`](https://github.com/bj-data-eng/surgeist-generator/tree/17f6159a4adb18f0d03cab58f81ad635660e2054).
The original project homepages and immutable citations below preserve those
source origins; current package facts come from this checkout's manifests.

The checkout also retains attribution for adapted source and bundled test assets
carried by the CSS, layout, and render crates. The copies linked below preserve
their upstream legal text and modification notices. All 15 imported `LICENSE` files contain
identical text and share one copy here; the root project's own license remains
separate.

This source checkout does not contain a linked application. The summary below
does not enumerate every crate's transitive, optional, platform, or compiled
release dependencies. Crate-local notices retain their stated inventory scopes;
the [crate notice index](#crate-notices) locates that material. A compiled release
requires an inventory matching the material, targets, and features actually
distributed. API-generator
dependencies are development tooling, not root facade runtime dependencies;
their entries do not imply that their source or compiled libraries are bundled
with this checkout. Distribute this notice and its accompanying `licenses/`
files together, retaining crate notices, their linked legal material, and source
provenance with redistributed crate source. The local crate links cover the
complete source checkout; a separately packaged subset needs its own valid
notice and license layout.

## Dependencies

### Ahem

The layout and render crates include Ahem test fonts. Layout embeds WOFF2 bytes
retained from Taffy's browser-test support and credits Todd Fahrner and Paul
Nelson through Kozea/Ahem. Render bundles a W3C-distributed TTF and credits Todd
Fahrner and Myles C. Maxfield in the included upstream README.

* License: [Public domain with the upstream Creative Commons Zero fallback](licenses/ahem/COPYING)
* Homepages: [Kozea/Ahem](https://github.com/Kozea/Ahem) and [W3C Ahem](https://www.w3.org/Style/CSS/Test/Fonts/Ahem/)
* Authorship and usage: [W3C upstream README](licenses/ahem/UPSTREAM-README)

Both crates carry byte-identical `COPYING` declarations, shared here. Layout's
[source notice](https://github.com/bj-data-eng/surgeist-layout/blob/b65a18c042655f47d0870f712c7b5e647b5e28a6/NOTICE.md#ahem)
identifies the declaration's Kozea revision; render's
[fixture provenance](https://github.com/bj-data-eng/surgeist-render/blob/e44860e7df530795fef2d423024723d5fa5dc447/tests/fixtures/fonts/ahem/PROVENANCE.md)
records its font and notice hashes. These fonts are test assets.

### CSSTree

The CSS crate includes CSSTree test fixtures and derived neutral expectations
from commit `88e3d965c0b1628642a30a841745b410d6835052`, distributed by Roman Dvornov.

* License: [MIT](licenses/csstree/LICENSE)
* Homepage: [CSSTree](https://github.com/csstree/csstree)

The [pinned corpus README](https://github.com/bj-data-eng/surgeist-css/blob/14ff8f37b2417c5c836eb04986188aadbc9310f1/tests/corpus/csstree/README.md)
records the source tree, import, and transformation provenance. The source
license is included verbatim, retaining the 2016–2026 copyright attribution.

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

### surgeist-generator 0.2.0

The source checkout includes surgeist-generator, distributed by bj-data-eng.
It supplies shared CSS corpus tooling and browser-corpus infrastructure, used
by the layout corpus adapter; it is not a dependency of the root facade.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-generator](https://github.com/bj-data-eng/surgeist-generator)
* Direct dependency attribution: [crate notice](crates/surgeist-generator/NOTICE.md) and [license material](crates/surgeist-generator/licenses/)

The imported notice covers nine direct dependencies, including optional browser
features and the Apple-Silicon macOS dependency. Preserve its license
alternatives, Rustix exception text, and copyright material with the crate.

### surgeist-layout 0.1.0

The root facade depends on surgeist-layout, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-layout](https://github.com/bj-data-eng/surgeist-layout)

The crate records adaptation from Taffy 0.10.1 and continued use of Taffy layout
fixtures and browser-test support. The [retained adaptation notice](licenses/taffy/NOTICE.md)
records that source lineage; the
[snapshot source notice](https://github.com/bj-data-eng/surgeist-layout/blob/b65a18c042655f47d0870f712c7b5e647b5e28a6/NOTICE.md#taffy)
identifies the incorporated material and fixture pins. The current
[Taffy MIT license](licenses/taffy/LICENSE.md) is included verbatim and retains
the original authors' copyright statement and the 2018 Visly Inc. attribution
for Stretch. Upstream:
[Taffy](https://github.com/DioxusLabs/taffy).

### surgeist-render 0.1.0

The root facade depends on surgeist-render, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-render](https://github.com/bj-data-eng/surgeist-render)

The crate includes private raster-engine source adapted from Vello 0.9.0,
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

The source checkout includes the surgeist-test workspace member,
distributed by bj-data-eng. It supplies shared test infrastructure and fixtures;
it is also an API-audit input and is not a dependency of the root facade.

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

### Web Platform Tests

The layout crate includes subgrid and grid-lanes browser fixtures adapted from
Web Platform Tests (WPT), distributed by the web-platform-tests contributors,
and generated expectations derived from those fixtures.

* License: [BSD-3-Clause](licenses/web-platform-tests/LICENSE.md), with [CC0-1.0](licenses/web-platform-tests/legalcode.txt) for individually dedicated source material
* Homepage: [Web Platform Tests](https://web-platform-tests.org)
* Source authors and individual dedications: [upstream notices](licenses/web-platform-tests/NOTICE.md)

The [snapshot source notice](https://github.com/bj-data-eng/surgeist-layout/blob/b65a18c042655f47d0870f712c7b5e647b5e28a6/NOTICE.md#web-platform-tests)
records revision `f01d00b6963a8374784e6aadc67e608b100069d0` for pinned source
URLs. Some adapted fixtures record only an upstream path, so their exact
adaptation revision remains unrecorded. The included license and source notices
were checked by the leaf against the named files at the pinned revision.

## Crate Notices

These local records accompany the imported source and describe each crate's
own attribution scope. Keep their relative license paths and bundled-asset
provenance intact. Their direct-dependency inventories do not imply a complete
transitive binary-release inventory.

| Crate | Local attribution |
| --- | --- |
| surgeist-animation | [NOTICE.md](crates/surgeist-animation/NOTICE.md) |
| surgeist-css | [NOTICE.md](crates/surgeist-css/NOTICE.md) |
| surgeist-dialog | [LICENSE](crates/surgeist-dialog/LICENSE); no separate notice |
| surgeist-generator | [NOTICE.md](crates/surgeist-generator/NOTICE.md) |
| surgeist-layout | [NOTICE.md](crates/surgeist-layout/NOTICE.md) |
| surgeist-render | [NOTICE.md](crates/surgeist-render/NOTICE.md) |
| surgeist-retained | [NOTICE.md](crates/surgeist-retained/NOTICE.md) |
| surgeist-runtime | [NOTICE.md](crates/surgeist-runtime/NOTICE.md) |
| surgeist-shape | [NOTICE.md](crates/surgeist-shape/NOTICE.md) |
| surgeist-style | [NOTICE.md](crates/surgeist-style/NOTICE.md) |
| surgeist-task | [NOTICE.md](crates/surgeist-task/NOTICE.md) |
| surgeist-template | [NOTICE.md](crates/surgeist-template/NOTICE.md) |
| surgeist-test | [NOTICE.md](crates/surgeist-test/NOTICE.md) |
| surgeist-text | [NOTICE.md](crates/surgeist-text/NOTICE.md) |
| surgeist-window | [NOTICE.md](crates/surgeist-window/NOTICE.md) |
