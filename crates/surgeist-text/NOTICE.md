# surgeist-text

## Dependencies

This notice covers the direct dependencies declared for `surgeist-text`
0.1.0 in [Cargo.toml](Cargo.toml), including the default configuration, the
optional `text-accessibility` feature.
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

### icu_properties 2.2.0

This product depends on ICU4X properties, distributed by the ICU4X Project Developers:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

Source-local whitespace processing uses compiled Unicode bidi-control,
East Asian Width and Script properties.

### icu_segmenter 2.2.0

This product depends on ICU4X segmentation, distributed by the ICU4X Project Developers:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

The Text interaction map uses compiled Unicode grapheme boundary data.

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
The `text-accessibility` feature projects the resulting layout through Text-owned
AccessKit integration. The adapted projection retains the upstream copyright
and SPDX attribution in its source; the [adaptation provenance](docs/accessibility-provenance.md)
identifies the exact input and retained mechanisms.

### skrifa 0.42.1

This product depends on Skrifa from Fontations:

* License: [MIT](licenses/fontations/LICENSE-MIT) or
  [Apache License 2.0](licenses/fontations/LICENSE-APACHE)
* Homepage: [Fontations](https://github.com/googlefonts/fontations)

The catalogue uses its safe font-file interface for whole-file face admission.

## Bundled test font

Ahem was developed by Todd Fahrner and Myles C. Maxfield. This repository includes
it for deterministic Text tests, copied from the existing Render fixture:

* License: [public-domain dedication with CC0 fallback](tests/fixtures/fonts/ahem/COPYING)
* Homepage: [Ahem](https://www.w3.org/Style/CSS/Test/Fonts/Ahem/)
* Original description: [upstream README](tests/fixtures/fonts/ahem/UPSTREAM-README)
* Exact file identity and facts: [provenance](tests/fixtures/fonts/ahem/PROVENANCE.md)

The font and license files are unchanged. The fixture is not a production font
or evidence of general script, variation or shaping conformance.
