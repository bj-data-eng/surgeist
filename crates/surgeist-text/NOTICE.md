# surgeist-text

## Dependencies

This notice covers the direct dependencies declared for `surgeist-text`
0.1.0 in [Cargo.toml](Cargo.toml), including the default configuration, the
`text-accessibility` and `text-render` features, and development dependencies.
It records source-level dependency attribution, not a resolved binary, release,
or transitive dependency inventory. A dependency declaration does not establish
that its code is redistributed in a particular artifact. Upstream example and
test assets are not bundled except for the Ahem font identified below.

The project's own MIT license remains in [LICENSE](LICENSE). Keep this notice
and its accompanying `licenses/` files together.

### accesskit 0.24.1

This product depends on accesskit, distributed by The AccessKit contributors:

* License: [MIT](licenses/accesskit/LICENSE-MIT) or
  [Apache License 2.0](licenses/accesskit/LICENSE-APACHE)
* Homepage: [AccessKit](https://accesskit.dev/)
* Additional material: [source notices](licenses/accesskit/NOTICE.md),
  [Chromium BSD license](licenses/accesskit/LICENSE.chromium), and
  [copyright authors](licenses/accesskit/AUTHORS)

This optional dependency is enabled by `text-accessibility`. AccessKit contains
code derived from Chromium and kurbo; the accompanying source notices retain
those attributions and their applicable license terms.

### fontique 0.9.0

This product depends on Fontique, distributed by the Parley Authors:

* License: [Apache License 2.0](licenses/fontique/LICENSE-APACHE) or
  [MIT](licenses/fontique/LICENSE-MIT)
* Homepage: [Fontique](https://github.com/linebender/parley)

Fontique supplies safe catalogue, registration and font selection interfaces.
Its native implementation remains an external dependency.

### linebender_resource_handle 0.1.1

This product depends on Linebender resource handles:

* License: [Apache License 2.0](licenses/linebender-resource-handle/LICENSE-APACHE) or
  [MIT](licenses/linebender-resource-handle/LICENSE-MIT)
* Homepage: [Linebender resource handles](https://github.com/linebender/raw_resource_handle)

The catalogue retains registered immutable font bytes through its safe Blob API.

### parley 0.9.0

This product depends on parley, distributed by the Parley Authors:

* License: [Apache License 2.0](licenses/parley/LICENSE-APACHE) or
  [MIT](licenses/parley/LICENSE-MIT)
* Homepage: [Parley](https://github.com/linebender/parley)

Parley provides the text layout implementation in the default configuration.
The `text-accessibility` feature also enables Parley's AccessKit integration.

### pollster 0.4.0

This product depends on pollster, distributed by Joshua Barretto:

* License: [Apache License 2.0](licenses/pollster/LICENSE-APACHE) or
  [MIT](licenses/pollster/LICENSE-MIT)
* Homepage: [Pollster](https://github.com/zesterer/pollster)

This development dependency is used by a renderer test under `text-render`.

### skrifa 0.42.1

This product depends on Skrifa from Fontations:

* License: [MIT](licenses/fontations/LICENSE-MIT) or
  [Apache License 2.0](licenses/fontations/LICENSE-APACHE)
* Homepage: [Fontations](https://github.com/googlefonts/fontations)

The catalogue uses its safe font-file interface for whole-file face admission.

### surgeist-render 0.1.0

This product depends on surgeist-render, distributed by bj-data-eng:

* License: [MIT](licenses/surgeist-render/LICENSE)
* Homepage: [surgeist-render](https://github.com/bj-data-eng/surgeist-render)

This optional sibling path dependency is enabled by `text-render`. Its license
preserves the pre-import sibling source attribution; its dependencies are outside
this notice's direct-dependency scope.

## Bundled test font

Ahem was developed by Todd Fahrner and Myles C. Maxfield. This repository includes
it for deterministic Text tests, copied from the existing Render fixture:

* License: [public-domain dedication with CC0 fallback](tests/fixtures/fonts/ahem/COPYING)
* Homepage: [Ahem](https://www.w3.org/Style/CSS/Test/Fonts/Ahem/)
* Original description: [upstream README](tests/fixtures/fonts/ahem/UPSTREAM-README)
* Exact file identity and facts: [provenance](tests/fixtures/fonts/ahem/PROVENANCE.md)

The font and license files are unchanged. The fixture is not a production font
or evidence of general script, variation or shaping conformance.
